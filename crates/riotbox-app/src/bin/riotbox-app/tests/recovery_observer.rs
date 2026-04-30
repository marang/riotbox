#[test]
fn observer_snapshot_records_recovery_startup_probe_without_selecting_candidate() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let autosave_path = temp
        .path()
        .join("session.autosave.artifact-ready-blocked.json");
    let captures_dir = temp.path().join("captures");
    fs::create_dir_all(&captures_dir).expect("create captures dir");
    fs::write(captures_dir.join("cap-01.wav"), [0_u8; 44]).expect("write capture artifact");
    save_session_json(
        &session_path,
        &SessionFile::new("canonical", "0.1.0", "2026-04-30T09:58:00Z"),
    )
    .expect("save canonical session");
    save_session_json(&autosave_path, &artifact_ready_blocked_autosave_session())
        .expect("save autosave session");

    let mode = LaunchMode::Load {
        session_path: session_path.clone(),
        source_graph_path: None,
    };
    let shell = shell_for_loaded_state(
        JamAppState::from_parts(
            SessionFile::new("loaded", "0.1.0", "2026-04-30T09:58:00Z"),
            None,
            ActionQueue::new(),
        ),
        &mode,
    );

    let snapshot = observer_snapshot(&shell);
    let recovery = &snapshot["recovery"];
    assert_eq!(recovery["present"], true);
    assert_eq!(recovery["has_manual_candidates"], true);
    assert_eq!(recovery["selected_candidate"], serde_json::Value::Null);
    assert_eq!(recovery["candidate_count"], 2);
    let dry_run = &recovery["manual_choice_dry_run"];
    assert!(
        dry_run["candidate_path"]
            .as_str()
            .expect("dry-run candidate path")
            .ends_with("session.autosave.artifact-ready-blocked.json")
    );
    assert_eq!(dry_run["selected_for_restore"], false);
    assert_eq!(
        dry_run["safety_note"],
        "Dry-run only: candidate inspected, not selected for restore."
    );
    assert_eq!(dry_run["decision"], "decision: blocked | replay unsupported");
    assert_eq!(dry_run["artifact_availability"], "artifacts ready: 1 capture(s)");
    assert_eq!(
        dry_run["payload_readiness"],
        "payload ready | snapshot restore ok"
    );
    assert_eq!(dry_run["replay_family"], "families Scene | suffix 1");

    let candidates = recovery["candidates"].as_array().expect("candidate array");
    assert_eq!(candidates[0]["kind"], "normal session path");
    assert_eq!(candidates[0]["trust"], "NormalLoadTarget");
    assert_eq!(candidates[1]["kind"], "autosave file");
    let autosave = candidates
        .iter()
        .find(|candidate| {
            candidate["path"].as_str().is_some_and(|path| {
                path.ends_with("session.autosave.artifact-ready-blocked.json")
            })
        })
        .expect("autosave recovery candidate");
    assert_eq!(autosave["trust"], "RecoverableClue");
    assert_eq!(autosave["artifact_availability"], "artifacts ready: 1 capture(s)");
    assert_eq!(autosave["payload_readiness"], "payload ready | snapshot restore ok");
    assert_eq!(
        autosave["replay_unsupported"],
        "unsupported suffix 1: mutate.scene"
    );
    assert_eq!(autosave["replay_family"], "families Scene | suffix 1");
    assert_eq!(autosave["guidance"], serde_json::Value::Null);
    assert!(session_path.exists());
    assert!(autosave_path.exists());
}

