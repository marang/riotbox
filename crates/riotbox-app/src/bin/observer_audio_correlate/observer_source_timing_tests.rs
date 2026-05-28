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

#[test]
fn strict_evidence_rejects_observer_source_timing_cue_policy_mismatch() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, observer_with_mismatched_source_timing_cue())
        .expect("write observer");
    fs::write(&manifest_path, passing_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("mismatched observer timing cue");

    assert!(
        error
            .to_string()
            .contains("malformed observer source timing")
    );
    assert!(render_markdown(&summary).contains("Observer source timing: `malformed`"));
}

#[test]
fn strict_evidence_rejects_observer_source_timing_actionability_policy_mismatch() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(
        &observer_path,
        observer_with_mismatched_source_timing_actionability(),
    )
    .expect("write observer");
    fs::write(&manifest_path, passing_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error =
        validate_required_evidence(&summary).expect_err("mismatched observer timing actionability");

    assert!(
        error
            .to_string()
            .contains("malformed observer source timing")
    );
    assert!(render_markdown(&summary).contains("Observer source timing: `malformed`"));
}

#[test]
fn strict_evidence_rejects_missing_observer_source_timing_actionability() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(
        &observer_path,
        observer_without_source_timing_actionability(),
    )
    .expect("write observer");
    fs::write(&manifest_path, passing_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error =
        validate_required_evidence(&summary).expect_err("missing observer timing actionability");

    assert!(
        error
            .to_string()
            .contains("malformed observer source timing")
    );
    assert!(render_markdown(&summary).contains("Observer source timing: `malformed`"));
}

#[test]
fn strict_evidence_rejects_unknown_observer_source_timing_policy() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, observer_with_unknown_source_timing_policy())
        .expect("write observer");
    fs::write(&manifest_path, passing_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("unknown observer timing policy");

    assert!(
        error
            .to_string()
            .contains("malformed observer source timing")
    );
}

#[test]
fn strict_evidence_rejects_invalid_observer_source_timing_detail() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, observer_with_invalid_source_timing_detail())
        .expect("write observer");
    fs::write(&manifest_path, passing_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("invalid observer timing detail");

    assert!(
        error
            .to_string()
            .contains("malformed observer source timing")
    );
}

#[test]
fn strict_evidence_rejects_invalid_observer_source_timing_downbeat_score() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(
        &observer_path,
        observer_with_invalid_source_timing_downbeat_score(),
    )
    .expect("write observer");
    fs::write(&manifest_path, passing_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("invalid downbeat score");

    assert!(
        error
            .to_string()
            .contains("malformed observer source timing")
    );
}

#[test]
fn strict_evidence_rejects_invalid_observer_source_timing_downbeat_score_gap() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(
        &observer_path,
        observer_with_invalid_source_timing_downbeat_score_gap(),
    )
    .expect("write observer");
    fs::write(&manifest_path, passing_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("invalid downbeat score gap");

    assert!(
        error
            .to_string()
            .contains("malformed observer source timing")
    );
}

#[test]
fn strict_evidence_rejects_invalid_observer_source_timing_alternate_downbeat_count() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(
        &observer_path,
        observer_with_invalid_source_timing_alternate_downbeat_count(),
    )
    .expect("write observer");
    fs::write(&manifest_path, passing_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("invalid alternate downbeat count");

    assert!(
        error
            .to_string()
            .contains("malformed observer source timing")
    );
}

#[test]
fn strict_evidence_rejects_observer_source_timing_grid_use_policy_mismatch() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(
        &observer_path,
        observer_with_mismatched_source_timing_grid_use(),
    )
    .expect("write observer");
    fs::write(&manifest_path, passing_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("mismatched observer grid use");

    assert!(
        error
            .to_string()
            .contains("malformed observer source timing")
    );
}

#[test]
fn strict_evidence_rejects_grid_status_without_beat_count() {
    assert_rejects_malformed_observer_counts(
        observer_with_locked_count_override(r#""beat_count":16"#, r#""beat_count":0"#),
        "grid beat count",
    );
}

#[test]
fn strict_evidence_rejects_bar_locked_without_bar_count() {
    assert_rejects_malformed_observer_counts(
        observer_with_locked_count_override(r#""bar_count":4"#, r#""bar_count":0"#),
        "bar-locked bar count",
    );
}

#[test]
fn strict_evidence_rejects_phrase_locked_without_phrase_count() {
    assert_rejects_malformed_observer_counts(
        observer_with_locked_count_override(r#""phrase_count":1"#, r#""phrase_count":0"#),
        "phrase-locked phrase count",
    );
}

#[test]
fn strict_evidence_rejects_non_locked_phrase_count() {
    assert_rejects_malformed_observer_counts(
        observer_with_locked_count_override(
            r#""phrase_status":"phrase_locked""#,
            r#""phrase_status":"uncertain""#,
        ),
        "non-locked phrase count",
    );
}

fn assert_rejects_malformed_observer_counts(observer: String, context: &str) {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, observer).expect("write observer");
    fs::write(&manifest_path, passing_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err(context);

    assert!(
        error
            .to_string()
            .contains("malformed observer source timing")
    );
    assert!(render_markdown(&summary).contains("Observer source timing: `malformed`"));
}

fn observer_with_malformed_source_timing() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":"nope","quality":"low","degraded_policy":"manual_confirm","cue":"needs confirm","actionability":"confirm grid first","grid_use":"manual_confirm_only","primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat"]}}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        "\n",
    )
}

fn observer_with_unknown_source_timing_policy() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":0.72,"quality":"low","degraded_policy":"surprise","cue":"unknown","grid_use":"manual_confirm_only","primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat"]}}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        "\n",
    )
}

