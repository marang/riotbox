use super::*;
use std::fs;

#[test]
fn strict_evidence_accepts_lane_recipe_manifest_for_recipe2_mc202_cases() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, recipe2_mc202_observer()).expect("write observer");
    fs::write(&manifest_path, lane_recipe_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");

    assert_eq!(summary.pack_id, "lane-recipe-listening-pack");
    assert_eq!(summary.lane_recipe_cases.len(), 7);
    assert!(output_path_present(&summary));
    validate_required_evidence(&summary).expect("lane recipe evidence");

    let json: Value = serde_json::from_str(&render_json(&summary).expect("json")).expect("json");
    let cases = json["output_path"]["lane_recipe_cases"]
        .as_array()
        .expect("lane recipe cases");
    assert_eq!(cases.len(), 7);
    assert_eq!(cases[0]["id"], "mc202-follower-to-answer");
    assert_eq!(cases[0]["result"], "pass");
    assert_eq!(cases[0]["mc202_phrase_grid"]["passed"], true);
    assert_eq!(cases[0]["mc202_phrase_grid"]["hit_ratio"], 1.0);
    assert_eq!(
        cases[0]["mc202_phrase_grid"]["starts_on_phrase_boundary"],
        true
    );
    assert_eq!(cases[0]["mc202_source_phrase_slot"]["passed"], true);
    assert_eq!(
        cases[0]["mc202_source_phrase_slot"]["phrase_grid_available"],
        true
    );
    assert_eq!(
        cases[0]["mc202_source_phrase_slot"]["starts_on_source_phrase_boundary"],
        true
    );
}

#[test]
fn strict_evidence_rejects_missing_lane_recipe_case() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, recipe2_mc202_observer()).expect("write observer");
    fs::write(&manifest_path, lane_recipe_manifest_missing_case()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("missing recipe case");

    assert!(error.to_string().contains("mc202-direct-to-hook-response"));
}

#[test]
fn strict_evidence_rejects_collapsed_lane_recipe_case() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, recipe2_mc202_observer()).expect("write observer");
    fs::write(&manifest_path, lane_recipe_manifest_collapsed_case()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("collapsed recipe case");

    assert!(error.to_string().contains("mc202-follower-to-pressure"));
    assert!(error.to_string().contains("signal_delta_rms"));
}

#[test]
fn strict_evidence_rejects_missing_mc202_phrase_grid_metric() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, recipe2_mc202_observer()).expect("write observer");
    fs::write(&manifest_path, lane_recipe_manifest_missing_phrase_grid()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("missing phrase grid metric");

    assert!(error.to_string().contains("mc202-follower-to-answer"));
    assert!(error.to_string().contains("mc202_phrase_grid=missing"));
}

#[test]
fn strict_evidence_rejects_missing_mc202_source_phrase_slot_metric() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, recipe2_mc202_observer()).expect("write observer");
    fs::write(
        &manifest_path,
        lane_recipe_manifest_missing_source_phrase_slot(),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error =
        validate_required_evidence(&summary).expect_err("missing source phrase slot metric");

    assert!(error.to_string().contains("mc202-follower-to-answer"));
    assert!(
        error
            .to_string()
            .contains("mc202_source_phrase_slot=missing")
    );
}

fn recipe2_mc202_observer() -> String {
    [
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic-recipe2-source.wav"}}"#,
        r#"{"event":"audio_runtime","status":"started"}"#,
        r#"{"event":"key_outcome","key":"space","outcome":"transport started"}"#,
        r#"{"event":"key_outcome","key":"g","outcome":"queue_mc202_generate_follower"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":301,"boundary":"Phrase","beat_index":16,"bar_index":4,"phrase_index":1,"commit_sequence":1}]}"#,
        r#"{"event":"key_outcome","key":"a","outcome":"queue_mc202_generate_answer"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":302,"boundary":"Phrase","beat_index":32,"bar_index":8,"phrase_index":2,"commit_sequence":1}]}"#,
        r#"{"event":"key_outcome","key":"P","outcome":"queue_mc202_generate_pressure"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":303,"boundary":"Phrase","beat_index":48,"bar_index":12,"phrase_index":3,"commit_sequence":1}]}"#,
        r#"{"event":"key_outcome","key":"I","outcome":"queue_mc202_generate_instigator"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":304,"boundary":"Phrase","beat_index":64,"bar_index":16,"phrase_index":4,"commit_sequence":1}]}"#,
        r#"{"event":"key_outcome","key":"G","outcome":"queue_mc202_mutate_phrase"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":305,"boundary":"Phrase","beat_index":80,"bar_index":20,"phrase_index":5,"commit_sequence":1}]}"#,
        r#"{"event":"key_outcome","key":">","outcome":"raise_mc202_touch"}"#,
    ]
    .join("\n")
        + "\n"
}

