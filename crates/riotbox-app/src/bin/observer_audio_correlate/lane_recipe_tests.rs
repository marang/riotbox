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

fn recipe2_mc202_observer() -> String {
    [
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic-recipe2-source.wav"}}"#,
        r#"{"event":"audio_runtime","status":"started"}"#,
        r#"{"event":"key_outcome","key":"space","outcome":"transport started"}"#,
        r#"{"event":"key_outcome","key":"g","outcome":"follower queued"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":301,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        r#"{"event":"key_outcome","key":"a","outcome":"answer queued"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":302,"boundary":"NextBar","beat_index":16,"bar_index":4,"phrase_index":1,"commit_sequence":2}]}"#,
        r#"{"event":"key_outcome","key":"P","outcome":"pressure queued"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":303,"boundary":"NextPhrase","beat_index":32,"bar_index":8,"phrase_index":2,"commit_sequence":3}]}"#,
        r#"{"event":"key_outcome","key":"I","outcome":"instigator queued"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":304,"boundary":"NextPhrase","beat_index":48,"bar_index":12,"phrase_index":3,"commit_sequence":4}]}"#,
        r#"{"event":"key_outcome","key":"G","outcome":"mutate queued"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":305,"boundary":"NextPhrase","beat_index":64,"bar_index":16,"phrase_index":4,"commit_sequence":5}]}"#,
        r#"{"event":"key_outcome","key":">","outcome":"touch raised"}"#,
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
    format!(
        r#"{{
  "id": "{id}",
  "result": "{result}",
  "thresholds": {{"min_signal_delta_rms": {min_signal_delta_rms}}},
  "metrics": {{
    "candidate": {{"rms": {candidate_rms}}},
    "signal_delta": {{"rms": {signal_delta_rms}}}
  }}
}}"#
    )
}
