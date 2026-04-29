#[test]
fn renders_log_w30_source_window_when_available() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].source_window =
        Some(riotbox_core::session::CaptureSourceWindow {
            source_id: SourceId::from("src-1"),
            start_seconds: 1.25,
            end_seconds: 3.75,
            start_frame: 60_000,
            end_frame: 180_000,
        });
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("win 1.25-3.75s src-1"), "{rendered}");
    assert_eq!(w30_capture_log_compact(&shell), "win 1.25-3.75s src-1");
}

#[test]
fn renders_capture_do_next_with_pending_capture_state() {
    let first_run_shell = first_run_shell_state();
    let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
    shell.app.queue_capture_bar(240);
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("queued [c] capture @ next_phrase"),
        "{rendered}"
    );
    assert!(
        rendered.contains("then [o] audition raw or [p] promote"),
        "{rendered}"
    );
    assert!(rendered.contains("[2] confirm capture"), "{rendered}");
}

#[test]
fn renders_capture_shell_snapshot_with_raw_capture_audition_cue() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_audition(260),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 160, 34);

    assert!(rendered.contains("pending W-30 cue audition"), "{rendered}");
    assert!(rendered.contains("bank-a/pad-01"), "{rendered}");
    assert!(rendered.contains("queued [o] audition raw @"), "{rendered}");
    assert!(
        rendered.contains("wait, then hear raw preview"),
        "{rendered}"
    );
    assert!(
        rendered.contains("hear cap-01 fallback: [o] raw -> [p]->[w]"),
        "{rendered}"
    );

    shell.active_screen = ShellScreen::Log;
    let rendered_log = render_jam_shell_snapshot(&shell, 160, 34);
    assert!(
        rendered_log.contains("w30.audition_raw_capture"),
        "{rendered_log}"
    );
}

#[test]
fn committed_raw_capture_audition_surfaces_source_fallback_readiness() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_audition(260),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        320,
    );
    shell.active_screen = ShellScreen::Jam;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("current preview audition"), "{rendered}");
    assert!(rendered.contains("raw/fallback"), "{rendered}");
    assert!(
        rendered.contains("fallback: [o] raw safe | 4 Capture"),
        "{rendered}"
    );

    shell.active_screen = ShellScreen::Capture;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(rendered.contains("| fallback"), "{rendered}");
}

#[test]
fn source_backed_raw_capture_audition_compact_label_uses_src_cue() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.runtime_state.lane_state.w30.preview_mode =
        Some(riotbox_core::session::W30PreviewModeState::RawCaptureAudition);
    shell.app.refresh_view();
    shell.app.runtime.w30_preview.source_window_preview =
        Some(riotbox_audio::w30::W30PreviewSampleWindow {
            source_start_frame: 0,
            source_end_frame: 64,
            sample_count: 64,
            samples: [0.0; riotbox_audio::w30::W30_PREVIEW_SAMPLE_WINDOW_LEN],
        });

    assert_eq!(w30_preview_mode_profile_compact(&shell), "audition raw/src");
    assert_eq!(w30_preview_source_readiness(&shell), Some("source-backed"));

    shell.active_screen = ShellScreen::Jam;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("current preview audition raw/src"), "{rendered}");
    assert!(
        rendered.contains("src: [o] raw source | 4 Capture"),
        "{rendered}"
    );
    assert!(
        !rendered.contains("fallback: ["),
        "source-backed W-30 preview should not show fallback action cue\n{rendered}"
    );
}

