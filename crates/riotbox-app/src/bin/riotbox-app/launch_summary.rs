fn launch_summary(launch: &AppLaunch) -> Value {
    match &launch.mode {
        LaunchMode::Load {
            session_path,
            source_graph_path,
        } => json!({
            "mode": "load",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::Ingest {
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path,
            analysis_seed,
        } => json!({
            "mode": "ingest",
            "source_path": source_path,
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "sidecar_script_path": sidecar_script_path,
            "analysis_seed": analysis_seed,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::StemPackageLocalCiDryRun {
            destination_path,
            claimed_stem_roles,
        } => json!({
            "mode": "stem_package_local_ci_dry_run",
            "destination_path": destination_path,
            "claimed_stem_roles": claimed_stem_roles
                .iter()
                .copied()
                .map(export_artifact_role_label)
                .collect::<Vec<_>>(),
            "observer_path": launch.observer_path,
        }),
        LaunchMode::StemPackageLocalCiExecute {
            session_path,
            source_graph_path,
            destination_path,
            claimed_stem_roles,
        } => json!({
            "mode": "stem_package_local_ci_execute",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "destination_path": destination_path,
            "claimed_stem_roles": claimed_stem_roles
                .iter()
                .copied()
                .map(export_artifact_role_label)
                .collect::<Vec<_>>(),
            "observer_path": launch.observer_path,
        }),
        LaunchMode::StemPackageLocalCiReport { session_path } => json!({
            "mode": "stem_package_local_ci_report",
            "session_path": session_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::LiveRecordingReadinessReport { session_path } => json!({
            "mode": "live_recording_readiness_report",
            "session_path": session_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawExportReadinessReport { session_path } => json!({
            "mode": "daw_export_readiness_report",
            "session_path": session_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionJsonPackageExecute {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_json_package_execute",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionJsonPackageEvidenceApply {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_json_package_evidence_apply",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionHostImportProofApply {
            session_path,
            proof_path,
        } => json!({
            "mode": "daw_session_host_import_proof_apply",
            "session_path": session_path,
            "proof_path": proof_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionAudibleOutputProofApply {
            session_path,
            proof_path,
        } => json!({
            "mode": "daw_session_audible_output_proof_apply",
            "session_path": session_path,
            "proof_path": proof_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionWriterProofExecute {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_writer_proof_execute",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionWriterProofApply {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_writer_proof_apply",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionWriterExportExecute {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_writer_export_execute",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionWriterPlan {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_writer_plan",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
    }
}
