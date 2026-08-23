use crate::{
    action::{ActionCommand, ActionParams, ActionStatus},
    ids::CaptureId,
    session::SessionFile,
};

/// Frozen active intensity for the first W-30 damage profile.
pub const W30_DAMAGE_PROFILE_ACTIVE_INTENSITY: f32 = 0.82;
/// Frozen fraction of a trigger step retained at full transient-bite intensity.
pub const W30_TRANSIENT_BITE_GATE_STEP_FRACTION: f32 = 0.44;

/// Returns the latest committed damage intensity for one exact W-30 capture.
///
/// Damage is action-log state rather than app-local state: an explicit zero
/// intensity bypasses the profile while preserving deterministic replay.
#[must_use]
pub fn latest_committed_w30_damage_intensity(
    session: &SessionFile,
    capture_id: &CaptureId,
) -> Option<f32> {
    session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| {
            action.status == ActionStatus::Committed
                && action.command == ActionCommand::W30ApplyDamageProfile
                && matches!(
                    &action.params,
                    ActionParams::Mutation {
                        target_id: Some(target_id),
                        ..
                    } if target_id == capture_id.as_str()
                )
        })
        .and_then(|action| match action.params {
            ActionParams::Mutation { intensity, .. } if intensity.is_finite() => {
                Some(intensity.clamp(0.0, 1.0))
            }
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use crate::{
        action::{
            Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget,
            ActorType, Quantization, UndoPolicy,
        },
        ids::{ActionId, CaptureId},
        session::SessionFile,
    };

    use super::latest_committed_w30_damage_intensity;

    fn committed_damage(action_id: u64, capture_id: &str, intensity: f32) -> Action {
        Action {
            id: ActionId(action_id),
            actor: ActorType::User,
            command: ActionCommand::W30ApplyDamageProfile,
            params: ActionParams::Mutation {
                intensity,
                target_id: Some(capture_id.to_owned()),
            },
            target: ActionTarget::default(),
            requested_at: action_id,
            quantization: Quantization::NextBar,
            status: ActionStatus::Committed,
            committed_at: Some(action_id),
            result: Some(ActionResult {
                accepted: true,
                summary: "damage profile state".into(),
            }),
            undo_policy: UndoPolicy::Undoable,
            explanation: None,
        }
    }

    #[test]
    fn damage_state_is_capture_scoped_and_latest_commit_wins() {
        let mut session = SessionFile::new("session", "test", "2026-08-23T00:00:00Z");
        session
            .action_log
            .actions
            .push(committed_damage(1, "cap-a", 0.82));
        session
            .action_log
            .actions
            .push(committed_damage(2, "cap-b", 0.41));
        session
            .action_log
            .actions
            .push(committed_damage(3, "cap-a", 0.0));

        assert_eq!(
            latest_committed_w30_damage_intensity(&session, &CaptureId::from("cap-a")),
            Some(0.0)
        );
        assert_eq!(
            latest_committed_w30_damage_intensity(&session, &CaptureId::from("cap-b")),
            Some(0.41)
        );
        assert_eq!(
            latest_committed_w30_damage_intensity(&session, &CaptureId::from("cap-c")),
            None
        );
    }
}
