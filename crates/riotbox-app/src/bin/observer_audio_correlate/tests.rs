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
    assert_eq!(summary.artifact_count, 5);
    assert_eq!(summary.grid_bpm_source, "source_timing");
    assert_eq!(summary.grid_bpm_decision_reason, "source_timing_ready");
    assert_eq!(summary.source_timing_bpm_delta, Some(0.0));
    assert_eq!(summary.commit_count, 1);
    assert_eq!(summary.commit_boundaries, ["NextBar"]);
    assert_eq!(
        summary
            .observer_source_timing
            .as_ref()
            .map(|timing| timing.quality.as_str()),
        Some("medium")
    );
    assert_eq!(
        summary
            .observer_source_timing
            .as_ref()
            .map(|timing| timing.phrase_status.as_str()),
        Some("uncertain")
    );
    assert!(summary.full_mix_rms.is_some_and(|rms| rms > 0.01));
    assert!(summary.full_mix_low_band_rms.is_some_and(|rms| rms > 0.01));
    assert_eq!(summary.mc202_question_answer_delta_rms, None);
    assert!(summary.source_timing.is_some());
    assert_eq!(
        summary.source_grid_output_drift,
        Some(SourceGridOutputDriftEvidence {
            hit_ratio: 1.0,
            max_peak_offset_ms: 1.27,
            max_allowed_peak_offset_ms: 70.0,
        })
    );
    assert_eq!(
        summary.tr909_source_grid_alignment,
        Some(SourceGridOutputDriftEvidence {
            hit_ratio: 1.0,
            max_peak_offset_ms: 1.27,
            max_allowed_peak_offset_ms: 70.0,
        })
    );
    assert_eq!(
        summary.w30_source_grid_alignment,
        Some(SourceGridOutputDriftEvidence {
            hit_ratio: 1.0,
            max_peak_offset_ms: 5.13,
            max_allowed_peak_offset_ms: 70.0,
        })
    );
    assert!(markdown.contains("Source timing phrase: `ambiguous_downbeat"));
    assert!(markdown.contains("Grid BPM source: `source_timing`"));
    assert!(markdown.contains("Grid BPM decision reason: `source_timing_ready`"));
    assert!(markdown.contains("Source timing BPM delta: `0.000000`"));
    assert!(markdown.contains("Source timing BPM agrees with grid: `yes`"));
    assert!(markdown.contains("Observer source timing: `src-beat08 cue=listen first"));
    assert!(markdown.contains("beat=tempo_only(0) downbeat=unknown(0) phrase=uncertain(0)"));
    assert!(markdown.contains("Source timing readiness: `grid locked readiness=ready"));
    assert!(markdown.contains("Source-grid output hit ratio: `1.000000`"));
    assert!(markdown.contains("TR-909 source-grid alignment: `hit_ratio=1.000000"));
    assert!(markdown.contains("W-30 source-grid alignment: `hit_ratio=1.000000"));
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
            "space -> toggle_transport",
            "c -> queue_capture_bar",
            "o -> queue_w30_audition",
            "p -> promote_last_capture",
            "w -> queue_w30_trigger_pad"
        ]
    );
    assert_eq!(summary.commit_count, 4);
    assert_eq!(summary.commit_boundaries, ["Phrase", "Bar", "Beat"]);
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
fn strict_evidence_rejects_invalid_observer_schema() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    fs::write(&observer_path, synthetic_observer_with_invalid_schema()).expect("write observer");

    let events = read_observer_events(&observer_path).expect("observer events");
    let error = validate_user_session_observer_events(&events).expect_err("invalid observer");

    assert!(
        error
            .to_string()
            .contains("riotbox.user_session_observer.v1")
    );
}

#[test]
fn strict_evidence_rejects_invalid_observer_launch_shape() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    fs::write(&observer_path, synthetic_observer_without_launch_source()).expect("write observer");

    let events = read_observer_events(&observer_path).expect("observer events");
    let error = validate_user_session_observer_events(&events).expect_err("invalid observer");

    assert!(error.to_string().contains("requires source_path or source"));
}

