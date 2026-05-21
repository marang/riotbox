use super::*;
use std::fs;

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

#[test]
fn strict_evidence_rejects_locked_observer_manual_manifest_grid_use_alignment() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, locked_observer_with_source_timing(128.0)).expect("write observer");
    fs::write(
        &manifest_path,
        manifest_with_grid_use_source_timing(128.397, &["PhraseUncertain"], "manual_confirm_only"),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let alignment = summary.source_timing_alignment.as_ref().expect("alignment");
    let error = validate_required_evidence(&summary).expect_err("grid-use alignment mismatch");
    let markdown = render_markdown(&summary);

    assert_eq!(alignment.grid_use_compatibility, "mismatch");
    assert!(error.to_string().contains(
        "source_timing_alignment.grid_use observer=locked_grid manifest=manual_confirm_only"
    ));
    assert!(markdown.contains("grid_use=mismatch"));
    assert!(markdown.contains("Output path present: `no`"));
}

#[test]
fn strict_evidence_rejects_source_timing_grid_with_manual_only_grid_use() {
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
        manifest_with_source_timing(128.397, &["PhraseUncertain"]).replace(
            r#""policy_profile": "dance_loop_auto_readiness","#,
            r#""policy_profile": "dance_loop_auto_readiness",
    "grid_use": "manual_confirm_only","#,
        ),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("grid-use policy mismatch");
    let markdown = render_markdown(&summary);

    assert!(
        error
            .to_string()
            .contains("source_timing_policy.grid_use=manual_confirm_only expected=locked_grid")
    );
    assert!(markdown.contains("Source timing grid use: `manual_confirm_only`"));
    assert!(markdown.contains("Output path present: `no`"));
}

#[test]
fn strict_evidence_rejects_static_default_with_locked_grid_use() {
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
        manifest_with_static_manual_confirm_source_timing(128.0).replace(
            r#""policy_profile": "dance_loop_auto_readiness","#,
            r#""policy_profile": "dance_loop_auto_readiness",
    "grid_use": "locked_grid","#,
        ),
    )
    .expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("grid-use static mismatch");
    let markdown = render_markdown(&summary);

    assert!(
        error
            .to_string()
            .contains("source_timing_policy.grid_use=locked_grid expected=not_locked_grid")
    );
    assert!(markdown.contains("Source timing grid use: `locked_grid`"));
    assert!(markdown.contains("Output path present: `no`"));
}
