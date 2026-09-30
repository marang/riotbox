use crate::ui::shell_state::JamShellState;
use riotbox_audio::w30::W30PreviewRenderMode;

pub(in crate::ui) fn w30_capture_compact(shell: &JamShellState) -> String {
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
}

pub(in crate::ui) fn w30_capture_log_compact(shell: &JamShellState) -> String {
    let capture_id = w30_capture_compact(shell);
    if let Some(source_window) = w30_latest_capture_source_window_compact(shell) {
        source_window
    } else {
        format!("cap {capture_id} | {}", w30_trigger_compact(shell))
    }
}

pub(in crate::ui) fn w30_latest_capture_source_window_compact(
    shell: &JamShellState,
) -> Option<String> {
    let capture_id = shell
        .app
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .as_ref()?;
    let capture = shell
        .app
        .session
        .captures
        .iter()
        .find(|capture| &capture.capture_id == capture_id)?;
    let source_window = capture.source_window.as_ref()?;

    Some(format_source_window_log_compact(source_window))
}

pub(in crate::ui) fn format_source_window_span(
    source_window: &riotbox_core::session::CaptureSourceWindow,
) -> String {
    format!(
        "{:.2}-{:.2}s",
        source_window.start_seconds, source_window.end_seconds
    )
}

pub(in crate::ui) fn format_source_window_log_compact(
    source_window: &riotbox_core::session::CaptureSourceWindow,
) -> String {
    format!(
        "win {} {}",
        format_source_window_span(source_window),
        source_window.source_id
    )
}

pub(in crate::ui) fn w30_trigger_compact(shell: &JamShellState) -> String {
    let render = &shell.app.runtime.w30_preview;
    if render.trigger_revision == 0 {
        if matches!(render.mode, W30PreviewRenderMode::Idle) {
            "unset".into()
        } else {
            "pending".into()
        }
    } else {
        format!(
            "r{}@{:.2}",
            render.trigger_revision, render.trigger_velocity
        )
    }
}

pub(in crate::ui) fn w30_action_target_compact(
    action: &riotbox_core::action::Action,
) -> Option<String> {
    action
        .target
        .bank_id
        .as_ref()
        .zip(action.target.pad_id.as_ref())
        .map(|(bank_id, pad_id)| format!("{bank_id}/{pad_id}"))
}

pub(in crate::ui) fn current_w30_lane_target(shell: &JamShellState) -> Option<(&str, &str)> {
    let lanes = &shell.app.jam_view.lanes;
    Some((
        lanes.w30_active_bank.as_deref()?,
        lanes.w30_focused_pad.as_deref()?,
    ))
}

pub(in crate::ui) fn latest_committed_w30_action_for_current_target(
    shell: &JamShellState,
    command: riotbox_core::action::ActionCommand,
) -> Option<&riotbox_core::action::Action> {
    let (active_bank, focused_pad) = current_w30_lane_target(shell)?;
    shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| {
            action.status == riotbox_core::action::ActionStatus::Committed
                && action.command == command
                && action
                    .target
                    .bank_id
                    .as_ref()
                    .map(|bank_id| bank_id.as_str())
                    == Some(active_bank)
                && action.target.pad_id.as_ref().map(|pad_id| pad_id.as_str()) == Some(focused_pad)
        })
}

pub(in crate::ui) fn w30_bank_manager_compact(shell: &JamShellState) -> String {
    if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_bank_swap_target
        .as_deref()
    {
        format!("next {target}")
    } else if let Some(target) = latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30SwapBank,
    )
    .and_then(w30_action_target_compact)
    {
        target
    } else {
        "idle".into()
    }
}

pub(in crate::ui) fn w30_bank_manager_status_compact(shell: &JamShellState) -> &'static str {
    if shell
        .app
        .jam_view
        .lanes
        .w30_pending_bank_swap_target
        .is_some()
    {
        "next-swap"
    } else if latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30SwapBank,
    )
    .is_some()
    {
        "swap"
    } else {
        "idle"
    }
}

pub(in crate::ui) fn w30_damage_profile_compact(shell: &JamShellState) -> String {
    if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_damage_profile_target
        .as_deref()
    {
        format!("next {target}")
    } else if let Some(target) = latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30ApplyDamageProfile,
    )
    .and_then(w30_action_target_compact)
    {
        target
    } else {
        "idle".into()
    }
}

pub(in crate::ui) fn w30_damage_profile_status_compact(shell: &JamShellState) -> &'static str {
    if shell
        .app
        .jam_view
        .lanes
        .w30_pending_damage_profile_target
        .is_some()
    {
        "next-shred"
    } else if latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30ApplyDamageProfile,
    )
    .is_some()
    {
        "shred"
    } else {
        "idle"
    }
}

pub(in crate::ui) fn w30_loop_freeze_compact(shell: &JamShellState) -> String {
    if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_loop_freeze_target
        .as_deref()
    {
        format!("next {target}")
    } else if let Some(target) = latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30LoopFreeze,
    )
    .and_then(w30_action_target_compact)
    {
        target
    } else {
        "idle".into()
    }
}

pub(in crate::ui) fn w30_loop_freeze_status_compact(shell: &JamShellState) -> &'static str {
    if shell
        .app
        .jam_view
        .lanes
        .w30_pending_loop_freeze_target
        .is_some()
    {
        "next-freeze"
    } else if latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30LoopFreeze,
    )
    .is_some()
    {
        "freeze"
    } else {
        "idle"
    }
}

pub(in crate::ui) fn w30_operation_status_compact(shell: &JamShellState) -> String {
    let operations = [
        w30_bank_manager_status_compact(shell),
        w30_damage_profile_status_compact(shell),
        w30_loop_freeze_status_compact(shell),
    ]
    .into_iter()
    .filter(|status| *status != "idle")
    .collect::<Vec<_>>();

    if operations.is_empty() {
        "idle".into()
    } else {
        operations.join("+")
    }
}
