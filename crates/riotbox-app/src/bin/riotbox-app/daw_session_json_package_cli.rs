use riotbox_app::jam_app::{
    DAW_SESSION_JSON_PACKAGE_WRITER_BOUNDARY_ID, WrittenDawSessionJsonPackage,
    daw_session_json_package_report, write_daw_session_json_package,
};

fn run_daw_session_json_package_execute(
    launch: &AppLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = daw_session_json_package_execute_summary(launch)?;
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn daw_session_json_package_execute_summary(
    launch: &AppLaunch,
) -> Result<Value, Box<dyn std::error::Error>> {
    let LaunchMode::DawSessionJsonPackageExecute {
        session_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a DAW session JSON package execute launch".into());
    };
    let session = riotbox_core::persistence::load_session_json(session_path)?;
    let surface_gate =
        riotbox_app::jam_app::daw_session_export_surface_gate_for_session(&session);

    match write_daw_session_json_package(&session, session_path.parent(), destination_path) {
        Ok(written) => {
            let package_report = daw_session_json_package_report(destination_path);
            Ok(json!({
                "mode": "daw_session_json_package_execute",
                "status": "ready",
                "ready": true,
                "writes_files": true,
                "mutates_session": false,
                "observer_events": false,
                "boundary": DAW_SESSION_JSON_PACKAGE_WRITER_BOUNDARY_ID,
                "session_path": session_path,
                "destination_path": destination_path,
                "written_package": written_daw_session_json_package_summary(&written),
                "package_report": package_report,
                "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
                "release_blockers": [
                    "developer_proof_only",
                    "daw_writer_missing",
                    "daw_host_import_proof_missing",
                    "audible_output_proof_missing",
                ],
            }))
        }
        Err(error) => Ok(json!({
            "mode": "daw_session_json_package_execute",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "mutates_session": false,
            "observer_events": false,
            "boundary": DAW_SESSION_JSON_PACKAGE_WRITER_BOUNDARY_ID,
            "session_path": session_path,
            "destination_path": destination_path,
            "readiness_blockers": [error.to_string()],
            "written_package": null,
            "package_report": null,
            "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
            "release_blockers": [
                "developer_proof_only",
                "daw_writer_missing",
                "daw_host_import_proof_missing",
                "audible_output_proof_missing",
            ],
        })),
    }
}

fn written_daw_session_json_package_summary(written: &WrittenDawSessionJsonPackage) -> Value {
    json!({
        "package_dir": written.package_dir,
        "manifest_path": written.manifest_path,
        "tempo_map_path": written.tempo_map_path,
        "proof_path": written.proof_path,
        "manifest_sha256": written.manifest_sha256,
        "tempo_map_sha256": written.tempo_map_sha256,
        "proof_sha256": written.proof_sha256,
        "proof_manifest_sha256": written.proof_manifest_sha256,
    })
}

fn daw_session_surface_gate_summary(
    gate: &riotbox_app::jam_app::DawSessionExportSurfaceGate,
) -> Value {
    json!({
        "status": gate.status.as_str(),
        "runnable": gate.runnable(),
        "blockers": gate
            .blockers
            .iter()
            .map(|blocker| blocker.as_str())
            .collect::<Vec<_>>(),
        "blocker_labels": gate
            .blockers
            .iter()
            .map(|blocker| blocker.musician_label())
            .collect::<Vec<_>>(),
    })
}
