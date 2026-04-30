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

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

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
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Session recovery"), "{rendered}");
    assert!(
        rendered.contains("Manual recovery only: Riotbox did not choose, load, replace, or delete"),
        "{rendered}"
    );
    assert!(
        rendered.contains("No candidate is selected here; reload an explicit reviewed path"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Restore replay: no replay entries"),
        "{rendered}"
    );
    assert!(
        rendered.contains("autosave file | parseable session JSON | artifacts n/a | no captures"),
        "{rendered}"
    );
    assert!(
        rendered.contains("artifacts n/a | no captures"),
        "{rendered}"
    );
    assert!(
        rendered.contains("payload none | full replay"),
        "{rendered}"
    );
    assert!(rendered.contains("suffix none@0"), "{rendered}");
    assert!(
        rendered.contains("review before manual"),
        "{rendered}"
    );
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
        command: ActionCommand::W30CaptureToPad,
        params: ActionParams::Capture { bars: Some(2) },
        target: ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-a")),
            pad_id: Some(PadId::from("pad-01")),
            ..Default::default()
        },
        requested_at: 700,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(900),
        result: Some(ActionResult {
            accepted: true,
            summary: "capture to W-30 pad committed".into(),
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

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("Restore replay: blocked: 1 unsupported suffix"),
        "{rendered}"
    );
    assert!(rendered.contains("w30.capture_to_pad"), "{rendered}");
    assert!(
        rendered.contains("autosave file | parseable session JSON | artifacts blocked: 1 of 1"),
        "{rendered}"
    );
    assert!(
        rendered.contains("payload ready"),
        "{rendered}"
    );
    assert!(
        rendered.contains("unsupported suffix"),
        "{rendered}"
    );
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
        command: ActionCommand::W30CaptureToPad,
        params: ActionParams::Capture { bars: Some(2) },
        target: ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-a")),
            pad_id: Some(PadId::from("pad-01")),
            ..Default::default()
        },
        requested_at: 700,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(900),
        result: Some(ActionResult {
            accepted: true,
            summary: "capture to W-30 pad committed".into(),
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

    let rendered = render_jam_shell_snapshot(&shell, 120, 36);

    assert!(
        rendered.contains("autosave file | parseable session JSON | artifacts ready: 1 capture"),
        "{rendered}"
    );
    assert!(
        rendered.contains("unsupported suffix w30.capture_to_pad"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Artifact note: audio present"),
        "{rendered}"
    );
    assert!(
        rendered.contains("not built yet"),
        "{rendered}"
    );
    assert!(rendered.contains("No candidate is selected here"), "{rendered}");
}
