//! Live public commits are the oracle for generated legacy touch restoration.
use riotbox_core::{
    action::{ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization},
    ids::{ActionId, SnapshotId},
    queue::ActionQueue,
    replay::apply_replay_target_suffix_to_session,
    session::{SessionFile, Snapshot, SnapshotPayload},
};

use super::{JamAppState, assert_no_hydration, commit, confirmed_app_with_persisted_plan};

#[test]
fn target_and_payload_legacy_restores_match_live_set_role_touch_reduction() {
    let owner = tempfile::tempdir().unwrap();
    let source = owner.path().join("never-created-touch-source.wav");
    let seed = confirmed_app_with_persisted_plan(&source);
    let graph = seed.source_graph.unwrap();
    let mut session = seed.session;
    session.action_log.actions.truncate(1);
    session.action_log.commit_records.truncate(1);
    session.runtime_state = SessionFile::new("generated-touch", "test", "2026-10-16").runtime_state;
    apply_replay_target_suffix_to_session(&mut session, 1, Some(&graph)).unwrap();
    let anchor_runtime = session.runtime_state.clone();
    assert!(anchor_runtime.source_timing.confirmed_grid.is_some());
    assert!(anchor_runtime.lane_state.mc202.source_phrase_plan.is_none());
    let mut live = JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());
    for (command, touch, timestamp) in [
        (ActionCommand::Mc202GenerateFollower, 0.9, 100),
        (ActionCommand::Mc202SetRole, 0.2, 200),
        (ActionCommand::Mc202GenerateAnswer, 0.1, 300),
    ] {
        let mut draft = ActionDraft::new(
            ActorType::User,
            command,
            Quantization::Immediate,
            ActionTarget::default(),
        );
        draft.params = ActionParams::Mutation {
            intensity: touch,
            target_id: (command == ActionCommand::Mc202SetRole).then(|| "follower".into()),
        };
        live.queue.enqueue(draft, timestamp).unwrap();
        commit(&mut live, timestamp + 1);
    }
    assert_no_hydration(&live);
    let expected = (
        live.session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .as_ref()
            .expect("actual accepted live writer must produce an explicit plan")
            .touch,
        live.session.runtime_state.macro_state.mc202_touch,
    );
    assert_eq!(
        expected,
        (0.2, 0.2),
        "live SetRole must reduce touch before the subsequent Answer"
    );

    // Model a historical log missing only the per-commit phrase decisions.
    // Keep actual committed actions/results/records from the public live writer.
    let mut historical = live.session;
    for record in historical.action_log.commit_records.iter_mut().skip(1) {
        assert!(record.mc202_source_phrase_plan.is_some());
        record.mc202_source_phrase_plan = None;
    }
    let id = SnapshotId::from("generated-touch-before-mc202");
    historical.snapshots.push(Snapshot {
        snapshot_id: id.clone(),
        created_at: "2026-10-16".into(),
        label: "generated confirmed timing before touch history".into(),
        action_cursor: 1,
        payload: Some(SnapshotPayload::from_runtime_state(&id, 1, &anchor_runtime)),
    });

    let mut observed = Vec::new();
    for payload in [false, true] {
        let mut app =
            JamAppState::from_parts(historical.clone(), Some(graph.clone()), ActionQueue::new());
        // Target-suffix callers supply the selected anchor's materialized state;
        // payload restore independently installs this state from its payload.
        // Constructor output is deliberately not used as the expected result.
        app.session.runtime_state = anchor_runtime.clone();
        let report = if payload {
            app.apply_restore_target_from_snapshot_payload(4).unwrap()
        } else {
            app.apply_restore_target_suffix(4).unwrap()
        };
        assert_eq!(report.anchor_action_cursor, Some(1));
        assert_eq!(
            report.applied_action_ids,
            vec![ActionId(2), ActionId(3), ActionId(4)]
        );
        assert_no_hydration(&app);
        let actual = (
            app.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan
                .as_ref()
                .expect("missing-plan replay must produce an explicit plan")
                .touch,
            app.session.runtime_state.macro_state.mc202_touch,
        );
        eprintln!("payload={payload}: restored {actual:?}; actual live writer {expected:?}");
        observed.push(actual);
    }
    assert!(!source.exists());
    assert_eq!(
        observed,
        vec![expected, expected],
        "both public restore seams must preserve live SetRole's touch reduction"
    );
}
