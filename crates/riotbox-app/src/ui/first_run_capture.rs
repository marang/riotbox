mod onramp_history;
mod pending_capture;
mod routing;

use ratatui::{text::Line, widgets::ListItem};
use riotbox_audio::runtime::{AudioRuntimeLifecycle, SourceMonitorAudioRoute};
use riotbox_core::{
    action::{ActionCommand, CaptureLengthIntent, SourceMonitorMode},
    view::jam::{CaptureHandoffReadinessView, CaptureTargetKindView, SceneJumpAvailabilityView},
};

use super::{JamShellState, transport_label, w30_preview_source_readiness};

use onramp_history::{
    committed_monitor_handoff_index_after, has_completed_first_run_onramp,
    latest_source_backed_promotion_index,
};
#[cfg(test)]
pub(super) use pending_capture::{capture_pending_detail_line, capture_pending_intent_line};
use pending_capture::{pending_capture_do_next_lines, pending_w30_audition_do_next_lines};
use routing::capture_heard_path_label;
pub(super) use routing::capture_routing_lines;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FirstRunOnrampStage {
    Capture,
    CapturePending,
    CaptureReadiness,
    PromotionPending,
    Monitor,
    Performance,
}

pub(super) fn first_run_onramp_stage(shell: &JamShellState) -> Option<FirstRunOnrampStage> {
    if !shell.first_run_onramp {
        return None;
    }

    if has_completed_first_run_onramp(shell) && audio_runtime_is_running(shell) {
        return None;
    }

    let capture = &shell.app.jam_view.capture;

    if has_pending_capture(shell) {
        return Some(FirstRunOnrampStage::CapturePending);
    }

    if capture.capture_count == 0 {
        return Some(FirstRunOnrampStage::Capture);
    }

    if !capture_handoff_is_ready(shell) {
        return Some(FirstRunOnrampStage::CaptureReadiness);
    }

    if capture.last_capture_target_kind.is_none() {
        return Some(if has_pending_promotion(shell) {
            FirstRunOnrampStage::PromotionPending
        } else {
            FirstRunOnrampStage::CaptureReadiness
        });
    }

    let Some(promotion_index) = latest_source_backed_promotion_index(shell) else {
        return Some(FirstRunOnrampStage::CaptureReadiness);
    };
    let monitor_is_playable = committed_monitor_handoff_index_after(shell, promotion_index)
        .is_some()
        && current_monitor_is_playable(shell);

    if !monitor_is_playable {
        return Some(FirstRunOnrampStage::Monitor);
    }

    Some(FirstRunOnrampStage::Performance)
}

pub(super) fn first_run_onramp_lines(shell: &JamShellState) -> Vec<String> {
    match first_run_onramp_stage(shell) {
        Some(FirstRunOnrampStage::Capture) => {
            let transport_step = match (
                audio_runtime_lifecycle(shell),
                shell.app.runtime.source_monitor_audio_route,
            ) {
                (Some(AudioRuntimeLifecycle::Running), SourceMonitorAudioRoute::SourceOnly)
                | (Some(AudioRuntimeLifecycle::Running), SourceMonitorAudioRoute::Blend) => {
                    "1 [Space] start; source monitor is ready"
                }
                (Some(AudioRuntimeLifecycle::Running), SourceMonitorAudioRoute::RiotboxOnly) => {
                    "1 [Space] start; Riotbox-only monitor active"
                }
                (Some(AudioRuntimeLifecycle::Faulted), _) => {
                    "1 Audio output faulted; playback is not ready"
                }
                _ => "1 Audio output is not running; playback is not ready",
            };
            vec![
                transport_step.into(),
                "2 [C] confirm grid if Timing asks".into(),
                "3 [c] capture a keeper at the shown boundary".into(),
                "Then [o] audition raw -> [p] promote".into(),
            ]
        }
        Some(FirstRunOnrampStage::CapturePending) => vec![
            "Capture armed; wait for the boundary shown in Next".into(),
            "Do not press capture again while it is pending".into(),
            "After landing: [o] audition raw".into(),
            "Keep it with [p] promote".into(),
        ],
        Some(FirstRunOnrampStage::CaptureReadiness) => capture_readiness_onramp_lines(shell),
        Some(FirstRunOnrampStage::PromotionPending) => vec![
            "Promotion armed; wait for the boundary shown in Next".into(),
            "The raw capture remains the current audition target".into(),
            "After landing, check Monitor before Blend".into(),
            "Then [w] [f] [s] [y] perform".into(),
        ],
        Some(FirstRunOnrampStage::Monitor) => monitor_onramp_lines(shell),
        Some(FirstRunOnrampStage::Performance) => performance_onramp_lines(shell),
        None => Vec::new(),
    }
}

