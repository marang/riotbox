#[test]
fn parse_args_builds_daw_session_json_package_execute_mode() {
    let launch = parse_args([
        "--daw-session-json-package-execute".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
    ])
    .expect("parse DAW session JSON package execute mode");

    assert_eq!(launch.observer_path, None);
    match launch.mode {
        LaunchMode::DawSessionJsonPackageExecute {
            session_path,
            destination_path,
        } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
            assert_eq!(destination_path, PathBuf::from("exports/daw-package"));
        }
        LaunchMode::Load { .. }
        | LaunchMode::Ingest { .. }
        | LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. }
        | LaunchMode::DawExportReadinessReport { .. }
        | LaunchMode::DawSessionJsonPackageEvidenceApply { .. }
        | LaunchMode::DawSessionHostImportProofApply { .. }
        | LaunchMode::DawSessionAudibleOutputProofApply { .. }
        | LaunchMode::DawSessionWriterPlan { .. } => {
            panic!("expected DAW session JSON package execute mode")
        }
    }
}

#[test]
fn parse_args_builds_daw_session_json_package_evidence_apply_mode() {
    let launch = parse_args([
        "--daw-session-json-package-evidence-apply".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
    ])
    .expect("parse DAW session JSON package evidence apply mode");

    assert_eq!(launch.observer_path, None);
    match launch.mode {
        LaunchMode::DawSessionJsonPackageEvidenceApply {
            session_path,
            destination_path,
        } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
            assert_eq!(destination_path, PathBuf::from("exports/daw-package"));
        }
        LaunchMode::Load { .. }
        | LaunchMode::Ingest { .. }
        | LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. }
        | LaunchMode::DawExportReadinessReport { .. }
        | LaunchMode::DawSessionJsonPackageExecute { .. }
        | LaunchMode::DawSessionHostImportProofApply { .. }
        | LaunchMode::DawSessionAudibleOutputProofApply { .. }
        | LaunchMode::DawSessionWriterPlan { .. } => {
            panic!("expected DAW session JSON package evidence apply mode")
        }
    }
}

#[test]
fn parse_args_builds_daw_session_host_import_proof_apply_mode() {
    let launch = parse_args([
        "--daw-session-host-import-proof-apply".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-host-import-proof".into(),
        "exports/daw-package/host_import_proof.json".into(),
    ])
    .expect("parse DAW session host import proof apply mode");

    assert_eq!(launch.observer_path, None);
    match launch.mode {
        LaunchMode::DawSessionHostImportProofApply {
            session_path,
            proof_path,
        } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
            assert_eq!(
                proof_path,
                PathBuf::from("exports/daw-package/host_import_proof.json")
            );
        }
        LaunchMode::Load { .. }
        | LaunchMode::Ingest { .. }
        | LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. }
        | LaunchMode::DawExportReadinessReport { .. }
        | LaunchMode::DawSessionJsonPackageExecute { .. }
        | LaunchMode::DawSessionJsonPackageEvidenceApply { .. }
        | LaunchMode::DawSessionAudibleOutputProofApply { .. }
        | LaunchMode::DawSessionWriterPlan { .. } => {
            panic!("expected DAW session host import proof apply mode")
        }
    }
}

#[test]
fn parse_args_builds_daw_session_audible_output_proof_apply_mode() {
    let launch = parse_args([
        "--daw-session-audible-output-proof-apply".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-audible-output-proof".into(),
        "exports/daw-package/audible_output_proof.json".into(),
    ])
    .expect("parse DAW session audible output proof apply mode");

    assert_eq!(launch.observer_path, None);
    match launch.mode {
        LaunchMode::DawSessionAudibleOutputProofApply {
            session_path,
            proof_path,
        } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
            assert_eq!(
                proof_path,
                PathBuf::from("exports/daw-package/audible_output_proof.json")
            );
        }
        LaunchMode::Load { .. }
        | LaunchMode::Ingest { .. }
        | LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. }
        | LaunchMode::DawExportReadinessReport { .. }
        | LaunchMode::DawSessionJsonPackageExecute { .. }
        | LaunchMode::DawSessionJsonPackageEvidenceApply { .. }
        | LaunchMode::DawSessionHostImportProofApply { .. }
        | LaunchMode::DawSessionWriterPlan { .. } => {
            panic!("expected DAW session audible output proof apply mode")
        }
    }
}

