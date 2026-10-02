use crate::ui::ShellLaunchMode;
use riotbox_core::action::LiveRecordingDuration;
use riotbox_core::action::LiveRecordingExportBoundary;
use riotbox_core::session::ExportArtifactRole;
use std::path::PathBuf;

pub(in crate::cli) fn live_master_recording_boundary(
    duration: LiveRecordingDuration,
) -> LiveRecordingExportBoundary {
    match duration {
        LiveRecordingDuration::TwoBars => LiveRecordingExportBoundary::RuntimeMasterBarWindowV2,
        LiveRecordingDuration::EightBars | LiveRecordingDuration::SixteenBars => {
            LiveRecordingExportBoundary::RuntimeMasterBarWindowV3
        }
    }
}

#[derive(Clone, Debug)]
pub(in crate::cli) enum LaunchMode {
    Load {
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        product_mix_export_handoff: Option<ProductMixExportHandoff>,
    },
    Ingest {
        source_path: PathBuf,
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        sidecar_script_path: PathBuf,
        analysis_seed: u64,
        explicit_source_bpm: Option<f32>,
        explicit_source_downbeat_seconds: Option<f32>,
        product_mix_export_handoff: Option<ProductMixExportHandoff>,
    },
    StemPackageLocalCiDryRun {
        destination_path: PathBuf,
        claimed_stem_roles: Vec<ExportArtifactRole>,
    },
    StemPackageLocalCiExecute {
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        destination_path: PathBuf,
        claimed_stem_roles: Vec<ExportArtifactRole>,
    },
    StemPackageSourceMatchedExecute {
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        handoff_proof_path: PathBuf,
        destination_path: PathBuf,
    },
    StemPackageW30HookExecute {
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        destination_path: PathBuf,
    },
    StemPackageLocalCiReport {
        session_path: PathBuf,
    },
    LiveRecordingReadinessReport {
        session_path: PathBuf,
    },
    LiveMasterRecordingExecute {
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        destination_path: PathBuf,
        duration: LiveRecordingDuration,
    },
    DawExportReadinessReport {
        session_path: PathBuf,
    },
    DawSessionJsonPackageExecute {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    DawSessionJsonPackageEvidenceApply {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    DawSessionHostImportProofApply {
        session_path: PathBuf,
        proof_path: PathBuf,
    },
    DawSessionHostImportProofExportExecute {
        session_path: PathBuf,
        proof_path: PathBuf,
    },
    DawSessionAudibleOutputProofApply {
        session_path: PathBuf,
        proof_path: PathBuf,
    },
    DawSessionWriterProofExecute {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    DawSessionWriterProofApply {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    DawSessionWriterExportExecute {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    W30HookDawprojectExecute {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    LiveMasterDawprojectExecute {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    DawSessionWriterPlan {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::cli) struct ProductMixExportHandoff {
    pub(in crate::cli) proof_path: PathBuf,
    pub(in crate::cli) destination_path: PathBuf,
}

impl LaunchMode {
    pub(in crate::cli) fn product_mix_export_handoff(&self) -> Option<&ProductMixExportHandoff> {
        match self {
            Self::Load {
                product_mix_export_handoff,
                ..
            }
            | Self::Ingest {
                product_mix_export_handoff,
                ..
            } => product_mix_export_handoff.as_ref(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(in crate::cli) struct AppLaunch {
    pub(in crate::cli) mode: LaunchMode,
    pub(in crate::cli) observer_path: Option<PathBuf>,
}

impl LaunchMode {
    pub(in crate::cli) fn shell_launch_mode(&self) -> ShellLaunchMode {
        match self {
            Self::Load { .. } => ShellLaunchMode::Load,
            Self::Ingest { .. } => ShellLaunchMode::Ingest,
            Self::StemPackageLocalCiDryRun { .. }
            | Self::StemPackageLocalCiExecute { .. }
            | Self::StemPackageSourceMatchedExecute { .. }
            | Self::StemPackageW30HookExecute { .. }
            | Self::StemPackageLocalCiReport { .. }
            | Self::LiveRecordingReadinessReport { .. }
            | Self::LiveMasterRecordingExecute { .. }
            | Self::DawExportReadinessReport { .. }
            | Self::DawSessionJsonPackageExecute { .. }
            | Self::DawSessionJsonPackageEvidenceApply { .. }
            | Self::DawSessionHostImportProofApply { .. }
            | Self::DawSessionHostImportProofExportExecute { .. }
            | Self::DawSessionAudibleOutputProofApply { .. }
            | Self::DawSessionWriterProofExecute { .. }
            | Self::DawSessionWriterProofApply { .. }
            | Self::DawSessionWriterExportExecute { .. }
            | Self::W30HookDawprojectExecute { .. }
            | Self::LiveMasterDawprojectExecute { .. }
            | Self::DawSessionWriterPlan { .. } => ShellLaunchMode::Load,
        }
    }
}
