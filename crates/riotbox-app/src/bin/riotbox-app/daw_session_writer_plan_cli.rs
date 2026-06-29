use crate::jam_app::daw_session_writer_plan;

fn run_daw_session_writer_plan(launch: &AppLaunch) -> Result<(), Box<dyn std::error::Error>> {
    let summary = daw_session_writer_plan_summary(launch)?;
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn daw_session_writer_plan_summary(
    launch: &AppLaunch,
) -> Result<Value, Box<dyn std::error::Error>> {
    let LaunchMode::DawSessionWriterPlan {
        session_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a DAW session writer plan launch".into());
    };
    let session = riotbox_core::persistence::load_session_json(session_path)?;
    let plan = daw_session_writer_plan(&session, session_path.parent(), destination_path);

    Ok(json!({
        "mode": "daw_session_writer_plan",
        "session_path": session_path,
        "destination_path": destination_path,
        "status": plan.status,
        "ready_for_writer": plan.ready_for_writer,
        "writes_files": plan.writes_files,
        "boundary": plan.boundary_id,
        "readiness_blockers": plan.readiness_blockers,
        "writer_blockers": plan.writer_blockers,
        "receipt": plan.receipt,
        "placement_refs": plan.placement_refs,
        "tempo_map_ref": plan.tempo_map_ref,
        "source_artifacts": plan.source_artifacts,
        "planned_artifacts": plan.planned_artifacts,
        "payload_preview": plan.payload_preview,
        "operator_readiness": plan.operator_readiness,
    }))
}