#[test]
fn non_strict_summary_still_reports_malformed_observer_for_local_inspection() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer_with_invalid_schema()).expect("write observer");
    fs::write(&manifest_path, synthetic_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");

    assert_eq!(summary.observer_schema, "riotbox.unknown_observer.v1");
    assert!(control_path_present(&summary));
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
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"transport":{},"queue":{},"runtime":{},"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":0.72,"quality":"low","degraded_policy":"manual_confirm","cue":"needs confirm","beat_status":"tempo_only","beat_count":0,"downbeat_status":"ambiguous","bar_count":0,"phrase_status":"uncertain","phrase_count":0,"primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat","phrase_uncertain"]},"recovery":{"present":false,"has_manual_candidates":false,"selected_candidate":null,"candidate_count":0,"candidates":[],"manual_choice_dry_run":null}}}"#,
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
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"}}"#,
        r#"{"event":"audio_runtime","status":"started"}"#,
        r#"{"event":"key_outcome","key":"space","outcome":"transport started"}"#,
    ]
    .join("\n")
        + "\n"
}

fn synthetic_observer_with_invalid_schema() -> String {
    [
        r#"{"event":"observer_started","schema":"riotbox.unknown_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"}}"#,
        r#"{"event":"audio_runtime","status":"started"}"#,
        r#"{"event":"key_outcome","key":"space","outcome":"transport started"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
    ]
    .join("\n")
        + "\n"
}

fn synthetic_observer_without_launch_source() -> String {
    [
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest"}}"#,
        r#"{"event":"audio_runtime","status":"started"}"#,
        r#"{"event":"key_outcome","key":"space","outcome":"transport started"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
    ]
    .join("\n")
        + "\n"
}

fn synthetic_manifest() -> String {
    r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "grid_bpm_source": "source_timing",
  "grid_bpm_decision_reason": "source_timing_ready",
  "source_timing_bpm_delta": 0.0,
  "artifacts": [{}, {}, {}, {}, {}],
  "source_timing": {
    "source_id": "source.wav",
    "policy_profile": "dance_loop_auto_readiness",
    "readiness": "ready",
    "requires_manual_confirm": false,
    "primary_bpm": 128.397,
    "bpm_agrees_with_grid": true,
    "beat_status": "stable",
    "downbeat_status": "ambiguous",
    "primary_downbeat_offset_beats": 0,
    "confidence_result": "candidate_ambiguous",
    "drift_status": "stable",
    "phrase_status": "ambiguous_downbeat",
    "alternate_evidence_count": 2,
    "warning_codes": ["AmbiguousDownbeat", "PhraseUncertain"]
  },
  "metrics": {
    "full_grid_mix": {
      "signal": { "rms": 0.1 },
      "low_band": { "rms": 0.08 }
    },
    "source_grid_output_drift": {
      "beat_count": 8,
      "hit_count": 7,
      "hit_ratio": 0.875,
      "max_peak_offset_ms": 12.5,
      "max_allowed_peak_offset_ms": 70.0
    }
  }
}"#
    .to_string()
}

fn synthetic_manifest_without_metrics() -> String {
    r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "artifacts": [{}, {}, {}, {}, {}],
  "metrics": {}
}"#
    .to_string()
}

fn synthetic_manifest_with_collapsed_metrics() -> String {
    r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "artifacts": [{}, {}, {}, {}, {}],
  "metrics": {
    "full_grid_mix": {
      "signal": { "rms": 0.0 },
      "low_band": { "rms": 0.0 }
    }
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
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic-first-playable-source.wav"}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started","host":"headless-probe"}"#,
        "\n",
        r#"{"event":"key_outcome","key":"space","outcome":"toggle_transport"}"#,
        "\n",
        r#"{"event":"key_outcome","key":"c","outcome":"queue_capture_bar"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":101,"boundary":"Phrase","beat_index":16,"bar_index":4,"phrase_index":1,"commit_sequence":1}]}"#,
        "\n",
        r#"{"event":"key_outcome","key":"o","outcome":"queue_w30_audition"}"#,
        "\n",
        r#"{"event":"key_outcome","key":"p","outcome":"promote_last_capture"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":102,"boundary":"Bar","beat_index":32,"bar_index":8,"phrase_index":2,"commit_sequence":1},{"action_id":103,"boundary":"Bar","beat_index":32,"bar_index":8,"phrase_index":2,"commit_sequence":2}]}"#,
        "\n",
        r#"{"event":"key_outcome","key":"w","outcome":"queue_w30_trigger_pad"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":104,"boundary":"Beat","beat_index":48,"bar_index":12,"phrase_index":3,"commit_sequence":1}]}"#,
        "\n",
    )
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
