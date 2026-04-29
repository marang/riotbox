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

#[test]
fn accepted_ghost_action_snapshot_replay_plan_uses_restored_commit_records() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: 32.0,
        beat_index: 32,
        bar_index: 8,
        phrase_index: 2,
        current_scene: Some(SceneId::from("scene-a")),
    });
    state.set_current_ghost_suggestion(ghost_fill_suggestion());

    assert!(matches!(
        state.accept_current_ghost_suggestion(1_000),
        GhostSuggestionQueueResult::Enqueued(_)
    ));
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        1_500,
    );
    assert_eq!(committed.len(), 1);
    state.session.snapshots = vec![
        Snapshot {
            snapshot_id: SnapshotId::from("before-ghost"),
            created_at: "2026-04-29T19:12:00Z".into(),
            label: "before ghost fill".into(),
            action_cursor: 0,
        },
        Snapshot {
            snapshot_id: SnapshotId::from("after-ghost"),
            created_at: "2026-04-29T19:12:01Z".into(),
            label: "after ghost fill".into(),
            action_cursor: state.session.action_log.actions.len(),
        },
    ];

    let tempdir = tempdir().expect("create ghost snapshot replay tempdir");
    let session_path = tempdir.path().join("ghost-snapshot-replay-session.json");
    save_session_json(&session_path, &state.session).expect("save ghost replay session");
    let reloaded = load_session_json(&session_path).expect("reload ghost replay session");

    let before_snapshot = reloaded
        .snapshots
        .iter()
        .find(|snapshot| snapshot.snapshot_id.as_str() == "before-ghost")
        .expect("before snapshot restored");
    let after_snapshot = reloaded
        .snapshots
        .iter()
        .find(|snapshot| snapshot.snapshot_id.as_str() == "after-ghost")
        .expect("after snapshot restored");
    let before_comparison = riotbox_core::replay::build_snapshot_replay_plan_comparison(
        &reloaded.action_log,
        before_snapshot,
    )
    .expect("before snapshot replay plan");
    let after_comparison =
        riotbox_core::replay::build_snapshot_replay_plan_comparison(&reloaded.action_log, after_snapshot)
            .expect("after snapshot replay plan");

    assert_eq!(before_comparison.origin.len(), 1);
    let ghost_entry = &before_comparison.origin[0];
    assert_eq!(ghost_entry.action.actor, ActorType::Ghost);
    assert_eq!(ghost_entry.action.command, ActionCommand::Tr909FillNext);
    assert_eq!(before_comparison.snapshot_suffix.len(), 1);
    assert_eq!(
        ghost_entry.commit_record.action_id,
        before_comparison.snapshot_suffix[0].action.id
    );
    assert!(after_comparison.snapshot_suffix.is_empty());

    let target_plan_from_before = riotbox_core::replay::build_replay_target_plan(
        &reloaded.action_log,
        std::slice::from_ref(before_snapshot),
        reloaded.action_log.actions.len(),
    )
    .expect("before snapshot target replay plan");
    let target_plan_from_after = riotbox_core::replay::build_replay_target_plan(
        &reloaded.action_log,
        &reloaded.snapshots,
        reloaded.action_log.actions.len(),
    )
    .expect("after snapshot target replay plan");

    assert_eq!(
        target_plan_from_before
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("before-ghost")
    );
    assert_eq!(target_plan_from_before.suffix.len(), 1);
    assert_eq!(target_plan_from_before.suffix[0].action.actor, ActorType::Ghost);
    let mut before_only_session = reloaded.clone();
    before_only_session.snapshots = vec![before_snapshot.clone()];
    let before_only_state = JamAppState::from_parts(before_only_session, None, ActionQueue::new());
    let dry_run_summary = before_only_state
        .restore_target_dry_run_summary(reloaded.action_log.actions.len())
        .expect("app dry-run summary for before snapshot");

    assert_eq!(
        dry_run_summary.target_action_cursor,
        reloaded.action_log.actions.len()
    );
    assert_eq!(dry_run_summary.origin_action_count, 1);
    assert_eq!(dry_run_summary.suffix_action_count, 1);
    assert!(dry_run_summary.needs_replay);
    assert_eq!(dry_run_summary.anchor_snapshot_id.as_deref(), Some("before-ghost"));
    assert_eq!(dry_run_summary.anchor_action_cursor, Some(0));
    assert_eq!(
        dry_run_summary.suffix_action_ids,
        vec![target_plan_from_before.suffix[0].action.id]
    );
    assert_eq!(
        dry_run_summary.suffix_commands,
        vec![ActionCommand::Tr909FillNext]
    );
    assert_eq!(
        target_plan_from_after
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("after-ghost")
    );
    assert!(target_plan_from_after.suffix.is_empty());
    let restored_state =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect("restore ghost replay");
    let exact_anchor_summary = restored_state
        .restore_target_dry_run_summary(reloaded.action_log.actions.len())
        .expect("app dry-run summary for exact anchor");
    assert_eq!(
        exact_anchor_summary.anchor_snapshot_id.as_deref(),
        Some("after-ghost")
    );
    assert_eq!(exact_anchor_summary.suffix_action_count, 0);
    assert!(!exact_anchor_summary.needs_replay);

    let no_snapshot_convergence =
        riotbox_core::replay::build_latest_snapshot_replay_convergence_summary(
            &reloaded.action_log,
            &[],
        )
        .expect("no-snapshot convergence summary");
    assert_eq!(
        no_snapshot_convergence.target_action_cursor,
        reloaded.action_log.actions.len()
    );
    assert_eq!(
        no_snapshot_convergence.origin_action_count,
        reloaded.action_log.actions.len()
    );
    assert_eq!(no_snapshot_convergence.origin_replay_entry_count, 1);
    assert_eq!(no_snapshot_convergence.snapshot_count, 0);
    assert_eq!(no_snapshot_convergence.anchor_snapshot_id, None);
    assert_eq!(no_snapshot_convergence.suffix_action_count, 1);
    assert!(no_snapshot_convergence.needs_replay);
    assert!(no_snapshot_convergence.needs_full_replay);

    let before_snapshot_convergence =
        riotbox_core::replay::build_latest_snapshot_replay_convergence_summary(
            &reloaded.action_log,
            std::slice::from_ref(before_snapshot),
        )
        .expect("before-snapshot convergence summary");
    assert_eq!(
        before_snapshot_convergence.anchor_snapshot_id.as_deref(),
        Some("before-ghost")
    );
    assert_eq!(before_snapshot_convergence.anchor_action_cursor, Some(0));
    assert_eq!(before_snapshot_convergence.suffix_action_count, 1);
    assert!(before_snapshot_convergence.needs_replay);
    assert!(!before_snapshot_convergence.needs_full_replay);
    assert_eq!(
        before_snapshot_convergence.suffix_action_ids,
        vec![target_plan_from_before.suffix[0].action.id]
    );
    assert_eq!(
        before_snapshot_convergence.suffix_commands,
        vec![ActionCommand::Tr909FillNext]
    );

    let latest_snapshot_convergence =
        riotbox_core::replay::build_latest_snapshot_replay_convergence_summary(
            &reloaded.action_log,
            &reloaded.snapshots,
        )
        .expect("latest-snapshot convergence summary");
    assert_eq!(
        latest_snapshot_convergence.target_action_cursor,
        reloaded.action_log.actions.len()
    );
    assert_eq!(
        latest_snapshot_convergence.origin_action_count,
        reloaded.action_log.actions.len()
    );
    assert_eq!(latest_snapshot_convergence.origin_replay_entry_count, 1);
    assert_eq!(latest_snapshot_convergence.snapshot_count, 2);
    assert_eq!(
        latest_snapshot_convergence.anchor_snapshot_id.as_deref(),
        Some("after-ghost")
    );
    assert_eq!(
        latest_snapshot_convergence.anchor_action_cursor,
        Some(reloaded.action_log.actions.len())
    );
    assert_eq!(latest_snapshot_convergence.suffix_action_count, 0);
    assert!(!latest_snapshot_convergence.needs_replay);
    assert!(!latest_snapshot_convergence.needs_full_replay);
    assert!(latest_snapshot_convergence.suffix_action_ids.is_empty());
    assert!(latest_snapshot_convergence.suffix_commands.is_empty());
}

