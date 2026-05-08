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
    let anchor_alignment = summary
        .source_timing_anchor_alignment
        .as_ref()
        .expect("anchor alignment");
    assert_eq!(anchor_alignment.status, "aligned");
    assert_eq!(
        anchor_alignment
            .observer
            .as_ref()
            .expect("observer anchors")
            .primary_anchor_count,
        4
    );
    assert_eq!(
        anchor_alignment
            .manifest
            .as_ref()
            .expect("manifest anchors")
            .primary_anchor_count,
        8
    );
    assert!(anchor_alignment.issues.is_empty());
    let groove_alignment = summary
        .source_timing_groove_alignment
        .as_ref()
        .expect("groove alignment");
    assert_eq!(groove_alignment.status, "aligned");
    assert_eq!(
        groove_alignment
            .observer
            .as_ref()
            .expect("observer groove")
            .primary_groove_residual_count,
        2
    );
    assert_eq!(
        groove_alignment
            .manifest
            .as_ref()
            .expect("manifest groove")
            .primary_groove_residual_count,
        2
    );
    assert!(groove_alignment.issues.is_empty());
    assert!(markdown.contains("Source timing alignment: `aligned"));
    assert!(markdown.contains("Source timing anchor alignment: `aligned"));
    assert!(markdown.contains("Source timing groove alignment: `aligned"));
    assert_eq!(
        json["output_path"]["source_timing_alignment"]["status"],
        "aligned"
    );
    assert_eq!(
        json["output_path"]["source_timing_anchor_alignment"]["status"],
        "aligned"
    );
    assert_eq!(
        json["output_path"]["source_timing_groove_alignment"]["status"],
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

#[test]
fn strict_evidence_rejects_contradictory_source_timing_anchor_alignment() {
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
        manifest_with_source_timing_anchor_counts(
            128.397,
            &["PhraseUncertain"],
            SourceTimingAnchorEvidence {
                primary_anchor_count: 0,
                primary_kick_anchor_count: 0,
                primary_backbeat_anchor_count: 0,
                primary_transient_anchor_count: 0,
            },
        ),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("anchor mismatch");
    let markdown = render_markdown(&summary);

    assert!(
        error
            .to_string()
            .contains("source_timing_anchor_alignment.manifest_anchor_count=0")
    );
    assert!(
        error
            .to_string()
            .contains("source_timing_anchor_alignment.manifest_kick_anchor_count=0")
    );
    assert!(markdown.contains("Source timing anchor alignment: `mismatch"));
    assert!(markdown.contains("Output path present: `no`"));
}

#[test]
fn strict_evidence_rejects_contradictory_source_timing_groove_alignment() {
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
        manifest_with_source_timing_groove_counts(
            128.397,
            &["PhraseUncertain"],
            SourceTimingGrooveEvidence {
                primary_groove_residual_count: 0,
                primary_max_abs_offset_ms: 0.0,
                primary_groove_preview: Vec::new(),
            },
        ),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("groove mismatch");
    let markdown = render_markdown(&summary);

    assert!(
        error
            .to_string()
            .contains("source_timing_groove_alignment.manifest_residual_count=0")
    );
    assert!(markdown.contains("Source timing groove alignment: `mismatch"));
    assert!(markdown.contains("Output path present: `no`"));
}

#[test]
fn strict_evidence_rejects_locked_observer_static_output_policy() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, locked_observer_with_source_timing(128.0)).expect("write observer");
    fs::write(
        &manifest_path,
        manifest_with_static_manual_confirm_source_timing(128.0),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("locked/static mismatch");
    let markdown = render_markdown(&summary);

    assert!(
        error
            .to_string()
            .contains("source_timing_policy.locked_observer_grid_bpm_source=static_default")
    );
    assert!(
        error
            .to_string()
            .contains("source_timing_policy.locked_observer_requires_manual_confirm=true")
    );
    assert!(markdown.contains("Observer source timing: `src-timing cue=grid locked"));
    assert!(markdown.contains("Source timing alignment: `aligned"));
    assert!(markdown.contains("Output path present: `no`"));
}

