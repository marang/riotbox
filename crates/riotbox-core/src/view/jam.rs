// Compatibility facade: projections consume existing Core/Session truth.
mod arrangement_contract;
mod build_view_model;
mod capture_actions;
mod model;
mod performer_state;
mod scene_launch;
mod source_map;
mod source_summary;
mod source_timing_summary;

#[cfg(test)]
mod tests;

pub use crate::session::source_timing_confirmation_matches_graph;
pub use arrangement_contract::{
    ArrangementSceneActionSurfaceView, ArrangementSceneBoundedExtensionView,
    ArrangementSceneContractReadinessView, ArrangementSceneContractView,
    ArrangementSceneTruthSourceView, arrangement_scene_contract_view,
};
pub use capture_actions::{
    CaptureHandoffReadinessView, CaptureSummaryView, CaptureTargetKindView,
    PendingCaptureActionView,
};
pub use model::{JamTransportView, JamViewModel};
pub use performer_state::{
    GhostStatusView, LaneSummaryView, MacroStripView, PendingActionView, RecentActionView,
    W30PendingAuditionKind, W30PendingAuditionView,
};
pub use scene_launch::{
    SceneJumpAvailabilityView, SceneLaunchCandidateView, SceneLaunchTargetReason,
    SceneMovementView, SceneSummaryView, SceneTransitionDirectionView, SceneTransitionKindView,
    SceneTransitionLaneIntentView, SceneTransitionPolicyView, SceneTransitionW30IntentView,
    next_scene_launch_candidate, next_scene_launch_candidate_with_reason,
};
pub use source_map::{SourceMapModeView, SourceMapView};
pub use source_summary::{FeralScorecardView, SourceSummaryView};
pub use source_timing_summary::{
    SourceTimingConsumerReadiness, SourceTimingGrooveResidualView, SourceTimingSummaryView,
    source_timing_consumer_readiness,
};
