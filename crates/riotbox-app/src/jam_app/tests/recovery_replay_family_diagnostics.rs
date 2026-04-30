#[test]
fn recovery_surface_reports_replay_families_for_supported_mixed_suffix() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let autosave_path = dir.path().join("session.autosave.mixed-families.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T19:05:00Z"),
    )
    .expect("save canonical session");

    let mut candidate_session = SessionFile::new(
        "mixed-family-autosave",
        "riotbox-test",
        "2026-04-30T19:05:01Z",
    );
    candidate_session.snapshots.push(snapshot_payload_at_origin(
        "before-family-suffix",
        &candidate_session.runtime_state,
    ));
    push_family_action(
        &mut candidate_session,
        1,
        1,
        ActionCommand::Mc202GenerateAnswer,
    );
    push_family_action(&mut candidate_session, 2, 2, ActionCommand::Tr909FillNext);
    push_family_action(&mut candidate_session, 3, 3, ActionCommand::SceneRestore);
    save_session_json(&autosave_path, &candidate_session)
        .expect("save mixed-family autosave session");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");
    let candidate = surface
        .candidates
        .iter()
        .find(|candidate| candidate.path == autosave_path)
        .expect("mixed-family autosave candidate");

    assert_eq!(
        candidate.replay_readiness_label,
        "ready: replay 3 suffix action(s)"
    );
    assert_eq!(
        candidate.replay_suffix_label,
        "suffix 3 action(s): mc202.generate_answer, tr909.fill_next, scene.restore"
    );
    assert_eq!(
        candidate.replay_family_label,
        "families MC-202, TR-909, Scene | suffix 3"
    );
    assert_eq!(candidate.replay_unsupported_label, "unsupported none");

    let dry_run = surface
        .dry_run_manual_choice(&autosave_path)
        .expect("manual choice dry-run for mixed-family candidate");
    assert_eq!(
        dry_run.replay_family_label,
        "families MC-202, TR-909, Scene | suffix 3"
    );
    assert!(!dry_run.selected_for_restore);
}

#[test]
fn recovery_surface_reports_replay_family_for_unsupported_suffix() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let autosave_path = dir.path().join("session.autosave.unsupported-family.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T19:10:00Z"),
    )
    .expect("save canonical session");

    let mut candidate_session = SessionFile::new(
        "unsupported-family-autosave",
        "riotbox-test",
        "2026-04-30T19:10:01Z",
    );
    candidate_session.snapshots.push(snapshot_payload_at_origin(
        "before-unsupported-family",
        &candidate_session.runtime_state,
    ));
    candidate_session
        .action_log
        .actions
        .push(family_action(44, ActionCommand::MutateScene));
    candidate_session
        .action_log
        .commit_records
        .push(family_commit_record(44, 1));
    save_session_json(&autosave_path, &candidate_session)
        .expect("save unsupported-family autosave session");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");
    let candidate = surface
        .candidates
        .iter()
        .find(|candidate| candidate.path == autosave_path)
        .expect("unsupported-family autosave candidate");

    assert_eq!(
        candidate.replay_readiness_label,
        "blocked: 1 unsupported suffix action(s)"
    );
    assert_eq!(candidate.replay_family_label, "families Scene | suffix 1");
    assert_eq!(
        candidate.replay_unsupported_label,
        "unsupported suffix 1: mutate.scene"
    );
    assert_eq!(
        candidate.decision_label,
        "decision: blocked | replay unsupported"
    );
}

#[test]
fn recovery_surface_reports_empty_replay_family_without_replay_entries() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let autosave_path = dir.path().join("session.autosave.no-replay-family.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T19:15:00Z"),
    )
    .expect("save canonical session");
    save_session_json(
        &autosave_path,
        &SessionFile::new("no-replay-autosave", "riotbox-test", "2026-04-30T19:15:01Z"),
    )
    .expect("save no-replay autosave session");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");
    let candidate = surface
        .candidates
        .iter()
        .find(|candidate| candidate.path == autosave_path)
        .expect("no-replay autosave candidate");

    assert_eq!(candidate.replay_readiness_label, "ready: no replay entries");
    assert_eq!(candidate.replay_family_label, "families none | no replay");
}

fn snapshot_payload_at_origin(
    snapshot_id: &str,
    runtime_state: &riotbox_core::session::RuntimeState,
) -> Snapshot {
    Snapshot {
        snapshot_id: SnapshotId::from(snapshot_id),
        created_at: "2026-04-30T19:00:00Z".into(),
        label: "before replay-family suffix".into(),
        action_cursor: 0,
        payload: Some(riotbox_core::session::SnapshotPayload::from_runtime_state(
            &SnapshotId::from(snapshot_id),
            0,
            runtime_state,
        )),
    }
}

fn push_family_action(
    session: &mut SessionFile,
    action_id: u64,
    commit_sequence: u32,
    command: ActionCommand,
) {
    session
        .action_log
        .actions
        .push(family_action(action_id, command));
    session
        .action_log
        .commit_records
        .push(family_commit_record(action_id, commit_sequence));
}

fn family_action(action_id: u64, command: ActionCommand) -> Action {
    Action {
        id: ActionId(action_id),
        actor: ActorType::User,
        command,
        params: ActionParams::Empty,
        target: ActionTarget::default(),
        requested_at: 900 + action_id,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(1_000 + action_id),
        result: Some(ActionResult {
            accepted: true,
            summary: format!("{} committed", command.as_str()),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: None,
    }
}

fn family_commit_record(action_id: u64, commit_sequence: u32) -> ActionCommitRecord {
    ActionCommitRecord {
        action_id: ActionId(action_id),
        boundary: CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 40,
            bar_index: 10,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        commit_sequence,
        committed_at: 1_000 + action_id,
    }
}
