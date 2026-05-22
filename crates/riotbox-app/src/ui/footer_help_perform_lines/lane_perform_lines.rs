use ratatui::text::Line;
use riotbox_audio::w30::W30PreviewRenderMode;

use super::{
    JamShellState, w30_operation_status_compact, w30_pending_cue_label,
    w30_preview_mode_profile_compact, w30_target_compact,
};

pub(super) fn mc202_perform_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lanes = &shell.app.jam_view.lanes;
    let next = if let Some(role) = lanes.mc202_pending_role.as_deref() {
        format!("next voice {role}")
    } else if lanes.mc202_pending_answer_generation {
        "next answer".into()
    } else if lanes.mc202_pending_pressure_generation {
        "next pressure".into()
    } else if lanes.mc202_pending_instigator_generation {
        "next instigate".into()
    } else if lanes.mc202_pending_follower_generation {
        "next follow".into()
    } else if lanes.mc202_pending_phrase_mutation {
        "next phrase mutation".into()
    } else {
        "next none".into()
    };

    vec![
        Line::from(format!(
            "current voice {}",
            lanes.mc202_role.as_deref().unwrap_or("unset")
        )),
        Line::from(next),
        Line::from(format!(
            "current phrase {}",
            lanes.mc202_phrase_ref.as_deref().unwrap_or("unset")
        )),
        Line::from(format!(
            "variant {}",
            lanes.mc202_phrase_variant.as_deref().unwrap_or("base")
        )),
        Line::from(format!(
            "sound {} / {} | touch {:.2}",
            shell.app.runtime_view.mc202_render_mode,
            shell.app.runtime_view.mc202_render_phrase_shape,
            shell.app.runtime.mc202_render.touch
        )),
    ]
}

pub(super) fn w30_perform_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let next = if w30_pending_cue_label(shell) != "idle" {
        format!("next {}", w30_pending_cue_label(shell))
    } else {
        format!("next {}", w30_operation_status_compact(shell))
    };

    let mut lines = vec![
        Line::from(format!("current pad {}", w30_target_compact(shell))),
        Line::from(format!(
            "current preview {}",
            w30_preview_mode_profile_compact(shell)
        )),
        Line::from(next),
    ];
    if let Some(action_cue) = w30_perform_action_cue(shell) {
        lines.push(Line::from(action_cue));
    }
    lines
}

fn w30_perform_action_cue(shell: &JamShellState) -> Option<&'static str> {
    let render = &shell.app.runtime.w30_preview;
    if matches!(render.mode, W30PreviewRenderMode::Idle) {
        return None;
    }

    if render.source_window_preview.is_some() {
        match render.mode {
            W30PreviewRenderMode::RawCaptureAudition => Some("src: [o] raw source | 4 Capture"),
            W30PreviewRenderMode::PromotedAudition => Some("src: [o] source | 4 Capture"),
            W30PreviewRenderMode::LiveRecall => Some("src: [w] source | 4 Capture"),
            W30PreviewRenderMode::Idle => None,
        }
    } else {
        match render.mode {
            W30PreviewRenderMode::RawCaptureAudition => Some("fallback: [o] raw safe | 4 Capture"),
            W30PreviewRenderMode::PromotedAudition => Some("fallback: [o] safe | 4 Capture"),
            W30PreviewRenderMode::LiveRecall => Some("fallback: [w] safe | 4 Capture"),
            W30PreviewRenderMode::Idle => None,
        }
    }
}

pub(super) fn tr909_perform_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let next = tr909_next_line(shell);

    vec![
        Line::from(format!(
            "current mode {}",
            if shell.app.jam_view.lanes.tr909_takeover_enabled {
                "takeover"
            } else {
                "support"
            }
        )),
        Line::from(format!(
            "current fill {} | slam {:.2}",
            if shell.app.jam_view.lanes.tr909_fill_armed_next_bar {
                "armed"
            } else {
                "idle"
            },
            shell.app.jam_view.macros.tr909_slam
        )),
        Line::from(format!("next {next}")),
    ]
}

fn tr909_next_line(shell: &JamShellState) -> String {
    use riotbox_core::action::ActionCommand::{
        Tr909FillNext, Tr909ReinforceBreak, Tr909Release, Tr909SceneLock, Tr909SetSlam,
        Tr909Takeover,
    };

    shell
        .app
        .queue
        .pending_actions()
        .iter()
        .find_map(|action| match action.command {
            Tr909FillNext => Some("fill".into()),
            Tr909ReinforceBreak => Some("push".into()),
            Tr909SetSlam => Some("slam".into()),
            Tr909Takeover => Some("takeover".into()),
            Tr909SceneLock => Some("lock".into()),
            Tr909Release => Some("release".into()),
            _ => None,
        })
        .unwrap_or_else(|| {
            if shell.app.jam_view.lanes.tr909_fill_armed_next_bar {
                "fill armed".into()
            } else {
                "none".into()
            }
        })
}

pub(super) fn tr909_inspect_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let render = &shell.app.runtime_view;
    let last_boundary = shell
        .app
        .runtime
        .last_commit_boundary
        .as_ref()
        .map(|boundary| {
            format!(
                "{:?} b{} p{}",
                boundary.kind, boundary.bar_index, boundary.phrase_index
            )
        })
        .unwrap_or_else(|| "none".into());

    vec![
        Line::from(format!(
            "mode {} | next {}",
            render.tr909_render_mode,
            tr909_next_line(shell)
        )),
        Line::from(format!(
            "profile {} | context {} | accent {} | reason {} | route {}",
            render.tr909_render_profile,
            render.tr909_render_support_context,
            render.tr909_render_support_accent,
            render.tr909_render_support_reason,
            render.tr909_render_routing
        )),
        Line::from(format!(
            "{} | {}",
            render.tr909_render_pattern_adoption, render.tr909_render_phrase_variation
        )),
        Line::from(render.tr909_render_mix_summary.clone()),
        Line::from(format!(
            "{} | boundary {last_boundary}",
            render.tr909_render_alignment
        )),
    ]
}
