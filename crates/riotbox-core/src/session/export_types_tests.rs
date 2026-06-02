use super::*;

use crate::{
    export_qa::validate_stem_package_artifact_set_evidence,
    export_readiness::{
        EXPORT_READINESS_CONTRACT_SCHEMA, PRODUCT_EXPORT_PACK_ID, PRODUCT_EXPORT_PROOF_SCHEMA,
    },
    ids::ActionId,
    session::SessionFile,
};

fn fixture_contract() -> ExportReadinessContract {
    ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::ProductMix,
        boundary: ProductExportBoundary::FeralGridGeneratedSupport,
        pack_id: PRODUCT_EXPORT_PACK_ID.into(),
        export_role: ProductExportRole::FullGridMix,
        export_artifact: "run-a/full_grid_mix.wav".into(),
        source_sha256: "eeee".into(),
        export_sha256: "aaaa".into(),
        normalized_manifest_sha256: "dddd".into(),
        unsupported_scopes: vec![UnsupportedExportScope::StemPackage],
    }
}

fn fixture_receipt() -> ExportReceiptState {
    ExportReceiptState::from_readiness_contract(
        ActionId(7),
        900,
        &fixture_contract(),
        "exports/full_grid_mix.wav",
        "exports/product_export_proof.json",
        Some("exports/manifest.json".into()),
    )
}

