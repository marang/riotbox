use crate::cli::args::parse_args;
use crate::cli::live_recording_report::live_recording_host_audio_ref_summary;
use crate::cli::live_recording_report::live_recording_readiness_report_summary;
use crate::cli::live_recording_report::live_recording_receipt_summary;
use crate::cli::model::AppLaunch;
use crate::cli::model::LaunchMode;
use riotbox_core::action::LiveRecordingDuration;
use riotbox_core::export_readiness::EXPORT_READINESS_CONTRACT_SCHEMA;
use riotbox_core::export_readiness::ExportReadinessContract;
use riotbox_core::export_readiness::ExportReadinessStatus;
use riotbox_core::export_readiness::ExportScope;
use riotbox_core::export_readiness::LIVE_RECORDING_RECEIPT_PACK_ID;
use riotbox_core::export_readiness::PRODUCT_EXPORT_PROOF_SCHEMA;
use riotbox_core::export_readiness::ProductExportBoundary;
use riotbox_core::export_readiness::ProductExportRole;
use riotbox_core::export_readiness::UnsupportedExportScope;
use riotbox_core::ids::ActionId;
use riotbox_core::persistence::save_session_json;
use riotbox_core::session::ExportLiveRecordingCallbackGapSummary;
use riotbox_core::session::ExportLiveRecordingHostAudioRef;
use riotbox_core::session::ExportLiveRecordingStreamErrorSummary;
use riotbox_core::session::ExportLiveRecordingTimingWindow;
use riotbox_core::session::ExportReceiptState;
use riotbox_core::session::SessionFile;
use serde_json::json;
use std::path::PathBuf;

#[test]
fn live_recording_report_projects_v3_duration_and_exact_timing_geometry() {
    for duration in [
        LiveRecordingDuration::EightBars,
        LiveRecordingDuration::SixteenBars,
    ] {
        let mut receipt = ready_live_recording_receipt(ActionId(1551));
        receipt.export_boundary = ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV3;
        receipt.live_recording_duration = Some(duration);
        let summary = live_recording_receipt_summary(&receipt);
        assert_eq!(summary["live_recording_duration"], json!(duration));
        assert_eq!(
            summary["export_boundary"],
            "live_recording.runtime_master_bar_window_v3"
        );

        let evidence = &mut receipt.live_recording_host_audio_refs[0];
        evidence.timing_window = Some(ExportLiveRecordingTimingWindow {
            confirmed_bpm_micros: 120_000_000,
            bar_grid_anchor_position_microbeats: 0,
            beat_span_per_frame_nanobeats: 41_667,
            requested_start_position_microbeats: 4_000_000,
            captured_start_position_microbeats: 4_000_000,
            captured_end_position_microbeats: 4_000_000
                + u64::from(duration.duration_beats()) * 1_000_000,
            start_alignment_error_frame_micros: 0,
            duration_error_frame_micros: 0,
            beats_per_bar: 4,
            duration_beats: duration.duration_beats(),
        });
        let host_summary = live_recording_host_audio_ref_summary(evidence);
        assert_eq!(host_summary["timing_window"], json!(evidence.timing_window));
        assert_eq!(
            host_summary["timing_window"]["duration_beats"],
            json!(duration.duration_beats())
        );
    }
}

#[test]
fn parse_args_builds_live_recording_readiness_report_mode() {
    let launch = parse_args([
        "--live-recording-readiness-report".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect("parse live recording readiness report mode");

    assert_eq!(launch.observer_path, None);
    match launch.mode {
        LaunchMode::LiveRecordingReadinessReport { session_path } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
        }
        LaunchMode::Load { .. }
        | LaunchMode::Ingest { .. }
        | LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageSourceMatchedExecute { .. }
        | LaunchMode::StemPackageW30HookExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. }
        | LaunchMode::LiveMasterRecordingExecute { .. }
        | LaunchMode::DawExportReadinessReport { .. }
        | LaunchMode::DawSessionJsonPackageExecute { .. }
        | LaunchMode::DawSessionJsonPackageEvidenceApply { .. }
        | LaunchMode::DawSessionHostImportProofApply { .. }
        | LaunchMode::DawSessionHostImportProofExportExecute { .. }
        | LaunchMode::DawSessionAudibleOutputProofApply { .. }
        | LaunchMode::DawSessionWriterProofExecute { .. }
        | LaunchMode::DawSessionWriterProofApply { .. }
        | LaunchMode::DawSessionWriterExportExecute { .. }
        | LaunchMode::W30HookDawprojectExecute { .. }
        | LaunchMode::LiveMasterDawprojectExecute { .. }
        | LaunchMode::DawSessionWriterPlan { .. } => {
            panic!("expected live recording readiness report mode")
        }
    }
}

