use super::*;
use std::fs;

#[test]
fn summarizes_source_timing_downbeat_and_phrase_evidence() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer()).expect("write observer");
    fs::write(&manifest_path, synthetic_manifest_with_source_timing()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let markdown = render_markdown(&summary);
    let json: Value = serde_json::from_str(&render_json(&summary).expect("json")).expect("json");

    assert_eq!(
        summary.source_timing,
        Some(SourceTimingEvidence {
            source_id: "source.wav".to_string(),
            policy_profile: "dance_loop_auto_readiness".to_string(),
            readiness: "weak".to_string(),
            requires_manual_confirm: true,
            primary_bpm: Some(128.397),
            bpm_agrees_with_grid: Some(true),
            beat_status: "stable".to_string(),
            downbeat_status: "ambiguous".to_string(),
            primary_downbeat_offset_beats: Some(0),
            confidence_result: "candidate_ambiguous".to_string(),
            drift_status: "stable".to_string(),
            phrase_status: "ambiguous_downbeat".to_string(),
            alternate_evidence_count: 2,
            anchor_evidence: None,
            groove_evidence: None,
            warning_codes: vec![
                "PhraseUncertain".to_string(),
                "AmbiguousDownbeat".to_string(),
            ],
        })
    );
    assert!(markdown.contains("Source timing downbeat: `ambiguous offset=0`"));
    assert!(markdown.contains("Source timing readiness: `needs confirm readiness=weak"));
    assert!(markdown.contains(
        "Source timing phrase: `ambiguous_downbeat confidence=candidate_ambiguous drift=stable alternates=2`"
    ));
    assert_eq!(json["output_path"]["source_timing"]["cue"], "needs confirm");
    assert_eq!(
        json["output_path"]["source_timing"]["phrase_status"],
        "ambiguous_downbeat"
    );
    assert_eq!(json["output_path"]["grid_bpm_source"], "static_default");
    assert_eq!(
        json["output_path"]["grid_bpm_decision_reason"],
        "source_timing_requires_manual_confirm"
    );
    assert_eq!(json["output_path"]["source_timing_bpm_delta"], 0.397);
}

#[test]
fn strict_evidence_rejects_malformed_source_timing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, synthetic_observer()).expect("write observer");
    fs::write(
        &manifest_path,
        synthetic_manifest_with_malformed_source_timing(),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("malformed timing evidence");
    let markdown = render_markdown(&summary);

    assert!(error.to_string().contains("source_timing=malformed"));
    assert!(markdown.contains("Source timing phrase: `malformed`"));
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

fn synthetic_manifest_with_source_timing() -> String {
    r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "grid_bpm_source": "static_default",
  "grid_bpm_decision_reason": "source_timing_requires_manual_confirm",
  "source_timing_bpm_delta": 0.397,
  "artifacts": [{}, {}, {}, {}, {}],
  "source_timing": {
    "source_id": "source.wav",
    "policy_profile": "dance_loop_auto_readiness",
    "readiness": "weak",
    "requires_manual_confirm": true,
    "primary_bpm": 128.397,
    "bpm_agrees_with_grid": true,
    "beat_status": "stable",
    "downbeat_status": "ambiguous",
    "primary_downbeat_offset_beats": 0,
    "confidence_result": "candidate_ambiguous",
    "drift_status": "stable",
    "phrase_status": "ambiguous_downbeat",
    "alternate_evidence_count": 2,
    "warning_codes": ["PhraseUncertain", "AmbiguousDownbeat"]
  },
  "metrics": {
    "full_grid_mix": {
      "signal": { "rms": 0.1 },
      "low_band": { "rms": 0.08 }
    }
  }
}"#
    .to_string()
}

fn synthetic_manifest_with_malformed_source_timing() -> String {
    r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "grid_bpm_source": "static_default",
  "grid_bpm_decision_reason": "source_timing_requires_manual_confirm",
  "source_timing_bpm_delta": 0.397,
  "artifacts": [{}, {}, {}, {}, {}],
  "source_timing": {
    "source_id": "source.wav",
    "policy_profile": "dance_loop_auto_readiness",
    "readiness": "weak",
    "requires_manual_confirm": true,
    "primary_bpm": 128.397,
    "bpm_agrees_with_grid": true,
    "beat_status": "stable",
    "downbeat_status": "ambiguous",
    "primary_downbeat_offset_beats": 0,
    "confidence_result": "candidate_ambiguous",
    "drift_status": "stable",
    "alternate_evidence_count": 2,
    "warning_codes": ["PhraseUncertain", "AmbiguousDownbeat"]
  },
  "metrics": {
    "full_grid_mix": {
      "signal": { "rms": 0.1 },
      "low_band": { "rms": 0.08 }
    }
  }
}"#
    .to_string()
}
