//! Reachable Capture/Source/Section identities and initial/legacy compatibility.
use super::fixtures::{
    action_error, assert_refused, capture, capture_session, graph, graph_ref, owner, payload,
    phrase_session, reference_error, session,
};
use crate::{
    action::ActionCommand,
    ids::ActionId,
    replay::{
        SnapshotRuntimeActionError as ActionError, SnapshotRuntimeField as Field,
        SnapshotRuntimeReference as Reference, SnapshotRuntimeReferenceError as ReferenceError,
        hydrate_replay_target_from_snapshot_payload as hydrate,
    },
    session::{CaptureSourceWindow, RuntimeState, SceneSourceBinding},
};

#[test]
fn declared_graph_storage_owns_section_resolution_not_a_supplied_override() {
    let mut candidate = phrase_session(false, Some("section-a"));
    candidate.source_graph_refs.push(graph_ref(Some(graph())));
    let mut conflicting = graph();
    conflicting.sections[0].section_id = "different-section".into();
    // A conflicting supplied graph must not reach traversal/resolver either.
    assert_refused(
        &candidate,
        0,
        Some(&conflicting),
        reference_error(
            Field::SourceGraphContext,
            Reference::Graph("source-a".into()),
            ReferenceError::IdentityMismatch,
        ),
    );
    candidate.source_graph_refs[0].embedded_graph = Some(conflicting);
    // Nor may a supplied same-SourceId graph rescue a missing embedded section.
    assert_refused(
        &candidate,
        0,
        Some(&graph()),
        reference_error(
            Field::SourceGraphContext,
            Reference::Graph("source-a".into()),
            ReferenceError::IdentityMismatch,
        ),
    );
    candidate.source_graph_refs[0].storage_mode = crate::session::GraphStorageMode::External;
    candidate.source_graph_refs[0].embedded_graph = Some(graph());
    // Incidental embedded bytes on an external reference are not its authority.
    assert_refused(
        &candidate,
        0,
        None,
        reference_error(
            Field::Mc202SourcePhrase,
            Reference::Graph("source-a".into()),
            ReferenceError::Unavailable,
        ),
    );
    hydrate(&candidate, 0, Some(&graph())).unwrap();
}

#[test]
fn lane_and_nested_undo_plans_resolve_sources_uniquely_and_preserve_legacy_absent_sections() {
    for nested in [false, true] {
        let field = if nested {
            Field::Mc202Undo
        } else {
            Field::Mc202SourcePhrase
        };
        let candidate = phrase_session(nested, None);
        let cursor = candidate.action_log.actions.len();
        hydrate(&candidate, cursor, None).unwrap();
        let mut missing = candidate.clone();
        missing.source_refs.clear();
        assert_refused(
            &missing,
            cursor,
            None,
            reference_error(
                field,
                Reference::Source("source-a".into()),
                ReferenceError::Missing,
            ),
        );
        let mut duplicate = candidate;
        duplicate.source_refs.push(duplicate.source_refs[0].clone());
        assert_refused(
            &duplicate,
            cursor,
            None,
            reference_error(
                field,
                Reference::Source("source-a".into()),
                ReferenceError::Ambiguous,
            ),
        );
    }
}

