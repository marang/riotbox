use crate::jam_app::{
    DAW_SESSION_LOCAL_PROJECT_WRITER_BOUNDARY_ID, WrittenDawSessionWriterProofSkeleton,
    attach_daw_session_writer_proof_evidence_to_receipt, daw_session_writer_proof_report,
    write_daw_session_writer_proof_skeleton,
};

fn run_daw_session_writer_proof_execute(
    launch: &AppLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = daw_session_writer_proof_execute_summary(launch)?;
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn run_daw_session_writer_proof_apply(
    launch: &AppLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = daw_session_writer_proof_apply_summary(launch)?;
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn daw_session_writer_proof_execute_summary(
    launch: &AppLaunch,
) -> Result<Value, Box<dyn std::error::Error>> {
    let LaunchMode::DawSessionWriterProofExecute {
        session_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a DAW session writer proof execute launch".into());
    };
    let session = riotbox_core::persistence::load_session_json(session_path)?;
    let surface_gate =
        crate::jam_app::daw_session_export_surface_gate_for_session(&session);

    match write_daw_session_writer_proof_skeleton(
        &session,
        session_path.parent(),
        destination_path,
    ) {
        Ok(written) => {
            let proof_report = daw_session_writer_proof_report(destination_path);
            Ok(json!({
                "mode": "daw_session_writer_proof_execute",
                "status": "ready",
                "ready": true,
                "writes_files": true,
                "mutates_session": false,
                "observer_events": false,
                "boundary": DAW_SESSION_LOCAL_PROJECT_WRITER_BOUNDARY_ID,
                "session_path": session_path,
                "destination_path": destination_path,
                "written_proof": written_daw_session_writer_proof_summary(&written),
                "proof_report": proof_report,
                "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
                "scope_note": "writer proof only; host import and audible output remain unproven",
            }))
        }
        Err(error) => Ok(json!({
            "mode": "daw_session_writer_proof_execute",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "mutates_session": false,
            "observer_events": false,
            "boundary": DAW_SESSION_LOCAL_PROJECT_WRITER_BOUNDARY_ID,
            "session_path": session_path,
            "destination_path": destination_path,
            "readiness_blockers": [error.to_string()],
            "written_proof": null,
            "proof_report": null,
            "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
            "scope_note": "writer proof only; host import and audible output remain unproven",
        })),
    }
}

fn daw_session_writer_proof_apply_summary(
    launch: &AppLaunch,
) -> Result<Value, Box<dyn std::error::Error>> {
    let LaunchMode::DawSessionWriterProofApply {
        session_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a DAW session writer proof apply launch".into());
    };
    let mut session = riotbox_core::persistence::load_session_json(session_path)?;
    let proof_report = daw_session_writer_proof_report(destination_path);

    let Some(receipt_index) = session.export_receipts.iter().rposition(|receipt| {
        receipt.export_scope == riotbox_core::export_readiness::ExportScope::DawSession
    }) else {
        let surface_gate =
            crate::jam_app::daw_session_export_surface_gate_for_session(&session);
        return Ok(json!({
            "mode": "daw_session_writer_proof_apply",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "mutates_session": false,
            "observer_events": false,
            "session_path": session_path,
            "destination_path": destination_path,
            "readiness_blockers": ["no_daw_session_receipt"],
            "proof_report": proof_report,
            "receipt": null,
            "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
        }));
    };

    attach_daw_session_writer_proof_evidence_to_receipt(
        &mut session.export_receipts[receipt_index],
        &proof_report,
    )
    .map_err(|error| format!("DAW session writer proof apply failed: {error:?}"))?;
    riotbox_core::persistence::save_session_json(session_path, &session)?;

    let surface_gate = crate::jam_app::daw_session_export_surface_gate_for_session(&session);
    let receipt = &session.export_receipts[receipt_index];
    Ok(json!({
        "mode": "daw_session_writer_proof_apply",
        "status": if proof_report.ready { "ready" } else { "blocked" },
        "ready": proof_report.ready,
        "writes_files": false,
        "mutates_session": true,
        "observer_events": false,
        "session_path": session_path,
        "destination_path": destination_path,
        "readiness_blockers": proof_report.gate_blockers(),
        "proof_report": proof_report,
        "receipt": daw_session_writer_proof_receipt_summary(receipt),
        "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
        "scope_note": "writer proof only; host import and audible output remain unproven",
    }))
}

fn written_daw_session_writer_proof_summary(
    written: &WrittenDawSessionWriterProofSkeleton,
) -> Value {
    json!({
        "package_dir": written.package_dir,
        "project_skeleton_path": written.project_skeleton_path,
        "proof_path": written.proof_path,
        "project_skeleton_sha256": written.project_skeleton_sha256,
        "proof_sha256": written.proof_sha256,
    })
}

fn daw_session_writer_proof_receipt_summary(
    receipt: &riotbox_core::session::ExportReceiptState,
) -> Value {
    let writer_gate = receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == riotbox_core::session::DAW_SESSION_WRITER_QA_GATE_ID);
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
        "daw_writer_gate": writer_gate,
    })
}
