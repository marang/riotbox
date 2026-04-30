use super::*;
use std::{fs, path::PathBuf};

#[test]
fn parses_required_paths_and_output() {
    let parsed = Args::parse([
        "--observer".to_string(),
        "events.ndjson".to_string(),
        "--manifest".to_string(),
        "manifest.json".to_string(),
        "--output".to_string(),
        "summary.md".to_string(),
    ])
    .expect("parse args");

    assert_eq!(parsed.observer_path, PathBuf::from("events.ndjson"));
    assert_eq!(parsed.manifest_path, PathBuf::from("manifest.json"));
    assert_eq!(parsed.output_path, Some(PathBuf::from("summary.md")));
    assert!(!parsed.require_evidence);
    assert!(!parsed.json_output);
}

#[test]
fn parses_strict_evidence_flag() {
    let parsed = Args::parse([
        "--observer".to_string(),
        "events.ndjson".to_string(),
        "--manifest".to_string(),
        "manifest.json".to_string(),
        "--require-evidence".to_string(),
    ])
    .expect("parse args");

    assert!(parsed.require_evidence);
}

#[test]
fn parses_json_output_flag() {
    let parsed = Args::parse([
        "--observer".to_string(),
        "events.ndjson".to_string(),
        "--manifest".to_string(),
        "manifest.json".to_string(),
        "--json".to_string(),
    ])
    .expect("parse args");

    assert!(parsed.json_output);
}

#[test]
fn rejects_missing_required_paths() {
    assert!(Args::parse(Vec::<String>::new()).is_err());
}

#[test]
fn accepts_help_without_required_paths() {
    let parsed = Args::parse(["--help".to_string()]).expect("parse help");

    assert!(parsed.show_help);
}

#[test]
fn summarizes_synthetic_observer_and_manifest() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer()).expect("write observer");
    fs::write(&manifest_path, synthetic_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let markdown = render_markdown(&summary);

    assert_eq!(summary.observer_schema, "riotbox.user_session_observer.v1");
    assert_eq!(summary.launch_mode, "ingest");
    assert_eq!(summary.audio_runtime_status, "started");
    assert_eq!(
        summary.key_outcomes,
        ["space -> transport started", "f -> queued"]
    );
    assert!(summary.first_commit.contains("action 2 at NextBar"));
    assert_eq!(summary.commit_count, 1);
    assert_eq!(summary.commit_boundaries, ["NextBar"]);
    assert_eq!(summary.pack_id, "feral-grid-demo");
    assert_eq!(summary.manifest_result, "pass");
    assert_eq!(summary.artifact_count, 6);
    assert_eq!(summary.full_mix_rms, Some(0.1));
    assert_eq!(summary.w30_candidate_rms, None);
    assert!(markdown.contains("Control path present: `yes`"));
    assert!(markdown.contains("Output path present: `yes`"));
    assert!(markdown.contains("Output path issues: `none`"));
    let json: Value = serde_json::from_str(&render_json(&summary).expect("json")).expect("json");
    assert_eq!(json["schema"], SUMMARY_SCHEMA);
    assert_eq!(
        json["schema_version"].as_u64(),
        Some(u64::from(SUMMARY_SCHEMA_VERSION))
    );
    assert_eq!(json["control_path"]["present"], true);
    assert_eq!(json["control_path"]["commit_count"], 1);
    assert_eq!(json["control_path"]["commit_boundaries"][0], "NextBar");
    assert_eq!(json["output_path"]["present"], true);
    assert_eq!(
        json["output_path"]["issues"]
            .as_array()
            .expect("issues")
            .len(),
        0
    );
    assert_eq!(
        json["output_path"]["metrics"]["full_mix_rms"].as_f64(),
        Some(0.1)
    );
}

#[test]
fn summarizes_committed_fixture_observer_and_manifest() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, fixture_observer()).expect("write observer");
    fs::write(&manifest_path, fixture_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let markdown = render_markdown(&summary);

    assert_eq!(summary.observer_schema, "riotbox.user_session_observer.v1");
    assert_eq!(summary.launch_mode, "ingest");
    assert_eq!(summary.audio_runtime_status, "started");
    assert_eq!(summary.pack_id, "feral-grid-demo");
    assert_eq!(summary.manifest_result, "pass");
    assert_eq!(summary.artifact_count, 6);
    assert_eq!(summary.commit_count, 1);
    assert_eq!(summary.commit_boundaries, ["NextBar"]);
    assert!(summary.full_mix_rms.is_some_and(|rms| rms > 0.01));
    assert!(summary.full_mix_low_band_rms.is_some_and(|rms| rms > 0.01));
    assert!(
        summary
            .mc202_question_answer_delta_rms
            .is_some_and(|rms| rms > 0.001)
    );
    assert!(markdown.contains("Key outcomes: `space -> transport started, f -> queued`"));
    assert!(markdown.contains("Control path present: `yes`"));
    assert!(markdown.contains("Output path present: `yes`"));
    assert!(markdown.contains("Needs human listening: `yes`"));
    validate_manifest_envelope_file(&manifest_path).expect("fixture manifest envelope");
    validate_required_evidence(&summary).expect("fixture evidence");
}

