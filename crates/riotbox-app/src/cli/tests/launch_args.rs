use crate::cli::args::parse_args;
use crate::cli::model::LaunchMode;
use std::path::PathBuf;

#[test]
fn parse_args_builds_ingest_mode() {
    let mode = parse_args([
        "--source".into(),
        "input.wav".into(),
        "--session".into(),
        "session.json".into(),
        "--graph".into(),
        "graph.json".into(),
    ])
    .expect("parse ingest mode");

    assert_eq!(mode.observer_path, None);
    match mode.mode {
        LaunchMode::Ingest {
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path,
            analysis_seed,
            ..
        } => {
            assert_eq!(source_path, PathBuf::from("input.wav"));
            assert_eq!(session_path, PathBuf::from("session.json"));
            assert_eq!(source_graph_path, Some(PathBuf::from("graph.json")));
            assert!(sidecar_script_path.is_absolute());
            assert!(sidecar_script_path.is_file());
            assert_eq!(analysis_seed, 19);
        }
        LaunchMode::Load { .. } => panic!("expected ingest mode"),
        LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageSourceMatchedExecute { .. }
        | LaunchMode::StemPackageW30HookExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. }
        | LaunchMode::LiveRecordingReadinessReport { .. }
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
        | LaunchMode::DawSessionWriterPlan { .. } => panic!("expected ingest mode"),
    }
}

#[test]
fn parse_args_retains_explicit_sidecar_override() {
    let mode = parse_args([
        "--source".into(),
        "input.wav".into(),
        "--sidecar".into(),
        "custom/provider.py".into(),
    ])
    .expect("parse explicit sidecar override");

    match mode.mode {
        LaunchMode::Ingest {
            sidecar_script_path,
            ..
        } => assert_eq!(sidecar_script_path, PathBuf::from("custom/provider.py")),
        _ => panic!("expected ingest mode"),
    }
}

#[test]
fn parse_args_accepts_explicit_source_bpm_only_for_ingest() {
    let launch = parse_args([
        "--source".into(),
        "input.wav".into(),
        "--source-bpm".into(),
        "130.0".into(),
    ])
    .expect("parse explicit source BPM");

    match launch.mode {
        LaunchMode::Ingest {
            explicit_source_bpm,
            ..
        } => assert_eq!(explicit_source_bpm, Some(130.0)),
        _ => panic!("expected ingest mode"),
    }

    let error = parse_args([
        "--session".into(),
        "session.json".into(),
        "--source-bpm".into(),
        "130.0".into(),
    ])
    .expect_err("source BPM requires ingest");
    assert!(error.contains("--source-bpm requires --source"));
}

#[test]
fn parse_args_requires_bpm_for_explicit_source_downbeat() {
    let launch = parse_args([
        "--source".into(),
        "input.wav".into(),
        "--source-bpm".into(),
        "120".into(),
        "--source-downbeat-seconds".into(),
        "0.25".into(),
    ])
    .expect("parse explicit manual source grid");

    match launch.mode {
        LaunchMode::Ingest {
            explicit_source_bpm,
            explicit_source_downbeat_seconds,
            ..
        } => {
            assert_eq!(explicit_source_bpm, Some(120.0));
            assert_eq!(explicit_source_downbeat_seconds, Some(0.25));
        }
        _ => panic!("expected ingest mode"),
    }

    let error = parse_args([
        "--source".into(),
        "input.wav".into(),
        "--source-downbeat-seconds".into(),
        "0".into(),
    ])
    .expect_err("manual source phase requires BPM");
    assert!(error.contains("requires --source-bpm"));
}

#[test]
fn parse_args_defaults_ingest_to_embedded_graph_storage() {
    let mode = parse_args([
        "--source".into(),
        "input.wav".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect("parse ingest mode");

    match mode.mode {
        LaunchMode::Ingest {
            source_graph_path, ..
        } => {
            assert_eq!(source_graph_path, None);
        }
        LaunchMode::Load { .. } => panic!("expected ingest mode"),
        LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageSourceMatchedExecute { .. }
        | LaunchMode::StemPackageW30HookExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. }
        | LaunchMode::LiveRecordingReadinessReport { .. }
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
        | LaunchMode::DawSessionWriterPlan { .. } => panic!("expected ingest mode"),
    }
}

#[test]
fn parse_args_builds_load_mode() {
    let mode = parse_args([
        "--session".into(),
        "session.json".into(),
        "--graph".into(),
        "graph.json".into(),
    ])
    .expect("parse load mode");

    match mode.mode {
        LaunchMode::Load {
            session_path,
            source_graph_path,
            ..
        } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
            assert_eq!(source_graph_path, Some(PathBuf::from("graph.json")));
        }
        LaunchMode::Ingest { .. } => panic!("expected load mode"),
        LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageSourceMatchedExecute { .. }
        | LaunchMode::StemPackageW30HookExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. }
        | LaunchMode::LiveRecordingReadinessReport { .. }
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
        | LaunchMode::DawSessionWriterPlan { .. } => panic!("expected load mode"),
    }
}

#[test]
fn parse_args_allows_session_only_for_load_mode() {
    let mode = parse_args(["--session".into(), "session.json".into()]).expect("session-only load");

    match mode.mode {
        LaunchMode::Load {
            session_path,
            source_graph_path,
            ..
        } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
            assert_eq!(source_graph_path, None);
        }
        LaunchMode::Ingest { .. } => panic!("expected load mode"),
        LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageSourceMatchedExecute { .. }
        | LaunchMode::StemPackageW30HookExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. }
        | LaunchMode::LiveRecordingReadinessReport { .. }
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
        | LaunchMode::DawSessionWriterPlan { .. } => panic!("expected load mode"),
    }
}

#[test]
fn parse_args_accepts_observer_path() {
    let launch = parse_args([
        "--session".into(),
        "session.json".into(),
        "--observer".into(),
        "artifacts/audio_qa/live/events.ndjson".into(),
    ])
    .expect("parse observer path");

    assert_eq!(
        launch.observer_path,
        Some(PathBuf::from("artifacts/audio_qa/live/events.ndjson"))
    );
}
