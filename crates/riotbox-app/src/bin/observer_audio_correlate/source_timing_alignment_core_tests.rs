use super::*;
use serde_json::Value;
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
        manifest_with_grid_use_source_timing(
            128.397,
            &["PhraseUncertain", "AmbiguousDownbeat"],
            "locked_grid",
        ),
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
    assert_eq!(alignment.observer_grid_use, "manual_confirm_only");
    assert_eq!(alignment.manifest_grid_use.as_deref(), Some("locked_grid"));
    assert_eq!(alignment.grid_use_compatibility, "compatible");
    assert_eq!(alignment.observer_downbeat_offset_beats, Some(0));
    assert_eq!(alignment.manifest_downbeat_offset_beats, Some(0));
    assert_eq!(alignment.downbeat_offset_compatibility, "aligned");
    assert_eq!(alignment.downbeat_ambiguity_compatibility, "aligned");
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
    assert!(markdown.contains("grid_use=compatible"));
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
    assert_eq!(
        json["output_path"]["source_timing_alignment"]["observer_grid_use"],
        "manual_confirm_only"
    );
    assert_eq!(
        json["output_path"]["source_timing_alignment"]["manifest_grid_use"],
        "locked_grid"
    );
    assert_eq!(
        json["output_path"]["source_timing_alignment"]["grid_use_compatibility"],
        "compatible"
    );
    assert_eq!(
        json["output_path"]["source_timing_alignment"]["observer_downbeat_offset_beats"],
        0
    );
    assert_eq!(
        json["output_path"]["source_timing_alignment"]["manifest_downbeat_offset_beats"],
        0
    );
    assert_eq!(
        json["output_path"]["source_timing_alignment"]["downbeat_offset_compatibility"],
        "aligned"
    );
    assert_eq!(
        json["output_path"]["source_timing_alignment"]["downbeat_ambiguity_compatibility"],
        "aligned"
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
fn strict_evidence_rejects_source_timing_downbeat_offset_mismatch() {
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
        manifest_with_grid_use_source_timing(
            128.397,
            &["PhraseUncertain", "AmbiguousDownbeat"],
            "locked_grid",
        )
        .replace(
            r#""primary_downbeat_offset_beats": 0,"#,
            r#""primary_downbeat_offset_beats": 2,"#,
        ),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("downbeat offset mismatch");
    let markdown = render_markdown(&summary);

    assert!(
        error
            .to_string()
            .contains("source_timing_alignment.downbeat_offset observer=0 manifest=2")
    );
    assert!(markdown.contains("Source timing alignment: `mismatch"));
    assert!(markdown.contains("downbeat_offset=mismatch"));
    assert!(markdown.contains("Output path present: `no`"));
}

#[test]
fn strict_evidence_rejects_source_timing_downbeat_ambiguity_mismatch() {
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
        manifest_with_grid_use_source_timing(
            128.397,
            &["PhraseUncertain", "AmbiguousDownbeat"],
            "locked_grid",
        )
        .replace(
            r#""alternate_downbeat_phase_count": 3,"#,
            r#""alternate_downbeat_phase_count": 1,"#,
        ),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("downbeat ambiguity mismatch");
    let markdown = render_markdown(&summary);

    assert!(
        error
            .to_string()
            .contains("source_timing_alignment.downbeat_alternates observer=3 manifest=1")
    );
    assert!(markdown.contains("Source timing alignment: `mismatch"));
    assert!(markdown.contains("downbeat_ambiguity=mismatch"));
    assert!(markdown.contains("Output path present: `no`"));
}

#[test]
fn one_sided_downbeat_ambiguity_score_evidence_stays_partial() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(
        &observer_path,
        observer_with_source_timing(128.0, "phrase_uncertain")
            .replace(r#","primary_downbeat_score":0.273"#, "")
            .replace(r#","primary_downbeat_score_gap":0.005"#, "")
            .replace(r#","alternate_downbeat_phase_count":3"#, ""),
    )
    .expect("write observer");
    fs::write(
        &manifest_path,
        manifest_with_grid_use_source_timing(128.0, &["PhraseUncertain"], "locked_grid"),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let alignment = summary.source_timing_alignment.as_ref().expect("alignment");

    assert_eq!(alignment.status, "aligned");
    assert_eq!(alignment.downbeat_offset_compatibility, "aligned");
    assert_eq!(alignment.downbeat_ambiguity_compatibility, "partial");
    assert!(alignment.issues.is_empty());
}
