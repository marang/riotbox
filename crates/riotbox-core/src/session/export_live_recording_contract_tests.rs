use super::*;

use crate::{
    export_readiness::{
        EXPORT_READINESS_CONTRACT_SCHEMA, LIVE_RECORDING_RECEIPT_PACK_ID, PRODUCT_EXPORT_PACK_ID,
        PRODUCT_EXPORT_PROOF_SCHEMA,
    },
    ids::ActionId,
    session::{
        ExportLiveRecordingCallbackGapSummary, ExportLiveRecordingHostAudioRef,
        ExportLiveRecordingStreamErrorSummary,
    },
};

fn live_recording_fixture_receipt() -> ExportReceiptState {
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::LiveRecording,
        boundary: ProductExportBoundary::LiveRecordingReceiptContractV1,
        pack_id: LIVE_RECORDING_RECEIPT_PACK_ID.into(),
        export_role: ProductExportRole::LiveRecordingCapture,
        export_artifact: "exports/live/recording.wav".into(),
        source_sha256: "source-sha".into(),
        export_sha256: "1212121212121212121212121212121212121212121212121212121212121212".into(),
        normalized_manifest_sha256:
            "3434343434343434343434343434343434343434343434343434343434343434".into(),
        unsupported_scopes: vec![UnsupportedExportScope::LiveRecording],
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        ActionId(1171),
        1_171,
        &contract,
        "exports/live/recording.wav",
        "exports/live/live_recording_proof.json",
        Some("exports/live/manifest.json".into()),
    );
    receipt.artifact_set = vec![
        ExportArtifactSetEntry::live_recording_capture(
            "exports/live/recording.wav",
            "1212121212121212121212121212121212121212121212121212121212121212",
        ),
        ExportArtifactSetEntry::export_manifest(
            "exports/live/manifest.json",
            "3434343434343434343434343434343434343434343434343434343434343434",
        ),
        ExportArtifactSetEntry::product_export_proof(
            "exports/live/live_recording_proof.json",
            "5656565656565656565656565656565656565656565656565656565656565656",
        ),
    ];
    receipt.qa_gates.clear();
    receipt
}

#[test]
fn live_recording_capture_artifact_entry_uses_wav_identity() {
    let entry = ExportArtifactSetEntry::live_recording_capture(
        "exports/live/recording.wav",
        "1212121212121212121212121212121212121212121212121212121212121212",
    );

    assert_eq!(entry.role, ExportArtifactRole::LiveRecordingCapture);
    assert_eq!(
        entry.location,
        ExportArtifactLocation::LocalPath {
            path: "exports/live/recording.wav".into()
        }
    );
    assert_eq!(entry.media_type, ExportArtifactMediaType::AudioWav);
    assert_eq!(
        entry.sha256,
        "1212121212121212121212121212121212121212121212121212121212121212"
    );
    assert_eq!(entry.normalized_manifest_hash, None);
    assert_eq!(entry.audio_metrics, None);
}

#[test]
fn live_recording_receipt_contract_roundtrips_without_writer_side_effects() {
    let receipt = live_recording_fixture_receipt();

    let json = serde_json::to_value(&receipt).expect("serialize live receipt");
    assert_eq!(json["export_scope"], "live_recording");
    assert_eq!(json["export_role"], "live_recording_capture");
    assert_eq!(
        json["export_boundary"],
        "live_recording_receipt_contract_v1"
    );
    assert_eq!(json["artifact_set"][0]["role"], "live_recording_capture");

    let roundtrip: ExportReceiptState = serde_json::from_value(json).expect("deserialize receipt");
    assert_eq!(roundtrip.export_scope, ExportScope::LiveRecording);
    assert_eq!(roundtrip.pack_id, LIVE_RECORDING_RECEIPT_PACK_ID);
    assert_eq!(
        roundtrip.export_role,
        ProductExportRole::LiveRecordingCapture
    );
    assert_eq!(
        roundtrip.export_boundary,
        ProductExportBoundary::LiveRecordingReceiptContractV1
    );
    assert_eq!(roundtrip.artifact_set.len(), 3);
    assert!(roundtrip.qa_gates.is_empty());
    assert!(
        roundtrip
            .unsupported_scopes
            .contains(&UnsupportedExportScope::LiveRecording)
    );
}

