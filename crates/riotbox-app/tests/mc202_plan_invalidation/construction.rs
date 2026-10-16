//! Unvalidated construction must not resurrect known invalidated metadata.
use riotbox_app::jam_app::JamAppState;
use riotbox_core::{queue::ActionQueue, session::ReplayPolicy};

use super::{assert_no_hydration, invalidated_history};

#[test]
fn constructor_preserves_matching_revert_invalidation_after_reconfirmation() {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("never-created-constructor-source.wav");
    let history = invalidated_history(&source, ReplayPolicy::DeterministicCommittedOrderV2);
    assert!(
        history
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
    for graph_available in [false, true] {
        let app = JamAppState::from_parts(
            history.session.clone(),
            graph_available.then(|| history.source_graph.clone().unwrap()),
            ActionQueue::new(),
        );
        assert_eq!(app.session.action_log, history.session.action_log);
        assert_no_hydration(&app);
        assert!(!source.exists());
        assert!(
            app.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan
                .is_none(),
            "constructor must not resurrect the invalidated plan because the Source was reconfirmed"
        );
    }
}

#[test]
fn constructor_keeps_each_prefix_and_committed_filter_without_changing_history() {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("never-created-constructor-prefix.wav");
    for policy in [
        ReplayPolicy::DeterministicCommittedOrderV2,
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
    ] {
        let history = invalidated_history(&source, policy);
        for cursor in 0..=4 {
            for graph_available in [false, true] {
                let mut input = history.session.clone();
                input.action_log.actions.truncate(cursor);
                input.action_log.commit_records.truncate(cursor);
                input.runtime_state.lane_state.mc202.source_phrase_plan = None;
                let log = input.action_log.clone();
                let app = JamAppState::from_parts(
                    input,
                    graph_available.then(|| history.source_graph.clone().unwrap()),
                    ActionQueue::new(),
                );
                assert_eq!(
                    app.session
                        .runtime_state
                        .lane_state
                        .mc202
                        .source_phrase_plan,
                    (cursor == 2).then(super::generated_plan)
                );
                assert_eq!(app.session.action_log, log);
                assert_no_hydration(&app);
            }
        }
        // Unvalidated construction must still ignore Undone phrase records.
        let mut input = history.session.clone();
        input.action_log.actions[1].status = riotbox_core::action::ActionStatus::Undone;
        let log = input.action_log.clone();
        let app = JamAppState::from_parts(input, history.source_graph, ActionQueue::new());
        assert!(
            app.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan
                .is_none()
        );
        assert_eq!(app.session.action_log, log);
    }
    assert!(!source.exists());
}

#[test]
fn unvalidated_phrase_metadata_remains_compatible_but_malformed_timing_fails_closed() {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("never-created-malformed-constructor.wav");
    let history = invalidated_history(&source, ReplayPolicy::DeterministicCommittedOrderV2);
    for malformed_timing in [false, true] {
        let mut session = history.session.clone();
        session.action_log.actions.truncate(2);
        session.action_log.commit_records.truncate(2);
        if malformed_timing {
            session.action_log.actions[0].params = riotbox_core::action::ActionParams::Empty;
        } else {
            session.action_log.actions[1].command =
                riotbox_core::action::ActionCommand::Mc202SetRole;
            session.action_log.actions[1].params = riotbox_core::action::ActionParams::Empty;
        }
        let before = session.clone();
        let app = JamAppState::from_parts(session, None, ActionQueue::new());
        assert_eq!(
            app.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan,
            (!malformed_timing).then(super::generated_plan)
        );
        assert_eq!(app.session.action_log, before.action_log);
        assert_eq!(
            app.session.runtime_state.source_timing,
            before.runtime_state.source_timing
        );
        assert_eq!(
            app.session.runtime_state.undo_state,
            before.runtime_state.undo_state
        );
        assert_no_hydration(&app);
    }
    assert!(!source.exists());
}
