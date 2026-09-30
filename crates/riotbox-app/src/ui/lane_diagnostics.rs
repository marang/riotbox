use crate::ui::shell_state::JamShellState;
use crate::ui::w30_cue_labels::last_committed_w30_action;
use crate::ui::w30_cue_labels::short_w30_action_label;
use crate::ui::w30_cue_labels::w30_pending_cue_label;
use crate::ui::w30_operations::w30_capture_log_compact;
use crate::ui::w30_operations::w30_operation_status_compact;
use crate::ui::w30_preview_labels::w30_preview_log_compact;
use crate::ui::w30_resample_labels::w30_resample_lineage_active;
use crate::ui::w30_resample_labels::w30_resample_log_focus_compact;
use crate::ui::w30_resample_labels::w30_resample_mix_log_compact;
use crate::ui::w30_slice_pool::w30_slice_pool_log_compact;
use crate::ui::w30_slice_pool::w30_slice_pool_relevant;
use ratatui::text::Line;

pub(in crate::ui) fn tr909_log_header_line(shell: &JamShellState) -> String {
    let scene = shell
        .app
        .runtime
        .transport
        .current_scene
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "none".into());

    if shell.app.runtime_view.tr909_render_support_reason == "feral break lift" {
        return format!("feral break lift | scene {scene}");
    }

    let transport = if shell.app.runtime.transport.is_playing {
        format!(
            "running @ {:.1}",
            shell.app.runtime.transport.position_beats
        )
    } else {
        format!(
            "stopped @ {:.1}",
            shell.app.runtime.transport.position_beats
        )
    };
    let boundary = shell
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
        .unwrap_or_else(|| "boundary none".into());

    format!("{transport} | scene {scene} | {boundary}")
}

pub(in crate::ui) fn tr909_compact_reason_line(shell: &JamShellState) -> Option<String> {
    match shell.app.runtime_view.tr909_render_support_reason.as_str() {
        "feral break lift" => Some("reason feral break lift".into()),
        _ => None,
    }
}

pub(in crate::ui) fn mc202_pending_role_label(shell: &JamShellState) -> &'static str {
    if shell.app.jam_view.lanes.mc202_pending_role.is_some() {
        "voice queued"
    } else if shell.app.jam_view.lanes.mc202_pending_answer_generation {
        "answer queued"
    } else if shell.app.jam_view.lanes.mc202_pending_pressure_generation {
        "pressure queued"
    } else if shell.app.jam_view.lanes.mc202_pending_instigator_generation {
        "instigate queued"
    } else if shell.app.jam_view.lanes.mc202_pending_follower_generation {
        "follow queued"
    } else {
        "stable"
    }
}

pub(in crate::ui) fn mc202_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lanes = &shell.app.jam_view.lanes;
    let last_mc202_action = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| {
            matches!(
                action.command,
                riotbox_core::action::ActionCommand::Mc202SetRole
                    | riotbox_core::action::ActionCommand::Mc202GenerateFollower
                    | riotbox_core::action::ActionCommand::Mc202GenerateAnswer
                    | riotbox_core::action::ActionCommand::Mc202GeneratePressure
                    | riotbox_core::action::ActionCommand::Mc202GenerateInstigator
                    | riotbox_core::action::ActionCommand::Mc202MutatePhrase
            )
        })
        .map(|action| action.command.to_string())
        .unwrap_or_else(|| "none".into());

    vec![
        Line::from(format!(
            "role {} | next {}",
            lanes.mc202_role.as_deref().unwrap_or("unset"),
            lanes.mc202_pending_role.as_deref().unwrap_or("none")
        )),
        Line::from(format!(
            "phrase {} | variant {} | gen {}",
            lanes.mc202_phrase_ref.as_deref().unwrap_or("unset"),
            lanes.mc202_phrase_variant.as_deref().unwrap_or("base"),
            if lanes.mc202_pending_answer_generation {
                "queued answer"
            } else if lanes.mc202_pending_pressure_generation {
                "queued pressure"
            } else if lanes.mc202_pending_instigator_generation {
                "queued instigate"
            } else if lanes.mc202_pending_follower_generation {
                "queued"
            } else if lanes.mc202_pending_phrase_mutation {
                "queued mutation"
            } else {
                "idle"
            }
        )),
        Line::from(format!(
            "touch {:.2} | last {}",
            shell.app.jam_view.macros.mc202_touch, last_mc202_action
        )),
        Line::from(format!(
            "render {} | {}",
            shell.app.runtime_view.mc202_render_routing,
            shell.app.runtime_view.mc202_render_mix_summary
        )),
        Line::from(format!("diagnostic {}", mc202_pending_role_label(shell))),
    ]
}

pub(in crate::ui) fn w30_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lanes = &shell.app.jam_view.lanes;
    let recent = last_committed_w30_action(shell);
    let recent_label = recent
        .map(|action| short_w30_action_label(&action.command))
        .unwrap_or("none");
    let lineage_active = w30_resample_lineage_active(shell);
    let slice_pool_relevant = w30_slice_pool_relevant(shell);

    vec![
        Line::from(format!(
            "bank {}/{}",
            lanes.w30_active_bank.as_deref().unwrap_or("unset"),
            lanes.w30_focused_pad.as_deref().unwrap_or("unset")
        )),
        Line::from(format!(
            "cue {} | {recent_label}",
            w30_pending_cue_label(shell)
        )),
        Line::from(format!("prev {}", w30_preview_log_compact(shell))),
        Line::from(if lineage_active {
            format!("tapmix {}", w30_resample_mix_log_compact(shell))
        } else {
            format!(
                "mix {} {}",
                w30_mix_log_compact(shell),
                w30_operation_status_compact(shell),
            )
        }),
        if lineage_active {
            Line::from(w30_resample_log_focus_compact(shell))
        } else {
            if slice_pool_relevant {
                Line::from(format!("pool {}", w30_slice_pool_log_compact(shell)))
            } else {
                Line::from(w30_capture_log_compact(shell))
            }
        },
    ]
}

pub(in crate::ui) fn w30_target_compact(shell: &JamShellState) -> String {
    format!(
        "{}/{}",
        shell
            .app
            .jam_view
            .lanes
            .w30_active_bank
            .as_deref()
            .unwrap_or("unset"),
        shell
            .app
            .jam_view
            .lanes
            .w30_focused_pad
            .as_deref()
            .unwrap_or("unset")
    )
}

pub(in crate::ui) fn w30_mix_log_compact(shell: &JamShellState) -> String {
    format!(
        "{:.2}/{:.2}",
        shell.app.runtime.w30_preview.music_bus_level, shell.app.runtime.w30_preview.grit_level
    )
}
