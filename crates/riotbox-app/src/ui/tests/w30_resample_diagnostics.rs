#[test]
fn renders_capture_shell_snapshot_with_committed_w30_resample_lineage_diagnostics() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.captures[0].lineage_capture_refs = vec!["cap-root".into()];
    shell.app.session.captures[0].resample_generation_depth = 1;
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_internal_resample(220),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Phrase,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        240,
    );
    assert_eq!(committed.len(), 1);
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("forge idle | tap"), "{rendered}");
    assert!(rendered.contains("ready/raw/unavailable"), "{rendered}");
    assert!(rendered.contains("g2"), "{rendered}");
    assert!(rendered.contains("lineage"));
    assert!(rendered.contains("tap src cap-02 g2/l2 |"), "{rendered}");
    assert!(rendered.contains("route silent"), "{rendered}");
    assert!(rendered.contains("tap src cap-02"), "{rendered}");
    assert!(
        rendered.matches("latest promoted").count() <= 1,
        "{rendered}"
    );
}

#[test]
fn renders_log_shell_snapshot_with_committed_w30_audition_diagnostics() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_promoted_audition(220),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        240,
    );
    assert_eq!(committed.len(), 1);
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("W-30 Lane"));
    assert!(rendered.contains("cue idle"));
    assert!(rendered.contains("auditioned cap-01"));
    assert!(rendered.contains("bank-b"));
    assert!(rendered.contains("pad-03"));
    assert!(rendered.contains("cue idle | audition"));
    assert!(rendered.contains("prev"));
    assert!(rendered.contains("audition/unavailable"));
    assert!(rendered.contains("mix 0.00/0.68"));
    assert!(rendered.contains("result auditioned cap-01"), "{rendered}");
}

#[test]
fn renders_log_shell_snapshot_with_committed_w30_trigger_preview_diagnostics() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_trigger_pad(230),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Beat,
            beat_index: 34,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        250,
    );
    assert_eq!(committed.len(), 1);
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("W-30 Lane"));
    assert!(rendered.contains("cue idle | trigger"));
    assert!(rendered.contains("prev"));
    assert!(rendered.contains("recall/unavailable"));
    assert!(rendered.contains("mix 0.00/0.69"));
    assert!(rendered.contains("result triggered cap-01"), "{rendered}");
}

#[test]
fn renders_log_shell_snapshot_with_committed_w30_resample_lineage_diagnostics() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.captures[0].lineage_capture_refs = vec!["cap-root".into()];
    shell.app.session.captures[0].resample_generation_depth = 1;
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_internal_resample(245),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Phrase,
            beat_index: 34,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        260,
    );
    assert_eq!(committed.len(), 1);
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("W-30 Lane"));
    assert!(rendered.contains("cue idle | resample"));
    assert!(rendered.contains("tapmix 0.64/0.50"), "{rendered}");
    assert!(rendered.contains("tapmix 0.64/0.50"), "{rendered}");
}

#[test]
fn renders_w30_resample_lab_diagnostics_across_shell_surfaces() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.captures[0].lineage_capture_refs = vec!["cap-root".into()];
    shell.app.session.captures[0].resample_generation_depth = 1;
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();

    assert_eq!(
        shell.app.queue_w30_internal_resample(265),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Phrase,
            beat_index: 36,
            bar_index: 10,
            phrase_index: 3,
            scene_id: Some(SceneId::from("scene-a")),
        },
        280,
    );
    assert_eq!(committed.len(), 1);

    let observer = crate::observer::observer_snapshot(&shell);
    assert_eq!(
        observer
            .pointer("/runtime/w30_resample_tap_availability")
            .and_then(serde_json::Value::as_str),
        Some("source_audio_unavailable")
    );
    assert_eq!(
        observer
            .pointer("/runtime/w30_resample_tap_variation")
            .and_then(serde_json::Value::as_str),
        Some("base")
    );

    let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        jam_rendered.contains("current pad bank-b/pad-03"),
        "{jam_rendered}"
    );
    assert!(jam_rendered.contains("next idle"), "{jam_rendered}");

    shell.active_screen = ShellScreen::Capture;
    let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        capture_rendered.contains("tap src cap-02 g2/l2 |"),
        "{capture_rendered}"
    );
    assert!(
        capture_rendered.contains("route silent"),
        "{capture_rendered}"
    );
    assert!(
        capture_rendered.contains("tap mix 0.64/0.50"),
        "{capture_rendered}"
    );

    shell.active_screen = ShellScreen::Log;
    let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(log_rendered.contains("tapmix 0.64/0.50"), "{log_rendered}");

    assert_eq!(
        shell.app.queue_w30_apply_damage_profile(290),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let damage = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 40,
            bar_index: 11,
            phrase_index: 3,
            scene_id: Some(SceneId::from("scene-a")),
        },
        300,
    );
    assert_eq!(damage.len(), 1);
    let hard_observer = crate::observer::observer_snapshot(&shell);
    assert_eq!(
        hard_observer
            .pointer("/runtime/w30_resample_tap_variation")
            .and_then(serde_json::Value::as_str),
        Some("hard_damage")
    );
    assert!(
        hard_observer
            .pointer("/runtime/w30_resample_tap_variation_intensity")
            .and_then(serde_json::Value::as_f64)
            .is_some_and(|intensity| (intensity - 0.82).abs() < 0.000_001)
    );
    assert!(
        hard_observer
            .pointer("/runtime/w30_resample_tap_variation_revision")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|revision| revision > 0)
    );
}