#[test]
fn live_recording_host_audio_refs_roundtrip_as_receipt_evidence_only() {
    let mut receipt = live_recording_fixture_receipt();
    receipt
        .live_recording_host_audio_refs
        .push(ExportLiveRecordingHostAudioRef {
            host: "Alsa".into(),
            device: "pipewire-default".into(),
            recording_duration_ms: 16_000,
            callback_gap_summary: ExportLiveRecordingCallbackGapSummary {
                max_gap_ms: Some(3),
                over_threshold_count: 0,
            },
            stream_error_summary: ExportLiveRecordingStreamErrorSummary {
                error_count: 0,
                last_error: None,
            },
        });

    let json = serde_json::to_value(&receipt).expect("serialize live receipt");
    assert_eq!(json["live_recording_host_audio_refs"][0]["host"], "Alsa");
    assert_eq!(
        json["live_recording_host_audio_refs"][0]["device"],
        "pipewire-default"
    );
    assert_eq!(
        json["live_recording_host_audio_refs"][0]["recording_duration_ms"],
        16_000
    );
    assert_eq!(
        json["live_recording_host_audio_refs"][0]["callback_gap_summary"]["max_gap_ms"],
        3
    );
    assert_eq!(
        json["live_recording_host_audio_refs"][0]["stream_error_summary"]["error_count"],
        0
    );

    let roundtrip: ExportReceiptState = serde_json::from_value(json).expect("deserialize receipt");
    assert_eq!(roundtrip.live_recording_host_audio_refs.len(), 1);
    assert_eq!(
        roundtrip.live_recording_host_audio_refs[0],
        ExportLiveRecordingHostAudioRef {
            host: "Alsa".into(),
            device: "pipewire-default".into(),
            recording_duration_ms: 16_000,
            callback_gap_summary: ExportLiveRecordingCallbackGapSummary {
                max_gap_ms: Some(3),
                over_threshold_count: 0,
            },
            stream_error_summary: ExportLiveRecordingStreamErrorSummary {
                error_count: 0,
                last_error: None,
            },
        }
    );
    assert!(roundtrip.qa_gates.is_empty());
}

#[test]
fn missing_live_recording_host_audio_refs_default_to_empty_for_older_receipts() {
    let mut json = serde_json::to_value(live_recording_fixture_receipt())
        .expect("serialize live recording receipt");
    json.as_object_mut()
        .expect("receipt json object")
        .remove("live_recording_host_audio_refs");

    let receipt: ExportReceiptState =
        serde_json::from_value(json).expect("deserialize older receipt");

    assert!(receipt.live_recording_host_audio_refs.is_empty());
}

#[test]
fn live_recording_export_contract_names_are_stable_but_not_product_mix_defaults() {
    assert_eq!(ExportScope::LiveRecording.as_str(), "live_recording");
    assert_eq!(
        ExportScope::LiveRecording.musician_label(),
        "live recording export"
    );
    assert_eq!(
        ProductExportRole::LiveRecordingCapture.as_str(),
        "live_recording_capture"
    );
    assert_eq!(
        ProductExportBoundary::LiveRecordingReceiptContractV1.as_proof_str(),
        "live_recording.receipt_contract_v1"
    );
    assert_eq!(
        LIVE_RECORDING_RECEIPT_PACK_ID,
        "live-recording-receipt-contract"
    );
    assert_eq!(PRODUCT_EXPORT_PACK_ID, "feral-grid-demo");
    assert_eq!(default_export_scope(), ExportScope::ProductMix);
}
