use std::{
    collections::BTreeMap,
    error::Error,
    fmt::{self, Display, Formatter},
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use riotbox_audio::{
    mc202::Mc202RenderState,
    runtime::{AudioRuntimeHealth, AudioRuntimeTimingSnapshot, render_w30_resample_tap_offline},
    source_audio::{SourceAudioCache, SourceAudioWindow, write_interleaved_pcm16_wav},
    tr909::Tr909RenderState,
    w30::{
        W30PreviewRenderState, W30ResampleTapMode, W30ResampleTapRouting,
        W30ResampleTapSourceProfile, W30ResampleTapState,
    },
};
use riotbox_core::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus, ActionTarget,
        ActorType, Quantization, TargetScope,
    },
    ghost::GhostWatchSuggestion,
    ids::{ActionId, CaptureId, SourceId},
    persistence::{
        PersistenceError, SessionRecoveryCandidateKind, SessionRecoveryCandidateStatus,
        load_session_json, load_source_graph_json, save_session_json, save_source_graph_json,
        scan_session_recovery_candidates,
    },
    queue::{ActionQueue, CommittedActionRef},
    session::{
        ActionCommitRecord, CaptureRef, CaptureTarget, GraphStorageMode, Mc202UndoSnapshotState,
        SessionFile, SourceGraphRef, SourceRef, Tr909TakeoverProfileState,
    },
    source_graph::{DecodeProfile, SourceGraph},
    transport::{CommitBoundaryState, TransportClockState},
    view::jam::JamViewModel,
};
use riotbox_sidecar::client::{ClientError as SidecarClientError, StdioSidecarClient};
use sha2::{Digest, Sha256};

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
use helpers::{is_mc202_phrase_action, max_action_id, next_action_id_from_session};
use projection::{
    build_mc202_render_state, build_tr909_render_state, build_w30_preview_render_state,
    build_w30_resample_tap_state, normalize_w30_preview_mode,
};
pub use recovery::{RecoveryCandidateTrust, SessionRecoveryCandidateView, SessionRecoverySurface};
pub use runtime_view::JamRuntimeView;
use side_effects::{
    apply_ghost_side_effects, apply_mc202_side_effects, apply_scene_side_effects,
    apply_tr909_side_effects, apply_w30_side_effects,
};
pub use state::{
    AppRuntimeState, JamAppError, JamAppState, JamFileSet, QueueControlResult, SidecarState,
    TransportDriverState,
};
use transport_helpers::{
    crossed_commit_boundary, normalize_scene_candidates, transport_clock_for_state,
    transport_clock_from_state,
};

impl JamAppState {
    pub(super) const W30_DAMAGE_PROFILE_LABEL: &str = "shred";
    pub(super) const W30_DAMAGE_PROFILE_GRIT: f32 = 0.82;
    pub(super) const W30_LOOP_FREEZE_LABEL: &str = "freeze";
}

#[cfg(test)]
mod tests;
