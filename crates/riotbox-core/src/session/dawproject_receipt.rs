//! Metadata integrity of DAWproject document and live-master archive receipts.
//! No file access or host-import claim belongs to this check.

use crate::action::{DawSessionExportBoundary, LiveRecordingDuration};

use super::{
    ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactRole, ExportReceiptQaGateStatus,
    ExportReceiptState,
    export_qa_gates::{DAWPROJECT_ARCHIVE_QA_GATE_ID, DAWPROJECT_XML_DOCUMENT_QA_GATE_ID},
};

impl ExportReceiptState {
    /// Match persisted live-master Action identity; readiness is checked separately.
    #[must_use]
    pub fn live_master_dawproject_action_contract_matches(
        &self,
        boundary: DawSessionExportBoundary,
        duration: Option<LiveRecordingDuration>,
    ) -> bool {
        boundary.valid_duration(duration)
            && self.live_recording_duration == duration
            && match boundary {
                DawSessionExportBoundary::LiveMasterDawprojectV1 => {
                    self.is_live_master_dawproject_v1()
                }
                DawSessionExportBoundary::LiveMasterDawprojectV2 => {
                    self.is_live_master_dawproject_v2()
                }
                DawSessionExportBoundary::LiveMasterDawprojectV3 => {
                    self.is_live_master_dawproject_v3()
                }
                _ => false,
            }
    }

    /// Bind a new extended archive to its admitted V3 recording without file I/O.
    /// Historical two-bar handoffs retain their existing source-validation policy.
    #[must_use]
    pub fn live_master_dawproject_source_contract_matches(&self, source: &Self) -> bool {
        if !self.is_live_master_dawproject_v3()
            || !self.live_master_dawproject_archive_ready()
            || !source.is_live_recording_runtime_master_bar_window_v3()
            || !source.live_recording_runtime_master_ready()
            || self.live_recording_duration != source.live_recording_duration
        {
            return false;
        }
        let Some(audio) = self
            .artifact_set
            .iter()
            .find(|entry| entry.role == ExportArtifactRole::LiveRecordingCapture)
        else {
            return false;
        };
        let Some(source_audio) = source
            .artifact_set
            .iter()
            .find(|entry| entry.role == ExportArtifactRole::LiveRecordingCapture)
        else {
            return false;
        };
        let source_tempo = source
            .live_recording_host_audio_refs
            .first()
            .and_then(|host| host.timing_window.as_ref())
            .map(|window| window.confirmed_bpm_micros);
        audio.sha256 == source_audio.sha256
            && audio.sample_rate_hz == source_audio.sample_rate_hz
            && audio.channel_count == source_audio.channel_count
            && audio.duration_ms == source_audio.duration_ms
            && audio.audio_metrics == source_audio.audio_metrics
            && audio.source_graph_ref == source_audio.source_graph_ref
            && audio.timing_grid_ref == source_audio.timing_grid_ref
            && audio.source_capture_refs == source_audio.source_capture_refs
            && audio.lineage_capture_refs == source_audio.lineage_capture_refs
            && self
                .daw_tempo_map_ref
                .as_ref()
                .map(|tempo| u64::from(tempo.bpm_micros))
                == source_tempo
    }

    #[must_use]
    pub fn dawproject_xml_document_ready(&self) -> bool {
        let mut matching = self
            .qa_gates
            .iter()
            .filter(|gate| gate.gate_id == DAWPROJECT_XML_DOCUMENT_QA_GATE_ID);
        let Some(gate) = matching.next() else {
            return false;
        };
        self.is_dawproject_archive_receipt()
            && matching.next().is_none()
            && gate.status == ExportReceiptQaGateStatus::Passed
            && gate.artifact_roles.len() == 2
            && gate
                .artifact_roles
                .contains(&ExportArtifactRole::DawProjectFile)
            && gate
                .artifact_roles
                .contains(&ExportArtifactRole::ExportManifest)
    }

