#[test]
fn renders_log_shell_snapshot_with_committed_w30_slice_pool_browse_diagnostics() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell
        .app
        .session
        .captures
        .push(riotbox_core::session::CaptureRef {
            capture_id: "cap-02".into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: vec!["cap-01".into()],
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            }),
            is_pinned: false,
            notes: Some("alt slice".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_browse_slice_pool(320),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.app.commit_ready_actions(
        riotbox_core::transport::CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Beat,
            beat_index: 42,
            bar_index: 11,
            phrase_index: 3,
            scene_id: Some("scene-1".into()),
        },
        420,
    );
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("cue idle | browse"), "{rendered}");
    assert!(rendered.contains("bank bank-a/pad-01"), "{rendered}");
    assert!(rendered.contains("tap cap-02 g0/l1 int"), "{rendered}");
}

#[test]
fn renders_capture_shell_snapshot_with_w30_damage_profile_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_apply_damage_profile(210),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("damage"));
    assert!(rendered.contains("bank-a/pad-01"));
    assert!(rendered.contains("next bank-a/pad-01"), "{rendered}");
}

#[test]
fn renders_w30_bank_manager_and_damage_profile_diagnostics_across_shell_surfaces() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell
        .app
        .session
        .captures
        .push(riotbox_core::session::CaptureRef {
            capture_id: "cap-02".into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-01".into(),
            }),
            is_pinned: false,
            notes: Some("bank b".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();

    assert_eq!(
        shell.app.queue_w30_swap_bank(208),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 17,
            bar_index: 5,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        220,
    );
    assert_eq!(committed.len(), 1);

    assert_eq!(
        shell.app.queue_w30_apply_damage_profile(222),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 21,
            bar_index: 6,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        240,
    );
    assert_eq!(committed.len(), 1);

    let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        jam_rendered.contains("current pad bank-b/pad-01"),
        "{jam_rendered}"
    );
    assert!(jam_rendered.contains("next swap+shred"), "{jam_rendered}");

    shell.active_screen = ShellScreen::Capture;
    let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        capture_rendered.contains("mgr bank-b/pad-01"),
        "{capture_rendered}"
    );
    assert!(
        capture_rendered.contains("forge bank-b/pad-01"),
        "{capture_rendered}"
    );

    shell.active_screen = ShellScreen::Log;
    let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        log_rendered.contains("bank bank-b/pad-01"),
        "{log_rendered}"
    );
    assert!(log_rendered.contains("cue idle | damage"), "{log_rendered}");
    assert!(log_rendered.contains("mix 0.64/0.82"), "{log_rendered}");
    assert!(log_rendered.contains("swap+shred"), "{log_rendered}");
}

#[test]
fn w30_operation_diagnostics_follow_current_lane_target() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell
        .app
        .session
        .captures
        .push(riotbox_core::session::CaptureRef {
            capture_id: "cap-02".into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-01".into(),
            }),
            is_pinned: false,
            notes: Some("bank b".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-02".into());
    shell.app.refresh_view();

    assert_eq!(
        shell.app.queue_w30_swap_bank(208),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 17,
            bar_index: 5,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        220,
    );
    assert_eq!(committed.len(), 1);

    assert_eq!(
        shell.app.queue_w30_apply_damage_profile(222),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 21,
            bar_index: 6,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        240,
    );
    assert_eq!(committed.len(), 1);

    assert_eq!(
        shell.app.queue_w30_loop_freeze(245),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Phrase,
            beat_index: 29,
            bar_index: 8,
            phrase_index: 3,
            scene_id: Some(SceneId::from("scene-a")),
        },
        260,
    );
    assert_eq!(committed.len(), 1);

    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-c".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();

    let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        jam_rendered.contains("current pad bank-c/pad-01"),
        "{jam_rendered}"
    );
    assert!(jam_rendered.contains("next idle"), "{jam_rendered}");

    shell.active_screen = ShellScreen::Capture;
    let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        capture_rendered.contains("bank/pad bank-c/pad-01"),
        "{capture_rendered}"
    );
    assert!(capture_rendered.contains("mgr idle"), "{capture_rendered}");
    assert!(
        capture_rendered.contains("forge idle"),
        "{capture_rendered}"
    );
    assert!(
        capture_rendered.contains("freeze idle"),
        "{capture_rendered}"
    );

    shell.active_screen = ShellScreen::Log;
    let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        log_rendered.contains("mix 0.64/0.82 idle"),
        "{log_rendered}"
    );
}

#[test]
fn renders_capture_shell_snapshot_with_w30_audition_cue() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_promoted_audition(210),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("audition"));
    assert!(rendered.contains("[w]/[o]"), "{rendered}");
    assert!(rendered.contains("queued [o] audition pad @"), "{rendered}");
    assert!(
        rendered.contains("wait, then hear promoted preview"),
        "{rendered}"
    );
    assert_eq!(
        shell
            .app
            .jam_view
            .capture
            .latest_w30_promoted_capture_label
            .as_deref(),
        Some("cap-01 -> bank-b/pad-03")
    );
    assert!(rendered.contains("latest promoted cap-01 ->"), "{rendered}");
    assert!(rendered.contains("cap-01"));
}

#[test]
fn renders_capture_heard_path_for_scene_targets_without_w30_audition_keys() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::Scene("drop-1".into()));
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Capture;

    assert_eq!(
        shell.app.jam_view.capture.last_capture_target_kind,
        Some(CaptureTargetKindView::Scene)
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("hear cap-01->scene drop-1 ready"),
        "{rendered}"
    );
    assert!(rendered.contains("scene target scene drop-1"), "{rendered}");
}

#[test]
fn renders_capture_handoff_source_readiness_for_w30_targets() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.captures[0].source_window =
        Some(riotbox_core::session::CaptureSourceWindow {
            source_id: SourceId::from("src-1"),
            start_seconds: 1.25,
            end_seconds: 3.75,
            start_frame: 60_000,
            end_frame: 180_000,
        });
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Capture;

    assert_eq!(
        shell.app.jam_view.capture.last_capture_handoff_readiness,
        Some(CaptureHandoffReadinessView::Source)
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("hear cap-01->pad bank-b/pad-03"),
        "{rendered}"
    );
    assert!(rendered.contains("[w]/[o] src"), "{rendered}");
    assert!(
        rendered.contains("hear now: [w] hit pad bank-b/pad-03"),
        "{rendered}"
    );
    assert!(rendered.contains("(src)"), "{rendered}");
}

#[test]
fn renders_capture_handoff_fallback_as_actionable_w30_cue() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Capture;

    assert_eq!(
        shell.app.jam_view.capture.last_capture_handoff_readiness,
        Some(CaptureHandoffReadinessView::Fallback)
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("fallback: [w]/[o] safe pad"),
        "{rendered}"
    );
    assert!(rendered.contains("bank-b/pad-03"), "{rendered}");
    assert!(rendered.contains("[3] Source shows why"), "{rendered}");
    assert!(
        rendered.contains("[c] new capture can become src"),
        "{rendered}"
    );
}

#[test]
fn renders_capture_shell_snapshot_with_w30_resample_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_internal_resample(215),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("+1 more in [2] Log"));
    assert!(rendered.contains("resample"));
    assert!(rendered.contains("cap-01"));
}
