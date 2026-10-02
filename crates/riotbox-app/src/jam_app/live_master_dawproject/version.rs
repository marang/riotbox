//! Version adaptation for supported, already-recorded bounded master contracts.
//! Core Action/receipt boundaries remain the persisted identity.

use riotbox_core::{
    action::{DawSessionExportBoundary, LiveRecordingDuration, LiveRecordingExportBoundary},
    export_readiness::{
        LIVE_MASTER_DAWPROJECT_PACK_ID, LIVE_MASTER_DAWPROJECT_V2_PACK_ID,
        LIVE_MASTER_DAWPROJECT_V3_PACK_ID, ProductExportBoundary,
    },
};

use super::{
    LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA, LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA_V2,
    LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA_V3,
};
use crate::jam_app::live_master_recording::{
    LIVE_MASTER_RECORDING_PROOF_SCHEMA, LIVE_MASTER_RECORDING_PROOF_SCHEMA_V3,
    LIVE_MASTER_RECORDING_PROOF_SCHEMA_V4,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LiveMasterDawprojectVersion {
    LegacyV1,
    CanonicalV2,
    ExtendedV3(LiveRecordingDuration),
}

impl LiveMasterDawprojectVersion {
    pub(super) fn supports_recording(boundary: ProductExportBoundary) -> bool {
        matches!(
            boundary,
            ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2
                | ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV3
                | ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4
        )
    }

    pub(super) fn for_recording(
        boundary: ProductExportBoundary,
        duration: Option<LiveRecordingDuration>,
    ) -> Option<Self> {
        match (boundary, duration) {
            (ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2, None) => {
                Some(Self::LegacyV1)
            }
            (
                ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4,
                Some(LiveRecordingDuration::TwoBars),
            ) => Some(Self::CanonicalV2),
            (
                ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV3,
                Some(
                    duration @ (LiveRecordingDuration::EightBars
                    | LiveRecordingDuration::SixteenBars),
                ),
            ) => Some(Self::ExtendedV3(duration)),
            _ => None,
        }
    }

    pub(super) fn for_action(
        boundary: DawSessionExportBoundary,
        duration: Option<LiveRecordingDuration>,
    ) -> Option<Self> {
        match (boundary, duration) {
            (DawSessionExportBoundary::LiveMasterDawprojectV1, None) => Some(Self::LegacyV1),
            (DawSessionExportBoundary::LiveMasterDawprojectV2, None) => Some(Self::CanonicalV2),
            (
                DawSessionExportBoundary::LiveMasterDawprojectV3,
                Some(
                    duration @ (LiveRecordingDuration::EightBars
                    | LiveRecordingDuration::SixteenBars),
                ),
            ) => Some(Self::ExtendedV3(duration)),
            _ => None,
        }
    }

    pub(super) fn action_boundary(self) -> DawSessionExportBoundary {
        match self {
            Self::LegacyV1 => DawSessionExportBoundary::LiveMasterDawprojectV1,
            Self::CanonicalV2 => DawSessionExportBoundary::LiveMasterDawprojectV2,
            Self::ExtendedV3(_) => DawSessionExportBoundary::LiveMasterDawprojectV3,
        }
    }

    pub(super) fn receipt_boundary(self) -> ProductExportBoundary {
        match self {
            Self::LegacyV1 => ProductExportBoundary::DawSessionLiveMasterDawprojectV1,
            Self::CanonicalV2 => ProductExportBoundary::DawSessionLiveMasterDawprojectV2,
            Self::ExtendedV3(_) => ProductExportBoundary::DawSessionLiveMasterDawprojectV3,
        }
    }

    pub(super) fn recording_boundary(self) -> ProductExportBoundary {
        match self {
            Self::LegacyV1 => ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2,
            Self::CanonicalV2 => ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4,
            Self::ExtendedV3(_) => ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV3,
        }
    }

    pub(super) fn recording_action_boundary(self) -> LiveRecordingExportBoundary {
        match self {
            Self::LegacyV1 => LiveRecordingExportBoundary::RuntimeMasterBarWindowV2,
            Self::CanonicalV2 => LiveRecordingExportBoundary::RuntimeMasterBarWindowV4,
            Self::ExtendedV3(_) => LiveRecordingExportBoundary::RuntimeMasterBarWindowV3,
        }
    }

    pub(super) fn recording_duration(self) -> Option<LiveRecordingDuration> {
        match self {
            Self::LegacyV1 => None,
            Self::CanonicalV2 => Some(LiveRecordingDuration::TwoBars),
            Self::ExtendedV3(duration) => Some(duration),
        }
    }

    /// Only the new DAW contract persists duration; older DAW identities stay unchanged.
    pub(super) fn export_duration(self) -> Option<LiveRecordingDuration> {
        match self {
            Self::ExtendedV3(duration) => Some(duration),
            Self::LegacyV1 | Self::CanonicalV2 => None,
        }
    }

    pub(super) fn duration(self) -> LiveRecordingDuration {
        self.recording_duration()
            .unwrap_or(LiveRecordingDuration::TwoBars)
    }

    pub(super) fn recording_proof_schema(self) -> &'static str {
        match self {
            Self::LegacyV1 => LIVE_MASTER_RECORDING_PROOF_SCHEMA,
            Self::CanonicalV2 => LIVE_MASTER_RECORDING_PROOF_SCHEMA_V4,
            Self::ExtendedV3(_) => LIVE_MASTER_RECORDING_PROOF_SCHEMA_V3,
        }
    }

    pub(super) fn proof_schema(self) -> &'static str {
        match self {
            Self::LegacyV1 => LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA,
            Self::CanonicalV2 => LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA_V2,
            Self::ExtendedV3(_) => LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA_V3,
        }
    }

    pub(super) fn pack_id(self) -> &'static str {
        match self {
            Self::LegacyV1 => LIVE_MASTER_DAWPROJECT_PACK_ID,
            Self::CanonicalV2 => LIVE_MASTER_DAWPROJECT_V2_PACK_ID,
            Self::ExtendedV3(_) => LIVE_MASTER_DAWPROJECT_V3_PACK_ID,
        }
    }
}