#[test]
fn parse_args_rejects_daw_session_json_package_execute_without_required_inputs_or_with_observer() {
    let missing_session = parse_args([
        "--daw-session-json-package-execute".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
    ])
    .expect_err("session is required");
    assert!(missing_session.contains("--session"));

    let missing_destination = parse_args([
        "--daw-session-json-package-execute".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect_err("destination is required");
    assert!(missing_destination.contains("--daw-session-destination"));

    let observer_arg = parse_args([
        "--daw-session-json-package-execute".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
        "--observer".into(),
        "observer.ndjson".into(),
    ])
    .expect_err("execute should not write observer files");
    assert!(observer_arg.contains("reads only an explicit session and destination"));

    let plan_mix = parse_args([
        "--daw-session-json-package-execute".into(),
        "--daw-session-writer-plan".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
    ])
    .expect_err("execute should not mix with writer plan");
    assert!(plan_mix.contains("cannot be combined"));
}

#[test]
fn parse_args_rejects_daw_session_json_package_evidence_apply_without_inputs_or_with_observer() {
    let missing_session = parse_args([
        "--daw-session-json-package-evidence-apply".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
    ])
    .expect_err("session is required");
    assert!(missing_session.contains("--session"));

    let missing_destination = parse_args([
        "--daw-session-json-package-evidence-apply".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect_err("destination is required");
    assert!(missing_destination.contains("--daw-session-destination"));

    let observer_arg = parse_args([
        "--daw-session-json-package-evidence-apply".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
        "--observer".into(),
        "observer.ndjson".into(),
    ])
    .expect_err("apply should not write observer files");
    assert!(observer_arg.contains("reads only an explicit session and destination"));

    let execute_mix = parse_args([
        "--daw-session-json-package-evidence-apply".into(),
        "--daw-session-json-package-execute".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
    ])
    .expect_err("apply should not mix with execute");
    assert!(execute_mix.contains("cannot be combined"));
}

#[test]
fn parse_args_rejects_daw_session_host_import_proof_apply_without_inputs_or_with_observer() {
    let missing_session = parse_args([
        "--daw-session-host-import-proof-apply".into(),
        "--daw-session-host-import-proof".into(),
        "host_import_proof.json".into(),
    ])
    .expect_err("session is required");
    assert!(missing_session.contains("--session"));

    let missing_proof = parse_args([
        "--daw-session-host-import-proof-apply".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect_err("proof path is required");
    assert!(missing_proof.contains("--daw-session-host-import-proof"));

    let observer_arg = parse_args([
        "--daw-session-host-import-proof-apply".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-host-import-proof".into(),
        "host_import_proof.json".into(),
        "--observer".into(),
        "observer.ndjson".into(),
    ])
    .expect_err("apply should not write observer files");
    assert!(observer_arg.contains("reads only an explicit session and proof file"));

    let package_mix = parse_args([
        "--daw-session-host-import-proof-apply".into(),
        "--daw-session-json-package-evidence-apply".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-host-import-proof".into(),
        "host_import_proof.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
    ])
    .expect_err("host import proof apply should not mix with package apply");
    assert!(package_mix.contains("cannot be combined"));
}

#[test]
fn parse_args_rejects_daw_session_audible_output_proof_apply_without_inputs_or_with_observer() {
    let missing_session = parse_args([
        "--daw-session-audible-output-proof-apply".into(),
        "--daw-session-audible-output-proof".into(),
        "audible_output_proof.json".into(),
    ])
    .expect_err("session is required");
    assert!(missing_session.contains("--session"));

    let missing_proof = parse_args([
        "--daw-session-audible-output-proof-apply".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect_err("proof path is required");
    assert!(missing_proof.contains("--daw-session-audible-output-proof"));

    let observer_arg = parse_args([
        "--daw-session-audible-output-proof-apply".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-audible-output-proof".into(),
        "audible_output_proof.json".into(),
        "--observer".into(),
        "observer.ndjson".into(),
    ])
    .expect_err("apply should not write observer files");
    assert!(observer_arg.contains("reads only an explicit session and proof file"));

    let host_mix = parse_args([
        "--daw-session-audible-output-proof-apply".into(),
        "--daw-session-host-import-proof-apply".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-audible-output-proof".into(),
        "audible_output_proof.json".into(),
        "--daw-session-host-import-proof".into(),
        "host_import_proof.json".into(),
    ])
    .expect_err("audible output proof apply should not mix with host import proof apply");
    assert!(host_mix.contains("cannot be combined"));
}
