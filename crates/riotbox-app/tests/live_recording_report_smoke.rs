use std::process::Command;

use riotbox_core::{
    export_readiness::{
        EXPORT_READINESS_CONTRACT_SCHEMA, ExportReadinessContract, ExportReadinessStatus,
        ExportScope, LIVE_RECORDING_RECEIPT_PACK_ID, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportRole, UnsupportedExportScope,
    },
    ids::ActionId,
    persistence::save_session_json,
    session::{
        ExportLiveRecordingCallbackGapSummary, ExportLiveRecordingHostAudioRef,
        ExportLiveRecordingStreamErrorSummary, ExportReceiptState, SessionFile,
    },
};
use serde_json::Value;

#[test]
fn live_recording_readiness_report_smoke_covers_ready_and_blocked_receipts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let mut session = SessionFile::new(
        "live-recording-report-smoke",
        "riotbox-test",
        "2026-06-04T08:30:00Z",
    );
    session
        .export_receipts
        .push(ready_live_recording_receipt(ActionId(177)));
    save_session_json(&session_path, &session).expect("save smoke session");

    let ready_report = run_report(&session_path);
    assert_eq!(ready_report["status"], "ready");
    assert_eq!(ready_report["ready"], true);
    assert_eq!(ready_report["writes_files"], false);
    assert_eq!(
        ready_report["developer_proof_status"],
        "live_recording_host_audio_ready"
    );
    assert_eq!(
        ready_report["musician_export_readiness"],
        "not_runnable_live_recording_export"
    );
    assert_eq!(ready_report["readiness_blockers"], Value::Array(Vec::new()));
    assert_eq!(
        ready_report["receipt"]["pack_id"],
        LIVE_RECORDING_RECEIPT_PACK_ID
    );
    assert_eq!(
        ready_report["receipt"]["export_role"],
        "live_recording_capture"
    );

    let latest = session.export_receipts.last_mut().expect("receipt");
    latest.live_recording_host_audio_refs.clear();
    latest
        .unsupported_scopes
        .push(UnsupportedExportScope::LiveRecording);
    save_session_json(&session_path, &session).expect("save blocked smoke session");

    let blocked_report = run_report(&session_path);
    assert_eq!(blocked_report["status"], "blocked");
    assert_eq!(blocked_report["ready"], false);
    assert_eq!(
        blocked_report["developer_proof_status"],
        "live_recording_host_audio_blocked"
    );
    assert_eq!(
        blocked_report["readiness_blockers"],
        Value::Array(vec![
            "unsupported_scope_flag_present".into(),
            "missing_host_audio_evidence".into(),
        ])
    );
}

fn run_report(session_path: &std::path::Path) -> Value {
    run_riotbox_app_json([
        "--live-recording-readiness-report",
        "--session",
        session_path.to_str().expect("session path"),
    ])
}

fn run_riotbox_app_json<const N: usize>(args: [&str; N]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_riotbox-app"))
        .args(args)
        .output()
        .expect("run riotbox-app");
    if !output.status.success() {
        panic!(
            "riotbox-app failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout).expect("parse riotbox-app stdout json")
}

fn ready_live_recording_receipt(created_by_action: ActionId) -> ExportReceiptState {
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::LiveRecording,
        boundary: ProductExportBoundary::LiveRecordingReceiptContractV1,
        pack_id: LIVE_RECORDING_RECEIPT_PACK_ID.into(),
        export_role: ProductExportRole::LiveRecordingCapture,
        export_artifact: "live-recording.wav".into(),
        source_sha256: "source-sha".into(),
        export_sha256: "export-sha".into(),
        normalized_manifest_sha256: "manifest-sha".into(),
        unsupported_scopes: Vec::new(),
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        created_by_action,
        117_700,
        &contract,
        "live-recording.wav",
        "live-recording-proof.json",
        None,
    );
    receipt.live_recording_host_audio_refs = vec![ExportLiveRecordingHostAudioRef {
        host: "Alsa".into(),
        device: "PipeWire default".into(),
        recording_duration_ms: 2_000,
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