fn observer_with_source_timing(bpm: f64, warning_code: &str) -> String {
    format!(
        r#"{{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{{"mode":"ingest","source":"synthetic.wav"}},"snapshot":{{"transport":{{}},"queue":{{}},"runtime":{{}},"source_timing":{{"present":true,"source_id":"src-timing","bpm_estimate":{bpm},"bpm_confidence":0.86,"quality":"medium","degraded_policy":"cautious","cue":"listen first","beat_status":"tempo_only","beat_count":0,"downbeat_status":"unknown","bar_count":0,"phrase_status":"uncertain","phrase_count":0,"primary_hypothesis_id":"probe-primary","hypothesis_count":2,"anchor_evidence":{{"primary_anchor_count":4,"primary_kick_anchor_count":1,"primary_backbeat_anchor_count":2,"primary_transient_anchor_count":1}},"groove_evidence":{{"primary_groove_residual_count":2,"primary_max_abs_offset_ms":12.5,"primary_groove_preview":[{{"subdivision":"eighth","offset_ms":-12.5,"confidence":0.72}},{{"subdivision":"sixteenth","offset_ms":6.25,"confidence":0.61}}]}},"primary_warning_code":"{warning_code}","warning_codes":["{warning_code}"]}},"recovery":{{"present":false,"has_manual_candidates":false,"selected_candidate":null,"candidate_count":0,"candidates":[],"manual_choice_dry_run":null}}}}}}"#,
    ) + "\n"
        + r#"{"event":"audio_runtime","status":"started"}"#
        + "\n"
        + r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#
        + "\n"
}

fn locked_observer_with_source_timing(bpm: f64) -> String {
    format!(
        r#"{{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{{"mode":"ingest","source":"synthetic.wav"}},"snapshot":{{"transport":{{}},"queue":{{}},"runtime":{{}},"source_timing":{{"present":true,"source_id":"src-timing","bpm_estimate":{bpm},"bpm_confidence":0.92,"quality":"high","degraded_policy":"locked","cue":"grid locked","beat_status":"grid","beat_count":16,"downbeat_status":"bar_locked","bar_count":4,"phrase_status":"phrase_locked","phrase_count":1,"primary_hypothesis_id":"probe-primary","hypothesis_count":1,"anchor_evidence":{{"primary_anchor_count":16,"primary_kick_anchor_count":4,"primary_backbeat_anchor_count":8,"primary_transient_anchor_count":4}},"groove_evidence":{{"primary_groove_residual_count":2,"primary_max_abs_offset_ms":6.0,"primary_groove_preview":[{{"subdivision":"eighth","offset_ms":-6.0,"confidence":0.78}},{{"subdivision":"sixteenth","offset_ms":3.5,"confidence":0.66}}]}},"primary_warning_code":null,"warning_codes":[]}},"recovery":{{"present":false,"has_manual_candidates":false,"selected_candidate":null,"candidate_count":0,"candidates":[],"manual_choice_dry_run":null}}}}}}"#,
    ) + "\n"
        + r#"{"event":"audio_runtime","status":"started"}"#
        + "\n"
        + r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#
        + "\n"
}

fn manifest_with_source_timing(primary_bpm: f64, warning_codes: &[&str]) -> String {
    manifest_with_source_timing_anchor_and_groove(
        primary_bpm,
        warning_codes,
        SourceTimingAnchorEvidence {
            primary_anchor_count: 8,
            primary_kick_anchor_count: 2,
            primary_backbeat_anchor_count: 4,
            primary_transient_anchor_count: 2,
        },
        source_timing_groove_evidence(),
    )
}

fn manifest_with_source_timing_anchor_counts(
    primary_bpm: f64,
    warning_codes: &[&str],
    anchor_evidence: SourceTimingAnchorEvidence,
) -> String {
    manifest_with_source_timing_anchor_and_groove(
        primary_bpm,
        warning_codes,
        anchor_evidence,
        source_timing_groove_evidence(),
    )
}