#[test]
fn observer_snapshot_reports_app_invalid_recovery_candidate_as_broken() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let autosave_path = temp.path().join("session.autosave.invalid-payload.json");
    save_session_json(
        &session_path,
        &SessionFile::new("canonical", "0.1.0", "2026-04-30T12:45:00Z"),
    )
    .expect("save canonical session");
    save_session_json(&autosave_path, &app_invalid_payload_autosave_session())
        .expect("save invalid autosave session");
    let autosave_before = fs::read_to_string(&autosave_path).expect("read autosave before");

    let mode = LaunchMode::Load {
        session_path: session_path.clone(),
        source_graph_path: None,
    };
    let shell = shell_for_loaded_state(
        JamAppState::from_parts(
            SessionFile::new("loaded", "0.1.0", "2026-04-30T12:45:00Z"),
            None,
            ActionQueue::new(),
        ),
        &mode,
    );

    let snapshot = observer_snapshot(&shell);
    let recovery = &snapshot["recovery"];
    assert_eq!(recovery["present"], true);
    assert_eq!(recovery["has_manual_candidates"], false);
    assert_eq!(recovery["selected_candidate"], serde_json::Value::Null);
    assert_eq!(recovery["candidate_count"], 2);
    assert_eq!(
        recovery["manual_choice_dry_run"],
        serde_json::Value::Null
    );

    let candidates = recovery["candidates"].as_array().expect("candidate array");
    let autosave = candidates
        .iter()
        .find(|candidate| {
            candidate["path"]
                .as_str()
                .is_some_and(|path| path.ends_with("session.autosave.invalid-payload.json"))
        })
        .expect("invalid autosave recovery candidate");
    assert_eq!(autosave["kind"], "autosave file");
    assert_eq!(autosave["status"], "app-invalid session");
    assert_eq!(autosave["trust"], "BrokenClue");
    assert_eq!(autosave["artifact_availability"], "artifacts unchecked");
    assert_eq!(
        autosave["payload_readiness"],
        "payload invalid | snapshot restore blocked"
    );
    assert_eq!(autosave["replay_family"], "families unchecked");
    assert_eq!(autosave["decision"], "decision: broken candidate");
    assert_eq!(autosave["guidance"], serde_json::Value::Null);
    assert_eq!(autosave["action_hint"], "do not recover automatically");
    assert_eq!(
        fs::read_to_string(&autosave_path).expect("read autosave after"),
        autosave_before
    );
    assert!(session_path.exists());
    assert!(autosave_path.exists());
}

fn artifact_ready_blocked_autosave_session() -> SessionFile {
    let mut session = SessionFile::new("autosave", "0.1.0", "2026-04-30T09:58:01Z");
    session.snapshots.push(Snapshot {
        snapshot_id: "snap-1".into(),
        created_at: "2026-04-30T09:58:02Z".into(),
        label: "before unsupported freeze".into(),
        action_cursor: 0,
        payload: Some(SnapshotPayload::from_runtime_state(
            &"snap-1".into(),
            0,
            &session.runtime_state,
        )),
    });
    session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-01"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["source-1".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: Some(ActionId(1)),
        storage_path: "captures/cap-01.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: false,
        notes: None,
    });
    session.action_log.actions.push(riotbox_core::action::Action {
        id: ActionId(88),
        actor: riotbox_core::action::ActorType::User,
        command: ActionCommand::MutateScene,
        params: riotbox_core::action::ActionParams::Mutation {
            intensity: 0.5,
            target_id: Some("scene-a".into()),
        },
        target: ActionTarget {
            scope: Some(TargetScope::Scene),
            ..Default::default()
        },
        requested_at: 480,
        quantization: Quantization::NextBar,
        status: riotbox_core::action::ActionStatus::Committed,
        committed_at: Some(500),
        result: None,
        undo_policy: riotbox_core::action::UndoPolicy::Undoable,
        explanation: Some("unsupported scene mutation action".into()),
    });
    session.action_log.commit_records.push(ActionCommitRecord {
        action_id: ActionId(88),
        boundary: riotbox_core::transport::CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 40,
            bar_index: 10,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        commit_sequence: 1,
        committed_at: 500,
    });
    session
}

fn app_invalid_payload_autosave_session() -> SessionFile {
    let mut session = SessionFile::new("autosave", "0.1.0", "2026-04-30T12:45:01Z");
    session.snapshots.push(Snapshot {
        snapshot_id: "snap-1".into(),
        created_at: "2026-04-30T12:45:02Z".into(),
        label: "invalid payload identity".into(),
        action_cursor: 0,
        payload: Some(SnapshotPayload {
            payload_version: riotbox_core::session::SnapshotPayloadVersion::V1,
            snapshot_id: "snap-other".into(),
            action_cursor: 0,
            runtime_state: session.runtime_state.clone(),
        }),
    });
    session
}
