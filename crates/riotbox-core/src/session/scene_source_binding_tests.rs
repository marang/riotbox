use super::*;
use crate::source_graph::{
    DecodeProfile, EnergyClass, GraphProvenance, SourceDescriptor,
    primary_grid_anchor_seconds_for_projected_scene,
};

fn graph() -> SourceGraph {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: "source".into(),
            path: "metadata-only.wav".into(),
            content_hash: "test".into(),
            duration_seconds: 16.0,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "test".into(),
            provider_set: vec![],
            generated_at: "test".into(),
            source_hash: "test".into(),
            analysis_seed: 0,
            run_notes: None,
        },
    );
    for (id, label, bar, energy) in [
        ("first", SectionLabelHint::Drop, 1, EnergyClass::High),
        ("second", SectionLabelHint::Break, 5, EnergyClass::Low),
    ] {
        graph.sections.push(Section {
            section_id: id.into(),
            label_hint: label,
            bar_start: bar,
            bar_end: bar + 3,
            start_seconds: (bar - 1) as f32 * 2.0,
            end_seconds: (bar + 3) as f32 * 2.0,
            energy_class: energy,
            confidence: 1.0,
            tags: vec![],
        });
    }
    graph
}

#[test]
fn migrated_scene_identity_survives_label_and_section_sort_order_changes() {
    let mut graph = graph();
    let mut state = SceneState {
        scenes: vec!["scene-01-old-name".into(), "scene-02-break".into()],
        ..SceneState::default()
    };
    let expected = state
        .source_section(&graph, &state.scenes[0])
        .unwrap()
        .section_id
        .clone();
    state.migrate_source_bindings(&graph);
    let bytes = serde_json::to_vec(&state).unwrap();
    let mut restored: SceneState = serde_json::from_slice(&bytes).unwrap();
    graph.sections[0].label_hint = SectionLabelHint::Outro;
    graph.sections[0].bar_start = 100;
    graph.sections.reverse();
    restored.migrate_source_bindings(&graph);
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    assert_eq!(
        restored
            .source_section(&graph, &state.scenes[0])
            .unwrap()
            .section_id,
        expected
    );
    assert!(restored.validate_source_bindings(&graph).is_ok());
}

#[test]
fn explicit_opaque_scene_binding_needs_no_label_or_index() {
    let graph = graph();
    let state = SceneState {
        source_bindings: Some(vec![SceneSourceBinding {
            scene_id: "performer-choice".into(),
            source_id: "source".into(),
            section_id: "second".into(),
        }]),
        ..SceneState::default()
    };
    let section = state
        .source_section(&graph, &"performer-choice".into())
        .unwrap();
    assert_eq!(section.section_id.as_str(), "second");
    // No implicit numeric-name fallback after an explicit binding list exists.
    assert!(
        state
            .source_section(&graph, &"scene-01-drop".into())
            .is_none()
    );
    assert!(
        primary_grid_anchor_seconds_for_projected_scene(&graph, &state, &"performer-choice".into())
            .is_none()
    );
}

#[test]
fn legacy_missing_field_is_distinct_from_explicit_unbound_and_invalid_refs() {
    let graph = graph();
    let legacy: SceneState = serde_json::from_value(serde_json::json!({
        "active_scene": "scene-01-drop", "scenes": ["scene-01-drop"], "restore_scene": null,
    }))
    .unwrap();
    assert!(legacy.source_bindings.is_none());
    assert!(
        legacy
            .source_section(&graph, &"scene-01-drop".into())
            .is_some()
    );
    for id in [
        "scene-0-drop",
        "scene-999-drop",
        "scene-nope-drop",
        "scene-01-",
        "unknown",
    ] {
        assert!(legacy.source_section(&graph, &id.into()).is_none(), "{id}");
    }
    let mut explicit = legacy;
    explicit.source_bindings = Some(vec![]);
    explicit.migrate_source_bindings(&graph);
    assert!(
        explicit
            .source_section(&graph, &"scene-01-drop".into())
            .is_none()
    );
    let invalid = SceneSourceBinding {
        scene_id: "scene-01-drop".into(),
        source_id: "other-source".into(),
        section_id: "first".into(),
    };
    explicit.source_bindings = Some(vec![invalid]);
    assert!(matches!(
        explicit.validate_source_bindings(&graph),
        Err(SceneSourceBindingError::InvalidTarget(_))
    ));
    assert!(
        explicit
            .source_section(&graph, &"scene-01-drop".into())
            .is_none()
    );
    explicit.source_bindings = Some(projected_scene_bindings(&graph));
    let duplicate = explicit.source_bindings.as_ref().unwrap()[0].clone();
    explicit.source_bindings.as_mut().unwrap().push(duplicate);
    assert!(matches!(
        explicit.validate_source_bindings(&graph),
        Err(SceneSourceBindingError::DuplicateScene(_))
    ));
    assert!(
        explicit
            .source_section(&graph, &"scene-01-drop".into())
            .is_none()
    );
}
