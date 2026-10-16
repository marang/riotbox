//! One missing-plan snapshot suffix, with generated in-memory Graph metadata.
use riotbox_core::{
    ids::{ActionId, SnapshotId},
    queue::ActionQueue,
    replay::apply_replay_target_suffix_to_session,
    session::{ReplayPolicy, SessionFile, Snapshot, SnapshotPayload},
};

use super::{
    JamAppState, assert_no_hydration, confirmed_app_with_persisted_plan, generated_plan,
    invalidated_history,
};

#[test]
fn snapshot_payload_suffix_reconstructs_missing_legacy_plan_from_trusted_pre_entry_state() {
    let owner = tempfile::tempdir().unwrap();
    let nonexistent_source = owner.path().join("never-created-payload-source.wav");
    let seed = confirmed_app_with_persisted_plan(&nonexistent_source);
    let graph = seed.source_graph.unwrap();
    let mut session = seed.session;
    assert_eq!(session.action_log.commit_records.len(), 2);
    session.action_log.commit_records[1].mc202_source_phrase_plan = None;

    // Materialize the exact pre-phrase anchor through ordinary Core replay.
    // Its confirmed timing is trusted, but it contains no historical plan.
    let mut anchor = session.clone();
    anchor.runtime_state = SessionFile::new("generated-anchor", "test", "2026-10-16").runtime_state;
    apply_replay_target_suffix_to_session(&mut anchor, 1, Some(&graph)).unwrap();
    assert!(anchor.runtime_state.source_timing.confirmed_grid.is_some());
    assert!(
        anchor
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
    let snapshot_id = SnapshotId::from("generated-before-mc202");
    session.runtime_state = anchor.runtime_state;
    session.snapshots.push(Snapshot {
        snapshot_id: snapshot_id.clone(),
        created_at: "2026-10-16".into(),
        label: "generated confirmed timing before missing-plan phrase".into(),
        action_cursor: 1,
        payload: Some(SnapshotPayload::from_runtime_state(
            &snapshot_id,
            1,
            &session.runtime_state,
        )),
    });

    // Independent existing-constructor control, not the payload implementation
    // under test. A degraded Some is still an explicit selector decision; this
    // fixture makes no source-intelligence or audible-output claim.
    let expected = {
        let control =
            JamAppState::from_parts(session.clone(), Some(graph.clone()), ActionQueue::new());
        assert_no_hydration(&control);
        control
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .expect("the existing legacy constructor must derive an explicit plan")
    };
    let mut app = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    let report = app.apply_restore_target_from_snapshot_payload(2).unwrap();
    assert_eq!(
        report.anchor_snapshot_id.as_deref(),
        Some("generated-before-mc202")
    );
    assert_eq!(report.anchor_action_cursor, Some(1));
    assert_eq!(report.applied_action_ids, vec![ActionId(2)]);
    assert_no_hydration(&app);
    assert!(!nonexistent_source.exists());
    let actual = app.session.runtime_state.lane_state.mc202.source_phrase_plan.as_ref()
        .expect("the actual missing MC-202 suffix entry must receive legacy resolution after payload hydration");
    assert_eq!(actual, &expected);
}

#[test]
fn payload_anchors_and_empty_suffixes_preserve_invalidation_instead_of_reselecting_history() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicCommittedOrderV2,
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
    ] {
        let source = owner
            .path()
            .join(format!("never-created-anchor-{policy:?}.wav"));
        let history = invalidated_history(&source, policy);
        let graph = history.source_graph.clone().unwrap();
        for anchor_cursor in 0..=4 {
            let mut anchor = SessionFile::new("generated-anchor-prefix", "test", "2026-10-16");
            anchor.action_log = history.session.action_log.clone();
            apply_replay_target_suffix_to_session(&mut anchor, anchor_cursor, Some(&graph))
                .unwrap();
            let id = SnapshotId::from(format!("generated-anchor-{anchor_cursor}"));
            let snapshot = Snapshot {
                snapshot_id: id.clone(),
                created_at: "2026-10-16".into(),
                label: "generated Core-materialized prefix".into(),
                action_cursor: anchor_cursor,
                payload: Some(SnapshotPayload::from_runtime_state(
                    &id,
                    anchor_cursor,
                    &anchor.runtime_state,
                )),
            };
            for graph_available in [false, true] {
                for target in anchor_cursor..=4 {
                    let mut session = history.session.clone();
                    session.snapshots.push(snapshot.clone());
                    let mut app = JamAppState::from_parts(
                        session,
                        graph_available.then(|| graph.clone()),
                        ActionQueue::new(),
                    );
                    // Hydration must replace the caller's current material,
                    // including when the selected suffix executes no action.
                    app.session
                        .runtime_state
                        .lane_state
                        .mc202
                        .source_phrase_plan = Some(generated_plan());
                    let log = app.session.action_log.clone();
                    let snapshots = app.session.snapshots.clone();
                    let report = app
                        .apply_restore_target_from_snapshot_payload(target)
                        .unwrap();
                    assert_eq!(report.anchor_action_cursor, Some(anchor_cursor));
                    assert_eq!(report.anchor_snapshot_id.as_deref(), Some(id.as_str()));
                    assert_eq!(
                        report.applied_action_ids,
                        [ActionId(1), ActionId(2), ActionId(3), ActionId(4)][anchor_cursor..target]
                    );
                    assert_eq!(
                        app.session
                            .runtime_state
                            .lane_state
                            .mc202
                            .source_phrase_plan,
                        (target == 2).then(generated_plan)
                    );
                    assert_eq!(app.session.action_log, log);
                    assert_eq!(app.session.snapshots, snapshots);
                    assert_no_hydration(&app);
                    assert!(!source.exists());
                }
            }
        }
    }
}