fn observer_with_mismatched_source_timing_cue() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":0.72,"quality":"low","degraded_policy":"manual_confirm","cue":"listen first","actionability":"confirm grid first","grid_use":"manual_confirm_only","primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat"]}}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        "\n",
    )
}

fn observer_with_mismatched_source_timing_actionability() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":0.72,"quality":"low","degraded_policy":"manual_confirm","cue":"needs confirm","actionability":"grid can steer moves","grid_use":"manual_confirm_only","primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat"]}}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        "\n",
    )
}

fn observer_without_source_timing_actionability() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":0.72,"quality":"low","degraded_policy":"manual_confirm","cue":"needs confirm","grid_use":"manual_confirm_only","primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat"]}}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        "\n",
    )
}

fn observer_with_invalid_source_timing_detail() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":0.72,"quality":"low","degraded_policy":"manual_confirm","cue":"needs confirm","actionability":"confirm grid first","grid_use":"manual_confirm_only","beat_status":"surprise","beat_count":0,"downbeat_status":"ambiguous","bar_count":0,"phrase_status":"uncertain","phrase_count":0,"primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat"]}}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        "\n",
    )
}

fn observer_with_invalid_source_timing_downbeat_score() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":0.72,"quality":"low","degraded_policy":"manual_confirm","cue":"needs confirm","actionability":"confirm grid first","grid_use":"manual_confirm_only","beat_status":"tempo_only","beat_count":0,"downbeat_status":"ambiguous","primary_downbeat_score":"hot","bar_count":0,"phrase_status":"uncertain","phrase_count":0,"primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat"]}}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        "\n",
    )
}

fn observer_with_invalid_source_timing_downbeat_score_gap() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":0.72,"quality":"low","degraded_policy":"manual_confirm","cue":"needs confirm","actionability":"confirm grid first","grid_use":"manual_confirm_only","beat_status":"tempo_only","beat_count":0,"downbeat_status":"ambiguous","primary_downbeat_score_gap":["nope"],"bar_count":0,"phrase_status":"uncertain","phrase_count":0,"primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat"]}}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        "\n",
    )
}

fn observer_with_invalid_source_timing_alternate_downbeat_count() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":0.72,"quality":"low","degraded_policy":"manual_confirm","cue":"needs confirm","actionability":"confirm grid first","grid_use":"manual_confirm_only","beat_status":"tempo_only","beat_count":0,"downbeat_status":"ambiguous","alternate_downbeat_phase_count":"three","bar_count":0,"phrase_status":"uncertain","phrase_count":0,"primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat"]}}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        "\n",
    )
}

fn observer_with_mismatched_source_timing_grid_use() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-timing","bpm_estimate":128.0,"bpm_confidence":0.72,"quality":"low","degraded_policy":"manual_confirm","cue":"needs confirm","actionability":"confirm grid first","grid_use":"locked_grid","beat_status":"tempo_only","beat_count":0,"downbeat_status":"ambiguous","bar_count":0,"phrase_status":"uncertain","phrase_count":0,"primary_hypothesis_id":"probe-primary","hypothesis_count":1,"primary_warning_code":"ambiguous_downbeat","warning_codes":["ambiguous_downbeat"]}}}"#,
        "\n",
        r#"{"event":"audio_runtime","status":"started"}"#,
        "\n",
        r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        "\n",
    )
}

fn observer_with_locked_count_override(target: &str, replacement: &str) -> String {
    observer_with_locked_source_timing().replace(target, replacement)
}

fn observer_with_locked_source_timing() -> &'static str {
    concat!(
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic.wav"},"snapshot":{"source_timing":{"present":true,"source_id":"src-locked","bpm_estimate":128.0,"bpm_confidence":0.92,"quality":"high","degraded_policy":"locked","cue":"grid locked","actionability":"grid can steer moves","grid_use":"locked_grid","beat_status":"grid","beat_count":16,"downbeat_status":"bar_locked","primary_downbeat_offset_beats":0,"bar_count":4,"phrase_status":"phrase_locked","phrase_count":1,"primary_hypothesis_id":"locked-primary","hypothesis_count":1,"anchor_evidence":{"primary_anchor_count":16,"primary_kick_anchor_count":4,"primary_backbeat_anchor_count":8,"primary_transient_anchor_count":4},"primary_anchor_cue":"anchors 16 | kick+backbeat","groove_evidence":{"primary_groove_residual_count":2,"primary_max_abs_offset_ms":6.0,"primary_groove_preview":[{"subdivision":"eighth","offset_ms":-6.0,"confidence":0.78},{"subdivision":"sixteenth","offset_ms":3.5,"confidence":0.66}]},"primary_warning_code":null,"warning_codes":[]}}}"#,
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
