use super::*;
use std::fs;

#[test]
fn strict_evidence_rejects_source_grid_output_drift_failures() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer()).expect("write observer");
    fs::write(&manifest_path, synthetic_manifest_with_drift_failures()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("drift output evidence");

    assert!(error.to_string().contains("output-path manifest evidence"));
    assert!(
        error
            .to_string()
            .contains("source_grid_output_drift.hit_ratio=0.250000")
    );
    assert!(
        error
            .to_string()
            .contains("source_grid_output_drift.max_peak_offset_ms=75.000000")
    );
    let markdown = render_markdown(&summary);
    assert!(markdown.contains("Source-grid output hit ratio: `0.250000`"));
    assert!(markdown.contains("Output path present: `no`"));
}

#[test]
fn strict_evidence_rejects_malformed_source_grid_output_drift() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer()).expect("write observer");
    fs::write(&manifest_path, synthetic_manifest_with_malformed_drift()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("malformed drift evidence");

    assert!(
        error
            .to_string()
            .contains("source_grid_output_drift=malformed")
    );
    assert!(render_markdown(&summary).contains("Output path present: `no`"));
}

fn synthetic_observer() -> String {
    [
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"}}"#,
        r#"{"event":"audio_runtime","status":"started"}"#,
        r#"{"event":"key_outcome","key":"space","outcome":"transport started"}"#,
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
    ]
    .join("\n")
        + "\n"
}

fn synthetic_manifest_with_drift_failures() -> String {
    r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "artifacts": [{}, {}, {}, {}, {}],
  "metrics": {
    "full_grid_mix": {
      "signal": { "rms": 0.1 },
      "low_band": { "rms": 0.08 }
    },
    "source_grid_output_drift": {
      "beat_count": 8,
      "hit_count": 2,
      "hit_ratio": 0.25,
      "max_peak_offset_ms": 75.0,
      "max_allowed_peak_offset_ms": 70.0
    }
  }
}"#
    .to_string()
}

fn synthetic_manifest_with_malformed_drift() -> String {
    r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "artifacts": [{}, {}, {}, {}, {}],
  "metrics": {
    "full_grid_mix": {
      "signal": { "rms": 0.1 },
      "low_band": { "rms": 0.08 }
    },
    "source_grid_output_drift": {
      "hit_ratio": 1.0,
      "max_allowed_peak_offset_ms": 70.0
    }
  }
}"#
    .to_string()
}