pub(super) fn first_run_onramp_compact_lines(shell: &JamShellState) -> Vec<String> {
    match first_run_onramp_stage(shell) {
        Some(FirstRunOnrampStage::Capture) if !audio_runtime_is_running(shell) => vec![
            "Audio output not running; playback is not ready".into(),
            "[C] grid if asked | [c] capture when ready".into(),
        ],
        Some(FirstRunOnrampStage::Capture) => vec![
            "[Space] start | [C] grid if asked".into(),
            "[c] capture -> [o] audition -> [p] promote".into(),
        ],
        Some(FirstRunOnrampStage::CapturePending) => vec![
            "[c] capture armed; wait for Next".into(),
            "Then [o] audition -> [p] promote".into(),
        ],
        Some(FirstRunOnrampStage::CaptureReadiness) if capture_handoff_is_ready(shell) => vec![
            "Capture ready: [o] audition raw".into(),
            "Keeper: [p] promote to W-30".into(),
        ],
        Some(FirstRunOnrampStage::CaptureReadiness) => vec![
            "Capture audio unavailable; [3] Source".into(),
            "Recapture only when source is ready".into(),
        ],
        Some(FirstRunOnrampStage::PromotionPending) => vec![
            "[p] promotion armed; wait for Next".into(),
            "Then check Monitor before Blend".into(),
        ],
        Some(FirstRunOnrampStage::Monitor) if !audio_runtime_is_running(shell) => vec![
            "Audio output not running; monitor is not playable".into(),
            "Fix output and restart Riotbox".into(),
        ],
        Some(FirstRunOnrampStage::Monitor) if source_monitor_blend_is_ready(shell) => vec![
            "Monitor ready: [M] choose Blend".into(),
            "Then [w] [f] [s] [y] perform".into(),
        ],
        Some(FirstRunOnrampStage::Monitor) => vec![
            "Blend unavailable; [3] inspect Source".into(),
            "[M] cycle to confirmed Riotbox-only".into(),
        ],
        Some(FirstRunOnrampStage::Performance)
            if shell.app.jam_view.scene.scene_jump_availability
                == SceneJumpAvailabilityView::WaitingForMoreScenes =>
        {
            vec![
                "Degraded: [w] hit | [f] fill | [s] slam".into(),
                "[y] waits for at least two scenes".into(),
            ]
        }
        Some(FirstRunOnrampStage::Performance) => vec![
            "Land all: [w] hit | [f] fill | [s] slam".into(),
            "[y] jump | [Y] restore".into(),
        ],
        None => Vec::new(),
    }
}

fn capture_readiness_onramp_lines(shell: &JamShellState) -> Vec<String> {
    if shell.app.jam_view.lanes.w30_pending_audition.is_some() {
        return vec![
            "Raw audition armed; wait for its shown boundary".into(),
            "Listen before deciding whether this is a keeper".into(),
            "If it works: [p] promote".into(),
            "If unavailable: [3] inspect Source and recapture".into(),
        ];
    }

    if capture_handoff_is_ready(shell) {
        vec![
            "Capture landed with a source-backed audio handoff".into(),
            "1 [o] audition the raw capture".into(),
            "2 [p] promote the keeper to the focused W-30 pad".into(),
            "Then check Monitor before choosing Blend".into(),
        ]
    } else {
        vec![
            "Capture landed, but its audio handoff is unavailable".into(),
            "Do not expect raw audition or source monitoring yet".into(),
            "[3] inspect Source readiness".into(),
            "[c] recapture only after source audio is ready".into(),
        ]
    }
}

