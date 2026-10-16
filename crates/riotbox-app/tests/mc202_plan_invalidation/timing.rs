//! Generated metadata controls for effective, rather than arbitrary, reverts.
use super::{JamAppState, assert_no_hydration, generated_plan, invalidated_history};
use riotbox_core::{
    action::{ActionParams, ActionStatus},
    ids::SnapshotId,
    queue::ActionQueue,
    replay::apply_replay_target_suffix_to_session,
    session::{ReplayPolicy, SessionFile, Snapshot, SnapshotPayload},
};

fn anchor_at_two(session: &mut SessionFile) {
    let mut anchor = session.clone();
    anchor.runtime_state = SessionFile::new("generated-anchor", "test", "2026-10-16").runtime_state;
    apply_replay_target_suffix_to_session(&mut anchor, 2, None).unwrap();
    let id = SnapshotId::from("generated-before-revert");
    session.snapshots.push(Snapshot {
        snapshot_id: id.clone(),
        created_at: "2026-10-16".into(),
        label: "generated pre-revert anchor".into(),
        action_cursor: 2,
        payload: Some(SnapshotPayload::from_runtime_state(
            &id,
            2,
            &anchor.runtime_state,
        )),
    });
}

#[test]
fn nonmatching_source_or_hypothesis_revert_does_not_invalidate_the_plan() {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("never-created-mismatched-source.wav");
    for mismatch_source in [false, true] {
        let history = invalidated_history(&source, ReplayPolicy::DeterministicCommittedOrderV2);
        let mut session = history.session;
        let ActionParams::SourceTimingGrid {
            source_id,
            hypothesis_id,
            ..
        } = &mut session.action_log.actions[2].params
        else {
            panic!("revert params")
        };
        if mismatch_source {
            *source_id = Some("generated-other-source".into());
        } else {
            *hypothesis_id = Some("generated-other-hypothesis".into());
        }
        for graph_available in [false, true] {
            let graph = graph_available.then(|| history.source_graph.clone().unwrap());
            let constructed =
                JamAppState::from_parts(session.clone(), graph.clone(), ActionQueue::new());
            assert_eq!(
                constructed
                    .session
                    .runtime_state
                    .lane_state
                    .mc202
                    .source_phrase_plan,
                Some(generated_plan())
            );
            assert_no_hydration(&constructed);
            for cursor in 0..=4 {
                let mut input = session.clone();
                input.runtime_state =
                    SessionFile::new("generated-clean", "test", "2026-10-16").runtime_state;
                let mut app = JamAppState::from_parts(input, graph.clone(), ActionQueue::new());
                let before = app.session.action_log.clone();
                app.apply_restore_target_suffix(cursor).unwrap();
                assert_eq!(
                    app.session
                        .runtime_state
                        .lane_state
                        .mc202
                        .source_phrase_plan,
                    (cursor >= 2).then(generated_plan)
                );
                assert_eq!(app.session.action_log, before);
                assert_no_hydration(&app);
            }
            let mut anchored = session.clone();
            anchor_at_two(&mut anchored);
            let mut app = JamAppState::from_parts(anchored, graph, ActionQueue::new());
            app.apply_restore_target_from_snapshot_payload(4).unwrap();
            assert_eq!(
                app.session
                    .runtime_state
                    .lane_state
                    .mc202
                    .source_phrase_plan,
                Some(generated_plan())
            );
            assert_no_hydration(&app);
        }
    }
    assert!(!source.exists());
}

#[test]
fn rejected_queued_or_unaccepted_reverts_do_not_execute_in_constructor_or_restore() {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("never-created-filtering-source.wav");
    for (status, accepted) in [
        (ActionStatus::Rejected, false),
        (ActionStatus::Queued, true),
        (ActionStatus::Committed, false),
    ] {
        let history = invalidated_history(&source, ReplayPolicy::DeterministicCommittedOrderV2);
        let mut session = history.session;
        session.action_log.actions[2].status = status;
        session.action_log.actions[2]
            .result
            .as_mut()
            .unwrap()
            .accepted = accepted;
        // A noncommitted action has no committed record. Accepted=false remains
        // a committed rejected-result record and is filtered by Core as such.
        if status != ActionStatus::Committed {
            session.action_log.commit_records.remove(2);
        }
        let mut app = JamAppState::from_parts(session, history.source_graph, ActionQueue::new());
        assert_eq!(
            app.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan,
            Some(generated_plan())
        );
        let log = app.session.action_log.clone();
        app.apply_restore_target_suffix(4).unwrap();
        assert_eq!(
            app.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan,
            Some(generated_plan())
        );
        assert_eq!(app.session.action_log, log);
        assert_no_hydration(&app);
    }
    assert!(!source.exists());
}