#[test]
fn explicit_sections_require_unique_available_session_graph_metadata_for_lane_and_undo_plans() {
    for nested in [false, true] {
        let field = if nested {
            Field::Mc202Undo
        } else {
            Field::Mc202SourcePhrase
        };
        let mut candidate = phrase_session(nested, Some("section-a"));
        let cursor = candidate.action_log.actions.len();
        let graph_reference = Reference::Graph("source-a".into());
        assert_refused(
            &candidate,
            cursor,
            None,
            reference_error(field, graph_reference.clone(), ReferenceError::Missing),
        );
        candidate.source_graph_refs.push(graph_ref(None));
        assert_refused(
            &candidate,
            cursor,
            None,
            reference_error(field, graph_reference.clone(), ReferenceError::Unavailable),
        );
        hydrate(&candidate, cursor, Some(&graph())).unwrap();
        candidate.source_graph_refs[0] = graph_ref(Some(graph()));
        hydrate(&candidate, cursor, None).unwrap();
        let mut wrong_graph = candidate.clone();
        wrong_graph.source_graph_refs[0]
            .embedded_graph
            .as_mut()
            .unwrap()
            .source
            .source_id = "foreign-source".into();
        assert_refused(
            &wrong_graph,
            cursor,
            None,
            reference_error(
                field,
                graph_reference.clone(),
                ReferenceError::IdentityMismatch,
            ),
        );
        let mut duplicate_graph = candidate.clone();
        duplicate_graph
            .source_graph_refs
            .push(duplicate_graph.source_graph_refs[0].clone());
        assert_refused(
            &duplicate_graph,
            cursor,
            None,
            reference_error(field, graph_reference, ReferenceError::Ambiguous),
        );
        let section_reference = Reference::Section {
            source_id: "source-a".into(),
            section_id: "section-a".into(),
        };
        candidate.source_graph_refs[0]
            .embedded_graph
            .as_mut()
            .unwrap()
            .sections
            .clear();
        assert_refused(
            &candidate,
            cursor,
            None,
            reference_error(field, section_reference.clone(), ReferenceError::Missing),
        );
        let mut duplicate_section = graph();
        duplicate_section
            .sections
            .push(duplicate_section.sections[0].clone());
        candidate.source_graph_refs[0] = graph_ref(Some(duplicate_section));
        assert_refused(
            &candidate,
            cursor,
            None,
            reference_error(field, section_reference, ReferenceError::Ambiguous),
        );
    }
}

#[test]
fn both_w30_capture_paths_reject_missing_and_duplicate_captures_and_accept_legacy_metadata() {
    for hook in [false, true] {
        let field = if hook {
            Field::W30HookArticulation
        } else {
            Field::W30LastCapture
        };
        let mut candidate = capture_session(hook);
        let reference = Reference::Capture("capture-a".into());
        assert_refused(
            &candidate,
            0,
            None,
            reference_error(field, reference.clone(), ReferenceError::Missing),
        );
        candidate.captures.push(capture("capture-a"));
        hydrate(&candidate, 0, None).unwrap();
        candidate.captures.push(capture("capture-a"));
        assert_refused(
            &candidate,
            0,
            None,
            reference_error(field, reference, ReferenceError::Ambiguous),
        );
    }
}

#[test]
fn reachable_capture_lineage_rejects_cycles_missing_members_future_and_wrong_producers() {
    let mut candidate = capture_session(false);
    let mut root = capture("capture-a");
    root.lineage_capture_refs.push("capture-b".into());
    candidate.captures.push(root);
    assert_refused(
        &candidate,
        0,
        None,
        reference_error(
            Field::CaptureLineage,
            Reference::Capture("capture-b".into()),
            ReferenceError::Missing,
        ),
    );
    let mut child = capture("capture-b");
    child.lineage_capture_refs.push("capture-a".into());
    candidate.captures.push(child);
    assert_refused(
        &candidate,
        0,
        None,
        reference_error(
            Field::CaptureLineage,
            Reference::Capture("capture-a".into()),
            ReferenceError::Cycle,
        ),
    );
    candidate.captures[1].lineage_capture_refs.clear();
    // Repeated references to a completed ancestor-free node are not a cycle.
    candidate.captures[0]
        .lineage_capture_refs
        .push("capture-b".into());
    hydrate(&candidate, 0, None).unwrap();

    let mut producer = owner(Field::CaptureLength, 900);
    producer.command = ActionCommand::CaptureLoop;
    let runtime = candidate.snapshots[0]
        .payload
        .as_ref()
        .unwrap()
        .runtime_state
        .clone();
    let mut owned = session(vec![producer], 0, runtime);
    owned.captures = candidate.captures;
    owned.captures[1].created_from_action = Some(ActionId(900));
    assert_refused(
        &owned,
        1,
        None,
        action_error(
            Field::CaptureProducer,
            900,
            ActionError::AfterAnchor {
                action_index: 0,
                action_cursor: 0,
            },
        ),
    );
    owned.snapshots[0].action_cursor = 1;
    owned.snapshots[0].payload.as_mut().unwrap().action_cursor = 1;
    hydrate(&owned, 1, None).unwrap();
    owned.action_log.actions[0].command = ActionCommand::TransportPlay;
    assert_refused(
        &owned,
        1,
        None,
        action_error(Field::CaptureProducer, 900, ActionError::WrongOwner),
    );
}