fn performance_onramp_lines(shell: &JamShellState) -> Vec<String> {
    if shell.app.jam_view.scene.scene_jump_availability
        == SceneJumpAvailabilityView::WaitingForMoreScenes
    {
        return vec![
            format!(
                "Playable monitor, degraded scene set: {}",
                source_monitor_route_help_label(shell)
            ),
            "Play now: [w] hit | [f] fill | [s] slam".into(),
            "[y] unavailable: it needs at least two scenes".into(),
            "Start Here stays open until all four gestures land".into(),
        ];
    }

    vec![
        format!(
            "Playable monitor: {}",
            source_monitor_route_help_label(shell)
        ),
        "Land all four: [w] hit | [f] fill | [s] slam".into(),
        "[y] request next scene | [Y] restore prior scene".into(),
        "All four landed gestures complete Start Here; [?] shows keys".into(),
    ]
}

fn monitor_onramp_lines(shell: &JamShellState) -> Vec<String> {
    if shell
        .app
        .queue
        .has_pending_command(ActionCommand::SourceMonitorSetMode)
    {
        let readiness = if audio_runtime_is_running(shell) {
            "Blend is playable only when the source route is ready"
        } else {
            "Audio output is not running; no playable handoff yet"
        };
        return vec![
            "Monitor change armed; wait for the committed mode".into(),
            format!("Current route: {}", source_monitor_route_help_label(shell)),
            readiness.into(),
            "Then [w] [f] [s] [y] perform".into(),
        ];
    }

    if !audio_runtime_is_running(shell) {
        let state = match audio_runtime_lifecycle(shell) {
            Some(AudioRuntimeLifecycle::Faulted) => "faulted",
            Some(AudioRuntimeLifecycle::Idle) => "idle",
            Some(AudioRuntimeLifecycle::Stopped) => "stopped",
            Some(AudioRuntimeLifecycle::Running) => unreachable!("handled above"),
            None => "not started",
        };
        return vec![
            format!("Audio output {state}; playable monitor is not confirmed"),
            format!("Current route: {}", source_monitor_route_help_label(shell)),
            "Fix output and restart Riotbox before judging sound".into(),
            "Start Here will continue at the monitor handoff".into(),
        ];
    }

    if source_monitor_blend_is_ready(shell) {
        vec![
            "Keeper promoted to the focused W-30 pad".into(),
            format!("Monitor ready: {}", source_monitor_route_help_label(shell)),
            "[M] choose Blend: source + Riotbox".into(),
            "Then [w] [f] [s] [y] perform".into(),
        ]
    } else {
        vec![
            "Keeper promoted; the W-30 pad is ready".into(),
            "Blend unavailable: source monitor has no audio".into(),
            "[3] inspect Source; do not expect source in the mix".into(),
            "[M] cycle until the route confirms Riotbox only".into(),
        ]
    }
}

fn has_pending_capture(shell: &JamShellState) -> bool {
    shell.app.queue.pending_actions().iter().any(|action| {
        matches!(
            action.command,
            ActionCommand::CaptureNow
                | ActionCommand::CaptureLoop
                | ActionCommand::CaptureBarGroup
                | ActionCommand::W30CaptureToPad
        )
    })
}

fn has_pending_promotion(shell: &JamShellState) -> bool {
    shell.app.queue.pending_actions().iter().any(|action| {
        matches!(
            action.command,
            ActionCommand::PromoteCaptureToPad | ActionCommand::PromoteCaptureToScene
        )
    })
}

fn capture_handoff_is_ready(shell: &JamShellState) -> bool {
    matches!(
        shell.app.jam_view.capture.last_capture_handoff_readiness,
        Some(CaptureHandoffReadinessView::Source)
    )
}

fn audio_runtime_lifecycle(shell: &JamShellState) -> Option<AudioRuntimeLifecycle> {
    shell
        .app
        .runtime
        .audio
        .as_ref()
        .map(|health| health.lifecycle)
}

fn audio_runtime_is_running(shell: &JamShellState) -> bool {
    audio_runtime_lifecycle(shell) == Some(AudioRuntimeLifecycle::Running)
}

fn current_monitor_is_playable(shell: &JamShellState) -> bool {
    audio_runtime_is_running(shell)
        && matches!(
            (
                shell.app.session.runtime_state.source_monitor.mode,
                shell.app.runtime.source_monitor_audio_route,
            ),
            (SourceMonitorMode::Blend, SourceMonitorAudioRoute::Blend)
                | (
                    SourceMonitorMode::Riotbox,
                    SourceMonitorAudioRoute::RiotboxOnly
                )
        )
}

