#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FirstRunOnrampStage {
    Start,
    QueuedFirstMove,
    FirstResult,
}

fn first_run_onramp_stage(shell: &JamShellState) -> Option<FirstRunOnrampStage> {
    if !shell.first_run_onramp {
        return None;
    }

    let committed_count = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .filter(|action| action.status == riotbox_core::action::ActionStatus::Committed)
        .count();
    let has_pending = !shell.app.jam_view.pending_actions.is_empty();
    let capture_count = shell.app.jam_view.capture.capture_count;

    if capture_count > 0 || committed_count > 1 {
        return None;
    }

    if committed_count == 0 {
        return Some(if has_pending {
            FirstRunOnrampStage::QueuedFirstMove
        } else {
            FirstRunOnrampStage::Start
        });
    }

    Some(FirstRunOnrampStage::FirstResult)
}

fn capture_lines(shell: &JamShellState) -> Vec<Line<'static>> {
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

fn capture_readiness_lines(shell: &JamShellState) -> Vec<Line<'static>> {
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
        Line::from(format!("pending capture actions {pending_capture_count}")),
        Line::from(format!("w30 bank {bank}")),
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

fn capture_latest_lines(shell: &JamShellState) -> Vec<Line<'static>> {
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

fn capture_do_next_lines(shell: &JamShellState) -> Vec<Line<'static>> {
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
            Line::from("1 [c] capture phrase"),
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
            if handoff_readiness == "fallback" {
                vec![
                    Line::from(format!("fallback: [w]/[o] safe {target}")),
                    Line::from("[3] Source shows why"),
                    Line::from("[c] new capture can become src"),
                    Line::from(format!("source {last_capture_id}")),
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

fn capture_handoff_readiness_label(shell: &JamShellState) -> &'static str {
    match shell.app.jam_view.capture.last_capture_handoff_readiness {
        Some(CaptureHandoffReadinessView::Source) => "src",
        Some(CaptureHandoffReadinessView::Fallback) | None => "fallback",
    }
}

fn capture_handoff_help_line(handoff_readiness: &str) -> &'static str {
    if handoff_readiness == "fallback" {
        "if still fallback: [3] Source"
    } else {
        "[2] confirm result"
    }
}

fn pending_w30_audition_do_next_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    let pending = shell.app.jam_view.lanes.w30_pending_audition.as_ref()?;

    match pending.kind {
        W30PendingAuditionKind::RawCapture => Some(vec![
            capture_pending_intent_line(format!(
                "queued [o] audition raw @ {}",
                pending.quantization
            )),
            capture_pending_detail_line("wait, then hear raw preview"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm audition"),
        ]),
        W30PendingAuditionKind::Promoted => Some(vec![
            capture_pending_intent_line(format!(
                "queued [o] audition pad @ {}",
                pending.quantization
            )),
            capture_pending_detail_line("wait, then hear promoted preview"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm audition"),
        ]),
    }
}

fn pending_capture_do_next_lines(
    capture: &riotbox_core::view::jam::CaptureSummaryView,
    handoff_readiness: &'static str,
) -> Option<Vec<Line<'static>>> {
    let pending = capture.pending_capture_items.first()?;

    if matches!(
        pending.command.as_str(),
        "capture.now" | "capture.loop" | "capture.bar_group" | "w30.capture_to_pad"
    ) {
        return Some(vec![
            capture_pending_intent_line(format!("queued [c] capture @ {}", pending.quantization)),
            capture_pending_detail_line("wait for commit"),
            capture_pending_detail_line("then [o] audition raw or [p] promote"),
            capture_pending_detail_line("[2] confirm capture"),
        ]);
    }

    if pending.command == "promote.capture_to_pad" {
        return Some(vec![
            capture_pending_intent_line(format!("queued [p] promote @ {}", pending.quantization)),
            capture_pending_detail_line(format!(
                "wait, then hear with [w] hit ({handoff_readiness})"
            )),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm promotion"),
        ]);
    }

    if pending.command == "promote.capture_to_scene" {
        return Some(vec![
            capture_pending_intent_line(format!("queued scene promote @ {}", pending.quantization)),
            capture_pending_detail_line("wait for scene target"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm promotion"),
        ]);
    }

    if pending.command == "w30.loop_freeze" || pending.command == "promote.resample" {
        return Some(vec![
            capture_pending_intent_line(format!("queued W-30 reshape @ {}", pending.quantization)),
            capture_pending_detail_line("wait for phrase seam"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm result"),
        ]);
    }

    None
}

fn capture_pending_intent_line(message: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(message.into(), style_pending_cue()))
}

fn capture_pending_detail_line(message: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(message.into(), style_pending_detail()))
}

