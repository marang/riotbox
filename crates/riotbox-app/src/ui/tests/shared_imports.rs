use crate::test_support::{scene_energy_for_label, scene_label_hint};
use riotbox_core::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus,
        ActionTarget, ActorType, CaptureLengthIntent, GhostMode, Quantization, TargetScope,
        UndoPolicy,
    },
    ghost::{
        GhostSuggestedAction, GhostSuggestionConfidence, GhostSuggestionSafety,
        GhostWatchSuggestion, GhostWatchTool,
    },
    export_readiness::{
        ExportReadinessStatus, ExportScope, ProductExportBoundary, ProductExportRole,
        STEM_PACKAGE_LOCAL_CI_PACK_ID, UnsupportedExportScope,
    },
    ids::{
        ActionId, AssetId, BankId, CaptureId, ExportReceiptId, PadId, SceneId, SectionId,
        SnapshotId, SourceId,
    },
    queue::ActionQueue,
    session::{
        ActionCommitRecord, ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactRole,
        ExportArtifactSetEntry, ExportReceiptQaGateResult, ExportReceiptQaGateStatus,
        ExportReceiptState, GhostSuggestionRecord, Mc202RoleState, SceneMovementDirectionState,
        SceneMovementKindState, STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
        STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID, STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
        STEM_PACKAGE_LINEAGE_QA_GATE_ID, STEM_PACKAGE_NON_SILENCE_QA_GATE_ID,
        SceneMovementLaneIntentState, SceneMovementState, SessionFile, Snapshot,
        SourceTimingGridConfirmationState, Tr909ReinforcementModeState, Tr909TakeoverProfileState,
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