pub(super) fn source_monitor_blend_is_ready(shell: &JamShellState) -> bool {
    audio_runtime_is_running(shell)
        && matches!(
            shell.app.runtime.source_monitor_audio_route,
            SourceMonitorAudioRoute::SourceOnly | SourceMonitorAudioRoute::Blend
        )
}

pub(super) fn source_monitor_route_compact_label(shell: &JamShellState) -> &'static str {
    match shell.app.runtime.source_monitor_audio_route {
        SourceMonitorAudioRoute::SourceOnly => "src",
        SourceMonitorAudioRoute::Blend => "mix",
        SourceMonitorAudioRoute::RiotboxOnly => "riot",
        SourceMonitorAudioRoute::SourceUnavailable => "no-src",
    }
}

pub(super) fn source_monitor_route_help_label(shell: &JamShellState) -> &'static str {
    match shell.app.runtime.source_monitor_audio_route {
        SourceMonitorAudioRoute::SourceOnly => "source ready",
        SourceMonitorAudioRoute::Blend => "source + Riotbox",
        SourceMonitorAudioRoute::RiotboxOnly => "Riotbox only",
        SourceMonitorAudioRoute::SourceUnavailable => "source unavailable",
    }
}

pub(super) const fn source_monitor_mode_compact_label(mode: SourceMonitorMode) -> &'static str {
    match mode {
        SourceMonitorMode::Source => "src",
        SourceMonitorMode::Blend => "mix",
        SourceMonitorMode::Riotbox => "riot",
    }
}

pub(super) fn capture_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    vec![
        Line::from(format!("captures {}", capture.capture_count)),
        Line::from(format!(
            "last {}",
            capture.last_capture_id.as_deref().unwrap_or("none")
        )),
        Line::from(format!(
            "target {}",
            capture
                .last_capture_target
                .as_deref()
                .unwrap_or("unassigned")
        )),
        Line::from(format!("origins {}", capture.last_capture_origin_count)),
        Line::from(
            capture
                .last_capture_notes
                .clone()
                .unwrap_or_else(|| "no capture note yet".into()),
        ),
    ]
}

pub(super) fn capture_readiness_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let pending_capture_count = shell.app.jam_view.capture.pending_capture_count;
    let bank = shell
        .app
        .jam_view
        .lanes
        .w30_active_bank
        .as_deref()
        .unwrap_or("unset");

    vec![
        Line::from(format!(
            "transport {} | beat {:.1}",
            transport_label(shell),
            shell.app.jam_view.transport.position_beats
        )),
        Line::from(format!("target {}", capture_target_boundary_label(shell))),
        Line::from(format!("pending {pending_capture_count} | w30 bank {bank}")),
        Line::from(format!(
            "last lane capture {}",
            shell
                .app
                .jam_view
                .capture
                .last_capture_id
                .as_deref()
                .unwrap_or("none")
        )),
    ]
}

pub(super) fn capture_latest_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    vec![
        Line::from(format!("captures total {}", capture.capture_count)),
        Line::from(format!(
            "pinned {} | promoted {}",
            capture.pinned_capture_count, capture.promoted_capture_count
        )),
        Line::from(format!("hear {}", capture_heard_path_label(shell))),
        Line::from(format!(
            "latest {}",
            capture.last_capture_id.as_deref().unwrap_or("none")
        )),
        Line::from(format!(
            "target {}",
            capture
                .last_capture_target
                .as_deref()
                .unwrap_or("unassigned")
        )),
        Line::from(format!("origin refs {}", capture.last_capture_origin_count)),
        Line::from(
            capture
                .last_promotion_result
                .clone()
                .or_else(|| capture.last_capture_notes.clone())
                .unwrap_or_else(|| "no capture note yet".into()),
        ),
    ]
}

