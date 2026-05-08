use super::*;
use std::fs;

#[test]
fn strict_evidence_rejects_malformed_observer_source_timing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, observer_with_malformed_source_timing()).expect("write observer");
    fs::write(&manifest_path, passing_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("malformed observer timing");

    assert!(
        error
            .to_string()
            .contains("malformed observer source timing")
    );
    assert!(render_markdown(&summary).contains("Observer source timing: `malformed`"));
}

fn observer_with_malformed_source_timing() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":"nope","quality":"low","degraded_policy":"manual_confirm","primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat"]}}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        "\n",
    )
}

fn passing_manifest() -> &'static str {
    r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "artifacts": [{}, {}, {}, {}, {}],
  "metrics": {
    "full_grid_mix": {
      "signal": { "rms": 0.1 },
      "low_band": { "rms": 0.08 }
    }
  }
}"#
}