fn lane_recipe_manifest() -> String {
    lane_recipe_manifest_with_cases(&[
        lane_recipe_case(
            "mc202-follower-to-answer",
            "pass",
            0.005675,
            0.008565,
            0.005,
        ),
        lane_recipe_case("mc202-touch-low-to-high", "pass", 0.009559, 0.006182, 0.006),
        lane_recipe_case(
            "mc202-follower-to-pressure",
            "pass",
            0.005752,
            0.009299,
            0.004,
        ),
        lane_recipe_case(
            "mc202-follower-to-instigator",
            "pass",
            0.005908,
            0.009383,
            0.009,
        ),
        lane_recipe_case(
            "mc202-follower-to-mutated-drive",
            "pass",
            0.009877,
            0.009514,
            0.005,
        ),
        lane_recipe_case(
            "mc202-neutral-to-lift-contour",
            "pass",
            0.008217,
            0.007847,
            0.004,
        ),
        lane_recipe_case(
            "mc202-direct-to-hook-response",
            "pass",
            0.003446,
            0.008681,
            0.004,
        ),
    ])
}

fn lane_recipe_manifest_missing_case() -> String {
    lane_recipe_manifest_with_cases(&[
        lane_recipe_case(
            "mc202-follower-to-answer",
            "pass",
            0.005675,
            0.008565,
            0.005,
        ),
        lane_recipe_case("mc202-touch-low-to-high", "pass", 0.009559, 0.006182, 0.006),
        lane_recipe_case(
            "mc202-follower-to-pressure",
            "pass",
            0.005752,
            0.009299,
            0.004,
        ),
        lane_recipe_case(
            "mc202-follower-to-instigator",
            "pass",
            0.005908,
            0.009383,
            0.009,
        ),
        lane_recipe_case(
            "mc202-follower-to-mutated-drive",
            "pass",
            0.009877,
            0.009514,
            0.005,
        ),
        lane_recipe_case(
            "mc202-neutral-to-lift-contour",
            "pass",
            0.008217,
            0.007847,
            0.004,
        ),
    ])
}

fn lane_recipe_manifest_collapsed_case() -> String {
    lane_recipe_manifest_with_cases(&[
        lane_recipe_case(
            "mc202-follower-to-answer",
            "pass",
            0.005675,
            0.008565,
            0.005,
        ),
        lane_recipe_case("mc202-touch-low-to-high", "pass", 0.009559, 0.006182, 0.006),
        lane_recipe_case(
            "mc202-follower-to-pressure",
            "pass",
            0.005752,
            0.000001,
            0.004,
        ),
        lane_recipe_case(
            "mc202-follower-to-instigator",
            "pass",
            0.005908,
            0.009383,
            0.009,
        ),
        lane_recipe_case(
            "mc202-follower-to-mutated-drive",
            "pass",
            0.009877,
            0.009514,
            0.005,
        ),
        lane_recipe_case(
            "mc202-neutral-to-lift-contour",
            "pass",
            0.008217,
            0.007847,
            0.004,
        ),
        lane_recipe_case(
            "mc202-direct-to-hook-response",
            "pass",
            0.003446,
            0.008681,
            0.004,
        ),
    ])
}

fn lane_recipe_manifest_missing_phrase_grid() -> String {
    lane_recipe_manifest_with_cases(&[
        lane_recipe_case_without_phrase_grid(
            "mc202-follower-to-answer",
            "pass",
            0.005675,
            0.008565,
            0.005,
        ),
        lane_recipe_case("mc202-touch-low-to-high", "pass", 0.009559, 0.006182, 0.006),
        lane_recipe_case(
            "mc202-follower-to-pressure",
            "pass",
            0.005752,
            0.009299,
            0.004,
        ),
        lane_recipe_case(
            "mc202-follower-to-instigator",
            "pass",
            0.005908,
            0.009383,
            0.009,
        ),
        lane_recipe_case(
            "mc202-follower-to-mutated-drive",
            "pass",
            0.009877,
            0.009514,
            0.005,
        ),
        lane_recipe_case(
            "mc202-neutral-to-lift-contour",
            "pass",
            0.008217,
            0.007847,
            0.004,
        ),
        lane_recipe_case(
            "mc202-direct-to-hook-response",
            "pass",
            0.003446,
            0.008681,
            0.004,
        ),
    ])
}

