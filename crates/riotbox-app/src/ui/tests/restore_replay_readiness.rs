#[test]
fn renders_log_shell_with_restore_replay_readiness_cues() {
    let mut shell = sample_shell_state();
    shell.app.session.action_log.actions.clear();
    shell.app.session.action_log.commit_records.clear();
    shell.app.session.snapshots = vec![Snapshot {
        snapshot_id: SnapshotId::from("before-fill"),
        created_at: "2026-04-29T22:56:00Z".into(),
        label: "before fill".into(),
        action_cursor: 0,
        payload: None,
    }];
    shell.app.session.action_log.actions.push(Action {
        id: ActionId(41),
        actor: ActorType::User,
        command: ActionCommand::Tr909FillNext,
        params: ActionParams::Empty,
        target: ActionTarget {
            scope: Some(TargetScope::LaneTr909),
            ..Default::default()
        },
        requested_at: 500,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(900),
        result: Some(ActionResult {
            accepted: true,
            summary: "fill committed".into(),
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
            action_id: ActionId(41),
            boundary: CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 64,
                bar_index: 16,
                phrase_index: 4,
                scene_id: Some(SceneId::from("scene-a")),
            },
            commit_sequence: 1,
            committed_at: 900,
        });
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Warnings / Restore"), "{rendered}");
    assert!(rendered.contains("replay 1 suffix"), "{rendered}");
    assert!(rendered.contains("anchor before-fill@0"), "{rendered}");
    assert!(
        rendered.contains("suffix tr909.fill_next"),
        "{rendered}"
    );
}
