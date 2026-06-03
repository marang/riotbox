use serde::{Deserialize, Serialize};

use super::export_types::ExportReceiptState;
use crate::{
    TimestampMs,
    export_readiness::{ExportScope, UnsupportedExportScope},
    ids::{ActionId, SourceId},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportDawTempoMapRef {
    pub source_id: SourceId,
    #[serde(default)]
    pub hypothesis_id: Option<String>,
    pub confirmed_by_action: ActionId,
    pub confirmed_at: TimestampMs,
    pub start_beat: u64,
    pub end_beat: u64,
    pub bpm_micros: u32,
}

impl ExportDawTempoMapRef {
    #[must_use]
    pub fn confirmed_grid(
        source_id: impl Into<SourceId>,
        hypothesis_id: Option<String>,
        confirmed_by_action: ActionId,
        confirmed_at: TimestampMs,
        start_beat: u64,
        end_beat: u64,
        bpm_micros: u32,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            hypothesis_id,
            confirmed_by_action,
            confirmed_at,
            start_beat,
            end_beat,
            bpm_micros,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DawTempoMapReadinessReport {
    pub status: DawTempoMapReadinessStatus,
    pub blockers: Vec<DawTempoMapReadinessBlocker>,
}

impl DawTempoMapReadinessReport {
    #[must_use]
    pub fn ready(&self) -> bool {
        self.status == DawTempoMapReadinessStatus::Ready
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DawTempoMapReadinessStatus {
    Ready,
    Blocked,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DawTempoMapReadinessBlocker {
    NotDawSessionScope,
    UnsupportedDawExportFlagPresent,
    MissingTempoMapRef,
    BlankSourceRef,
    InvalidBeatRange,
    InvalidTempo,
}

#[must_use]
pub fn validate_daw_tempo_map_readiness(
    receipt: &ExportReceiptState,
) -> DawTempoMapReadinessReport {
    let mut blockers = Vec::new();
    if receipt.export_scope != ExportScope::DawSession {
        blockers.push(DawTempoMapReadinessBlocker::NotDawSessionScope);
    }
    if receipt
        .unsupported_scopes
        .contains(&UnsupportedExportScope::DawExport)
    {
        blockers.push(DawTempoMapReadinessBlocker::UnsupportedDawExportFlagPresent);
    }

    match &receipt.daw_tempo_map_ref {
        Some(tempo_map) => {
            if tempo_map.source_id.as_str().trim().is_empty() {
                blockers.push(DawTempoMapReadinessBlocker::BlankSourceRef);
            }
            if tempo_map.end_beat <= tempo_map.start_beat {
                blockers.push(DawTempoMapReadinessBlocker::InvalidBeatRange);
            }
            if tempo_map.bpm_micros == 0 {
                blockers.push(DawTempoMapReadinessBlocker::InvalidTempo);
            }
        }
        None => blockers.push(DawTempoMapReadinessBlocker::MissingTempoMapRef),
    }

    blockers.sort_by_key(|blocker| daw_tempo_map_blocker_rank(*blocker));
    blockers.dedup();

    let status = if blockers.is_empty() {
        DawTempoMapReadinessStatus::Ready
    } else {
        DawTempoMapReadinessStatus::Blocked
    };
    DawTempoMapReadinessReport { status, blockers }
}

const fn daw_tempo_map_blocker_rank(blocker: DawTempoMapReadinessBlocker) -> u8 {
    match blocker {
        DawTempoMapReadinessBlocker::NotDawSessionScope => 0,
        DawTempoMapReadinessBlocker::UnsupportedDawExportFlagPresent => 1,
        DawTempoMapReadinessBlocker::MissingTempoMapRef => 2,
        DawTempoMapReadinessBlocker::BlankSourceRef => 3,
        DawTempoMapReadinessBlocker::InvalidBeatRange => 4,
        DawTempoMapReadinessBlocker::InvalidTempo => 5,
    }
}

#[cfg(test)]
#[path = "daw_tempo_map_tests.rs"]
mod daw_tempo_map_tests;
