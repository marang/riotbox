use super::*;
use crate::{
    export_qa::validate_stem_package_artifact_set_evidence,
    session::{ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactSetEntry},
};

#[test]
fn stem_package_artifact_set_gate_result_records_deferred_structure() {
    let artifact_set = vec![stem_artifact(
        ExportArtifactRole::StemDrums,
        "exports/stems/drums.wav",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )];
    let report = validate_stem_package_artifact_set_evidence(
        &artifact_set,
        &[ExportArtifactRole::StemDrums],
    );

    let gate = ExportReceiptQaGateResult::stem_package_artifact_set_evidence(&report);

    assert_eq!(gate.gate_id, STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID);
    assert_eq!(gate.status, ExportReceiptQaGateStatus::Deferred);
    assert_eq!(gate.artifact_roles, vec![ExportArtifactRole::StemDrums]);
    assert!(
        gate.summary
            .as_deref()
            .expect("summary")
            .contains("deferred QA check(s) remain")
    );
}

#[test]
fn stem_package_artifact_set_gate_result_records_failed_structure() {
    let report = validate_stem_package_artifact_set_evidence(&[], &[ExportArtifactRole::StemBass]);

    let gate = ExportReceiptQaGateResult::stem_package_artifact_set_evidence(&report);

    assert_eq!(gate.status, ExportReceiptQaGateStatus::Failed);
    assert_eq!(gate.artifact_roles, vec![ExportArtifactRole::StemBass]);
    assert!(
        gate.summary
            .as_deref()
            .expect("summary")
            .contains("evidence failed")
    );
}

#[test]
fn stem_package_artifact_set_gate_result_roundtrips() {
    let artifact_set = vec![stem_artifact(
        ExportArtifactRole::StemVocals,
        "exports/stems/vocals.wav",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    )];
    let report = validate_stem_package_artifact_set_evidence(
        &artifact_set,
        &[ExportArtifactRole::StemVocals],
    );
    let gate = ExportReceiptQaGateResult::stem_package_artifact_set_evidence(&report);

    let json = serde_json::to_string_pretty(&gate).expect("serialize gate");
    let roundtrip: ExportReceiptQaGateResult =
        serde_json::from_str(&json).expect("deserialize gate");

    assert_eq!(roundtrip, gate);
    assert_eq!(roundtrip.status, ExportReceiptQaGateStatus::Deferred);
}

#[test]
fn daw_session_json_package_gate_records_ready_and_blocked_integrity() {
    let package_roles = vec![
        ExportArtifactRole::ExportManifest,
        ExportArtifactRole::DawSessionTempoMap,
        ExportArtifactRole::ProductExportProof,
    ];

    let ready_gate = ExportReceiptQaGateResult::daw_session_json_package_integrity(
        true,
        &[],
        package_roles.clone(),
    );
    assert_eq!(ready_gate.gate_id, DAW_SESSION_JSON_PACKAGE_QA_GATE_ID);
    assert_eq!(ready_gate.status, ExportReceiptQaGateStatus::Passed);
    assert_eq!(ready_gate.artifact_roles, package_roles);

    let blocked_gate = ExportReceiptQaGateResult::daw_session_json_package_integrity(
        false,
        &["proof_manifest_hash_mismatch".into()],
        vec![ExportArtifactRole::ProductExportProof],
    );
    assert_eq!(blocked_gate.status, ExportReceiptQaGateStatus::Failed);
    assert!(
        blocked_gate
            .summary
            .as_deref()
            .expect("summary")
            .contains("proof_manifest_hash_mismatch")
    );
}

#[test]
fn daw_session_writer_gate_records_ready_and_blocked_proof() {
    let ready_gate = ExportReceiptQaGateResult::daw_session_writer_proof(
        true,
        &[],
        vec![ExportArtifactRole::DawSessionWriterProof],
    );
    assert_eq!(ready_gate.gate_id, DAW_SESSION_WRITER_QA_GATE_ID);
    assert_eq!(ready_gate.status, ExportReceiptQaGateStatus::Passed);
    assert_eq!(
        ready_gate.artifact_roles,
        vec![ExportArtifactRole::DawSessionWriterProof]
    );
    assert!(
        ready_gate
            .summary
            .as_deref()
            .expect("summary")
            .contains("accepted")
    );

    let blocked_gate = ExportReceiptQaGateResult::daw_session_writer_proof(
        false,
        &["project_skeleton_hash_mismatch".into()],
        vec![ExportArtifactRole::DawSessionWriterProof],
    );
    assert_eq!(blocked_gate.status, ExportReceiptQaGateStatus::Failed);
    assert!(
        blocked_gate
            .summary
            .as_deref()
            .expect("summary")
            .contains("project_skeleton_hash_mismatch")
    );
}

#[test]
fn daw_session_host_import_gate_records_ready_and_blocked_proof() {
    let ready_gate = ExportReceiptQaGateResult::daw_session_host_import_proof(true, &[]);
    assert_eq!(ready_gate.gate_id, DAW_SESSION_HOST_IMPORT_QA_GATE_ID);
    assert_eq!(ready_gate.status, ExportReceiptQaGateStatus::Passed);
    assert_eq!(ready_gate.artifact_roles, Vec::<ExportArtifactRole>::new());
    assert!(
        ready_gate
            .summary
            .as_deref()
            .expect("summary")
            .contains("accepted")
    );

    let blocked_gate = ExportReceiptQaGateResult::daw_session_host_import_proof(
        false,
        &["host_import_runner_missing".into()],
    );
    assert_eq!(blocked_gate.status, ExportReceiptQaGateStatus::Failed);
    assert!(
        blocked_gate
            .summary
            .as_deref()
            .expect("summary")
            .contains("host_import_runner_missing")
    );
}

#[test]
fn daw_session_audible_output_gate_records_ready_and_blocked_proof() {
    let ready_gate = ExportReceiptQaGateResult::daw_session_audible_output_proof(true, &[]);
    assert_eq!(ready_gate.gate_id, DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID);
    assert_eq!(ready_gate.status, ExportReceiptQaGateStatus::Passed);
    assert_eq!(ready_gate.artifact_roles, Vec::<ExportArtifactRole>::new());
    assert!(
        ready_gate
            .summary
            .as_deref()
            .expect("summary")
            .contains("accepted")
    );

    let blocked_gate = ExportReceiptQaGateResult::daw_session_audible_output_proof(
        false,
        &["host_audio_capture_missing".into()],
    );
    assert_eq!(blocked_gate.status, ExportReceiptQaGateStatus::Failed);
    assert!(
        blocked_gate
            .summary
            .as_deref()
            .expect("summary")
            .contains("host_audio_capture_missing")
    );
}

fn stem_artifact(
    role: ExportArtifactRole,
    path: impl Into<String>,
    sha256: impl Into<String>,
) -> ExportArtifactSetEntry {
    ExportArtifactSetEntry {
        role,
        location: ExportArtifactLocation::LocalPath { path: path.into() },
        media_type: ExportArtifactMediaType::AudioWav,
        sha256: sha256.into(),
        normalized_manifest_hash: None,
        source_graph_ref: None,
        timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: None,
        audio_metrics: None,
        sample_rate_hz: None,
        channel_count: None,
        duration_ms: None,
    }
}