#[test]
fn new_missing_phrase_after_reconfirmation_can_resolve_but_reconfirmation_alone_cannot() {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("never-created-new-phrase-source.wav");
    let mut live = invalidated_history(&source, ReplayPolicy::DeterministicCommittedOrderV2);
    assert!(
        live.session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
    live.queue_mc202_generate_answer(50).unwrap();
    let boundary = riotbox_core::transport::CommitBoundaryState {
        kind: riotbox_core::action::CommitBoundary::Phrase,
        beat_index: 0,
        bar_index: 1,
        phrase_index: 1,
        scene_id: None,
    };
    assert_eq!(live.commit_ready_actions(boundary, 51).unwrap().len(), 1);
    let expected = live
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .clone()
        .expect("the actual new live phrase owns an explicit selector decision");
    let mut input = live.session.clone();
    input
        .action_log
        .commit_records
        .last_mut()
        .unwrap()
        .mc202_source_phrase_plan = None;
    let mut anchor = input.clone();
    anchor.runtime_state =
        SessionFile::new("generated-new-anchor", "test", "2026-10-16").runtime_state;
    apply_replay_target_suffix_to_session(&mut anchor, 4, None).unwrap();
    assert!(
        anchor
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
    let id = SnapshotId::from("generated-after-reconfirmation");
    let mut anchored = input.clone();
    anchored.snapshots.push(Snapshot {
        snapshot_id: id.clone(),
        created_at: "2026-10-16".into(),
        label: "generated".into(),
        action_cursor: 4,
        payload: Some(SnapshotPayload::from_runtime_state(
            &id,
            4,
            &anchor.runtime_state,
        )),
    });
    for payload in [false, true] {
        let mut app = JamAppState::from_parts(
            if payload {
                anchored.clone()
            } else {
                input.clone()
            },
            live.source_graph.clone(),
            ActionQueue::new(),
        );
        if payload {
            app.apply_restore_target_from_snapshot_payload(5).unwrap();
        } else {
            app.apply_restore_target_suffix(5).unwrap();
        }
        assert_eq!(
            app.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan
                .as_ref(),
            Some(&expected)
        );
        assert_no_hydration(&app);
    }
    assert!(!source.exists());
}

#[test]
fn app_clears_a_persisted_plan_when_the_final_confirmed_source_is_different() {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("never-created-trust-source.wav");
    let history = invalidated_history(&source, ReplayPolicy::DeterministicCommittedOrderV2);
    let mut input = history.session;
    // The mismatch Revert is ineffective. Core legitimately retains the old
    // plan through a confirmation for another Source; App must still clear it.
    for index in [2, 3] {
        let ActionParams::SourceTimingGrid { source_id, .. } =
            &mut input.action_log.actions[index].params
        else {
            panic!("timing params")
        };
        *source_id = Some("generated-other-source".into());
    }
    let mut core = input.clone();
    apply_replay_target_suffix_to_session(&mut core, 4, None).unwrap();
    assert_eq!(
        core.runtime_state.lane_state.mc202.source_phrase_plan,
        Some(generated_plan())
    );
    let constructed = JamAppState::from_parts(core, None, ActionQueue::new());
    assert!(
        constructed
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
    anchor_at_two(&mut input);
    for payload in [false, true] {
        let mut app = JamAppState::from_parts(input.clone(), None, ActionQueue::new());
        if payload {
            app.apply_restore_target_from_snapshot_payload(4).unwrap();
        } else {
            app.apply_restore_target_suffix(4).unwrap();
        }
        assert!(
            app.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan
                .is_none()
        );
        assert_no_hydration(&app);
    }
    assert!(!source.exists());
}

#[test]
fn late_execution_failure_preserves_app_session_queue_transport_and_projections() {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("never-created-rollback-source.wav");
    let history = invalidated_history(&source, ReplayPolicy::DeterministicCommittedOrderV2);
    let mut input = history.session;
    input.action_log.actions.truncate(3);
    input.action_log.commit_records.truncate(3);
    input.action_log.commit_records[1].mc202_source_phrase_plan = None;
    input.action_log.actions[2].command = riotbox_core::action::ActionCommand::TransportSeek;
    input.action_log.actions[2].params = ActionParams::Empty;
    let mut anchor = input.clone();
    apply_replay_target_suffix_to_session(&mut anchor, 1, None).unwrap();
    let id = SnapshotId::from("generated-rollback-anchor");
    input.snapshots.push(Snapshot {
        snapshot_id: id.clone(),
        created_at: "2026-10-16".into(),
        label: "generated".into(),
        action_cursor: 1,
        payload: Some(SnapshotPayload::from_runtime_state(
            &id,
            1,
            &anchor.runtime_state,
        )),
    });
    for payload in [false, true] {
        let mut app = JamAppState::from_parts(
            input.clone(),
            history.source_graph.clone(),
            ActionQueue::new(),
        );
        let before = app.clone();
        if payload {
            assert!(app.apply_restore_target_from_snapshot_payload(3).is_err());
        } else {
            assert!(app.apply_restore_target_suffix(3).is_err());
        }
        assert_eq!(app.session, before.session);
        assert_eq!(app.queue.pending_actions(), before.queue.pending_actions());
        assert_eq!(app.queue.history(), before.queue.history());
        assert_eq!(
            app.queue.clone().allocate_action_id(),
            before.queue.clone().allocate_action_id()
        );
        assert_eq!(app.runtime, before.runtime);
        assert_eq!(app.jam_view, before.jam_view);
        assert_eq!(app.runtime_view, before.runtime_view);
        assert_no_hydration(&app);
    }
    assert!(!source.exists());
}
