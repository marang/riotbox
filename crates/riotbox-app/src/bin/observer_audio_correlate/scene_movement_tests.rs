use super::*;
use std::fs;

#[test]
fn correlates_scene_movement_observer_to_output_evidence() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&observer_path, scene_movement_observer()).expect("write observer");
    fs::write(&manifest_path, fixture_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let markdown = render_markdown(&summary);
    let movement = summary
        .observer_scene_movement
        .as_ref()
        .expect("observer scene movement");

    assert_eq!(movement.kind, "launch");
    assert_eq!(movement.direction, "rise");
    assert_eq!(movement.tr909_intent, "drive");
    assert_eq!(movement.mc202_intent, "lift");
    assert_eq!(movement.from_scene.as_deref(), Some("scene-01-break"));
    assert_eq!(movement.to_scene, "scene-02-drop");
    assert!(movement.can_use_source_locked_scene_movement);
    assert_eq!(movement.source_anchor_seconds, Some(16.0));
    assert!(scene_movement_audio_evidence_failures(&summary).is_empty());
    assert!(markdown.contains("Observer scene movement: `launch scene-01-break -> scene-02-drop"));
    assert!(markdown.contains("Scene movement/audio evidence: `pass`"));
    validate_required_evidence(&summary).expect("scene movement evidence");
}

#[test]
fn strict_evidence_rejects_source_locked_scene_movement_without_anchor() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(
        &observer_path,
        scene_movement_observer_without_source_anchor(),
    )
    .expect("write observer");
    fs::write(&manifest_path, fixture_manifest()).expect("write manifest");

    let summary = build_summary(&observer_path, &manifest_path).expect("summary");
    let error = validate_required_evidence(&summary).expect_err("missing scene anchor evidence");

    assert!(error.to_string().contains("output-path manifest evidence"));
    assert!(
        error
            .to_string()
            .contains("scene_movement.source_anchor_seconds=missing")
    );
}

fn scene_movement_observer() -> String {
    scene_movement_observer_with_anchor(Some(16.0))
}

fn scene_movement_observer_without_source_anchor() -> String {
    scene_movement_observer_with_anchor(None)
}

fn scene_movement_observer_with_anchor(source_anchor_seconds: Option<f64>) -> String {
    let source_anchor =
        source_anchor_seconds.map_or_else(|| "null".to_string(), |value| format!("{value:.1}"));
    [
        r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest","source":"synthetic-scene.wav"}}"#.to_string(),
        r#"{"event":"audio_runtime","status":"started","host":"headless-probe"}"#.to_string(),
        r#"{"event":"key_outcome","key":"y","outcome":"queue_scene_select"}"#.to_string(),
        format!(
            r#"{{"event":"transport_commit","committed":[{{"action_id":42,"boundary":"NextBar","beat_index":36,"bar_index":9,"phrase_index":2,"commit_sequence":1}}],"snapshot":{{"scene":{{"active_scene":"scene-02-drop","last_movement":{{"kind":"launch","direction":"rise","tr909_intent":"drive","mc202_intent":"lift","intensity":0.84,"from_scene":"scene-01-break","to_scene":"scene-02-drop","committed_bar_index":9,"committed_phrase_index":2}},"arrangement_contract":{{"can_use_source_locked_scene_movement":true}},"source_monitor":{{"source_anchor_seconds":{source_anchor},"source_anchor_position_beats":36.0}}}}}}}}"#
        ),
    ]
    .join("\n")
        + "\n"
}

fn fixture_manifest() -> &'static str {
    include_str!("../../../tests/fixtures/observer_audio_correlation/manifest.json")
}