#[test]
fn source_backed_promoted_and_recall_compact_labels_use_src_cue() {
    let mut shell = sample_shell_without_pending_queue();
    let sample_window = riotbox_audio::w30::W30PreviewSampleWindow {
        source_start_frame: 0,
        source_end_frame: 64,
        sample_count: 64,
        samples: [0.0; riotbox_audio::w30::W30_PREVIEW_SAMPLE_WINDOW_LEN],
    };

    shell.app.runtime.w30_preview.mode = W30PreviewRenderMode::PromotedAudition;
    shell.app.runtime.w30_preview.source_profile =
        Some(riotbox_audio::w30::W30PreviewSourceProfile::PromotedAudition);
    shell.app.runtime.w30_preview.source_window_preview = Some(sample_window.clone());

    assert_eq!(w30_preview_mode_profile_compact(&shell), "audition/src");
    assert_eq!(w30_preview_log_compact(&shell), "audition/src");
    assert_eq!(w30_preview_source_readiness(&shell), Some("source-backed"));

    shell.active_screen = ShellScreen::Jam;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(rendered.contains("src: [o] source | 4 Capture"), "{rendered}");
    assert!(
        !rendered.contains("fallback: ["),
        "source-backed promoted audition should not show fallback action cue\n{rendered}"
    );

    shell.app.runtime.w30_preview.mode = W30PreviewRenderMode::LiveRecall;
    shell.app.runtime.w30_preview.source_profile =
        Some(riotbox_audio::w30::W30PreviewSourceProfile::PromotedRecall);
    shell.app.runtime.w30_preview.source_window_preview = Some(sample_window);

    assert_eq!(
        w30_preview_mode_profile_compact(&shell),
        "recall/promoted/src"
    );
    assert_eq!(w30_preview_log_compact(&shell), "recall/src");
    assert_eq!(w30_preview_source_readiness(&shell), Some("source-backed"));

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(rendered.contains("src: [w] source | 4 Capture"), "{rendered}");
    assert!(
        !rendered.contains("fallback: ["),
        "source-backed live recall should not show fallback action cue\n{rendered}"
    );
}

#[test]
fn renders_capture_pending_cues_panel_as_first_item_with_log_overflow() {
    let first_run_shell = first_run_shell_state();
    let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
    shell.app.queue_capture_bar(240);
    shell.app.queue_capture_bar(241);

    let lines = pending_capture_lines(&shell);
    let rendered: Vec<String> = lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect();

    assert_eq!(rendered[0], "next user capture.bar_group");
    assert_eq!(rendered[1], "when next_phrase | target lanew30");
    assert_eq!(rendered[2], "note capture next phrase into W-30 path");
    assert_eq!(rendered[3], "+1 more in [2] Log");
    assert_eq!(rendered.len(), 4);
}

#[test]
fn renders_capture_shell_snapshot_with_w30_live_recall_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.captures[0].is_pinned = true;
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_live_recall(200),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("recall"));
}

#[test]
fn renders_capture_shell_snapshot_with_w30_trigger_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_trigger_pad(205),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("trigger"));
    assert!(rendered.contains("bank-a/pad-01"));
}

#[test]
fn renders_capture_shell_snapshot_with_w30_step_cue() {
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
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-04".into(),
            }),
            is_pinned: false,
            notes: Some("secondary".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_step_focus(207),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("step"));
    assert!(rendered.contains("bank-b/pad-04"));
}

#[test]
fn renders_capture_shell_snapshot_with_w30_bank_swap_cue() {
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
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_swap_bank(208),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("bank"));
    assert!(rendered.contains("bank-b/pad-01"));
    assert!(rendered.contains("pending W-30 cue bank"), "{rendered}");
    assert!(rendered.contains("mgr next bank-b/pad-01"), "{rendered}");
}

#[test]
fn renders_capture_shell_snapshot_with_w30_slice_pool_browse_cue() {
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
        shell.app.queue_w30_browse_slice_pool(209),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("browse"));
    assert!(rendered.contains("bank-a/pad-01"), "{rendered}");
    assert!(rendered.contains("bank/pad bank-a/pad-01"), "{rendered}");
    assert!(rendered.contains("pool cap-01 1/2 -> cap-02"), "{rendered}");
}

#[test]
fn renders_capture_shell_snapshot_with_feral_w30_slice_pool_browse_cue() {
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
            source_origin_refs: vec!["asset-hook".into()],
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
            notes: Some("feral hook slice".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_browse_slice_pool(210),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    assert_eq!(
        shell
            .app
            .jam_view
            .lanes
            .w30_pending_slice_pool_reason
            .as_deref(),
        Some("feral")
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("pending W-30 cue feral") && rendered.contains("browse cap-02"),
        "{rendered}"
    );
    assert!(
        rendered.contains("pool cap-01 1/2 -> feral") && rendered.contains("cap-02"),
        "{rendered}"
    );
}
