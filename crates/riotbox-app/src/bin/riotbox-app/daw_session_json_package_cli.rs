use riotbox_app::jam_app::{
    DAW_SESSION_JSON_PACKAGE_WRITER_BOUNDARY_ID, WrittenDawSessionJsonPackage,
    attach_daw_session_audible_output_proof_evidence_to_receipt,
    attach_daw_session_host_import_proof_evidence_to_receipt,
    attach_daw_session_json_package_evidence_to_receipt, daw_session_audible_output_proof_report,
    daw_session_host_import_proof_report, daw_session_json_package_report,
    write_daw_session_json_package,
};

fn run_daw_session_json_package_execute(
    launch: &AppLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = daw_session_json_package_execute_summary(launch)?;
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn run_daw_session_json_package_evidence_apply(
    launch: &AppLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = daw_session_json_package_evidence_apply_summary(launch)?;
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn run_daw_session_host_import_proof_apply(
    launch: &AppLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = daw_session_host_import_proof_apply_summary(launch)?;
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn run_daw_session_audible_output_proof_apply(
    launch: &AppLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = daw_session_audible_output_proof_apply_summary(launch)?;
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

fn daw_session_json_package_evidence_apply_summary(
    launch: &AppLaunch,
) -> Result<Value, Box<dyn std::error::Error>> {
    let LaunchMode::DawSessionJsonPackageEvidenceApply {
        session_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a DAW session JSON package evidence apply launch".into());
    };
    let mut session = riotbox_core::persistence::load_session_json(session_path)?;
    let package_report = daw_session_json_package_report(destination_path);

    let Some(receipt_index) = session
        .export_receipts
        .iter()
        .rposition(|receipt| {
            receipt.export_scope == riotbox_core::export_readiness::ExportScope::DawSession
        })
    else {
        let surface_gate =
            riotbox_app::jam_app::daw_session_export_surface_gate_for_session(&session);
        return Ok(json!({
            "mode": "daw_session_json_package_evidence_apply",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "mutates_session": false,
            "observer_events": false,
            "session_path": session_path,
            "destination_path": destination_path,
            "readiness_blockers": ["no_daw_session_receipt"],
            "package_report": package_report,
            "receipt": null,
            "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
            "release_blockers": daw_session_release_blockers(),
        }));
    };

    attach_daw_session_json_package_evidence_to_receipt(
        &mut session.export_receipts[receipt_index],
        &package_report,
    )
    .map_err(|error| format!("DAW session JSON package evidence apply failed: {error:?}"))?;
    riotbox_core::persistence::save_session_json(session_path, &session)?;

    let surface_gate = riotbox_app::jam_app::daw_session_export_surface_gate_for_session(&session);
    let receipt = &session.export_receipts[receipt_index];
    Ok(json!({
        "mode": "daw_session_json_package_evidence_apply",
        "status": if package_report.ready { "ready" } else { "blocked" },
        "ready": package_report.ready,
        "writes_files": false,
        "mutates_session": true,
        "observer_events": false,
        "session_path": session_path,
        "destination_path": destination_path,
        "readiness_blockers": package_report
            .blockers
            .iter()
            .map(|blocker| blocker.as_str())
            .collect::<Vec<_>>(),
        "package_report": package_report,
        "receipt": daw_session_json_package_receipt_summary(receipt),
        "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
        "release_blockers": daw_session_release_blockers(),
    }))
}

fn daw_session_host_import_proof_apply_summary(
    launch: &AppLaunch,
) -> Result<Value, Box<dyn std::error::Error>> {
    let LaunchMode::DawSessionHostImportProofApply {
        session_path,
        proof_path,
    } = &launch.mode
    else {
        return Err("not a DAW session host import proof apply launch".into());
    };
    let mut session = riotbox_core::persistence::load_session_json(session_path)?;
    let proof_report = daw_session_host_import_proof_report(proof_path);

    let Some(receipt_index) = session
        .export_receipts
        .iter()
        .rposition(|receipt| {
            receipt.export_scope == riotbox_core::export_readiness::ExportScope::DawSession
        })
    else {
        let surface_gate =
            riotbox_app::jam_app::daw_session_export_surface_gate_for_session(&session);
        return Ok(json!({
            "mode": "daw_session_host_import_proof_apply",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "mutates_session": false,
            "observer_events": false,
            "session_path": session_path,
            "proof_path": proof_path,
            "readiness_blockers": ["no_daw_session_receipt"],
            "proof_report": proof_report,
            "receipt": null,
            "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
            "release_blockers": daw_session_release_blockers(),
        }));
    };

    attach_daw_session_host_import_proof_evidence_to_receipt(
        &mut session.export_receipts[receipt_index],
        &proof_report,
    )
    .map_err(|error| format!("DAW session host import proof apply failed: {error:?}"))?;
    riotbox_core::persistence::save_session_json(session_path, &session)?;

    let surface_gate = riotbox_app::jam_app::daw_session_export_surface_gate_for_session(&session);
    let receipt = &session.export_receipts[receipt_index];
    let ready = proof_report.ready_for_receipt(receipt);
    let readiness_blockers = proof_report.gate_blockers_for_receipt(receipt);
    Ok(json!({
        "mode": "daw_session_host_import_proof_apply",
        "status": if ready { "ready" } else { "blocked" },
        "ready": ready,
        "writes_files": false,
        "mutates_session": true,
        "observer_events": false,
        "session_path": session_path,
        "proof_path": proof_path,
        "readiness_blockers": readiness_blockers,
        "proof_report": proof_report,
        "receipt": daw_session_host_import_receipt_summary(receipt),
        "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
        "release_blockers": daw_session_release_blockers(),
    }))
}

fn daw_session_audible_output_proof_apply_summary(
    launch: &AppLaunch,
) -> Result<Value, Box<dyn std::error::Error>> {
    let LaunchMode::DawSessionAudibleOutputProofApply {
        session_path,
        proof_path,
    } = &launch.mode
    else {
        return Err("not a DAW session audible output proof apply launch".into());
    };
    let mut session = riotbox_core::persistence::load_session_json(session_path)?;
    let proof_report = daw_session_audible_output_proof_report(proof_path);

    let Some(receipt_index) = session.export_receipts.iter().rposition(|receipt| {
        receipt.export_scope == riotbox_core::export_readiness::ExportScope::DawSession
    }) else {
        let surface_gate =
            riotbox_app::jam_app::daw_session_export_surface_gate_for_session(&session);
        return Ok(json!({
            "mode": "daw_session_audible_output_proof_apply",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "mutates_session": false,
            "observer_events": false,
            "session_path": session_path,
            "proof_path": proof_path,
            "readiness_blockers": ["no_daw_session_receipt"],
            "proof_report": proof_report,
            "receipt": null,
            "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
            "release_blockers": daw_session_release_blockers(),
        }));
    };

    attach_daw_session_audible_output_proof_evidence_to_receipt(
        &mut session.export_receipts[receipt_index],
        &proof_report,
    )
    .map_err(|error| format!("DAW session audible output proof apply failed: {error:?}"))?;
    riotbox_core::persistence::save_session_json(session_path, &session)?;

    let surface_gate = riotbox_app::jam_app::daw_session_export_surface_gate_for_session(&session);
    let receipt = &session.export_receipts[receipt_index];
    Ok(json!({
        "mode": "daw_session_audible_output_proof_apply",
        "status": if proof_report.ready { "ready" } else { "blocked" },
        "ready": proof_report.ready,
        "writes_files": false,
        "mutates_session": true,
        "observer_events": false,
        "session_path": session_path,
        "proof_path": proof_path,
        "readiness_blockers": proof_report.gate_blockers(),
        "proof_report": proof_report,
        "receipt": daw_session_audible_output_receipt_summary(receipt),
        "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
        "release_blockers": daw_session_release_blockers(),
    }))
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

fn daw_session_json_package_receipt_summary(
    receipt: &riotbox_core::session::ExportReceiptState,
) -> Value {
    let package_gate = receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == riotbox_core::session::DAW_SESSION_JSON_PACKAGE_QA_GATE_ID);
    json!({
        "receipt_id": receipt.receipt_id.as_str(),
        "export_scope": receipt.export_scope,
        "pack_id": receipt.pack_id,
        "export_role": receipt.export_role,
        "export_boundary": receipt.export_boundary,
        "artifact_set": receipt
            .artifact_set
            .iter()
            .map(|artifact| json!({
                "role": export_artifact_role_label(artifact.role),
                "location": artifact.location_identity(),
                "sha256": artifact.sha256,
            }))
            .collect::<Vec<_>>(),
        "daw_json_package_gate": package_gate,
    })
}

fn daw_session_host_import_receipt_summary(
    receipt: &riotbox_core::session::ExportReceiptState,
) -> Value {
    let host_import_gate = receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == riotbox_core::session::DAW_SESSION_HOST_IMPORT_QA_GATE_ID);
    json!({
        "receipt_id": receipt.receipt_id.as_str(),
        "export_scope": receipt.export_scope,
        "pack_id": receipt.pack_id,
        "export_role": receipt.export_role,
        "export_boundary": receipt.export_boundary,
        "daw_host_import_gate": host_import_gate,
    })
}

fn daw_session_audible_output_receipt_summary(
    receipt: &riotbox_core::session::ExportReceiptState,
) -> Value {
    let audible_output_gate = receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == riotbox_core::session::DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID);
    json!({
        "receipt_id": receipt.receipt_id.as_str(),
        "export_scope": receipt.export_scope,
        "pack_id": receipt.pack_id,
        "export_role": receipt.export_role,
        "export_boundary": receipt.export_boundary,
        "daw_audible_output_gate": audible_output_gate,
    })
}

fn daw_session_release_blockers() -> [&'static str; 4] {
    [
        "developer_proof_only",
        "daw_writer_missing",
        "daw_host_import_proof_missing",
        "audible_output_proof_missing",
    ]
}