#[test]
fn product_export_proof_artifact_entry_uses_json_media_type() {
    let entry = ExportArtifactSetEntry::product_export_proof(
        "exports/product_export_proof.json",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );

    assert_eq!(entry.role, ExportArtifactRole::ProductExportProof);
    assert_eq!(
        entry.location,
        ExportArtifactLocation::LocalPath {
            path: "exports/product_export_proof.json".into()
        }
    );
    assert_eq!(entry.media_type, ExportArtifactMediaType::Json);
    assert_eq!(
        entry.sha256,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    assert_eq!(entry.audio_metrics, None);
}

#[test]
fn export_receipts_roundtrip_with_session_file() {
    let mut session = SessionFile::new("session-export", "0.1.0", "2026-05-31T00:00:00Z");
    session.export_receipts.push(fixture_receipt());

    let json = serde_json::to_string_pretty(&session).expect("serialize session");
    let roundtrip: SessionFile = serde_json::from_str(&json).expect("deserialize session");

    assert_eq!(roundtrip.export_receipts.len(), 1);
    let receipt = &roundtrip.export_receipts[0];
    assert_eq!(
        receipt.receipt_id,
        ExportReceiptId::from("export-receipt-a-0007")
    );
    assert_eq!(receipt.created_by_action, ActionId(7));
    assert_eq!(receipt.export_scope, ExportScope::ProductMix);
    assert_eq!(receipt.pack_id, PRODUCT_EXPORT_PACK_ID);
    assert_eq!(receipt.export_role, ProductExportRole::FullGridMix);
    assert_eq!(
        receipt.export_boundary,
        ProductExportBoundary::FeralGridGeneratedSupport
    );
    assert_eq!(
        receipt.unsupported_scopes,
        vec![UnsupportedExportScope::StemPackage]
    );
    assert_eq!(
        receipt.artifact_set,
        vec![ExportArtifactSetEntry::product_mix(
            "exports/full_grid_mix.wav",
            "aaaa",
            Some("dddd".into()),
        )]
    );
    assert_eq!(
        receipt.qa_gates,
        vec![ExportReceiptQaGateResult::product_export_reproducibility()]
    );
}

#[test]
fn missing_export_receipts_default_to_empty_for_older_sessions() {
    let session = SessionFile::new("old-session", "0.1.0", "2026-05-31T00:00:00Z");
    let mut json = serde_json::to_value(&session).expect("serialize session");
    json.as_object_mut()
        .expect("session json object")
        .remove("export_receipts");

    let session: SessionFile = serde_json::from_value(json).expect("deserialize older session");

    assert!(session.export_receipts.is_empty());
}

#[test]
fn missing_artifact_set_defaults_to_empty_for_older_receipts() {
    let mut receipt = fixture_receipt();
    receipt.artifact_set.clear();
    let mut json = serde_json::to_value(&receipt).expect("serialize receipt");
    json.as_object_mut()
        .expect("receipt json object")
        .remove("artifact_set");

    let receipt: ExportReceiptState =
        serde_json::from_value(json).expect("deserialize older receipt");

    assert!(receipt.artifact_set.is_empty());
    assert_eq!(
        receipt.artifact_set_or_legacy(),
        vec![ExportArtifactSetEntry::product_mix(
            "exports/full_grid_mix.wav",
            "aaaa",
            Some("dddd".into()),
        )]
    );
}

#[test]
fn missing_export_scope_defaults_to_product_mix_for_older_receipts() {
    let mut json = serde_json::to_value(fixture_receipt()).expect("serialize receipt");
    json.as_object_mut()
        .expect("receipt json object")
        .remove("export_scope");

    let receipt: ExportReceiptState =
        serde_json::from_value(json).expect("deserialize older receipt");

    assert_eq!(receipt.export_scope, ExportScope::ProductMix);
}

#[test]
fn missing_pack_id_defaults_to_product_export_pack_for_older_receipts() {
    let mut json = serde_json::to_value(fixture_receipt()).expect("serialize receipt");
    json.as_object_mut()
        .expect("receipt json object")
        .remove("pack_id");

    let receipt: ExportReceiptState =
        serde_json::from_value(json).expect("deserialize older receipt");

    assert_eq!(receipt.pack_id, PRODUCT_EXPORT_PACK_ID);
}

#[test]
fn missing_qa_gates_defaults_to_empty_for_older_receipts() {
    let mut json = serde_json::to_value(fixture_receipt()).expect("serialize receipt");
    json.as_object_mut()
        .expect("receipt json object")
        .remove("qa_gates");

    let receipt: ExportReceiptState =
        serde_json::from_value(json).expect("deserialize older receipt");

    assert!(receipt.qa_gates.is_empty());
}

#[test]
fn product_export_receipt_records_reproducibility_gate_result() {
    let receipt = fixture_receipt();

    assert_eq!(
        receipt.qa_gates,
        vec![ExportReceiptQaGateResult {
            gate_id: PRODUCT_EXPORT_REPRODUCIBILITY_QA_GATE_ID.into(),
            status: ExportReceiptQaGateStatus::Passed,
            artifact_roles: vec![ExportArtifactRole::FullGridMix],
            summary: Some("product export proof and artifact hash accepted".into()),
        }]
    );
}

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
fn stem_package_artifact_set_gate_result_roundtrips_in_session_receipt() {
    let artifact_set = vec![stem_artifact(
        ExportArtifactRole::StemVocals,
        "exports/stems/vocals.wav",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    )];
    let report = validate_stem_package_artifact_set_evidence(
        &artifact_set,
        &[ExportArtifactRole::StemVocals],
    );
    let mut receipt = fixture_receipt();
    receipt.qa_gates = vec![ExportReceiptQaGateResult::stem_package_artifact_set_evidence(&report)];

    let json = serde_json::to_string_pretty(&receipt).expect("serialize receipt");
    let roundtrip: ExportReceiptState = serde_json::from_str(&json).expect("deserialize receipt");

    assert_eq!(roundtrip.qa_gates, receipt.qa_gates);
    assert_eq!(
        roundtrip.qa_gates[0].status,
        ExportReceiptQaGateStatus::Deferred
    );
}

#[test]
fn receipt_can_attach_source_graph_ref_to_matching_artifact_role() {
    let mut receipt = fixture_receipt();
    let source_graph_ref = ExportArtifactSourceGraphRef {
        source_id: SourceId::from("src-1"),
        graph_version: SourceGraphVersion::V1,
        graph_hash: "graph-hash-1".into(),
    };

    receipt.attach_artifact_source_graph_ref(
        ExportArtifactRole::FullGridMix,
        source_graph_ref.clone(),
    );

    assert_eq!(
        receipt.artifact_set[0].source_graph_ref,
        Some(source_graph_ref)
    );
}

#[test]
fn receipt_can_attach_timing_grid_ref_to_matching_artifact_role() {
    let mut receipt = fixture_receipt();
    let timing_grid_ref = ExportArtifactTimingGridRef {
        source_id: SourceId::from("src-1"),
        hypothesis_id: Some("primary-grid".into()),
        confirmed_by_action: ActionId(7),
        confirmed_at: 900,
    };

    receipt
        .attach_artifact_timing_grid_ref(ExportArtifactRole::FullGridMix, timing_grid_ref.clone());

    assert_eq!(
        receipt.artifact_set[0].timing_grid_ref,
        Some(timing_grid_ref)
    );
}

#[test]
fn artifact_set_entries_roundtrip_optional_audio_metrics() {
    let entry = ExportArtifactSetEntry {
        role: ExportArtifactRole::StemDrums,
        location: ExportArtifactLocation::LocalPath {
            path: "exports/stems/drums.wav".into(),
        },
        media_type: ExportArtifactMediaType::AudioWav,
        sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        normalized_manifest_hash: None,
        source_graph_ref: None,
        timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: None,
        audio_metrics: Some(ExportArtifactAudioMetrics {
            peak_milli_dbfs: Some(-120),
            rms_milli_dbfs: Some(-6_000),
            peak_amplitude_micros: Some(986_000),
            rms_amplitude_micros: Some(125_000),
            silent_frame_count: Some(0),
            total_frame_count: Some(96_000),
        }),
        sample_rate_hz: Some(48_000),
        channel_count: Some(2),
        duration_ms: Some(2_000),
    };

    let json = serde_json::to_string_pretty(&entry).expect("serialize artifact entry");
    let roundtrip: ExportArtifactSetEntry =
        serde_json::from_str(&json).expect("deserialize artifact entry");

    assert_eq!(roundtrip, entry);
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

#[test]
fn artifact_set_entries_roundtrip_source_and_capture_lineage_refs() {
    let entry = ExportArtifactSetEntry {
        role: ExportArtifactRole::StemDrums,
        location: ExportArtifactLocation::LocalPath {
            path: "exports/stems/drums.wav".into(),
        },
        media_type: ExportArtifactMediaType::AudioWav,
        sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        normalized_manifest_hash: Some("manifest-hash-1".into()),
        source_graph_ref: Some(ExportArtifactSourceGraphRef {
            source_id: SourceId::from("src-1"),
            graph_version: SourceGraphVersion::V1,
            graph_hash: "graph-hash-1".into(),
        }),
        timing_grid_ref: None,
        source_capture_refs: vec![CaptureId::from("cap-source")],
        lineage_capture_refs: vec![CaptureId::from("cap-root"), CaptureId::from("cap-print")],
        fallback_comparison: Some(ExportArtifactFallbackComparisonEvidence {
            comparison_kind: ExportArtifactFallbackComparisonKind::SourceVsFallback,
            reference_identity: "fallback://stem-drums".into(),
            rms_difference_micros: Some(125_000),
            normalized_correlation_micros: Some(420_000),
        }),
        audio_metrics: None,
        sample_rate_hz: Some(48_000),
        channel_count: Some(2),
        duration_ms: Some(2_000),
    };

    let json = serde_json::to_string_pretty(&entry).expect("serialize artifact entry");
    let roundtrip: ExportArtifactSetEntry =
        serde_json::from_str(&json).expect("deserialize artifact entry");

    assert_eq!(roundtrip, entry);
}

#[test]
fn missing_optional_evidence_defaults_for_older_artifact_entries() {
    let entry: ExportArtifactSetEntry = serde_json::from_value(serde_json::json!({
        "role": "stem_drums",
        "location": {
            "kind": "local_path",
            "path": "exports/stems/drums.wav"
        },
        "media_type": "audio_wav",
        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    }))
    .expect("deserialize older artifact entry");

    assert_eq!(entry.audio_metrics, None);
    assert_eq!(entry.source_graph_ref, None);
    assert_eq!(entry.timing_grid_ref, None);
    assert!(entry.source_capture_refs.is_empty());
    assert!(entry.lineage_capture_refs.is_empty());
    assert_eq!(entry.fallback_comparison, None);
    assert_eq!(entry.normalized_manifest_hash, None);
}
