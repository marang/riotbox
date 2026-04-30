mod artifact_hydration;
mod executor;
mod plan;
mod scene_movement;
mod summary;
mod target_execution;

pub use artifact_hydration::{
    W30ArtifactReplayHydrationError, W30ArtifactReplayHydrationPlan,
    plan_w30_artifact_replay_hydration,
};
pub use executor::{
    ReplayExecutionError, ReplayExecutionReport, apply_replay_entry_to_session,
    apply_replay_plan_to_session, replay_supported_action_commands,
};
pub use plan::{
    ReplayPlanEntry, ReplayPlanError, ReplayTargetPlan, SnapshotReplayPlanComparison,
    build_committed_replay_plan, build_replay_target_plan, build_snapshot_replay_plan_comparison,
    select_replay_snapshot_anchor,
};
pub use scene_movement::{
    apply_graph_aware_replay_plan_to_session, derive_scene_movement_for_replay_entry,
    derive_scene_movement_state,
};
pub use summary::{
    LatestSnapshotReplayConvergenceSummary, ReplayTargetDryRunSummary, SnapshotPayloadReadiness,
    build_latest_snapshot_replay_convergence_summary, build_replay_target_dry_run_summary,
};
pub use target_execution::{
    ReplayTargetExecutionError, ReplayTargetExecutionReport, SnapshotPayloadHydrationError,
    SnapshotPayloadHydrationReport, apply_replay_target_suffix_to_session,
    hydrate_replay_target_from_snapshot_payload,
};
