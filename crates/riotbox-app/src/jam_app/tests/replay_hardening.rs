#[test]
fn replay_from_zero_restore_rebuilds_commit_boundary_and_queue_cursor() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut live = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let first = live.queue.enqueue(
        ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909FillNext,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        ),
        300,
    );
    let second = live.queue.enqueue(
        ActionDraft::new(
            ActorType::User,
            ActionCommand::MutateScene,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::Scene),
                scene_id: Some(SceneId::from("scene-1")),
                ..Default::default()
            },
        ),
        301,
    );
    let boundary = CommitBoundaryState {
        kind: CommitBoundary::Bar,
        beat_index: 80,
        bar_index: 20,
        phrase_index: 5,
        scene_id: Some(SceneId::from("scene-1")),
    };

    let committed = live.commit_ready_actions(boundary.clone(), 500);

    assert_eq!(committed.len(), 2);
    assert_eq!(committed[0].action_id, first);
    assert_eq!(committed[0].commit_sequence, 1);
    assert_eq!(committed[1].action_id, second);
    assert_eq!(committed[1].commit_sequence, 2);
    assert_eq!(live.runtime.last_commit_boundary, Some(boundary.clone()));

    let tempdir = tempdir().expect("create replay hardening tempdir");
    let session_path = tempdir.path().join("replay-from-zero-session.json");
    save_session_json(&session_path, &live.session).expect("save replay fixture session");

    let mut restored =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect("restore from session");

    assert_eq!(restored.runtime.last_commit_boundary, Some(boundary));
    assert_eq!(restored.queue.pending_actions().len(), 0);
    assert_eq!(restored.session.action_log.commit_records.len(), 2);
    assert_eq!(restored.session.action_log.commit_records[0].action_id, first);
    assert_eq!(restored.session.action_log.commit_records[1].action_id, second);
    assert_eq!(restored.jam_view.recent_actions[0].id, second.to_string());
    assert_eq!(restored.jam_view.recent_actions[1].id, first.to_string());

    let next_id = restored.queue.enqueue(
        ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909FillNext,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        ),
        800,
    );

    assert_eq!(next_id, ActionId(4));
}
