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

#[test]
fn strict_evidence_rejects_missing_required_source_grid_alignment() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer()).expect("write observer");
    fs::write(
        &manifest_path,
        synthetic_manifest_without_required_alignment(),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("missing source-grid alignment");

    assert!(
        error
            .to_string()
            .contains("source_grid_output_drift=missing")
    );
    assert!(
        error
            .to_string()
            .contains("tr909_source_grid_alignment=missing")
    );
    assert!(
        error
            .to_string()
            .contains("w30_source_grid_alignment=missing")
    );
    let markdown = render_markdown(&summary);
    assert!(markdown.contains("Output path present: `no`"));
    assert!(markdown.contains("source_grid_output_drift=missing"));
}

#[test]
fn strict_evidence_rejects_lane_source_grid_alignment_failures() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer()).expect("write observer");
    fs::write(
        &manifest_path,
        synthetic_manifest_with_lane_alignment_failures(),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("lane alignment evidence");

    assert!(error.to_string().contains("output-path manifest evidence"));
    assert!(
        error
            .to_string()
            .contains("w30_source_grid_alignment.hit_ratio=0.250000")
    );
    assert!(render_markdown(&summary).contains("W-30 source-grid alignment: `hit_ratio=0.250000"));
    assert!(render_markdown(&summary).contains("Output path present: `no`"));
}

#[test]
fn strict_evidence_rejects_malformed_lane_source_grid_alignment() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer()).expect("write observer");
    fs::write(
        &manifest_path,
        synthetic_manifest_with_malformed_lane_alignment(),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("malformed lane alignment");

    assert!(
        error
            .to_string()
            .contains("tr909_source_grid_alignment=malformed")
    );
    assert!(render_markdown(&summary).contains("Output path present: `no`"));
}

#[test]
fn strict_evidence_rejects_w30_source_loop_closure_failures() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer()).expect("write observer");
    fs::write(
        &manifest_path,
        synthetic_manifest_with_w30_loop_closure_failure(),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("w30 loop closure evidence");

    assert!(error.to_string().contains("output-path manifest evidence"));
    assert!(
        error
            .to_string()
            .contains("w30_source_loop_closure.passed=false")
    );
    assert!(
        error
            .to_string()
            .contains("w30_source_loop_closure.edge_delta_abs=0.120000")
    );
    let markdown = render_markdown(&summary);
    assert!(markdown.contains("W-30 source-loop closure: `passed=no"));
    assert!(markdown.contains("Output path present: `no`"));
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

fn synthetic_manifest_without_required_alignment() -> String {
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
    .to_string()
}

fn synthetic_manifest_with_lane_alignment_failures() -> String {
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
      "hit_count": 8,
      "hit_ratio": 1.0,
      "max_peak_offset_ms": 1.27,
      "max_allowed_peak_offset_ms": 70.0
    },
    "w30_source_grid_alignment": {
      "beat_count": 8,
      "hit_count": 2,
      "hit_ratio": 0.25,
      "max_peak_offset_ms": 85.0,
      "max_allowed_peak_offset_ms": 70.0
    }
  }
}"#
    .to_string()
}

fn synthetic_manifest_with_malformed_lane_alignment() -> String {
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
      "hit_count": 8,
      "hit_ratio": 1.0,
      "max_peak_offset_ms": 1.27,
      "max_allowed_peak_offset_ms": 70.0
    },
    "tr909_source_grid_alignment": {
      "hit_ratio": 1.0,
      "max_allowed_peak_offset_ms": 70.0
    }
  }
}"#
    .to_string()
}

fn synthetic_manifest_with_w30_loop_closure_failure() -> String {
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
      "hit_count": 8,
      "hit_ratio": 1.0,
      "max_peak_offset_ms": 1.27,
      "max_allowed_peak_offset_ms": 70.0
    },
    "tr909_source_grid_alignment": {
      "beat_count": 8,
      "hit_count": 8,
      "hit_ratio": 1.0,
      "max_peak_offset_ms": 1.27,
      "max_allowed_peak_offset_ms": 70.0
    },
    "w30_source_grid_alignment": {
      "beat_count": 8,
      "hit_count": 8,
      "hit_ratio": 1.0,
      "max_peak_offset_ms": 5.13,
      "max_allowed_peak_offset_ms": 70.0
    },
    "w30_source_loop_closure": {
      "passed": false,
      "selected_frame_count": 2048,
      "preview_rms": 0.145,
      "edge_delta_abs": 0.12,
      "max_allowed_edge_delta_abs": 0.06,
      "edge_abs_max": 0.08,
      "max_allowed_edge_abs": 0.04,
      "source_contains_selection": true,
      "reason": "source_chop_loop_closure_out_of_budget"
    }
  }
}"#
    .to_string()
}