#[test]
fn reached_capture_windows_resolve_sources_but_unreached_bad_capture_metadata_is_ignored() {
    let mut candidate = capture_session(false);
    let mut reached = capture("capture-a");
    reached.source_window = Some(CaptureSourceWindow {
        source_id: "missing-source".into(),
        start_seconds: 0.0,
        end_seconds: 1.0,
        start_frame: 0,
        end_frame: 48_000,
        hook_selection: None,
    });
    candidate.captures.push(reached);
    assert_refused(
        &candidate,
        0,
        None,
        reference_error(
            Field::CaptureSourceWindow,
            Reference::Source("missing-source".into()),
            ReferenceError::Missing,
        ),
    );
    candidate.captures[0]
        .source_window
        .as_mut()
        .unwrap()
        .source_id = "source-a".into();
    let mut unreached = capture("unreached");
    unreached.created_from_action = Some(ActionId(444));
    unreached.lineage_capture_refs.push("absent-lineage".into());
    candidate.captures.push(unreached);
    hydrate(&candidate, 0, None).unwrap();
}

#[test]
fn explicit_scene_bindings_are_unique_and_resolve_sections_without_renaming_custom_scenes() {
    let binding = SceneSourceBinding {
        scene_id: "custom/scene:not-a-number".into(),
        source_id: "source-a".into(),
        section_id: "section-a".into(),
    };
    let mut runtime = RuntimeState::default();
    runtime.scene_state.source_bindings = Some(vec![binding.clone()]);
    let mut candidate = session(vec![], 0, runtime);
    candidate.source_graph_refs.push(graph_ref(Some(graph())));
    hydrate(&candidate, 0, None).unwrap();
    payload(&mut candidate)
        .scene_state
        .source_bindings
        .as_mut()
        .unwrap()
        .push(binding.clone());
    assert_refused(
        &candidate,
        0,
        None,
        reference_error(
            Field::SceneSourceBinding,
            Reference::SceneBinding(binding.scene_id),
            ReferenceError::Ambiguous,
        ),
    );
    payload(&mut candidate)
        .scene_state
        .source_bindings
        .as_mut()
        .unwrap()
        .pop();
    payload(&mut candidate)
        .scene_state
        .source_bindings
        .as_mut()
        .unwrap()[0]
        .section_id = "missing-section".into();
    assert_refused(
        &candidate,
        0,
        None,
        reference_error(
            Field::SceneSourceBinding,
            Reference::Section {
                source_id: "source-a".into(),
                section_id: "missing-section".into(),
            },
            ReferenceError::Missing,
        ),
    );
}

#[test]
fn initial_defaults_unbound_scenes_empty_pad_focus_and_opaque_state_are_preserved() {
    for bindings in [None, Some(vec![])] {
        let mut runtime = RuntimeState::default();
        runtime.transport.current_scene = Some("arbitrary-initial-scene".into());
        runtime.scene_state.active_scene = runtime.transport.current_scene.clone();
        runtime
            .scene_state
            .scenes
            .push("arbitrary-initial-scene".into());
        runtime.scene_state.source_bindings = bindings;
        runtime.lane_state.w30.active_bank = Some("initial-bank".into());
        runtime.lane_state.w30.focused_pad = Some("empty-pad".into());
        runtime.lane_state.mc202.phrase_ref = Some("opaque future-looking phrase-99999".into());
        runtime.lane_state.tr909.pattern_ref = Some("opaque-bar-99999".into());
        runtime
            .lock_state
            .locked_object_ids
            .push("opaque:custom-object".into());
        let candidate = session(vec![], 0, runtime.clone());
        let restored = hydrate(&candidate, 0, None).unwrap();
        assert_eq!(restored.session.runtime_state, runtime);
    }
    hydrate(&session(vec![], 0, RuntimeState::default()), 0, None).unwrap();
}
