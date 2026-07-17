use riotbox_core::action::{Action, ActionCommand, ActionParams, ActionStatus, SourceMonitorMode};

use super::JamShellState;

pub(super) fn has_completed_first_run_onramp(shell: &JamShellState) -> bool {
    let actions = &shell.app.session.action_log.actions;
    actions.iter().enumerate().any(|(promotion_index, action)| {
        action_is_source_backed_promotion(shell, action)
            && actions
                .iter()
                .enumerate()
                .skip(promotion_index + 1)
                .filter(|(_, action)| action_is_monitor_handoff(action))
                .any(|(handoff_index, _)| {
                    has_all_first_run_gestures(monitor_epoch_after(actions, handoff_index))
                })
    })
}

fn monitor_epoch_after(actions: &[Action], handoff_index: usize) -> &[Action] {
    let epoch_tail = &actions[handoff_index + 1..];
    let epoch_end = epoch_tail
        .iter()
        .position(action_is_committed_monitor_mode_change)
        .unwrap_or(epoch_tail.len());
    &epoch_tail[..epoch_end]
}

pub(super) fn latest_source_backed_promotion_index(shell: &JamShellState) -> Option<usize> {
    let latest_capture = shell.app.session.captures.last()?;
    if latest_capture.source_window.is_none() || latest_capture.assigned_target.is_none() {
        return None;
    }

    shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .enumerate()
        .rfind(|(_, action)| {
            action.status == ActionStatus::Committed
                && matches!(
                    action.command,
                    ActionCommand::PromoteCaptureToPad | ActionCommand::PromoteCaptureToScene
                )
                && matches!(
                    &action.params,
                    ActionParams::Promotion {
                        capture_id: Some(capture_id),
                        ..
                    } if capture_id == &latest_capture.capture_id
                )
        })
        .map(|(index, _)| index)
}

pub(super) fn committed_monitor_handoff_index_after(
    shell: &JamShellState,
    promotion_index: usize,
) -> Option<usize> {
    shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .enumerate()
        .skip(promotion_index + 1)
        .rfind(|(_, action)| action_is_monitor_handoff(action))
        .map(|(index, _)| index)
}

fn action_is_source_backed_promotion(shell: &JamShellState, action: &Action) -> bool {
    if action.status != ActionStatus::Committed
        || !matches!(
            action.command,
            ActionCommand::PromoteCaptureToPad | ActionCommand::PromoteCaptureToScene
        )
    {
        return false;
    }

    let ActionParams::Promotion {
        capture_id: Some(capture_id),
        ..
    } = &action.params
    else {
        return false;
    };

    shell.app.session.captures.iter().any(|capture| {
        capture.capture_id == *capture_id
            && capture.source_window.is_some()
            && capture.assigned_target.is_some()
    })
}

fn action_is_monitor_handoff(action: &Action) -> bool {
    action.status == ActionStatus::Committed
        && action.command == ActionCommand::SourceMonitorSetMode
        && matches!(
            &action.params,
            ActionParams::SourceMonitor {
                mode: Some(SourceMonitorMode::Blend | SourceMonitorMode::Riotbox)
            }
        )
}

fn action_is_committed_monitor_mode_change(action: &Action) -> bool {
    action.status == ActionStatus::Committed
        && action.command == ActionCommand::SourceMonitorSetMode
}

fn has_all_first_run_gestures(actions: &[Action]) -> bool {
    let mut landed = [false; 4];
    for action in actions
        .iter()
        .filter(|action| action.status == ActionStatus::Committed)
    {
        match action.command {
            ActionCommand::W30TriggerPad => landed[0] = true,
            ActionCommand::Tr909FillNext => landed[1] = true,
            ActionCommand::Tr909SetSlam => landed[2] = true,
            ActionCommand::SceneLaunch => landed[3] = true,
            _ => {}
        }
    }
    landed.into_iter().all(|did_land| did_land)
}