fn capture_provenance_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lines = &shell.app.jam_view.capture.latest_capture_provenance_lines;
    if lines.is_empty() {
        return vec![Line::from("no captured material yet")];
    }

    lines.iter().cloned().map(Line::from).collect()
}

fn pending_capture_lines(shell: &JamShellState) -> Vec<Line<'static>> {
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

fn recent_capture_items(shell: &JamShellState) -> Vec<ListItem<'static>> {
    let rows = &shell.app.jam_view.capture.recent_capture_rows;
    if rows.is_empty() {
        return vec![ListItem::new("no captures stored yet")];
    }

    rows.iter().cloned().map(ListItem::new).collect()
}

fn capture_routing_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let latest_promoted = shell
        .app
        .jam_view
        .capture
        .latest_w30_promoted_capture_label
        .as_deref()
        .unwrap_or("none");
    let pending_w30 = w30_pending_cue_label(shell);
    let bank_or_pool_line = if w30_slice_pool_relevant(shell) {
        format!(
            "bank/pad {} | pool {}",
            w30_target_compact(shell),
            w30_slice_pool_compact(shell)
        )
    } else {
        format!(
            "bank/pad {} | mgr {}",
            w30_target_compact(shell),
            w30_bank_manager_compact(shell)
        )
    };
    let mut lines = vec![
        Line::from(format!("pending W-30 cue {pending_w30}")),
        Line::from(bank_or_pool_line),
        Line::from({
            let mut line = format!(
                "preview {} | {}",
                shell.app.runtime_view.w30_preview_mode,
                shell.app.runtime_view.w30_preview_mix_summary,
            );
            if let Some(readiness) = w30_preview_source_readiness(shell) {
                line.push_str(" | ");
                line.push_str(readiness);
            }
            line
        }),
        Line::from(format!(
            "forge {} | tap {}",
            w30_damage_profile_compact(shell),
            w30_resample_tap_compact(shell),
        )),
    ];

    if w30_resample_lineage_active(shell) {
        lines.push(Line::from(format!(
            "tap {} | route {}",
            w30_resample_source_compact(shell),
            w30_resample_route_compact(shell),
        )));
        lines.push(Line::from(format!(
            "tap mix {}",
            w30_resample_mix_log_compact(shell)
        )));
        lines.push(Line::from(format!(
            "freeze {}",
            w30_loop_freeze_compact(shell)
        )));
        lines.push(Line::from(format!(
            "lineage {}",
            w30_capture_lineage_compact(shell)
        )));
    } else {
        let last_target = shell
            .app
            .jam_view
            .capture
            .last_capture_target
            .as_deref()
            .unwrap_or("unassigned");
        lines.push(Line::from(format!("route {last_target}")));
        lines.push(Line::from(
            shell
                .app
                .jam_view
                .capture
                .last_promotion_result
                .clone()
                .unwrap_or_else(|| "promotion result pending".into()),
        ));
        lines.push(Line::from(format!(
            "freeze {}",
            w30_loop_freeze_compact(shell)
        )));
        lines.push(Line::from(format!("latest promoted {latest_promoted}")));
        lines.push(Line::from(format!(
            "last lane capture {}",
            shell
                .app
                .session
                .runtime_state
                .lane_state
                .w30
                .last_capture
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "none".into())
        )));
        lines.push(Line::from(format!(
            "next shell cue {}",
            capture_or_recall_cue_label(shell)
        )));
        lines.push(Line::from(
            "audition and recall stay on the shared next-bar seam",
        ));
        return lines;
    }

    lines.push(Line::from(format!("latest promoted {latest_promoted}")));
    lines
}

fn capture_heard_path_label(shell: &JamShellState) -> String {
    let capture = &shell.app.jam_view.capture;
    let Some(last_capture_id) = capture.last_capture_id.as_deref() else {
        return "[c] first, then [p]->[w]".into();
    };

    match (
        capture.last_capture_target_kind,
        capture.last_capture_target.as_deref(),
    ) {
        (Some(CaptureTargetKindView::W30Pad), Some(target)) => {
            format!(
                "{last_capture_id}->{target} [w]/[o] {}",
                capture_handoff_readiness_label(shell)
            )
        }
        (Some(CaptureTargetKindView::Scene), Some(target)) => {
            format!("{last_capture_id}->{target} ready")
        }
        (_, Some(target)) if target != "unassigned" => format!("{last_capture_id}->{target} ready"),
        _ => {
            let readiness = capture_handoff_readiness_label(shell);
            if readiness == "fallback" {
                format!("{last_capture_id} fallback: [o] raw -> [p]->[w]")
            } else {
                format!("{last_capture_id} src: [o] raw -> [p]->[w]")
            }
        }
    }
}
