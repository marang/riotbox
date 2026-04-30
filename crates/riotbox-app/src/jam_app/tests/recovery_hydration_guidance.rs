#[test]
fn recovery_surface_reports_supported_artifact_hydration_blocker_guidance() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let blocked_autosave_path = dir
        .path()
        .join("session.autosave.supported-hydration-blocked.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T12:20:00Z"),
    )
    .expect("save canonical session");

    let mut blocked_session =
        SessionFile::new("supported-blocked", "riotbox-test", "2026-04-30T12:20:01Z");
    blocked_session.snapshots.push(Snapshot {
        snapshot_id: SnapshotId::from("before-capture-loop"),
        created_at: "2026-04-30T12:20:02Z".into(),
        label: "before capture loop".into(),
        action_cursor: 0,
        payload: Some(riotbox_core::session::SnapshotPayload::from_runtime_state(
            &SnapshotId::from("before-capture-loop"),
            0,
            &blocked_session.runtime_state,
        )),
    });
    blocked_session.action_log.actions.push(Action {
        id: ActionId(89),
        actor: ActorType::User,
        command: ActionCommand::CaptureLoop,
        params: ActionParams::Capture { bars: Some(2) },
        target: ActionTarget {
            scope: Some(TargetScope::LaneW30),
            ..Default::default()
        },
        requested_at: 480,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(500),
        result: Some(ActionResult {
            accepted: true,
            summary: "capture loop committed".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("capture source-window loop".into()),
    });
    blocked_session
        .action_log
        .commit_records
        .push(loop_freeze_commit_record(89));
    blocked_session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Loop,
        source_origin_refs: vec!["src-1".into()],
        source_window: Some(CaptureSourceWindow {
            source_id: SourceId::from("src-1"),
            start_seconds: 0.0,
            end_seconds: 2.0,
            start_frame: 0,
            end_frame: 96_000,
        }),
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: Some(ActionId(89)),
        storage_path: " ".into(),
        assigned_target: None,
        is_pinned: false,
        notes: Some("missing storage identity".into()),
    });
    save_session_json(&blocked_autosave_path, &blocked_session)
        .expect("save supported hydration-blocked autosave session");
    let autosave_before =
        fs::read(&blocked_autosave_path).expect("read hydration-blocked autosave before scan");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");
    let blocked_candidate = surface
        .candidates
        .iter()
        .find(|candidate| matches!(candidate.trust, RecoveryCandidateTrust::RecoverableClue))
        .expect("supported hydration-blocked autosave candidate");

    assert_eq!(
        blocked_candidate.artifact_availability_label,
        "artifacts blocked: 1 of 1 | 1 missing identity"
    );
    assert_eq!(
        blocked_candidate.replay_readiness_label,
        "ready: replay 1 suffix action(s)"
    );
    assert_eq!(
        blocked_candidate.payload_readiness_label,
        "payload ready | snapshot restore ok"
    );
    assert_eq!(
        blocked_candidate.replay_suffix_label,
        "suffix 1 action(s): capture.loop"
    );
    assert_eq!(blocked_candidate.replay_unsupported_label, "unsupported none");
    assert_eq!(
        blocked_candidate.decision_label,
        "decision: blocked | replay hydration and artifacts"
    );
    assert_eq!(
        blocked_candidate.guidance,
        Some(
            RecoveryCandidateGuidance::SupportedArtifactReplayHydrationBlocked {
                detail: "capture.loop action a-0089 cannot hydrate persisted artifact: missing storage path for capture cap-02".into(),
            }
        )
    );
    assert_eq!(surface.selected_candidate, None);
    assert_eq!(
        fs::read(&blocked_autosave_path).expect("read hydration-blocked autosave after scan"),
        autosave_before
    );
}
