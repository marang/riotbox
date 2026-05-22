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
