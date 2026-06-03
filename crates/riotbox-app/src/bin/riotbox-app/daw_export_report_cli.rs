use riotbox_app::jam_app::{
    daw_export_operator_readiness_report, daw_session_export_surface_gate_for_session,
};

fn run_daw_export_readiness_report(
    launch: &AppLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = daw_export_readiness_report_summary(launch)?;
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn daw_export_readiness_report_summary(
    launch: &AppLaunch,
) -> Result<Value, Box<dyn std::error::Error>> {
    let LaunchMode::DawExportReadinessReport { session_path } = &launch.mode else {
        return Err("not a DAW export readiness report launch".into());
    };
    let session = riotbox_core::persistence::load_session_json(session_path)?;
    let report = daw_export_operator_readiness_report(&session, session_path.parent());
    let surface_gate = daw_session_export_surface_gate_for_session(&session);

    Ok(json!({
        "mode": "daw_export_readiness_report",
        "session_path": session_path,
        "status": report.status,
        "ready_for_next_gate": report.ready_for_next_gate,
        "writes_files": report.writes_files,
        "developer_proof_status": report.developer_proof_status,
        "musician_export_readiness": report.musician_export_readiness,
        "release_blockers": report.release_blockers,
        "proof_gates": report.proof_gates,
        "daw_session_surface_gate": {
            "status": surface_gate.status.as_str(),
            "runnable": surface_gate.runnable(),
            "blockers": surface_gate
                .blockers
                .iter()
                .map(|blocker| blocker.as_str())
                .collect::<Vec<_>>(),
            "blocker_labels": surface_gate
                .blockers
                .iter()
                .map(|blocker| blocker.musician_label())
                .collect::<Vec<_>>(),
        },
        "readiness_blockers": report.readiness_blockers,
        "daw_session_receipt_count": report.daw_session_receipt_count,
        "receipt": report.receipt,
        "arrangement_placement_readiness": report.arrangement_placement_readiness,
        "daw_tempo_map_readiness": report.daw_tempo_map_readiness,
        "artifact_preflight": report.artifact_preflight,
    }))
}
