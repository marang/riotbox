use super::*;
use std::fs;

#[test]
fn summarizes_aligned_observer_and_manifest_source_timing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(
        &observer_path,
        observer_with_source_timing(128.0, "phrase_uncertain"),
    )
    .expect("write observer");
    fs::write(
        &manifest_path,
        manifest_with_source_timing(128.397, &["PhraseUncertain", "AmbiguousDownbeat"]),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let markdown = render_markdown(&summary);
    let json: Value = serde_json::from_str(&render_json(&summary).expect("json")).expect("json");

    let alignment = summary.source_timing_alignment.as_ref().expect("alignment");
    assert_eq!(alignment.status, "aligned");
    assert!((alignment.bpm_delta.expect("bpm delta") - 0.397).abs() < 1.0e-9);
    assert_eq!(
        alignment.bpm_tolerance,
        SOURCE_TIMING_BPM_ALIGNMENT_TOLERANCE
    );
    assert_eq!(alignment.warning_overlap, vec!["phrase_uncertain"]);
    assert!(alignment.issues.is_empty());
    assert!(markdown.contains("Source timing alignment: `aligned"));
    assert_eq!(
        json["output_path"]["source_timing_alignment"]["status"],
        "aligned"
    );
    assert_eq!(
        json["output_path"]["source_timing_alignment"]["warning_overlap"][0],
        "phrase_uncertain"
    );
}

#[test]
fn strict_evidence_rejects_source_timing_alignment_mismatch() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(
        &observer_path,
        observer_with_source_timing(128.0, "phrase_uncertain"),
    )
    .expect("write observer");
    fs::write(
        &manifest_path,
        manifest_with_source_timing(132.5, &["HighDrift"]),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("timing mismatch");
    let markdown = render_markdown(&summary);

    assert!(
        error
            .to_string()
            .contains("source_timing_alignment.bpm_delta")
    );
    assert!(
        error
            .to_string()
            .contains("source_timing_alignment.warning_codes=no_overlap")
    );
    assert!(markdown.contains("Source timing alignment: `mismatch"));
    assert!(markdown.contains("Output path present: `no`"));
}

fn observer_with_source_timing(bpm: f64, warning_code: &str) -> String {
    format!(
        r#"{{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{{"mode":"ingest","source":"synthetic.wav"}},"snapshot":{{"transport":{{}},"queue":{{}},"runtime":{{}},"source_timing":{{"present":true,"source_id":"src-timing","bpm_estimate":{bpm},"bpm_confidence":0.86,"quality":"medium","degraded_policy":"cautious","primary_hypothesis_id":"probe-primary","hypothesis_count":2,"primary_warning_code":"{warning_code}","warning_codes":["{warning_code}"]}},"recovery":{{"present":false,"has_manual_candidates":false,"selected_candidate":null,"candidate_count":0,"candidates":[],"manual_choice_dry_run":null}}}}}}"#,
    ) + "\n"
        + r#"{"event":"audio_runtime","status":"started"}"#
        + "\n"
        + r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#
        + "\n"
}

fn manifest_with_source_timing(primary_bpm: f64, warning_codes: &[&str]) -> String {
    let warning_codes = warning_codes
        .iter()
        .map(|code| format!(r#""{code}""#))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"{{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "artifacts": [{{}}, {{}}, {{}}, {{}}, {{}}],
  "source_timing": {{
    "schema": "riotbox.source_timing_probe_readiness.v1",
    "schema_version": 1,
    "source_id": "source.wav",
    "policy_profile": "dance_loop_auto_readiness",
    "readiness": "weak",
    "requires_manual_confirm": true,
    "primary_bpm": {primary_bpm},
    "bpm_agrees_with_grid": true,
    "beat_status": "stable",
    "downbeat_status": "ambiguous",
    "primary_downbeat_offset_beats": 0,
    "confidence_result": "candidate_ambiguous",
    "drift_status": "stable",
    "phrase_status": "ambiguous_downbeat",
    "alternate_evidence_count": 2,
    "warning_codes": [{warning_codes}]
  }},
  "metrics": {{
    "full_grid_mix": {{
      "signal": {{ "rms": 0.1 }},
      "low_band": {{ "rms": 0.08 }}
    }},
    "source_grid_output_drift": {{
      "hit_ratio": 1.0,
      "max_peak_offset_ms": 1.27,
      "max_allowed_peak_offset_ms": 70.0
    }}
  }}
}}"#
    )
}