#[test]
fn parse_args_rejects_live_recording_report_without_session_or_with_write_args() {
    let missing_session =
        parse_args(["--live-recording-readiness-report".into()]).expect_err("session is required");
    assert!(missing_session.contains("--session"));

    let observer_arg = parse_args([
        "--live-recording-readiness-report".into(),
        "--session".into(),
        "session.json".into(),
        "--observer".into(),
        "observer.ndjson".into(),
    ])
    .expect_err("report should not write observer files");
    assert!(observer_arg.contains("reads only an explicit session"));

    let destination_arg = parse_args([
        "--live-recording-readiness-report".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-session".into(),
    ])
    .expect_err("report should not accept write-shaped args");
    assert!(destination_arg.contains("reads only an explicit session"));
}

#[test]
fn live_recording_readiness_report_blocks_without_live_receipt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    save_session_json(
        &session_path,
        &SessionFile::new(
            "live-report-session",
            "riotbox-test",
            "2026-06-04T08:00:00Z",
        ),
    )
    .expect("save session");
    let launch = AppLaunch {
        mode: LaunchMode::LiveRecordingReadinessReport { session_path },
        observer_path: None,
    };

    let summary = live_recording_readiness_report_summary(&launch).expect("report summary");

    assert_eq!(summary["mode"], "live_recording_readiness_report");
    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["writes_files"], false);
    assert_eq!(
        summary["developer_proof_status"],
        "no_live_recording_receipt"
    );
    assert_eq!(
        summary["musician_export_readiness"],
        "not_runnable_live_recording_export"
    );
    assert_eq!(
        summary["readiness_blockers"],
        json!(["no_live_recording_receipt"])
    );
    assert_eq!(summary["live_recording_receipt_count"], 0);
    assert_eq!(summary["host_audio_refs"], json!([]));
}

#[test]
fn live_recording_readiness_report_summarizes_ready_host_audio_evidence() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let mut session = SessionFile::new(
        "ready-live-report-session",
        "riotbox-test",
        "2026-06-04T08:01:00Z",
    );
    session
        .export_receipts
        .push(ready_live_recording_receipt(ActionId(1177)));
    save_session_json(&session_path, &session).expect("save session");
    let launch = AppLaunch {
        mode: LaunchMode::LiveRecordingReadinessReport { session_path },
        observer_path: None,
    };

    let summary = live_recording_readiness_report_summary(&launch).expect("report summary");

    assert_eq!(summary["status"], "ready");
    assert_eq!(summary["ready"], true);
    assert_eq!(summary["writes_files"], false);
    assert_eq!(
        summary["developer_proof_status"],
        "live_recording_host_audio_ready"
    );
    assert_eq!(summary["readiness_blockers"], json!([]));
    assert_eq!(
        summary["receipt"]["pack_id"],
        LIVE_RECORDING_RECEIPT_PACK_ID
    );
    assert_eq!(summary["receipt"]["export_scope"], "live_recording");
    assert_eq!(summary["receipt"]["export_role"], "live_recording_capture");
    assert_eq!(
        summary["receipt"]["export_boundary"],
        "live_recording.receipt_contract_v1"
    );
    assert_eq!(summary["host_audio_refs"][0]["host"], "Alsa");
    assert_eq!(summary["host_audio_refs"][0]["device"], "PipeWire default");
    assert_eq!(
        summary["host_audio_refs"][0]["recording_duration_ms"],
        2_000
    );
}

#[test]
fn live_recording_readiness_report_reports_blocked_missing_host_audio() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let mut session = SessionFile::new(
        "blocked-live-report-session",
        "riotbox-test",
        "2026-06-04T08:02:00Z",
    );
    let mut receipt = ready_live_recording_receipt(ActionId(1178));
    receipt.live_recording_host_audio_refs.clear();
    receipt
        .unsupported_scopes
        .push(UnsupportedExportScope::LiveRecording);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save session");
    let launch = AppLaunch {
        mode: LaunchMode::LiveRecordingReadinessReport { session_path },
        observer_path: None,
    };

    let summary = live_recording_readiness_report_summary(&launch).expect("report summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready"], false);
    assert_eq!(
        summary["developer_proof_status"],
        "live_recording_host_audio_blocked"
    );
    assert_eq!(
        summary["readiness_blockers"],
        json!([
            "unsupported_scope_flag_present",
            "missing_host_audio_evidence"
        ])
    );
    assert_eq!(summary["host_audio_refs"], json!([]));
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
        timing_window: None,
    }];
    receipt
}
