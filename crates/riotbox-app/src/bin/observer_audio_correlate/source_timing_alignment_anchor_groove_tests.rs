use super::*;
use std::fs;

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
