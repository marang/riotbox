#[test]
fn commit_pipeline_logs_action_before_promotion_summary_side_effect() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert!(state.queue_promote_last_capture(300));
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 1,
            scene_id: Some(SceneId::from("scene-1")),
        },
        360,
    );

    assert_eq!(committed.len(), 1);
    let action_id = committed[0].action_id;
    let logged_action = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.id == action_id)
        .expect("logged committed action");
    let commit_record = state
        .session
        .action_log
        .commit_records
        .iter()
        .find(|record| record.action_id == action_id)
        .expect("commit record");

    assert_eq!(commit_record.commit_sequence, committed[0].commit_sequence);
    assert_eq!(commit_record.committed_at, 360);
    assert_eq!(
        logged_action.result.as_ref().map(|result| result.summary.as_str()),
        Some("keeper | promoted to pad bank-a/pad-01")
    );
    assert_eq!(
        state.session.captures[0]
            .assigned_target
            .as_ref()
            .map(capture_target_label)
            .as_deref(),
        Some("bank-a/pad-01")
    );
}

#[test]
fn commit_pipeline_materializes_loop_freeze_capture_before_w30_side_effects() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_w30_loop_freeze(410),
        Some(QueueControlResult::Enqueued)
    );
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 64,
            bar_index: 16,
            phrase_index: 4,
            scene_id: Some(SceneId::from("scene-1")),
        },
        512,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .map(ToString::to_string),
        Some("cap-02".into())
    );
    let frozen_capture = state
        .session
        .captures
        .iter()
        .find(|capture| capture.capture_id == CaptureId::from("cap-02"))
        .expect("materialized frozen capture");
    assert!(frozen_capture.is_pinned);
    assert_eq!(frozen_capture.lineage_capture_refs, vec![CaptureId::from("cap-01")]);
    assert_eq!(
        frozen_capture.assigned_target.as_ref().map(capture_target_label).as_deref(),
        Some("bank-a/pad-01")
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("froze cap-01 into cap-02 for W-30 reuse on bank-a/pad-01")
    );
}

#[test]
fn commit_pipeline_mirrors_scene_side_effects_after_scene_commit() {
    let mut graph = sample_graph();
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Break,
        start_seconds: 16.0,
        end_seconds: 24.0,
        bar_start: 9,
        bar_end: 12,
        energy_class: EnergyClass::Medium,
        confidence: 0.84,
        tags: vec!["contrast".into()],
    });

    let mut session = sample_session(&graph);
    session.runtime_state.transport.current_scene = None;
    session.runtime_state.scene_state.active_scene = None;
    session.runtime_state.scene_state.scenes.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.queue_scene_select(610), QueueControlResult::Enqueued);
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 1,
            scene_id: Some(SceneId::from("scene-01-drop")),
        },
        700,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.transport.current_scene,
        Some(SceneId::from("scene-02-break"))
    );
    assert_eq!(
        state.runtime.transport.current_scene,
        state.session.runtime_state.transport.current_scene
    );
    assert_eq!(
        state.jam_view.scene.active_scene.as_deref(),
        Some("scene-02-break")
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("launched scene scene-02-break at bar 8 / phrase 1")
    );
}

fn capture_target_label(target: &CaptureTarget) -> String {
    match target {
        CaptureTarget::W30Pad { bank_id, pad_id } => format!("{bank_id}/{pad_id}"),
        CaptureTarget::Scene(scene_id) => scene_id.to_string(),
    }
}
