use riotbox_core::{
    action::{Action, ActionCommand, ActionParams, ActionResult},
    session::{SessionFile, Tr909ReinforcementModeState, Tr909TakeoverProfileState},
    transport::CommitBoundaryState,
};

use crate::jam_app::helpers::update_logged_action_result;

pub(in crate::jam_app) fn apply_tr909_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
) {
    match action.command {
        ActionCommand::Tr909SetSlam => {
            let intensity = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => session.runtime_state.macro_state.tr909_slam,
            };
            session.runtime_state.macro_state.tr909_slam = intensity;
            session.runtime_state.lane_state.tr909.slam_enabled = intensity > 0.0;

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                let summary = if intensity > 0.0 {
                    format!("enabled TR-909 slam at {:.2}", intensity)
                } else {
                    "disabled TR-909 slam".into()
                };
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary,
                });
            }
        }
        ActionCommand::Tr909FillNext => {
            session.runtime_state.lane_state.tr909.fill_armed_next_bar = false;
            session.runtime_state.lane_state.tr909.last_fill_bar =
                boundary.map(|boundary| boundary.bar_index);
            session.runtime_state.lane_state.tr909.pattern_ref =
                boundary.map(|boundary| format!("fill-bar-{}", boundary.bar_index));
            session.runtime_state.lane_state.tr909.reinforcement_mode =
                Some(Tr909ReinforcementModeState::Fills);
            update_logged_action_result(
                session,
                action.id,
                tr909_result_summary("triggered TR-909 fill", boundary),
            );
        }
        ActionCommand::Tr909ReinforceBreak => {
            session.runtime_state.lane_state.tr909.reinforcement_mode =
                Some(Tr909ReinforcementModeState::BreakReinforce);
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("reinforce-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("reinforce-{scene_id}"),
                )
            });
            update_logged_action_result(
                session,
                action.id,
                tr909_result_summary("reinforced TR-909 break", boundary),
            );
        }
        ActionCommand::Tr909Takeover => {
            session.runtime_state.lane_state.tr909.takeover_enabled = true;
            session.runtime_state.lane_state.tr909.takeover_profile =
                Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("takeover-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("takeover-{scene_id}"),
                )
            });
            session.runtime_state.lane_state.tr909.reinforcement_mode =
                Some(Tr909ReinforcementModeState::Takeover);
            update_logged_action_result(
                session,
                action.id,
                tr909_result_summary("engaged TR-909 takeover", boundary),
            );
        }
        ActionCommand::Tr909SceneLock => {
            session.runtime_state.lane_state.tr909.takeover_enabled = true;
            session.runtime_state.lane_state.tr909.takeover_profile =
                Some(Tr909TakeoverProfileState::SceneLockTakeover);
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("lock-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("lock-{scene_id}"),
                )
            });
            session.runtime_state.lane_state.tr909.reinforcement_mode =
                Some(Tr909ReinforcementModeState::Takeover);
            update_logged_action_result(
                session,
                action.id,
                tr909_result_summary("engaged TR-909 scene lock", boundary),
            );
        }
        ActionCommand::Tr909Release => {
            session.runtime_state.lane_state.tr909.takeover_enabled = false;
            session.runtime_state.lane_state.tr909.takeover_profile = None;
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("release-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("release-{scene_id}"),
                )
            });
            if session.runtime_state.lane_state.tr909.reinforcement_mode
                == Some(Tr909ReinforcementModeState::Takeover)
            {
                session.runtime_state.lane_state.tr909.reinforcement_mode =
                    Some(Tr909ReinforcementModeState::SourceSupport);
            }
            update_logged_action_result(
                session,
                action.id,
                tr909_result_summary("released TR-909 takeover", boundary),
            );
        }
        _ => {}
    }
}

fn tr909_result_summary(gesture: &str, boundary: Option<&CommitBoundaryState>) -> String {
    boundary.map_or_else(
        || gesture.to_string(),
        |boundary| {
            format!(
                "{gesture} at beat {}, bar {}, phrase {}",
                boundary.beat_index, boundary.bar_index, boundary.phrase_index
            )
        },
    )
}
