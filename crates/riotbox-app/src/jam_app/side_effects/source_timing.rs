use riotbox_core::{
    action::{Action, ActionCommand, ActionParams},
    session::{SessionFile, SourceTimingGridConfirmationState},
};

pub(in crate::jam_app) fn apply_source_timing_side_effects(
    session: &mut SessionFile,
    action: &Action,
) -> bool {
    match action.command {
        ActionCommand::SourceTimingConfirmGrid => {
            let ActionParams::SourceTimingGrid {
                source_id: Some(source_id),
                hypothesis_id,
            } = &action.params
            else {
                return false;
            };

            session.runtime_state.source_timing.confirmed_grid =
                Some(SourceTimingGridConfirmationState {
                    source_id: source_id.clone(),
                    hypothesis_id: hypothesis_id.clone(),
                    confirmed_by_action: action.id,
                    confirmed_at: action.committed_at.unwrap_or(action.requested_at),
                });
            true
        }
        ActionCommand::SourceTimingRevertGrid => {
            let ActionParams::SourceTimingGrid {
                source_id: Some(source_id),
                hypothesis_id,
            } = &action.params
            else {
                return false;
            };
            if session
                .runtime_state
                .source_timing
                .confirmed_grid
                .as_ref()
                .is_some_and(|confirmed| {
                    confirmed.source_id == *source_id
                        && confirmed.hypothesis_id.as_deref() == hypothesis_id.as_deref()
                })
            {
                session.runtime_state.source_timing.confirmed_grid = None;
                if session
                    .runtime_state
                    .lane_state
                    .mc202
                    .source_phrase_plan
                    .as_ref()
                    .is_some_and(|plan| plan.source_id == *source_id)
                {
                    session.runtime_state.lane_state.mc202.source_phrase_plan = None;
                }
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        TimestampMs,
        action::{
            Action, ActionCommand, ActionParams, ActionStatus, ActionTarget, ActorType,
            Quantization, UndoPolicy,
        },
        ids::{ActionId, SourceId},
        session::{
            Mc202RoleState, Mc202SourcePhraseCandidateFamilyState,
            Mc202SourcePhraseNoteBudgetState, Mc202SourcePhrasePlanState,
            Mc202SourcePhraseSlotState, SessionFile,
        },
    };

    use super::apply_source_timing_side_effects;

    #[test]
    fn source_timing_side_effect_confirms_grid_in_session_runtime_state() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T12:20:00Z");
        let action = confirm_grid_action("src-1", Some("primary-grid"), 100);

        assert!(apply_source_timing_side_effects(&mut session, &action));

        let confirmed = session
            .runtime_state
            .source_timing
            .confirmed_grid
            .expect("grid confirmation");
        assert_eq!(confirmed.source_id, SourceId::from("src-1"));
        assert_eq!(confirmed.hypothesis_id.as_deref(), Some("primary-grid"));
        assert_eq!(confirmed.confirmed_by_action, ActionId(1));
        assert_eq!(confirmed.confirmed_at, 100);
    }

    #[test]
    fn source_timing_side_effect_rejects_missing_source_id() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T12:21:00Z");
        let mut action = confirm_grid_action("src-1", Some("primary-grid"), 100);
        action.params = ActionParams::SourceTimingGrid {
            source_id: None,
            hypothesis_id: Some("primary-grid".into()),
        };

        assert!(!apply_source_timing_side_effects(&mut session, &action));
        assert!(session.runtime_state.source_timing.confirmed_grid.is_none());
    }

    #[test]
    fn source_timing_side_effect_reverts_matching_grid_in_session_runtime_state() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T12:22:00Z");
        let confirm = confirm_grid_action("src-1", Some("primary-grid"), 100);
        assert!(apply_source_timing_side_effects(&mut session, &confirm));

        let revert = revert_grid_action("src-1", Some("primary-grid"), 120);

        assert!(apply_source_timing_side_effects(&mut session, &revert));
        assert!(session.runtime_state.source_timing.confirmed_grid.is_none());
    }

    #[test]
    fn source_timing_side_effect_revert_clears_matching_mc202_source_phrase_plan() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T12:22:30Z");
        let confirm = confirm_grid_action("src-1", Some("primary-grid"), 100);
        assert!(apply_source_timing_side_effects(&mut session, &confirm));
        session.runtime_state.lane_state.mc202.source_phrase_plan =
            Some(source_phrase_plan("src-1"));

        let revert = revert_grid_action("src-1", Some("primary-grid"), 120);

        assert!(apply_source_timing_side_effects(&mut session, &revert));
        assert!(session.runtime_state.source_timing.confirmed_grid.is_none());
        assert!(
            session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan
                .is_none()
        );
    }

    #[test]
    fn source_timing_side_effect_preserves_unmatched_confirmation_on_revert() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T12:23:00Z");
        let confirm = confirm_grid_action("src-1", Some("primary-grid"), 100);
        assert!(apply_source_timing_side_effects(&mut session, &confirm));
        session.runtime_state.lane_state.mc202.source_phrase_plan =
            Some(source_phrase_plan("src-1"));

        let revert = revert_grid_action("src-1", Some("alternate-grid"), 120);

        assert!(!apply_source_timing_side_effects(&mut session, &revert));
        assert!(session.runtime_state.source_timing.confirmed_grid.is_some());
        assert!(
            session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan
                .is_some()
        );
    }

    fn confirm_grid_action(
        source_id: &str,
        hypothesis_id: Option<&str>,
        requested_at: TimestampMs,
    ) -> Action {
        Action {
            id: ActionId(1),
            actor: ActorType::User,
            command: ActionCommand::SourceTimingConfirmGrid,
            params: ActionParams::SourceTimingGrid {
                source_id: Some(SourceId::from(source_id)),
                hypothesis_id: hypothesis_id.map(Into::into),
            },
            target: ActionTarget::default(),
            requested_at,
            quantization: Quantization::Immediate,
            status: ActionStatus::Committed,
            committed_at: Some(requested_at),
            result: None,
            undo_policy: UndoPolicy::Undoable,
            explanation: None,
        }
    }

    fn revert_grid_action(
        source_id: &str,
        hypothesis_id: Option<&str>,
        requested_at: TimestampMs,
    ) -> Action {
        let mut action = confirm_grid_action(source_id, hypothesis_id, requested_at);
        action.id = ActionId(2);
        action.command = ActionCommand::SourceTimingRevertGrid;
        action
    }

    fn source_phrase_plan(source_id: &str) -> Mc202SourcePhrasePlanState {
        Mc202SourcePhrasePlanState {
            source_id: SourceId::from(source_id),
            phrase_slot: Mc202SourcePhraseSlotState {
                phrase_index: 1,
                start_bar: 0,
                end_bar: 7,
            },
            role: Mc202RoleState::Answer,
            rhythm_cells: [
                None,
                Some(0),
                None,
                Some(5),
                None,
                Some(7),
                None,
                None,
                None,
                Some(3),
                None,
                Some(7),
                None,
                None,
                None,
                None,
            ],
            note_budget: Mc202SourcePhraseNoteBudgetState::Sparse,
            touch: 0.82,
            confidence: 0.86,
            candidate_family: Some(Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer),
            candidate_count: 3,
            rejected_candidate_count: 1,
            candidate_provenance_refs: vec!["candidate_family:sparse_offbeat_answer".into()],
            fallback_reason: None,
        }
    }
}
