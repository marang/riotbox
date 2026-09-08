//! Metadata integrity of DAWproject document and live-master archive receipts.
//! No file access or host-import claim belongs to this check.

use super::{
    ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactRole, ExportReceiptQaGateStatus,
    ExportReceiptState,
    export_qa_gates::{DAWPROJECT_ARCHIVE_QA_GATE_ID, DAWPROJECT_XML_DOCUMENT_QA_GATE_ID},
};

impl ExportReceiptState {
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
        if !self.is_live_master_dawproject_v1()
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
}
