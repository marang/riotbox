use super::*;
use std::fs;

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
    assert_eq!(
        summary.observer_source_timing,
        Some(ObserverSourceTimingReadiness {
            source_id: "src-timing".to_string(),
            bpm_estimate: Some(128.0),
            bpm_confidence: 0.72,
            cue: "needs confirm".to_string(),
            quality: "low".to_string(),
            degraded_policy: "manual_confirm".to_string(),
            beat_status: "tempo_only".to_string(),
            beat_count: 0,
            downbeat_status: "ambiguous".to_string(),
            bar_count: 0,
            phrase_status: "uncertain".to_string(),
            phrase_count: 0,
            primary_hypothesis_id: Some("probe-primary".to_string()),
            hypothesis_count: 1,
            primary_warning_code: Some("ambiguous_downbeat".to_string()),
            warning_codes: vec![
                "ambiguous_downbeat".to_string(),
                "phrase_uncertain".to_string()
            ],
        })
    );
    assert_eq!(summary.pack_id, "feral-grid-demo");
    assert_eq!(summary.manifest_result, "pass");
    assert_eq!(summary.artifact_count, 5);
    assert_eq!(summary.full_mix_rms, Some(0.1));
    assert_eq!(summary.w30_candidate_rms, None);
    assert_eq!(
        summary.source_grid_output_drift,
        Some(SourceGridOutputDriftEvidence {
            hit_ratio: 0.875,
            max_peak_offset_ms: 12.5,
            max_allowed_peak_offset_ms: 70.0,
        })
    );
    assert_eq!(
        summary.tr909_source_grid_alignment,
        Some(SourceGridOutputDriftEvidence {
            hit_ratio: 0.875,
            max_peak_offset_ms: 12.5,
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
    assert!(markdown.contains("Source-grid output hit ratio: `0.875000`"));
    assert!(markdown.contains("Source-grid output max peak offset: `12.500000`"));
    assert!(markdown.contains("TR-909 source-grid alignment: `hit_ratio=0.875000"));
    assert!(markdown.contains("W-30 source-grid alignment: `hit_ratio=1.000000"));
    assert!(markdown.contains("Observer source timing: `src-timing cue=needs confirm"));
    assert!(markdown.contains("Source timing readiness: `needs confirm readiness=weak"));
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
    assert_eq!(
        json["control_path"]["observer_source_timing"]["cue"],
        "needs confirm"
    );
    assert_eq!(
        json["control_path"]["observer_source_timing"]["quality"],
        "low"
    );
    assert_eq!(
        json["control_path"]["observer_source_timing"]["beat_status"],
        "tempo_only"
    );
    assert_eq!(
        json["control_path"]["observer_source_timing"]["downbeat_status"],
        "ambiguous"
    );
    assert_eq!(
        json["control_path"]["observer_source_timing"]["phrase_status"],
        "uncertain"
    );
    assert_eq!(json["output_path"]["source_timing"]["cue"], "needs confirm");
    assert_eq!(
        json["control_path"]["observer_source_timing"]["warning_codes"][1],
        "phrase_uncertain"
    );
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
    assert_eq!(
        json["output_path"]["metrics"]["source_grid_output_drift"]["hit_ratio"].as_f64(),
        Some(0.875)
    );
    assert_eq!(
        json["output_path"]["metrics"]["tr909_source_grid_alignment"]["hit_ratio"].as_f64(),
        Some(0.875)
    );
    assert_eq!(
        json["output_path"]["metrics"]["w30_source_grid_alignment"]["hit_ratio"].as_f64(),
        Some(1.0)
    );
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

fn synthetic_manifest() -> String {
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
    },
    "tr909_source_grid_alignment": {
      "beat_count": 8,
      "hit_count": 7,
      "hit_ratio": 0.875,
      "max_peak_offset_ms": 12.5,
      "max_allowed_peak_offset_ms": 70.0
    },
    "w30_source_grid_alignment": {
      "beat_count": 8,
      "hit_count": 8,
      "hit_ratio": 1.0,
      "max_peak_offset_ms": 5.13,
      "max_allowed_peak_offset_ms": 70.0
    }
  }
}"#
    .to_string()
}
