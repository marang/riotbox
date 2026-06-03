use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

use crate::{
    TimestampMs,
    export_readiness::{
        ExportScope, ProductExportBoundary, ProductExportDestinationKind, ProductExportRole,
        default_export_scope,
    },
    ids::{ActionId, AssetId, BankId, CaptureId, PadId, SceneId, SourceId},
    session::ExportArtifactRole,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorType {
    User,
    Ghost,
    System,
}

impl Display for ActorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::User => "user",
            Self::Ghost => "ghost",
            Self::System => "system",
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionStatus {
    Requested,
    Queued,
    PendingCommit,
    Committed,
    Rejected,
    Undone,
    Failed,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quantization {
    Immediate,
    NextBeat,
    NextHalfBar,
    NextBar,
    NextPhrase,
    NextScene,
}

impl Quantization {
    #[must_use]
    pub fn is_ready_for(self, boundary: CommitBoundary) -> bool {
        self.rank() <= boundary.rank()
    }

    const fn rank(self) -> u8 {
        match self {
            Self::Immediate => 0,
            Self::NextBeat => 1,
            Self::NextHalfBar => 2,
            Self::NextBar => 3,
            Self::NextPhrase => 4,
            Self::NextScene => 5,
        }
    }
}

impl Display for Quantization {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Immediate => "immediate",
            Self::NextBeat => "next_beat",
            Self::NextHalfBar => "next_half_bar",
            Self::NextBar => "next_bar",
            Self::NextPhrase => "next_phrase",
            Self::NextScene => "next_scene",
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitBoundary {
    Immediate,
    Beat,
    HalfBar,
    Bar,
    Phrase,
    Scene,
}

impl CommitBoundary {
    const fn rank(self) -> u8 {
        match self {
            Self::Immediate => 0,
            Self::Beat => 1,
            Self::HalfBar => 2,
            Self::Bar => 3,
            Self::Phrase => 4,
            Self::Scene => 5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetScope {
    Global,
    Scene,
    LaneMc202,
    LaneW30,
    LaneTr909,
    Mixer,
    Ghost,
    Session,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ActionTarget {
    pub scope: Option<TargetScope>,
    pub scene_id: Option<SceneId>,
    pub bank_id: Option<BankId>,
    pub pad_id: Option<PadId>,
    pub loop_id: Option<AssetId>,
    pub object_id: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GhostMode {
    Off,
    Watch,
    Assist,
    Perform,
}

impl Display for GhostMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Off => "off",
            Self::Watch => "watch",
            Self::Assist => "assist",
            Self::Perform => "perform",
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMonitorMode {
    #[default]
    Source,
    Blend,
    Riotbox,
}

impl SourceMonitorMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Blend => "blend",
            Self::Riotbox => "riotbox",
        }
    }
}

impl Display for SourceMonitorMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureLengthIntent {
    OneBeat,
    OneBar,
    #[default]
    FourBars,
    Phrase,
}

impl CaptureLengthIntent {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OneBeat => "1 beat",
            Self::OneBar => "1 bar",
            Self::FourBars => "4 bars",
            Self::Phrase => "phrase",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::OneBeat => Self::OneBar,
            Self::OneBar => Self::FourBars,
            Self::FourBars => Self::Phrase,
            Self::Phrase => Self::OneBeat,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::OneBeat => Self::Phrase,
            Self::OneBar => Self::OneBeat,
            Self::FourBars => Self::OneBar,
            Self::Phrase => Self::FourBars,
        }
    }
}

impl Display for CaptureLengthIntent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActionParams {
    Empty,
    Transport {
        position_beats: Option<u64>,
    },
    Mutation {
        intensity: f32,
        target_id: Option<String>,
    },
    Capture {
        bars: Option<u32>,
    },
    CaptureLength {
        intent: Option<CaptureLengthIntent>,
    },
    Promotion {
        capture_id: Option<CaptureId>,
        destination: Option<String>,
    },
    Scene {
        scene_id: Option<SceneId>,
    },
    Lock {
        object_id: String,
    },
    Snapshot {
        label: Option<String>,
    },
    Ghost {
        mode: Option<GhostMode>,
        proposal_id: Option<String>,
    },
    SourceMonitor {
        mode: Option<SourceMonitorMode>,
    },
    SourceTimingGrid {
        source_id: Option<SourceId>,
        hypothesis_id: Option<String>,
    },
    ProductExport {
        #[serde(default = "default_export_scope")]
        export_scope: ExportScope,
        export_role: ProductExportRole,
        boundary: ProductExportBoundary,
        include_manifest: bool,
        destination_kind: ProductExportDestinationKind,
        destination_path: Option<String>,
    },
    StemPackageExport {
        #[serde(default = "default_stem_package_export_scope")]
        export_scope: ExportScope,
        export_role: StemPackageExportRole,
        boundary: StemPackageExportBoundary,
        include_manifest: bool,
        destination_kind: ProductExportDestinationKind,
        destination_path: Option<String>,
        claimed_stem_roles: Vec<ExportArtifactRole>,
        lineage_policy: StemPackageLineagePolicy,
        fallback_comparison_policy: StemPackageFallbackComparisonPolicy,
    },
    LiveRecordingExport {
        #[serde(default = "default_live_recording_export_scope")]
        export_scope: ExportScope,
        export_role: LiveRecordingExportRole,
        boundary: LiveRecordingExportBoundary,
        include_manifest: bool,
        destination_kind: ProductExportDestinationKind,
        destination_path: Option<String>,
        receipt_id: Option<String>,
    },
    DawSessionExport {
        #[serde(default = "default_daw_session_export_scope")]
        export_scope: ExportScope,
        boundary: DawSessionExportBoundary,
        include_manifest: bool,
        destination_kind: ProductExportDestinationKind,
        destination_path: Option<String>,
        receipt_id: Option<String>,
    },
}

#[must_use]
pub const fn default_stem_package_export_scope() -> ExportScope {
    ExportScope::StemPackage
}

#[must_use]
pub const fn default_live_recording_export_scope() -> ExportScope {
    ExportScope::LiveRecording
}

#[must_use]
pub const fn default_daw_session_export_scope() -> ExportScope {
    ExportScope::DawSession
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageExportRole {
    PackageManifest,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageExportBoundary {
    ReservedContractOnly,
    LocalCiPackageV1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveRecordingExportRole {
    LiveRecordingCapture,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveRecordingExportBoundary {
    ReservedContractOnly,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionExportBoundary {
    ReservedContractOnly,
    LocalProjectWriterV1,
    HostImportProofV1,
    AudibleOutputProofV1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageLineagePolicy {
    NotRequired,
    RequireAnyCoreLineage,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageFallbackComparisonPolicy {
    NotRequired,
    Required,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionCommand {
    TransportPlay,
    TransportPause,
    TransportStop,
    TransportSeek,
    MutateScene,
    MutateLane,
    MutateLoop,
    MutatePattern,
    MutateHook,
    CaptureNow,
    CaptureLoop,
    CaptureBarGroup,
    CaptureSetLength,
    PromoteCaptureToPad,
    PromoteCaptureToScene,
    PromoteResample,
    SceneLaunch,
    SceneRestore,
    SceneRegenerate,
    SceneReinterpret,
    Mc202GenerateFollower,
    Mc202GenerateAnswer,
    Mc202GeneratePressure,
    Mc202GenerateInstigator,
    Mc202MutatePhrase,
    Mc202SetRole,
    W30CaptureToPad,
    W30LiveRecall,
    W30TriggerPad,
    W30AuditionRawCapture,
    W30AuditionPromoted,
    W30SwapBank,
    W30BrowseSlicePool,
    W30StepFocus,
    W30ApplyDamageProfile,
    W30LoopFreeze,
    Tr909FillNext,
    Tr909SetSlam,
    Tr909ReinforceBreak,
    Tr909Takeover,
    Tr909SceneLock,
    Tr909Release,
    LockObject,
    UnlockObject,
    SnapshotSave,
    SnapshotLoad,
    UndoLast,
    RedoLast,
    RestoreSource,
    SourceMonitorSetMode,
    SourceTimingConfirmGrid,
    SourceTimingRevertGrid,
    ExportProductMix,
    ExportStemPackage,
    ExportLiveRecording,
    ExportDawSession,
    GhostSetMode,
    GhostAcceptSuggestion,
    GhostRejectSuggestion,
    GhostExecuteTool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActionReplayCoverage {
    Supported,
    Unsupported,
}

impl ActionCommand {
    pub const ALL: &'static [Self] = &[
        Self::TransportPlay,
        Self::TransportPause,
        Self::TransportStop,
        Self::TransportSeek,
        Self::MutateScene,
        Self::MutateLane,
        Self::MutateLoop,
        Self::MutatePattern,
        Self::MutateHook,
        Self::CaptureNow,
        Self::CaptureLoop,
        Self::CaptureBarGroup,
        Self::CaptureSetLength,
        Self::PromoteCaptureToPad,
        Self::PromoteCaptureToScene,
        Self::PromoteResample,
        Self::SceneLaunch,
        Self::SceneRestore,
        Self::SceneRegenerate,
        Self::SceneReinterpret,
        Self::Mc202GenerateFollower,
        Self::Mc202GenerateAnswer,
        Self::Mc202GeneratePressure,
        Self::Mc202GenerateInstigator,
        Self::Mc202MutatePhrase,
        Self::Mc202SetRole,
        Self::W30CaptureToPad,
        Self::W30LiveRecall,
        Self::W30TriggerPad,
        Self::W30AuditionRawCapture,
        Self::W30AuditionPromoted,
        Self::W30SwapBank,
        Self::W30BrowseSlicePool,
        Self::W30StepFocus,
        Self::W30ApplyDamageProfile,
        Self::W30LoopFreeze,
        Self::Tr909FillNext,
        Self::Tr909SetSlam,
        Self::Tr909ReinforceBreak,
        Self::Tr909Takeover,
        Self::Tr909SceneLock,
        Self::Tr909Release,
        Self::LockObject,
        Self::UnlockObject,
        Self::SnapshotSave,
        Self::SnapshotLoad,
        Self::UndoLast,
        Self::RedoLast,
        Self::RestoreSource,
        Self::SourceMonitorSetMode,
        Self::SourceTimingConfirmGrid,
        Self::SourceTimingRevertGrid,
        Self::ExportProductMix,
        Self::ExportStemPackage,
        Self::ExportLiveRecording,
        Self::ExportDawSession,
        Self::GhostSetMode,
        Self::GhostAcceptSuggestion,
        Self::GhostRejectSuggestion,
        Self::GhostExecuteTool,
    ];

    #[must_use]
    pub const fn all() -> &'static [Self] {
        Self::ALL
    }

    #[must_use]
    pub const fn replay_coverage(self) -> ActionReplayCoverage {
        match self {
            Self::TransportPlay
            | Self::TransportPause
            | Self::TransportStop
            | Self::TransportSeek
            | Self::CaptureNow
            | Self::CaptureLoop
            | Self::CaptureBarGroup
            | Self::CaptureSetLength
            | Self::PromoteCaptureToPad
            | Self::PromoteCaptureToScene
            | Self::PromoteResample
            | Self::SceneLaunch
            | Self::SceneRestore
            | Self::Mc202GenerateFollower
            | Self::Mc202GenerateAnswer
            | Self::Mc202GeneratePressure
            | Self::Mc202GenerateInstigator
            | Self::Mc202MutatePhrase
            | Self::Mc202SetRole
            | Self::W30CaptureToPad
            | Self::W30LiveRecall
            | Self::W30TriggerPad
            | Self::W30AuditionRawCapture
            | Self::W30AuditionPromoted
            | Self::W30SwapBank
            | Self::W30BrowseSlicePool
            | Self::W30StepFocus
            | Self::W30ApplyDamageProfile
            | Self::W30LoopFreeze
            | Self::Tr909FillNext
            | Self::Tr909SetSlam
            | Self::Tr909ReinforceBreak
            | Self::Tr909Takeover
            | Self::Tr909SceneLock
            | Self::Tr909Release
            | Self::LockObject
            | Self::UnlockObject
            | Self::SourceMonitorSetMode
            | Self::SourceTimingConfirmGrid
            | Self::SourceTimingRevertGrid
            | Self::GhostSetMode
            | Self::MutateScene => ActionReplayCoverage::Supported,
            Self::MutateLane
            | Self::MutateLoop
            | Self::MutatePattern
            | Self::MutateHook
            | Self::SceneRegenerate
            | Self::SceneReinterpret
            | Self::SnapshotSave
            | Self::SnapshotLoad
            | Self::UndoLast
            | Self::RedoLast
            | Self::RestoreSource
            | Self::ExportProductMix
            | Self::ExportStemPackage
            | Self::ExportLiveRecording
            | Self::ExportDawSession
            | Self::GhostAcceptSuggestion
            | Self::GhostRejectSuggestion
            | Self::GhostExecuteTool => ActionReplayCoverage::Unsupported,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TransportPlay => "transport.play",
            Self::TransportPause => "transport.pause",
            Self::TransportStop => "transport.stop",
            Self::TransportSeek => "transport.seek",
            Self::MutateScene => "mutate.scene",
            Self::MutateLane => "mutate.lane",
            Self::MutateLoop => "mutate.loop",
            Self::MutatePattern => "mutate.pattern",
            Self::MutateHook => "mutate.hook",
            Self::CaptureNow => "capture.now",
            Self::CaptureLoop => "capture.loop",
            Self::CaptureBarGroup => "capture.bar_group",
            Self::CaptureSetLength => "capture.set_length",
            Self::PromoteCaptureToPad => "promote.capture_to_pad",
            Self::PromoteCaptureToScene => "promote.capture_to_scene",
            Self::PromoteResample => "promote.resample",
            Self::SceneLaunch => "scene.launch",
            Self::SceneRestore => "scene.restore",
            Self::SceneRegenerate => "scene.regenerate",
            Self::SceneReinterpret => "scene.reinterpret",
            Self::Mc202GenerateFollower => "mc202.generate_follower",
            Self::Mc202GenerateAnswer => "mc202.generate_answer",
            Self::Mc202GeneratePressure => "mc202.generate_pressure",
            Self::Mc202GenerateInstigator => "mc202.generate_instigator",
            Self::Mc202MutatePhrase => "mc202.mutate_phrase",
            Self::Mc202SetRole => "mc202.set_role",
            Self::W30CaptureToPad => "w30.capture_to_pad",
            Self::W30LiveRecall => "w30.live_recall",
            Self::W30TriggerPad => "w30.trigger_pad",
            Self::W30AuditionRawCapture => "w30.audition_raw_capture",
            Self::W30AuditionPromoted => "w30.audition_promoted",
            Self::W30SwapBank => "w30.swap_bank",
            Self::W30BrowseSlicePool => "w30.browse_slice_pool",
            Self::W30StepFocus => "w30.step_focus",
            Self::W30ApplyDamageProfile => "w30.apply_damage_profile",
            Self::W30LoopFreeze => "w30.loop_freeze",
            Self::Tr909FillNext => "tr909.fill_next",
            Self::Tr909SetSlam => "tr909.set_slam",
            Self::Tr909ReinforceBreak => "tr909.reinforce_break",
            Self::Tr909Takeover => "tr909.takeover",
            Self::Tr909SceneLock => "tr909.scene_lock",
            Self::Tr909Release => "tr909.release",
            Self::LockObject => "lock.object",
            Self::UnlockObject => "unlock.object",
            Self::SnapshotSave => "snapshot.save",
            Self::SnapshotLoad => "snapshot.load",
            Self::UndoLast => "undo.last",
            Self::RedoLast => "redo.last",
            Self::RestoreSource => "restore.source",
            Self::SourceMonitorSetMode => "source_monitor.set_mode",
            Self::SourceTimingConfirmGrid => "source_timing.confirm_grid",
            Self::SourceTimingRevertGrid => "source_timing.revert_grid",
            Self::ExportProductMix => "export.product_mix",
            Self::ExportStemPackage => "export.stem_package",
            Self::ExportLiveRecording => "export.live_recording",
            Self::ExportDawSession => "export.daw_session",
            Self::GhostSetMode => "ghost.set_mode",
            Self::GhostAcceptSuggestion => "ghost.accept_suggestion",
            Self::GhostRejectSuggestion => "ghost.reject_suggestion",
            Self::GhostExecuteTool => "ghost.execute_tool",
        }
    }
}

impl Display for ActionCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionResult {
    pub accepted: bool,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UndoPolicy {
    Undoable,
    NotUndoable { reason: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub id: ActionId,
    pub actor: ActorType,
    pub command: ActionCommand,
    pub params: ActionParams,
    pub target: ActionTarget,
    pub requested_at: TimestampMs,
    pub quantization: Quantization,
    pub status: ActionStatus,
    pub committed_at: Option<TimestampMs>,
    pub result: Option<ActionResult>,
    pub undo_policy: UndoPolicy,
    pub explanation: Option<String>,
}

impl Action {
    #[must_use]
    pub fn short_label(&self) -> String {
        match &self.explanation {
            Some(explanation) if !explanation.is_empty() => {
                format!("{} ({explanation})", self.command)
            }
            _ => self.command.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionDraft {
    pub actor: ActorType,
    pub command: ActionCommand,
    pub params: ActionParams,
    pub target: ActionTarget,
    pub quantization: Quantization,
    pub undo_policy: UndoPolicy,
    pub explanation: Option<String>,
}

impl ActionDraft {
    #[must_use]
    pub fn new(
        actor: ActorType,
        command: ActionCommand,
        quantization: Quantization,
        target: ActionTarget,
    ) -> Self {
        Self {
            actor,
            command,
            params: ActionParams::Empty,
            target,
            quantization,
            undo_policy: UndoPolicy::Undoable,
            explanation: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn action_command_lexicon_labels_are_unique_and_complete() {
        assert_eq!(ActionCommand::all().len(), 60);

        let labels = ActionCommand::all()
            .iter()
            .map(|command| command.as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(labels.len(), ActionCommand::all().len());
        assert!(!labels.contains(""));
    }

    #[test]
    fn action_command_replay_coverage_is_declared_for_every_command() {
        let supported = ActionCommand::all()
            .iter()
            .filter(|command| command.replay_coverage() == ActionReplayCoverage::Supported)
            .count();
        let unsupported = ActionCommand::all().len() - supported;

        assert_eq!(supported, 42);
        assert_eq!(unsupported, 18);
    }

    #[test]
    fn product_export_action_params_default_scope_for_older_logs() {
        let params: ActionParams = serde_json::from_value(serde_json::json!({
            "ProductExport": {
                "export_role": "full_grid_mix",
                "boundary": "feral_grid_generated_support",
                "include_manifest": true,
                "destination_kind": "local_artifact_directory",
                "destination_path": "exports"
            }
        }))
        .expect("older product export params deserialize");

        assert_eq!(
            params,
            ActionParams::ProductExport {
                export_scope: ExportScope::ProductMix,
                export_role: ProductExportRole::FullGridMix,
                boundary: ProductExportBoundary::FeralGridGeneratedSupport,
                include_manifest: true,
                destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
                destination_path: Some("exports".into()),
            }
        );
    }

    #[test]
    fn stem_package_export_action_contract_roundtrips_as_reserved_scope() {
        let action = Action {
            id: ActionId(1),
            actor: ActorType::User,
            command: ActionCommand::ExportStemPackage,
            params: ActionParams::StemPackageExport {
                export_scope: ExportScope::StemPackage,
                export_role: StemPackageExportRole::PackageManifest,
                boundary: StemPackageExportBoundary::ReservedContractOnly,
                include_manifest: true,
                destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
                destination_path: Some("exports/stem-package".into()),
                claimed_stem_roles: vec![
                    ExportArtifactRole::StemDrums,
                    ExportArtifactRole::StemBass,
                ],
                lineage_policy: StemPackageLineagePolicy::RequireAnyCoreLineage,
                fallback_comparison_policy: StemPackageFallbackComparisonPolicy::Required,
            },
            target: ActionTarget {
                scope: Some(TargetScope::Session),
                ..ActionTarget::default()
            },
            requested_at: 100,
            quantization: Quantization::Immediate,
            status: ActionStatus::Requested,
            committed_at: None,
            result: None,
            undo_policy: UndoPolicy::NotUndoable {
                reason: "reserved stem-package export writes files outside musical undo".into(),
            },
            explanation: Some("reserved contract only; not runnable yet".into()),
        };

        let json = serde_json::to_value(&action).expect("serialize reserved stem action");
        assert_eq!(json["command"], "ExportStemPackage");
        assert_eq!(
            json["params"]["StemPackageExport"]["export_scope"],
            "stem_package"
        );
        assert_eq!(
            json["params"]["StemPackageExport"]["claimed_stem_roles"],
            serde_json::json!(["stem_drums", "stem_bass"])
        );
        assert_eq!(
            json["params"]["StemPackageExport"]["lineage_policy"],
            "require_any_core_lineage"
        );
        assert_eq!(
            json["params"]["StemPackageExport"]["fallback_comparison_policy"],
            "required"
        );

        let roundtrip: Action =
            serde_json::from_value(json).expect("deserialize reserved stem action");
        assert_eq!(roundtrip, action);
        assert_eq!(
            roundtrip.command.replay_coverage(),
            ActionReplayCoverage::Unsupported
        );
        let local_ci_json = serde_json::to_value(StemPackageExportBoundary::LocalCiPackageV1)
            .expect("serialize local CI boundary");
        assert_eq!(local_ci_json, "local_ci_package_v1");
        let local_ci_boundary: StemPackageExportBoundary =
            serde_json::from_value(local_ci_json).expect("deserialize local CI boundary");
        assert_eq!(
            local_ci_boundary,
            StemPackageExportBoundary::LocalCiPackageV1
        );
    }

    #[test]
    fn live_recording_export_action_contract_roundtrips_as_reserved_scope() {
        let action = Action {
            id: ActionId(3),
            actor: ActorType::User,
            command: ActionCommand::ExportLiveRecording,
            params: ActionParams::LiveRecordingExport {
                export_scope: ExportScope::LiveRecording,
                export_role: LiveRecordingExportRole::LiveRecordingCapture,
                boundary: LiveRecordingExportBoundary::ReservedContractOnly,
                include_manifest: true,
                destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
                destination_path: Some("exports/live-recording".into()),
                receipt_id: Some("export-receipt-live-42".into()),
            },
            target: ActionTarget {
                scope: Some(TargetScope::Session),
                ..ActionTarget::default()
            },
            requested_at: 110,
            quantization: Quantization::Immediate,
            status: ActionStatus::Requested,
            committed_at: None,
            result: None,
            undo_policy: UndoPolicy::NotUndoable {
                reason: "reserved live recording export writes files outside musical undo".into(),
            },
            explanation: Some("reserved live recording export contract; not runnable yet".into()),
        };

        let json = serde_json::to_value(&action).expect("serialize reserved live action");
        assert_eq!(json["command"], "ExportLiveRecording");
        assert_eq!(
            json["params"]["LiveRecordingExport"]["export_scope"],
            "live_recording"
        );
        assert_eq!(
            json["params"]["LiveRecordingExport"]["export_role"],
            "live_recording_capture"
        );
        assert_eq!(
            json["params"]["LiveRecordingExport"]["boundary"],
            "reserved_contract_only"
        );
        assert_eq!(
            json["params"]["LiveRecordingExport"]["receipt_id"],
            "export-receipt-live-42"
        );

        let roundtrip: Action =
            serde_json::from_value(json).expect("deserialize reserved live action");
        assert_eq!(roundtrip, action);
        assert_eq!(
            roundtrip.command.replay_coverage(),
            ActionReplayCoverage::Unsupported
        );
        assert_eq!(
            ActionCommand::ExportLiveRecording.as_str(),
            "export.live_recording"
        );

        let older_params: ActionParams = serde_json::from_value(serde_json::json!({
            "LiveRecordingExport": {
                "export_role": "live_recording_capture",
                "boundary": "reserved_contract_only",
                "include_manifest": true,
                "destination_kind": "local_artifact_directory",
                "destination_path": "exports/live-recording",
                "receipt_id": null
            }
        }))
        .expect("older live recording export params deserialize");
        assert_eq!(
            older_params,
            ActionParams::LiveRecordingExport {
                export_scope: ExportScope::LiveRecording,
                export_role: LiveRecordingExportRole::LiveRecordingCapture,
                boundary: LiveRecordingExportBoundary::ReservedContractOnly,
                include_manifest: true,
                destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
                destination_path: Some("exports/live-recording".into()),
                receipt_id: None,
            }
        );
    }

    #[test]
    fn daw_session_export_action_contract_roundtrips_as_reserved_scope() {
        let action = Action {
            id: ActionId(2),
            actor: ActorType::User,
            command: ActionCommand::ExportDawSession,
            params: ActionParams::DawSessionExport {
                export_scope: ExportScope::DawSession,
                boundary: DawSessionExportBoundary::ReservedContractOnly,
                include_manifest: true,
                destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
                destination_path: Some("exports/daw-session".into()),
                receipt_id: Some("export-receipt-42".into()),
            },
            target: ActionTarget {
                scope: Some(TargetScope::Session),
                ..ActionTarget::default()
            },
            requested_at: 120,
            quantization: Quantization::Immediate,
            status: ActionStatus::Requested,
            committed_at: None,
            result: None,
            undo_policy: UndoPolicy::NotUndoable {
                reason: "reserved DAW session export writes files outside musical undo".into(),
            },
            explanation: Some("reserved DAW session export contract; not runnable yet".into()),
        };

        let json = serde_json::to_value(&action).expect("serialize reserved DAW action");
        assert_eq!(json["command"], "ExportDawSession");
        assert_eq!(
            json["params"]["DawSessionExport"]["export_scope"],
            "daw_session"
        );
        assert_eq!(
            json["params"]["DawSessionExport"]["boundary"],
            "reserved_contract_only"
        );
        assert_eq!(
            json["params"]["DawSessionExport"]["receipt_id"],
            "export-receipt-42"
        );

        let roundtrip: Action =
            serde_json::from_value(json).expect("deserialize reserved DAW action");
        assert_eq!(roundtrip, action);
        assert_eq!(
            roundtrip.command.replay_coverage(),
            ActionReplayCoverage::Unsupported
        );
        let writer_json = serde_json::to_value(DawSessionExportBoundary::LocalProjectWriterV1)
            .expect("serialize local DAW writer boundary");
        assert_eq!(writer_json, "local_project_writer_v1");
        let writer_boundary: DawSessionExportBoundary =
            serde_json::from_value(writer_json).expect("deserialize local DAW writer boundary");
        assert_eq!(
            writer_boundary,
            DawSessionExportBoundary::LocalProjectWriterV1
        );
        let host_import_json = serde_json::to_value(DawSessionExportBoundary::HostImportProofV1)
            .expect("serialize DAW host import proof boundary");
        assert_eq!(host_import_json, "host_import_proof_v1");
        let host_import_boundary: DawSessionExportBoundary =
            serde_json::from_value(host_import_json)
                .expect("deserialize DAW host import proof boundary");
        assert_eq!(
            host_import_boundary,
            DawSessionExportBoundary::HostImportProofV1
        );
        let audible_output_json =
            serde_json::to_value(DawSessionExportBoundary::AudibleOutputProofV1)
                .expect("serialize DAW audible output proof boundary");
        assert_eq!(audible_output_json, "audible_output_proof_v1");
        let audible_output_boundary: DawSessionExportBoundary =
            serde_json::from_value(audible_output_json)
                .expect("deserialize DAW audible output proof boundary");
        assert_eq!(
            audible_output_boundary,
            DawSessionExportBoundary::AudibleOutputProofV1
        );
    }
}
