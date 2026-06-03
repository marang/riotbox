use serde::{Deserialize, Serialize};

use super::export_types::ExportReceiptState;
use crate::{
    export_readiness::{ExportScope, UnsupportedExportScope},
    ids::{SceneId, SourceId},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArrangementPlacementRef {
    pub scene_id: SceneId,
    #[serde(default)]
    pub source_id: Option<SourceId>,
    pub start_bar: u32,
    pub end_bar: u32,
    pub start_beat: u64,
    pub end_beat: u64,
}

impl ExportArrangementPlacementRef {
    #[must_use]
    pub fn scene_range(
        scene_id: impl Into<SceneId>,
        source_id: Option<SourceId>,
        start_bar: u32,
        end_bar: u32,
        start_beat: u64,
        end_beat: u64,
    ) -> Self {
        Self {
            scene_id: scene_id.into(),
            source_id,
            start_bar,
            end_bar,
            start_beat,
            end_beat,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrangementExportPlacementReadinessReport {
    pub status: ArrangementExportPlacementReadinessStatus,
    pub blockers: Vec<ArrangementExportPlacementReadinessBlocker>,
}

impl ArrangementExportPlacementReadinessReport {
    #[must_use]
    pub fn ready(&self) -> bool {
        self.status == ArrangementExportPlacementReadinessStatus::Ready
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArrangementExportPlacementReadinessStatus {
    Ready,
    Blocked,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArrangementExportPlacementReadinessBlocker {
    NotDawSessionScope,
    UnsupportedDawExportFlagPresent,
    MissingPlacementRefs,
    BlankSceneRef,
    InvalidBarRange,
    InvalidBeatRange,
}

#[must_use]
pub fn validate_arrangement_export_placement_readiness(
    receipt: &ExportReceiptState,
) -> ArrangementExportPlacementReadinessReport {
    let mut blockers = Vec::new();
    if receipt.export_scope != ExportScope::DawSession {
        blockers.push(ArrangementExportPlacementReadinessBlocker::NotDawSessionScope);
    }
    if receipt
        .unsupported_scopes
        .contains(&UnsupportedExportScope::DawExport)
    {
        blockers.push(ArrangementExportPlacementReadinessBlocker::UnsupportedDawExportFlagPresent);
    }
    if receipt.arrangement_placement_refs.is_empty() {
        blockers.push(ArrangementExportPlacementReadinessBlocker::MissingPlacementRefs);
    }

    for placement in &receipt.arrangement_placement_refs {
        if placement.scene_id.as_str().trim().is_empty() {
            blockers.push(ArrangementExportPlacementReadinessBlocker::BlankSceneRef);
        }
        if placement.start_bar == 0
            || placement.end_bar == 0
            || placement.end_bar < placement.start_bar
        {
            blockers.push(ArrangementExportPlacementReadinessBlocker::InvalidBarRange);
        }
        if placement.end_beat <= placement.start_beat {
            blockers.push(ArrangementExportPlacementReadinessBlocker::InvalidBeatRange);
        }
    }

    blockers.sort_by_key(|blocker| arrangement_placement_blocker_rank(*blocker));
    blockers.dedup();

    let status = if blockers.is_empty() {
        ArrangementExportPlacementReadinessStatus::Ready
    } else {
        ArrangementExportPlacementReadinessStatus::Blocked
    };
    ArrangementExportPlacementReadinessReport { status, blockers }
}

const fn arrangement_placement_blocker_rank(
    blocker: ArrangementExportPlacementReadinessBlocker,
) -> u8 {
    match blocker {
        ArrangementExportPlacementReadinessBlocker::NotDawSessionScope => 0,
        ArrangementExportPlacementReadinessBlocker::UnsupportedDawExportFlagPresent => 1,
        ArrangementExportPlacementReadinessBlocker::MissingPlacementRefs => 2,
        ArrangementExportPlacementReadinessBlocker::BlankSceneRef => 3,
        ArrangementExportPlacementReadinessBlocker::InvalidBarRange => 4,
        ArrangementExportPlacementReadinessBlocker::InvalidBeatRange => 5,
    }
}

#[cfg(test)]
#[path = "arrangement_export_placement_tests.rs"]
mod arrangement_export_placement_tests;
