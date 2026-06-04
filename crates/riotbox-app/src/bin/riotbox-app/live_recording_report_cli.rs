use riotbox_core::{
    export_readiness::ExportScope as LiveRecordingReportExportScope,
    persistence::load_session_json as load_live_recording_report_session_json,
    session::{
        ExportLiveRecordingHostAudioRef as ReportExportLiveRecordingHostAudioRef,
        ExportReceiptState as LiveRecordingReportExportReceiptState,
        LiveRecordingHostAudioReadinessBlocker as ReportLiveRecordingHostAudioReadinessBlocker,
    },
};

fn run_live_recording_readiness_report(
    launch: &AppLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = live_recording_readiness_report_summary(launch)?;
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn live_recording_readiness_report_summary(
    launch: &AppLaunch,
) -> Result<Value, Box<dyn std::error::Error>> {
    let LaunchMode::LiveRecordingReadinessReport { session_path } = &launch.mode else {
        return Err("not a live recording readiness report launch".into());
    };
    let session = load_live_recording_report_session_json(session_path)?;
    let product_mix_receipt_count = session
        .export_receipts
        .iter()
        .filter(|receipt| receipt.export_scope == LiveRecordingReportExportScope::ProductMix)
        .count();
    let live_recording_receipt_count = session
        .export_receipts
        .iter()
        .filter(|receipt| receipt.export_scope == LiveRecordingReportExportScope::LiveRecording)
        .count();
    let Some(receipt) = session
        .export_receipts
        .iter()
        .rev()
        .find(|receipt| receipt.export_scope == LiveRecordingReportExportScope::LiveRecording)
    else {
        return Ok(json!({
            "mode": "live_recording_readiness_report",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "session_path": session_path,
            "developer_proof_status": "no_live_recording_receipt",
            "musician_export_readiness": "not_runnable_live_recording_export",
            "readiness_blockers": ["no_live_recording_receipt"],
            "product_mix_receipt_count": product_mix_receipt_count,
            "live_recording_receipt_count": live_recording_receipt_count,
            "receipt": null,
            "readiness_report": null,
            "host_audio_refs": [],
        }));
    };

    let readiness = receipt.live_recording_host_audio_readiness_report();
    let readiness_blockers = readiness
        .blockers
        .iter()
        .copied()
        .map(live_recording_readiness_blocker_label)
        .collect::<Vec<_>>();
    let ready = readiness.ready();
    let status = if ready { "ready" } else { "blocked" };
    let developer_proof_status = if ready {
        "live_recording_host_audio_ready"
    } else {
        "live_recording_host_audio_blocked"
    };

    Ok(json!({
        "mode": "live_recording_readiness_report",
        "status": status,
        "ready": ready,
        "writes_files": false,
        "session_path": session_path,
        "developer_proof_status": developer_proof_status,
        "musician_export_readiness": "not_runnable_live_recording_export",
        "readiness_blockers": readiness_blockers,
        "readiness_report": readiness,
        "product_mix_receipt_count": product_mix_receipt_count,
        "live_recording_receipt_count": live_recording_receipt_count,
        "receipt": live_recording_receipt_summary(receipt),
        "host_audio_refs": receipt
            .live_recording_host_audio_refs
            .iter()
            .map(live_recording_host_audio_ref_summary)
            .collect::<Vec<_>>(),
    }))
}

fn live_recording_receipt_summary(receipt: &LiveRecordingReportExportReceiptState) -> Value {
    json!({
        "receipt_id": receipt.receipt_id,
        "created_by_action": receipt.created_by_action,
        "pack_id": receipt.pack_id,
        "export_scope": receipt.export_scope.as_str(),
        "export_role": receipt.export_role.as_str(),
        "export_boundary": receipt.export_boundary.as_proof_str(),
        "readiness_status": receipt.readiness_status,
        "unsupported_scopes": receipt.unsupported_scopes,
    })
}

fn live_recording_host_audio_ref_summary(
    evidence: &ReportExportLiveRecordingHostAudioRef,
) -> Value {
    json!({
        "host": evidence.host,
        "device": evidence.device,
        "recording_duration_ms": evidence.recording_duration_ms,
        "callback_gap_summary": evidence.callback_gap_summary,
        "stream_error_summary": evidence.stream_error_summary,
    })
}

fn live_recording_readiness_blocker_label(
    blocker: ReportLiveRecordingHostAudioReadinessBlocker,
) -> &'static str {
    match blocker {
        ReportLiveRecordingHostAudioReadinessBlocker::NotLiveRecordingScope => {
            "not_live_recording_scope"
        }
        ReportLiveRecordingHostAudioReadinessBlocker::UnsupportedScopeFlagPresent => {
            "unsupported_scope_flag_present"
        }
        ReportLiveRecordingHostAudioReadinessBlocker::MissingHostAudioEvidence => {
            "missing_host_audio_evidence"
        }
        ReportLiveRecordingHostAudioReadinessBlocker::BlankHost => "blank_host",
        ReportLiveRecordingHostAudioReadinessBlocker::BlankDevice => "blank_device",
        ReportLiveRecordingHostAudioReadinessBlocker::ZeroRecordingDuration => {
            "zero_recording_duration"
        }
        ReportLiveRecordingHostAudioReadinessBlocker::CallbackGapOverThreshold => {
            "callback_gap_over_threshold"
        }
        ReportLiveRecordingHostAudioReadinessBlocker::StreamErrorReported => {
            "stream_error_reported"
        }
    }
}
