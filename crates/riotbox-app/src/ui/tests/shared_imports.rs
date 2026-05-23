use crate::test_support::{scene_energy_for_label, scene_label_hint};
use riotbox_core::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus,
        ActionTarget, ActorType, GhostMode, Quantization, TargetScope, UndoPolicy,
    },
    ghost::{
        GhostSuggestedAction, GhostSuggestionConfidence, GhostSuggestionSafety,
        GhostWatchSuggestion, GhostWatchTool,
    },
    ids::{
        ActionId, AssetId, BankId, CaptureId, PadId, SceneId, SectionId, SnapshotId, SourceId,
    },
    queue::ActionQueue,
    session::{
        ActionCommitRecord, GhostSuggestionRecord, SceneMovementDirectionState,
        SceneMovementKindState, SceneMovementLaneIntentState, SceneMovementState, SessionFile,
        Snapshot, SourceTimingGridConfirmationState, Tr909ReinforcementModeState,
        Tr909TakeoverProfileState,
    },
    source_graph::{
        AnalysisSummary, AnalysisWarning, Asset, AssetType, BarSpan, BeatPoint, Candidate,
        CandidateType, DecodeProfile, EnergyClass, GraphProvenance, MeterHint, PhraseSpan,
        QualityClass, Relationship, RelationshipType, Section, SectionLabelHint,
        SourceDescriptor, SourceGraph, SourceTimingAnchor, SourceTimingAnchorType,
        TimingDegradedPolicy, TimingHypothesis, TimingHypothesisKind, TimingQuality,
        TimingWarning, TimingWarningCode,
    },
    transport::{CommitBoundaryState, TransportClockState},
};
use serde::Deserialize;
use super::*;
