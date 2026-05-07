use riotbox_core::persistence::save_session_json;
use tempfile::tempdir;

#[test]
fn renders_manual_recovery_prompt_in_warnings_and_help() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let autosave_path = dir.path().join("session.autosave.2026-04-29T211500Z.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-29T21:15:00Z"),
    )
    .expect("save canonical session");
    save_session_json(
        &autosave_path,
        &SessionFile::new("autosave", "riotbox-test", "2026-04-29T21:15:01Z"),
    )
    .expect("save autosave session");

    let mut shell = sample_shell_state();
    shell.set_recovery_surface(
        JamAppState::scan_session_recovery_surface(&target_path)
            .expect("scan recovery surface"),
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 38);

    assert!(
        rendered.contains("recovery: 1 manual recovery"),
        "{rendered}"
    );
    assert!(
        rendered.contains("candidate(s) need explicit review"),
        "{rendered}"
    );
    assert!(rendered.contains("manual review only"), "{rendered}");
    assert!(!rendered.contains("Warnings clear"), "{rendered}");

    shell.show_help = true;
    let rendered = render_jam_shell_snapshot(&shell, 120, 38);

    assert!(rendered.contains("Session recovery"), "{rendered}");
    assert!(
        rendered.contains("Manual recovery only: Riotbox did not choose, load, replace, or"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Selected candidate: none | dry-run only | no auto-restore"),
        "{rendered}"
    );
    assert!(rendered.contains("no restore selected"), "{rendered}");
    assert!(
        rendered.contains("Review candidate: session.autosave.2026-04-29T211500Z.json"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Replay/artifacts: families none | no replay | artifacts n/a"),
        "{rendered}"
    );
    assert!(
        rendered.contains("payload none | full replay"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Dry-run result: Dry-run only: candidate inspected"),
        "{rendered}"
    );
    assert!(rendered.contains("Next: inspect that file outside Riotbox"), "{rendered}");
    assert!(
        shell
            .recovery_surface
            .as_ref()
            .is_some_and(|surface| surface.selected_candidate.is_none()),
        "{rendered}"
    );
    assert!(rendered.contains("artifacts n/a"), "{rendered}");
    assert!(rendered.contains("payload none"), "{rendered}");
    assert!(rendered.contains("families none"), "{rendered}");
    assert!(rendered.contains("reviewable"), "{rendered}");
    assert!(rendered.contains("parseable session JSON"), "{rendered}");
}

#[test]
fn renders_manual_recovery_prompt_with_blocked_restore_replay_state() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let autosave_path = dir.path().join("session.autosave.2026-04-29T231000Z.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-29T23:10:00Z"),
    )
    .expect("save canonical session");
    save_session_json(
        &autosave_path,
        &SessionFile::new("autosave", "riotbox-test", "2026-04-29T23:10:01Z"),
    )
    .expect("save autosave session");

    let mut shell = sample_shell_state();
    shell.app.session.action_log.actions.clear();
    shell.app.session.action_log.commit_records.clear();
    shell.app.session.snapshots = vec![Snapshot {
        snapshot_id: SnapshotId::from("before-freeze"),
        created_at: "2026-04-29T23:10:00Z".into(),
        label: "before unsupported freeze".into(),
        action_cursor: 0,
        payload: Some(riotbox_core::session::SnapshotPayload::from_runtime_state(
            &SnapshotId::from("before-freeze"),
            0,
            &shell.app.session.runtime_state,
        )),
    }];
    shell.app.session.action_log.actions.push(Action {
        id: ActionId(55),
        actor: ActorType::User,
        command: ActionCommand::MutateScene,
        params: ActionParams::Mutation {
            intensity: 0.5,
            target_id: Some("scene-a".into()),
        },
        target: ActionTarget {
            scope: Some(TargetScope::Scene),
            ..Default::default()
        },
        requested_at: 700,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(900),
        result: Some(ActionResult {
            accepted: true,
            summary: "scene mutation committed".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: None,
    });
    shell
        .app
        .session
        .action_log
        .commit_records
        .push(ActionCommitRecord {
            action_id: ActionId(55),
            boundary: CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 72,
                bar_index: 18,
                phrase_index: 4,
                scene_id: Some(SceneId::from("scene-a")),
            },
            commit_sequence: 1,
            committed_at: 900,
        });
    shell.app.refresh_view();
    save_session_json(&autosave_path, &shell.app.session).expect("save blocked autosave session");
    shell.set_recovery_surface(
        JamAppState::scan_session_recovery_surface(&target_path)
            .expect("scan recovery surface"),
    );
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 38);

    assert!(
        rendered.contains("Review candidate: session.autosave.2026-04-29T231000Z.json"),
        "{rendered}"
    );
    assert!(rendered.contains("multi-blocked"), "{rendered}");
    assert!(rendered.contains("parseable session JSON"), "{rendered}");
    assert!(
        rendered.contains("Replay/artifacts: families Scene | suffix 1"),
        "{rendered}"
    );
    assert!(rendered.contains("artifacts blocked"), "{rendered}");
    assert!(rendered.contains("1 missing"), "{rendered}");
    assert!(rendered.contains("payload ready"), "{rendered}");
    assert!(rendered.contains("families Scene"), "{rendered}");
    assert!(rendered.contains("multi-blocked"), "{rendered}");
}

