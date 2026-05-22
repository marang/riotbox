use riotbox_core::action::{Action, ActionCommand, ActionStatus};

use super::JamShellState;

pub(super) fn w30_pending_cue_label(shell: &JamShellState) -> String {
    if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_trigger_target
        .as_deref()
    {
        format!("trigger {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_focus_step_target
        .as_deref()
    {
        format!("step {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_audition
        .as_ref()
        .map(|pending| pending.target.as_str())
    {
        format!("audition {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_bank_swap_target
        .as_deref()
    {
        format!("bank {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_slice_pool_target
        .as_deref()
    {
        if shell
            .app
            .jam_view
            .lanes
            .w30_pending_slice_pool_reason
            .as_deref()
            == Some("feral")
        {
            let capture_id = shell
                .app
                .jam_view
                .lanes
                .w30_pending_slice_pool_capture_id
                .as_deref()
                .unwrap_or(target);
            format!("feral browse {capture_id}")
        } else {
            format!("browse {target}")
        }
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_damage_profile_target
        .as_deref()
    {
        format!("damage shred {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_loop_freeze_target
        .as_deref()
    {
        format!("freeze {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_recall_target
        .as_deref()
    {
        format!("recall {target}")
    } else if let Some(capture_id) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_resample_capture_id
        .as_deref()
    {
        format!("resample {capture_id}")
    } else {
        "idle".into()
    }
}

pub(super) fn last_committed_w30_action(shell: &JamShellState) -> Option<&Action> {
    shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| {
            action.status == ActionStatus::Committed
                && matches!(
                    action.command,
                    ActionCommand::W30TriggerPad
                        | ActionCommand::W30StepFocus
                        | ActionCommand::W30SwapBank
                        | ActionCommand::W30BrowseSlicePool
                        | ActionCommand::W30ApplyDamageProfile
                        | ActionCommand::W30LoopFreeze
                        | ActionCommand::W30LiveRecall
                        | ActionCommand::W30AuditionRawCapture
                        | ActionCommand::W30AuditionPromoted
                        | ActionCommand::PromoteResample
                )
        })
}

pub(super) fn short_w30_action_label(command: &ActionCommand) -> &'static str {
    match command {
        ActionCommand::W30TriggerPad => "trigger",
        ActionCommand::W30StepFocus => "step",
        ActionCommand::W30SwapBank => "bank",
        ActionCommand::W30BrowseSlicePool => "browse",
        ActionCommand::W30ApplyDamageProfile => "damage",
        ActionCommand::W30LoopFreeze => "freeze",
        ActionCommand::W30LiveRecall => "recall",
        ActionCommand::W30AuditionRawCapture => "audition raw",
        ActionCommand::W30AuditionPromoted => "audition",
        ActionCommand::PromoteResample => "resample",
        _ => "other",
    }
}
