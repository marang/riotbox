mod artifact_hydration;
mod executor;
mod export_receipt;
mod plan;
mod scene_movement;
mod snapshot_runtime;
mod summary;
mod target_execution;
mod traversal;

pub use artifact_hydration::{
    W30ArtifactReplayHydrationError, W30ArtifactReplayHydrationPlan,
    W30SourceCaptureReplayHydrationPlan, plan_source_window_loop_capture_replay_hydration,
    plan_source_window_pad_capture_replay_hydration, plan_w30_artifact_replay_hydration,
};
pub use executor::{
    ReplayExecutionError, ReplayExecutionReport, apply_replay_entry_to_session,
    apply_replay_plan_to_session, replay_supported_action_commands,
};
pub use export_receipt::{
    ExportReceiptReplayValidationError, ExportReceiptReplayValidationPlan,
    plan_export_receipt_replay_validation,
};
pub use plan::{
    ReplayPlanEntry, ReplayPlanError, ReplayPolicyUpgradeError, ReplayTargetPlan,
    SnapshotReplayPlanComparison, build_committed_replay_plan, build_replay_target_plan,
    build_snapshot_replay_plan_comparison, select_replay_snapshot_anchor,
    upgrade_replay_policy_to_committed_order,
};
pub use scene_movement::{
    apply_graph_aware_replay_plan_to_session, apply_scene_audio_projection_transition,
    derive_scene_movement_for_replay_entry, derive_scene_movement_state,
};
pub use snapshot_runtime::{
    SnapshotRuntimeActionError, SnapshotRuntimeField, SnapshotRuntimeReference,
    SnapshotRuntimeReferenceError, SnapshotRuntimeValidationError,
};
pub use summary::{
    LatestSnapshotReplayConvergenceSummary, ReplayTargetDryRunSummary, SnapshotPayloadReadiness,
    build_latest_snapshot_replay_convergence_summary, build_replay_target_dry_run_summary,
};
pub use target_execution::{
    ReplayTargetExecutionError, ReplayTargetExecutionReport, SnapshotPayloadHydrationError,
    SnapshotPayloadHydrationReport, apply_replay_target_suffix_to_session,
    apply_replay_target_suffix_with_mc202_legacy_resolver,
    hydrate_replay_target_from_snapshot_payload,
    hydrate_replay_target_from_snapshot_payload_with_mc202_legacy_resolver,
};
pub use traversal::apply_replay_entry_with_mc202_legacy_resolver;