#[test]
fn renders_artifact_ready_replay_blocker_hint_without_selecting_candidate() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let autosave_path = dir.path().join("session.autosave.2026-04-30T092900Z.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T09:29:00Z"),
    )
    .expect("save canonical session");

    let mut shell = sample_shell_state();
    shell.app.session.action_log.actions.clear();
    shell.app.session.action_log.commit_records.clear();
    shell.app.session.snapshots = vec![Snapshot {
        snapshot_id: SnapshotId::from("before-freeze"),
        created_at: "2026-04-30T09:29:00Z".into(),
        label: "before unsupported freeze".into(),
        action_cursor: 0,
        payload: Some(riotbox_core::session::SnapshotPayload::from_runtime_state(
            &SnapshotId::from("before-freeze"),
            0,
            &shell.app.session.runtime_state,
        )),
    }];
    shell.app.session.action_log.actions.push(Action {
        id: ActionId(56),
        actor: ActorType::User,
        command: ActionCommand::MutateScene,
        params: ActionParams::Mutation {
            intensity: 0.5,
            target_id: Some("scene-a".into()),
        },
        target: ActionTarget {
            scope: Some(TargetScope::Scene),
            ..Default::default()
        },
        requested_at: 700,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(900),
        result: Some(ActionResult {
            accepted: true,
            summary: "scene mutation committed".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: None,
    });
    shell
        .app
        .session
        .action_log
        .commit_records
        .push(ActionCommitRecord {
            action_id: ActionId(56),
            boundary: CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 72,
                bar_index: 18,
                phrase_index: 4,
                scene_id: Some(SceneId::from("scene-a")),
            },
            commit_sequence: 1,
            committed_at: 900,
        });
    shell.app.refresh_view();
    save_session_json(&autosave_path, &shell.app.session).expect("save blocked autosave session");
    let captures_dir = dir.path().join("captures");
    std::fs::create_dir_all(&captures_dir).expect("create capture artifacts dir");
    std::fs::write(captures_dir.join("cap-01.wav"), [0u8; 44])
        .expect("write ready capture artifact");

    shell.set_recovery_surface(
        JamAppState::scan_session_recovery_surface(&target_path)
            .expect("scan recovery surface"),
    );
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 38);

    assert!(rendered.contains("decision"), "{rendered}");
    assert!(rendered.contains("replay-blocked"), "{rendered}");
    assert!(rendered.contains("families Scene | suffix 1"), "{rendered}");
    assert!(
        rendered.contains("artifacts ready: 1"),
        "{rendered}"
    );
    assert!(rendered.contains("payload ready"), "{rendered}");
    assert!(
        !rendered.contains("Artifact note: audio present"),
        "{rendered}"
    );
    assert!(rendered.contains("Selected candidate: none"), "{rendered}");
}
