//! Version adaptation for the two supported, already-recorded two-bar contracts.
//! Core Action/receipt boundaries remain the persisted identity.

use riotbox_core::{
    action::{DawSessionExportBoundary, LiveRecordingDuration, LiveRecordingExportBoundary},
    export_readiness::{
        LIVE_MASTER_DAWPROJECT_PACK_ID, LIVE_MASTER_DAWPROJECT_V2_PACK_ID, ProductExportBoundary,
    },
};

use super::{LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA, LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA_V2};
use crate::jam_app::live_master_recording::{
    LIVE_MASTER_RECORDING_PROOF_SCHEMA, LIVE_MASTER_RECORDING_PROOF_SCHEMA_V4,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LiveMasterDawprojectVersion {
    LegacyV1,
    CanonicalV2,
}

impl LiveMasterDawprojectVersion {
    pub(super) fn for_recording(boundary: ProductExportBoundary) -> Option<Self> {
        match boundary {
            ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2 => Some(Self::LegacyV1),
            ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4 => Some(Self::CanonicalV2),
            _ => None,
        }
    }

    pub(super) fn for_action(boundary: DawSessionExportBoundary) -> Option<Self> {
        match boundary {
            DawSessionExportBoundary::LiveMasterDawprojectV1 => Some(Self::LegacyV1),
            DawSessionExportBoundary::LiveMasterDawprojectV2 => Some(Self::CanonicalV2),
            _ => None,
        }
    }

    pub(super) fn action_boundary(self) -> DawSessionExportBoundary {
        match self {
            Self::LegacyV1 => DawSessionExportBoundary::LiveMasterDawprojectV1,
            Self::CanonicalV2 => DawSessionExportBoundary::LiveMasterDawprojectV2,
        }
    }

    pub(super) fn receipt_boundary(self) -> ProductExportBoundary {
        match self {
            Self::LegacyV1 => ProductExportBoundary::DawSessionLiveMasterDawprojectV1,
            Self::CanonicalV2 => ProductExportBoundary::DawSessionLiveMasterDawprojectV2,
        }
    }

    pub(super) fn recording_boundary(self) -> ProductExportBoundary {
        match self {
            Self::LegacyV1 => ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2,
            Self::CanonicalV2 => ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4,
        }
    }

    pub(super) fn recording_action_boundary(self) -> LiveRecordingExportBoundary {
        match self {
            Self::LegacyV1 => LiveRecordingExportBoundary::RuntimeMasterBarWindowV2,
            Self::CanonicalV2 => LiveRecordingExportBoundary::RuntimeMasterBarWindowV4,
        }
    }

    pub(super) fn recording_duration(self) -> Option<LiveRecordingDuration> {
        match self {
            Self::LegacyV1 => None,
            Self::CanonicalV2 => Some(LiveRecordingDuration::TwoBars),
        }
    }

    pub(super) fn recording_proof_schema(self) -> &'static str {
        match self {
            Self::LegacyV1 => LIVE_MASTER_RECORDING_PROOF_SCHEMA,
            Self::CanonicalV2 => LIVE_MASTER_RECORDING_PROOF_SCHEMA_V4,
        }
    }

    pub(super) fn proof_schema(self) -> &'static str {
        match self {
            Self::LegacyV1 => LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA,
            Self::CanonicalV2 => LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA_V2,
        }
    }

    pub(super) fn pack_id(self) -> &'static str {
        match self {
            Self::LegacyV1 => LIVE_MASTER_DAWPROJECT_PACK_ID,
            Self::CanonicalV2 => LIVE_MASTER_DAWPROJECT_V2_PACK_ID,
        }
    }
}