fn lane_recipe_manifest_missing_source_phrase_slot() -> String {
    lane_recipe_manifest_with_cases(&[
        lane_recipe_case_without_source_phrase_slot(
            "mc202-follower-to-answer",
            "pass",
            0.005675,
            0.008565,
            0.005,
        ),
        lane_recipe_case("mc202-touch-low-to-high", "pass", 0.009559, 0.006182, 0.006),
        lane_recipe_case(
            "mc202-follower-to-pressure",
            "pass",
            0.005752,
            0.009299,
            0.004,
        ),
        lane_recipe_case(
            "mc202-follower-to-instigator",
            "pass",
            0.005908,
            0.009383,
            0.009,
        ),
        lane_recipe_case(
            "mc202-follower-to-mutated-drive",
            "pass",
            0.009877,
            0.009514,
            0.005,
        ),
        lane_recipe_case(
            "mc202-neutral-to-lift-contour",
            "pass",
            0.008217,
            0.007847,
            0.004,
        ),
        lane_recipe_case(
            "mc202-direct-to-hook-response",
            "pass",
            0.003446,
            0.008681,
            0.004,
        ),
    ])
}

fn lane_recipe_manifest_with_cases(cases: &[String]) -> String {
    format!(
        r#"{{
  "schema_version": 1,
  "pack_id": "lane-recipe-listening-pack",
  "result": "pass",
  "artifacts": [{{"role":"pack-summary","kind":"markdown_report","path":"pack-summary.md"}}],
  "case_count": {},
  "cases": [{}]
}}"#,
        cases.len(),
        cases.join(",")
    )
}

fn lane_recipe_case(
    id: &str,
    result: &str,
    candidate_rms: f64,
    signal_delta_rms: f64,
    min_signal_delta_rms: f64,
) -> String {
    lane_recipe_case_with_extra_metrics(
        id,
        result,
        candidate_rms,
        signal_delta_rms,
        min_signal_delta_rms,
        r#",
    "mc202_phrase_grid": {
      "resolution": "sixteenth",
      "phrase_length_steps": 64,
      "phrase_length_beats": 16.0,
      "position_beats": 32.0,
      "starts_on_phrase_boundary": true,
      "candidate_onset_count": 7,
      "grid_aligned_onset_count": 7,
      "hit_ratio": 1.0,
      "max_onset_offset_ms": 2.902494,
      "max_allowed_onset_offset_ms": 8.0,
      "passed": true
    },
    "mc202_source_phrase_slot": {
      "contract": "source_graph_phrase_grid.v0",
      "source_hypothesis_id": "lane-recipe-source-grid",
      "phrase_grid_available": true,
      "phrase_index": 3,
      "phrase_start_bar": 9,
      "phrase_end_bar": 12,
      "candidate_position_beats": 32.0,
      "candidate_bar_index": 9,
      "starts_on_source_phrase_boundary": true,
      "passed": true
    }"#,
    )
}

fn lane_recipe_case_without_phrase_grid(
    id: &str,
    result: &str,
    candidate_rms: f64,
    signal_delta_rms: f64,
    min_signal_delta_rms: f64,
) -> String {
    lane_recipe_case_with_extra_metrics(
        id,
        result,
        candidate_rms,
        signal_delta_rms,
        min_signal_delta_rms,
        "",
    )
}

fn lane_recipe_case_without_source_phrase_slot(
    id: &str,
    result: &str,
    candidate_rms: f64,
    signal_delta_rms: f64,
    min_signal_delta_rms: f64,
) -> String {
    lane_recipe_case_with_extra_metrics(
        id,
        result,
        candidate_rms,
        signal_delta_rms,
        min_signal_delta_rms,
        r#",
    "mc202_phrase_grid": {
      "resolution": "sixteenth",
      "phrase_length_steps": 64,
      "phrase_length_beats": 16.0,
      "position_beats": 32.0,
      "starts_on_phrase_boundary": true,
      "candidate_onset_count": 7,
      "grid_aligned_onset_count": 7,
      "hit_ratio": 1.0,
      "max_onset_offset_ms": 2.902494,
      "max_allowed_onset_offset_ms": 8.0,
      "passed": true
    }"#,
    )
}

fn lane_recipe_case_with_extra_metrics(
    id: &str,
    result: &str,
    candidate_rms: f64,
    signal_delta_rms: f64,
    min_signal_delta_rms: f64,
    extra_metrics: &str,
) -> String {
    format!(
        r#"{{
  "id": "{id}",
  "result": "{result}",
  "thresholds": {{"min_signal_delta_rms": {min_signal_delta_rms}}},
  "metrics": {{
    "candidate": {{"rms": {candidate_rms}}},
    "signal_delta": {{"rms": {signal_delta_rms}}}{extra_metrics}
  }}
}}"#
    )
}
