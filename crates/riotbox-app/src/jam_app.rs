use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use riotbox_audio::source_audio::SourceAudioCache;
use riotbox_core::{
    action::ActionStatus,
    ids::SourceId,
    persistence::{
        load_session_json, load_source_graph_json, save_session_json, save_source_graph_json,
    },
    queue::ActionQueue,
    session::{GraphStorageMode, SessionFile, SourceGraphRef, SourceRef},
    source_graph::{DecodeProfile, SourceGraph},
    view::jam::JamViewModel,
};
use riotbox_sidecar::client::StdioSidecarClient;
use sha2::{Digest, Sha256};

#[cfg(test)]
use riotbox_audio::runtime::AudioRuntimeTimingSnapshot;

mod capture_artifacts;
mod capture_helpers;
mod commit;
mod controls;
mod ghost_candidates;
mod ghost_queue;
mod helpers;
mod lifecycle;
mod mc202_queue;
mod persistence;
mod projection;
mod recovery;
mod restore_replay;
mod runtime_replay_warnings;
mod runtime_view;
mod scene_ops;
mod side_effects;
mod state;
mod tr909_queue;
mod transport;
mod transport_helpers;
mod w30_queue;
mod w30_targets;

use capture_helpers::{
    apply_capture_promotion_side_effects, capture_promotion_summary, capture_ref_from_action,
    capture_targets_specific_w30_pad, capture_targets_w30_pad,
};
pub use ghost_queue::{GhostSuggestionQueueResult, NO_CURRENT_GHOST_SUGGESTION_REASON};
use helpers::{
    is_mc202_phrase_action, max_action_id, next_action_id_from_session, update_logged_action_result,
};
use projection::normalize_w30_preview_mode;
pub use recovery::{
    RecoveryCandidateGuidance, RecoveryCandidateTrust, SessionRecoveryCandidateView,
    SessionRecoverySurface,
};
pub use runtime_view::JamRuntimeView;
use side_effects::{
    apply_ghost_side_effects, apply_mc202_side_effects, apply_scene_side_effects,
    apply_tr909_side_effects, apply_w30_side_effects,
};
pub use state::{
    AppRuntimeState, JamAppError, JamAppState, JamFileSet, QueueControlResult, SidecarState,
    TransportDriverState,
};
use transport_helpers::{normalize_scene_candidates, transport_clock_from_state};

#[cfg(test)]
use riotbox_core::TimestampMs;
#[cfg(test)]
use riotbox_core::transport::CommitBoundaryState;

impl JamAppState {
    pub(super) const W30_DAMAGE_PROFILE_LABEL: &str = "shred";
    pub(super) const W30_DAMAGE_PROFILE_GRIT: f32 = 0.82;
    pub(super) const W30_LOOP_FREEZE_LABEL: &str = "freeze";
}

#[cfg(test)]
mod tests;