fn manifest_with_source_timing_groove_counts(
    primary_bpm: f64,
    warning_codes: &[&str],
    groove_evidence: SourceTimingGrooveEvidence,
) -> String {
    manifest_with_source_timing_anchor_and_groove(
        primary_bpm,
        warning_codes,
        SourceTimingAnchorEvidence {
            primary_anchor_count: 8,
            primary_kick_anchor_count: 2,
            primary_backbeat_anchor_count: 4,
            primary_transient_anchor_count: 2,
        },
        groove_evidence,
    )
}

fn manifest_with_source_timing_anchor_and_groove(
    primary_bpm: f64,
    warning_codes: &[&str],
    anchor_evidence: SourceTimingAnchorEvidence,
    groove_evidence: SourceTimingGrooveEvidence,
) -> String {
    let warning_codes = warning_codes
        .iter()
        .map(|code| format!(r#""{code}""#))
        .collect::<Vec<_>>()
        .join(", ");
    let groove_preview = groove_evidence
        .primary_groove_preview
        .iter()
        .map(|residual| {
            format!(
                r#"{{"subdivision":"{}","offset_ms":{},"confidence":{}}}"#,
                residual.subdivision, residual.offset_ms, residual.confidence
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"{{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "grid_bpm_source": "source_timing",
  "grid_bpm_decision_reason": "source_timing_ready",
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
    "anchor_evidence": {{
      "primary_anchor_count": {},
      "primary_kick_anchor_count": {},
      "primary_backbeat_anchor_count": {},
      "primary_transient_anchor_count": {}
    }},
    "groove_evidence": {{
      "primary_groove_residual_count": {},
      "primary_max_abs_offset_ms": {},
      "primary_groove_preview": [{groove_preview}]
    }},
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
}}"#,
        anchor_evidence.primary_anchor_count,
        anchor_evidence.primary_kick_anchor_count,
        anchor_evidence.primary_backbeat_anchor_count,
        anchor_evidence.primary_transient_anchor_count,
        groove_evidence.primary_groove_residual_count,
        groove_evidence.primary_max_abs_offset_ms,
    )
}

fn source_timing_groove_evidence() -> SourceTimingGrooveEvidence {
    SourceTimingGrooveEvidence {
        primary_groove_residual_count: 2,
        primary_max_abs_offset_ms: 12.5,
        primary_groove_preview: vec![
            SourceTimingGrooveResidualEvidence {
                subdivision: "eighth".into(),
                offset_ms: -12.5,
                confidence: 0.72,
            },
            SourceTimingGrooveResidualEvidence {
                subdivision: "sixteenth".into(),
                offset_ms: 6.25,
                confidence: 0.61,
            },
        ],
    }
}

fn manifest_with_static_manual_confirm_source_timing(primary_bpm: f64) -> String {
    format!(
        r#"{{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "grid_bpm_source": "static_default",
  "grid_bpm_decision_reason": "source_timing_requires_manual_confirm",
  "artifacts": [{{}}, {{}}, {{}}, {{}}, {{}}],
  "source_timing": {{
    "schema": "riotbox.source_timing_probe_readiness.v1",
    "schema_version": 1,
    "source_id": "source.wav",
    "policy_profile": "dance_loop_auto_readiness",
    "readiness": "ready",
    "requires_manual_confirm": true,
    "primary_bpm": {primary_bpm},
    "bpm_agrees_with_grid": true,
    "beat_status": "stable",
    "downbeat_status": "stable",
    "primary_downbeat_offset_beats": 0,
    "confidence_result": "candidate_cautious",
    "drift_status": "stable",
    "phrase_status": "stable",
    "alternate_evidence_count": 0,
    "anchor_evidence": {{
      "primary_anchor_count": 16,
      "primary_kick_anchor_count": 4,
      "primary_backbeat_anchor_count": 8,
      "primary_transient_anchor_count": 4
    }},
    "groove_evidence": {{
      "primary_groove_residual_count": 2,
      "primary_max_abs_offset_ms": 6.0,
      "primary_groove_preview": [
        {{"subdivision": "eighth", "offset_ms": -6.0, "confidence": 0.78}},
        {{"subdivision": "sixteenth", "offset_ms": 3.5, "confidence": 0.66}}
      ]
    }},
    "warning_codes": []
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