#[test]
fn restored_runtime_view_warns_about_unsupported_replay_commands() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    let unsupported_action = Action {
        id: ActionId(77),
        actor: ActorType::User,
        command: ActionCommand::W30LoopFreeze,
        params: ActionParams::Mutation {
            intensity: 0.8,
            target_id: Some("cap-01".into()),
        },
        target: ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-a")),
            pad_id: Some(PadId::from("pad-01")),
            ..Default::default()
        },
        requested_at: 490,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(500),
        result: Some(ActionResult {
            accepted: true,
            summary: "loop-freeze committed".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("artifact-producing W-30 action".into()),
    };
    session.action_log.actions.push(unsupported_action);
    session.action_log.commit_records.push(ActionCommitRecord {
        action_id: ActionId(77),
        boundary: CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 40,
            bar_index: 10,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        commit_sequence: 1,
        committed_at: 500,
    });
    session.snapshots = vec![Snapshot {
        snapshot_id: SnapshotId::from("before-artifact"),
        created_at: "2026-04-29T22:10:00Z".into(),
        label: "before artifact action".into(),
        action_cursor: 0,
    }];

    let unsupported_action_cursor = session
        .action_log
        .actions
        .iter()
        .position(|action| action.id == ActionId(77))
        .expect("unsupported action exists in action log")
        + 1;
    let original_session = session.clone();
    let error =
        riotbox_core::replay::apply_replay_target_suffix_to_session(
            &mut session,
            unsupported_action_cursor,
            None,
        )
        .expect_err("unsupported W-30 suffix should reject");
    assert!(matches!(
        error,
        riotbox_core::replay::ReplayTargetExecutionError::Execution(
            riotbox_core::replay::ReplayExecutionError::UnsupportedAction {
                action_id: ActionId(77),
                command: ActionCommand::W30LoopFreeze,
            }
        )
    ));
    assert_eq!(
        session, original_session,
        "unsupported target suffix must not partially mutate restored state"
    );
    let diagnostic_state = JamAppState::from_parts(session.clone(), Some(graph), ActionQueue::new());
    let dry_run_summary = diagnostic_state
        .restore_target_dry_run_summary(unsupported_action_cursor)
        .expect("unsupported replay dry-run summary");
    assert_eq!(dry_run_summary.suffix_unsupported_action_count, 1);
    assert_eq!(
        dry_run_summary.suffix_unsupported_action_ids,
        vec![ActionId(77)]
    );
    assert_eq!(
        dry_run_summary.suffix_unsupported_commands,
        vec![ActionCommand::W30LoopFreeze]
    );

    let tempdir = tempdir().expect("create unsupported replay warning tempdir");
    let session_path = tempdir.path().join("unsupported-replay-session.json");
    save_session_json(&session_path, &session).expect("save unsupported replay fixture");

    let restored =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect("restore from session");

    assert!(restored.runtime_view.runtime_warnings.iter().any(|warning| {
        warning == "replay cannot cover 1 unsupported command(s) after snapshot: w30.loop_freeze"
    }));
}
