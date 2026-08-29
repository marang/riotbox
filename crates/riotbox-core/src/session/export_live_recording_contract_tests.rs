use super::*;

use crate::{
    export_readiness::{
        EXPORT_READINESS_CONTRACT_SCHEMA, LIVE_RECORDING_RECEIPT_PACK_ID,
        LIVE_RECORDING_RUNTIME_MASTER_PACK_ID, PRODUCT_EXPORT_PACK_ID, PRODUCT_EXPORT_PROOF_SCHEMA,
    },
    ids::ActionId,
    session::{
        ExportLiveRecordingCallbackGapSummary, ExportLiveRecordingHostAudioRef,
        ExportLiveRecordingStreamErrorSummary, LiveRecordingHostAudioReadinessBlocker,
        LiveRecordingHostAudioReadinessStatus, validate_live_recording_host_audio_readiness,
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
fn live_recording_host_audio_readiness_blocks_missing_evidence() {
    let receipt = live_recording_fixture_receipt();

    let report = receipt.live_recording_host_audio_readiness_report();

    assert_eq!(
        report.status,
        LiveRecordingHostAudioReadinessStatus::Blocked
    );
    assert_eq!(
        report.blockers,
        vec![
            LiveRecordingHostAudioReadinessBlocker::UnsupportedScopeFlagPresent,
            LiveRecordingHostAudioReadinessBlocker::MissingHostAudioEvidence,
        ]
    );
}

#[test]
fn live_recording_host_audio_readiness_blocks_bad_evidence() {
    let mut receipt = live_recording_fixture_receipt();
    receipt.unsupported_scopes.clear();
    receipt
        .live_recording_host_audio_refs
        .push(ExportLiveRecordingHostAudioRef {
            host: " ".into(),
            device: "".into(),
            recording_duration_ms: 0,
            callback_gap_summary: ExportLiveRecordingCallbackGapSummary {
                max_gap_ms: Some(98),
                over_threshold_count: 2,
            },
            stream_error_summary: ExportLiveRecordingStreamErrorSummary {
                error_count: 1,
                last_error: Some("xrun".into()),
            },
        });

    let report = validate_live_recording_host_audio_readiness(&receipt);

    assert_eq!(
        report.status,
        LiveRecordingHostAudioReadinessStatus::Blocked
    );
    assert_eq!(
        report.blockers,
        vec![
            LiveRecordingHostAudioReadinessBlocker::BlankHost,
            LiveRecordingHostAudioReadinessBlocker::BlankDevice,
            LiveRecordingHostAudioReadinessBlocker::ZeroRecordingDuration,
            LiveRecordingHostAudioReadinessBlocker::CallbackGapOverThreshold,
            LiveRecordingHostAudioReadinessBlocker::StreamErrorReported,
        ]
    );
}

#[test]
fn live_recording_host_audio_readiness_blocks_wrong_scope() {
    let mut receipt = live_recording_fixture_receipt();
    receipt.export_scope = ExportScope::ProductMix;
    receipt.unsupported_scopes.clear();
    receipt.live_recording_host_audio_refs = vec![ready_live_recording_host_audio_ref()];

    let report = validate_live_recording_host_audio_readiness(&receipt);

    assert_eq!(
        report.status,
        LiveRecordingHostAudioReadinessStatus::Blocked
    );
    assert_eq!(
        report.blockers,
        vec![LiveRecordingHostAudioReadinessBlocker::NotLiveRecordingScope]
    );
}

#[test]
fn live_recording_host_audio_readiness_passes_ready_receipt_evidence() {
    let mut receipt = live_recording_fixture_receipt();
    receipt.unsupported_scopes.clear();
    receipt.live_recording_host_audio_refs = vec![ready_live_recording_host_audio_ref()];

    let report = receipt.live_recording_host_audio_readiness_report();

    assert!(report.ready());
    assert_eq!(report.status, LiveRecordingHostAudioReadinessStatus::Ready);
    assert!(report.blockers.is_empty());
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
    assert_eq!(
        ProductExportBoundary::LiveRecordingRuntimeMasterCaptureV1.as_proof_str(),
        "live_recording.runtime_master_capture_v1"
    );
    assert_eq!(
        LIVE_RECORDING_RUNTIME_MASTER_PACK_ID,
        "live-recording-runtime-master"
    );
    assert_eq!(PRODUCT_EXPORT_PACK_ID, "feral-grid-demo");
    assert_eq!(default_export_scope(), ExportScope::ProductMix);
}

#[test]
fn runtime_master_readiness_requires_exact_receipt_artifact_and_gate_identity() {
    let receipt = runtime_master_fixture_receipt();
    assert!(receipt.live_recording_runtime_master_ready());

    let mut mismatched_hash = receipt.clone();
    mismatched_hash.export_hash =
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into();
    assert!(!mismatched_hash.live_recording_runtime_master_ready());

    let mut duplicate_gate = receipt.clone();
    duplicate_gate
        .qa_gates
        .push(ExportReceiptQaGateResult::live_recording_wav_readback());
    assert!(!duplicate_gate.live_recording_runtime_master_ready());

    let mut mismatched_duration = receipt;
    mismatched_duration.live_recording_host_audio_refs[0].recording_duration_ms += 1;
    assert!(!mismatched_duration.live_recording_runtime_master_ready());
}

fn runtime_master_fixture_receipt() -> ExportReceiptState {
    let wav_sha = "abababababababababababababababababababababababababababababababab";
    let proof_sha = "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: "riotbox.live_recording_runtime_master.v1".into(),
        export_scope: ExportScope::LiveRecording,
        boundary: ProductExportBoundary::LiveRecordingRuntimeMasterCaptureV1,
        pack_id: LIVE_RECORDING_RUNTIME_MASTER_PACK_ID.into(),
        export_role: ProductExportRole::LiveRecordingCapture,
        export_artifact: "exports/live/runtime-master.wav".into(),
        source_sha256: "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef".into(),
        export_sha256: wav_sha.into(),
        normalized_manifest_sha256: proof_sha.into(),
        unsupported_scopes: Vec::new(),
    };
    let proof_path = "exports/live/runtime-master.wav.riotbox.json";
    let mut receipt = ExportReceiptState::from_readiness_contract(
        ActionId(1485),
        1_485,
        &contract,
        "exports/live/runtime-master.wav",
        proof_path,
        Some(proof_path.into()),
    );
    let mut audio =
        ExportArtifactSetEntry::live_recording_capture("exports/live/runtime-master.wav", wav_sha);
    audio.sample_rate_hz = Some(48_000);
    audio.channel_count = Some(2);
    audio.duration_ms = Some(3_692);
    audio.audio_metrics = Some(ExportArtifactAudioMetrics {
        peak_milli_dbfs: None,
        rms_milli_dbfs: None,
        peak_amplitude_micros: Some(900_000),
        rms_amplitude_micros: Some(200_000),
        silent_frame_count: None,
        total_frame_count: Some(177_231),
    });
    receipt.artifact_set = vec![
        audio,
        ExportArtifactSetEntry::product_export_proof(proof_path, proof_sha),
    ];
    receipt.qa_gates = vec![
        ExportReceiptQaGateResult::live_recording_runtime_master_capture(),
        ExportReceiptQaGateResult::live_recording_wav_readback(),
    ];
    receipt.live_recording_host_audio_refs = vec![ExportLiveRecordingHostAudioRef {
        host: "Alsa".into(),
        device: "pipewire-default".into(),
        recording_duration_ms: 3_692,
        callback_gap_summary: ExportLiveRecordingCallbackGapSummary {
            max_gap_ms: Some(12),
            over_threshold_count: 0,
        },
        stream_error_summary: ExportLiveRecordingStreamErrorSummary {
            error_count: 0,
            last_error: None,
        },
    }];
    receipt
}

fn ready_live_recording_host_audio_ref() -> ExportLiveRecordingHostAudioRef {
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
}
