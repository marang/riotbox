use riotbox_core::{
    action::{Action, ActionCommand, ActionParams},
    session::SessionFile,
};

pub(in crate::jam_app) fn apply_preset_side_effects(
    session: &mut SessionFile,
    action: &Action,
) -> bool {
    if action.command != ActionCommand::PresetActivate {
        return false;
    }
    let ActionParams::Preset { preset_id } = action.params else {
        return false;
    };

    preset_id.apply_to_session(session);
    true
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        action::{
            Action, ActionCommand, ActionParams, ActionStatus, ActionTarget, ActorType,
            Quantization, UndoPolicy,
        },
        ids::ActionId,
        session::SessionFile,
        style::{PerformancePresetId, StyleProfileId},
    };

    use super::apply_preset_side_effects;

    #[test]
    fn preset_side_effect_applies_named_session_recipe() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-07-17T14:31:00Z");
        let action = Action {
            id: ActionId(1),
            actor: ActorType::User,
            command: ActionCommand::PresetActivate,
            params: ActionParams::Preset {
                preset_id: PerformancePresetId::FeralBreakAlphaV1,
            },
            target: ActionTarget::default(),
            requested_at: 100,
            quantization: Quantization::Immediate,
            status: ActionStatus::Committed,
            committed_at: Some(100),
            result: None,
            undo_policy: UndoPolicy::NotUndoable {
                reason: "test".into(),
            },
            explanation: None,
        };

        assert!(apply_preset_side_effects(&mut session, &action));
        assert_eq!(
            session.runtime_state.style.active_profile,
            Some(StyleProfileId::FeralRebuild)
        );
    }
}