pub(super) fn capture_do_next_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    let handoff_readiness = capture_handoff_readiness_label(shell);
    if let Some(lines) = pending_capture_do_next_lines(capture, handoff_readiness) {
        return lines;
    }
    if let Some(lines) = pending_w30_audition_do_next_lines(shell) {
        return lines;
    }

    let Some(last_capture_id) = capture.last_capture_id.as_deref() else {
        return vec![
            Line::from(format!("1 [c] {}", capture_target_boundary_label(shell))),
            Line::from("2 [p] promote keeper"),
            Line::from("3 [w] hit promoted pad"),
            Line::from("use Log to confirm"),
        ];
    };

    match (
        capture.last_capture_target_kind,
        capture.last_capture_target.as_deref(),
    ) {
        (Some(CaptureTargetKindView::W30Pad), Some(target)) => {
            if handoff_readiness == "unavailable" {
                vec![
                    Line::from("unavailable: no W-30 audio"),
                    Line::from(format!("target {target}")),
                    Line::from("[3] Source shows why"),
                    Line::from("[c] recapture source-backed"),
                ]
            } else {
                vec![
                    Line::from(format!("hear now: [w] hit {target} ({handoff_readiness})")),
                    Line::from("or [o] audition same pad"),
                    Line::from("[b]/[s] browse or swap"),
                    Line::from(format!("source {last_capture_id}")),
                ]
            }
        }
        (Some(CaptureTargetKindView::Scene), Some(target)) => vec![
            Line::from(format!("scene target {target}")),
            Line::from("use Jam scene controls"),
            Line::from("[2] confirm action trail"),
            Line::from(format!("source {last_capture_id}")),
        ],
        _ => vec![
            Line::from(format!(
                "1 hear it: [o] raw {last_capture_id} ({handoff_readiness})"
            )),
            Line::from(format!("2 keep it: [p] promote {last_capture_id}")),
            Line::from(format!(
                "3 play it: [w] hit after promote ({handoff_readiness})"
            )),
            Line::from(capture_handoff_help_line(handoff_readiness)),
        ],
    }
}

fn capture_target_boundary_label(shell: &JamShellState) -> String {
    let length_intent = shell.app.session.runtime_state.capture.length_intent;
    let source_map = &shell.app.jam_view.source.source_map;
    let has_capture_range =
        source_map.capture_range_row.contains('[') || source_map.capture_range_row.contains('*');

    if has_capture_range {
        return match length_intent {
            CaptureLengthIntent::Phrase if shell.app.jam_view.source.timing.phrase_count == 0 => {
                "phrase->4bar @ next bar".into()
            }
            _ => format!("{length_intent} @ next bar"),
        };
    }

    if source_map.capture_hint.contains("listen first") {
        format!("{length_intent} @ listen first")
    } else {
        format!("{length_intent} @ unavailable")
    }
}

fn capture_handoff_readiness_label(shell: &JamShellState) -> &'static str {
    if matches!(
        shell.app.jam_view.capture.last_capture_target_kind,
        Some(CaptureTargetKindView::W30Pad)
    ) {
        match w30_preview_source_readiness(shell) {
            Some("source-backed") => return "src",
            Some("artifact-backed") => return "artifact",
            Some("unavailable") => return "unavailable",
            _ => {}
        }
    }

    match shell.app.jam_view.capture.last_capture_handoff_readiness {
        Some(CaptureHandoffReadinessView::Source) => "src",
        Some(CaptureHandoffReadinessView::Unavailable) | None => "unavailable",
    }
}

fn capture_handoff_help_line(handoff_readiness: &str) -> &'static str {
    if handoff_readiness == "unavailable" {
        "if unavailable: [3] Source"
    } else {
        "[2] confirm result"
    }
}

pub(super) fn capture_provenance_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lines = &shell.app.jam_view.capture.latest_capture_provenance_lines;
    if lines.is_empty() {
        return vec![Line::from("no captured material yet")];
    }

    lines.iter().cloned().map(Line::from).collect()
}

pub(super) fn pending_capture_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let pending = &shell.app.jam_view.capture.pending_capture_items;
    if pending.is_empty() {
        return vec![Line::from("no queued capture actions")];
    }

    let action = &pending[0];
    let mut lines = vec![
        Line::from(format!("next {} {}", action.actor, action.command)),
        Line::from(format!(
            "when {} | target {}",
            action.quantization, action.target
        )),
    ];
    if let Some(explanation) = &action.explanation {
        lines.push(Line::from(format!("note {explanation}")));
    }

    let overflow_count = pending.len().saturating_sub(1);
    if overflow_count > 0 {
        lines.push(Line::from(format!("+{overflow_count} more in [2] Log")));
    }

    lines
}

pub(super) fn recent_capture_items(shell: &JamShellState) -> Vec<ListItem<'static>> {
    let rows = &shell.app.jam_view.capture.recent_capture_rows;
    if rows.is_empty() {
        return vec![ListItem::new("no captures stored yet")];
    }

    rows.iter().cloned().map(ListItem::new).collect()
}