    #[must_use]
    pub fn live_master_dawproject_archive_ready(&self) -> bool {
        use ExportArtifactMediaType::{AudioWav, DawProjectZip, Json, Xml};
        use ExportArtifactRole::{
            DawProjectFile, DawProjectProof, ExportManifest, LiveRecordingCapture,
        };
        let members = [
            (DawProjectFile, DawProjectZip, None),
            (ExportManifest, Xml, Some("project.xml")),
            (
                LiveRecordingCapture,
                AudioWav,
                Some("audio/live_master.wav"),
            ),
            (DawProjectProof, Json, Some("riotbox-proof.json")),
        ];
        if !self.is_live_master_dawproject()
            || (self.is_live_master_dawproject_v3() && !self.extended_live_master_geometry_ready())
            || !self.dawproject_xml_document_ready()
            || self.artifact_set.len() != members.len()
            || self.artifact_path.is_empty()
            || self.proof_path != self.artifact_path
            || self.manifest_path.as_deref() != Some(self.artifact_path.as_str())
        {
            return false;
        }
        let artifacts_ready = members.iter().all(|(role, media, member)| {
            let mut matching = self.artifact_set.iter().filter(|entry| entry.role == *role);
            let Some(entry) = matching.next() else {
                return false;
            };
            if matching.next().is_some()
                || entry.media_type != *media
                || entry.sha256.len() != 64
                || !entry.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return false;
            }
            let location_ready = match (&entry.location, member) {
                (ExportArtifactLocation::LocalPath { path }, None) => path == &self.artifact_path,
                (ExportArtifactLocation::Uri { uri }, Some(member)) => {
                    uri == &format!("{}#{member}", self.artifact_path)
                }
                _ => false,
            };
            location_ready
                && match role {
                    DawProjectFile => {
                        entry.sha256 == self.export_hash
                            && entry.normalized_manifest_hash.as_ref()
                                == Some(&self.normalized_manifest_hash)
                    }
                    ExportManifest => entry.sha256 == self.normalized_manifest_hash,
                    LiveRecordingCapture | DawProjectProof => true,
                    _ => false,
                }
        });
        let mut matching = self
            .qa_gates
            .iter()
            .filter(|gate| gate.gate_id == DAWPROJECT_ARCHIVE_QA_GATE_ID);
        let Some(gate) = matching.next() else {
            return false;
        };
        artifacts_ready
            && matching.next().is_none()
            && gate.status == ExportReceiptQaGateStatus::Passed
            && gate.artifact_roles.len() == members.len()
            && members
                .iter()
                .all(|(role, _, _)| gate.artifact_roles.contains(role))
    }

    fn extended_live_master_geometry_ready(&self) -> bool {
        let Some(
            duration @ (LiveRecordingDuration::EightBars | LiveRecordingDuration::SixteenBars),
        ) = self.live_recording_duration
        else {
            return false;
        };
        let [placement] = self.arrangement_placement_refs.as_slice() else {
            return false;
        };
        let Some(tempo) = &self.daw_tempo_map_ref else {
            return false;
        };
        if !self.arrangement_export_placement_report().ready()
            || !self.daw_tempo_map_report().ready()
            || placement.start_bar != 1
            || placement.end_bar != u32::from(duration.bars())
            || placement.start_beat != 0
            || placement.end_beat != u64::from(duration.duration_beats())
            || tempo.start_beat != 0
            || tempo.end_beat != placement.end_beat
            || placement.source_id.as_ref() != Some(&tempo.source_id)
        {
            return false;
        }
        let Some(audio) = self
            .artifact_set
            .iter()
            .find(|entry| entry.role == ExportArtifactRole::LiveRecordingCapture)
        else {
            return false;
        };
        let rate = audio.sample_rate_hz.unwrap_or_default();
        let Some(frames) = duration.target_frame_count(rate, u64::from(tempo.bpm_micros)) else {
            return false;
        };
        let millis = (u128::from(frames) * 1_000 + u128::from(rate) / 2)
            .checked_div(u128::from(rate))
            .and_then(|value| u64::try_from(value).ok());
        audio.channel_count.is_some_and(|channels| channels > 0)
            && audio
                .audio_metrics
                .as_ref()
                .and_then(|metrics| metrics.total_frame_count)
                == Some(frames)
            && millis.is_some()
            && audio.duration_ms == millis
            && audio.timing_grid_ref.as_ref().is_some_and(|grid| {
                grid.source_id == tempo.source_id
                    && grid.hypothesis_id == tempo.hypothesis_id
                    && grid.confirmed_by_action == tempo.confirmed_by_action
                    && grid.confirmed_at == tempo.confirmed_at
            })
    }
}