#[test]
fn summarizes_first_playable_w30_source_diff_manifest() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, first_playable_observer()).expect("write observer");
    fs::write(&manifest_path, w30_source_diff_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let markdown = render_markdown(&summary);

    assert_eq!(summary.pack_id, "w30-preview-smoke");
    assert_eq!(summary.manifest_result, "pass");
    assert_eq!(summary.artifact_count, 3);
    assert_eq!(
        summary.key_outcomes,
        [
            "space -> transport started",
            "c -> capture queued",
            "o -> audition raw/src",
            "p -> promote queued",
            "w -> recall/src"
        ]
    );
    assert_eq!(summary.commit_count, 2);
    assert_eq!(summary.commit_boundaries, ["NextBar"]);
    assert!(summary.w30_candidate_rms.is_some_and(|rms| rms > 0.001));
    assert!(
        summary
            .w30_candidate_active_sample_ratio
            .is_some_and(|ratio| ratio > 0.1)
    );
    assert!(summary.w30_rms_delta.is_some_and(|delta| delta > 0.001));
    assert!(markdown.contains("W-30 candidate RMS: `0.006986`"));
    assert!(markdown.contains("Output path present: `yes`"));
    validate_required_evidence(&summary).expect("first playable evidence");
}

#[test]
fn strict_evidence_rejects_invalid_manifest_envelope() {
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&manifest_path, synthetic_manifest()).expect("write manifest");

    let error = validate_manifest_envelope_file(&manifest_path).expect_err("invalid envelope");

    assert!(error.to_string().contains("schema_version"));
}

#[test]
fn strict_evidence_rejects_missing_commit() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer_without_commit()).expect("write observer");
    fs::write(&manifest_path, synthetic_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("missing control evidence");

    assert!(error.to_string().contains("control-path evidence"));
}

#[test]
fn strict_evidence_rejects_missing_output_metrics() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer()).expect("write observer");
    fs::write(&manifest_path, synthetic_manifest_without_metrics()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("missing output evidence");

    assert!(error.to_string().contains("output-path manifest evidence"));
    assert!(error.to_string().contains("full_mix_rms=missing"));
    assert!(error.to_string().contains("full_mix_low_band_rms=missing"));
    assert!(
        error
            .to_string()
            .contains("mc202_question_answer_delta_rms=missing")
    );
    let markdown = render_markdown(&summary);
    assert!(markdown.contains("Output path present: `no`"));
    assert!(markdown.contains("full_mix_rms=missing"));
}

#[test]
fn strict_evidence_rejects_collapsed_output_metrics() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer()).expect("write observer");
    fs::write(&manifest_path, synthetic_manifest_with_collapsed_metrics()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("collapsed output evidence");

    assert!(error.to_string().contains("output-path manifest evidence"));
    assert!(error.to_string().contains("full_mix_rms=0.000000"));
    assert!(error.to_string().contains("floor 0.000001"));
    let markdown = render_markdown(&summary);
    assert!(markdown.contains("Output path present: `no`"));
    assert!(markdown.contains("full_mix_rms=0.000000"));
}

fn synthetic_observer() -> String {
    [
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest"}}"#,
        r#"{"event":"audio_runtime","status":"started"}"#,
        r#"{"event":"key_outcome","key":"space","outcome":"transport started"}"#,
        r#"{"event":"key_outcome","key":"f","outcome":"queued"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
    ]
    .join("\n")
        + "\n"
}

fn synthetic_observer_without_commit() -> String {
    [
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest"}}"#,
        r#"{"event":"audio_runtime","status":"started"}"#,
        r#"{"event":"key_outcome","key":"space","outcome":"transport started"}"#,
    ]
    .join("\n")
        + "\n"
}

fn synthetic_manifest() -> String {
    r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "artifacts": [{}, {}, {}, {}, {}, {}],
  "metrics": {
    "full_grid_mix": {
      "signal": { "rms": 0.1 },
      "low_band": { "rms": 0.08 }
    },
    "mc202_question_answer_delta": { "rms": 0.01 }
  }
}"#
    .to_string()
}

fn synthetic_manifest_without_metrics() -> String {
    r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "artifacts": [{}, {}, {}, {}, {}, {}],
  "metrics": {}
}"#
    .to_string()
}

fn synthetic_manifest_with_collapsed_metrics() -> String {
    r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "artifacts": [{}, {}, {}, {}, {}, {}],
  "metrics": {
    "full_grid_mix": {
      "signal": { "rms": 0.0 },
      "low_band": { "rms": 0.0 }
    },
    "mc202_question_answer_delta": { "rms": 0.0 }
  }
}"#
    .to_string()
}

fn fixture_observer() -> &'static str {
    include_str!("../../../tests/fixtures/observer_audio_correlation/events.ndjson")
}

fn fixture_manifest() -> &'static str {
    include_str!("../../../tests/fixtures/observer_audio_correlation/manifest.json")
}

fn first_playable_observer() -> &'static str {
    include_str!("../../../tests/fixtures/first_playable_jam_probe/events.ndjson")
}

fn w30_source_diff_manifest() -> String {
    r#"{
  "schema_version": 1,
  "pack_id": "w30-preview-smoke",
  "case_id": "raw_capture_source_window_preview",
  "result": "pass",
  "artifacts": [{}, {}, {}],
  "metrics": {
    "candidate": {
      "rms": 0.006986,
      "active_sample_ratio": 0.900499
    },
    "deltas": {
      "rms": 0.001073
    }
  }
}"#
    .to_string()
}
