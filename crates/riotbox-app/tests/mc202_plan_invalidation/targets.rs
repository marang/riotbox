//! Core/App parity at each declared target, never source acquisition.
use riotbox_app::jam_app::JamAppState;
use riotbox_core::{
    ids::ActionId,
    queue::ActionQueue,
    replay::apply_replay_target_suffix_to_session,
    session::{ReplayPolicy, SessionFile},
};

use super::{assert_no_hydration, generated_plan, invalidated_history};

#[test]
fn every_target_prefix_retains_exact_persisted_plan_or_invalidation_for_all_policies() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicCommittedOrderV2,
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
    ] {
        let source = owner.path().join(format!("never-created-{policy:?}.wav"));
        let history = invalidated_history(&source, policy);
        for graph_available in [false, true] {
            for cursor in 0..=4 {
                let mut origin = SessionFile::new("generated-target-prefix", "test", "2026-10-16");
                origin.action_log = history.session.action_log.clone();
                let log = origin.action_log.clone();
                let mut core = origin.clone();
                let core_report =
                    apply_replay_target_suffix_to_session(&mut core, cursor, None).unwrap();
                let graph = graph_available.then(|| history.source_graph.clone().unwrap());
                let mut app = JamAppState::from_parts(origin, graph, ActionQueue::new());
                let report = app.apply_restore_target_suffix(cursor).unwrap();
                assert_eq!(report, core_report);
                assert_eq!(
                    report.applied_action_ids,
                    [ActionId(1), ActionId(2), ActionId(3), ActionId(4)][..cursor]
                );
                let expected = (cursor == 2).then(generated_plan);
                assert_eq!(
                    core.runtime_state.lane_state.mc202.source_phrase_plan,
                    expected
                );
                assert_eq!(
                    app.session
                        .runtime_state
                        .lane_state
                        .mc202
                        .source_phrase_plan,
                    expected
                );
                assert_eq!(
                    app.session.runtime_state.lane_state.mc202,
                    core.runtime_state.lane_state.mc202
                );
                assert_eq!(
                    app.session.runtime_state.source_timing,
                    core.runtime_state.source_timing
                );
                assert_eq!(
                    app.session.action_log, log,
                    "restore cannot mutate history or backfill records"
                );
                assert_no_hydration(&app);
                assert!(!source.exists());
            }
        }
    }
}
