#[test]
fn renders_help_overlay_with_restore_ready_cue() {
    let graph = scene_regression_graph(&["drop".into(), "break".into()]);
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-break"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Scene restore"), "{rendered}");
    assert!(
        rendered.contains("Y is live now for drop/high (rise)"),
        "{rendered}"
    );
    assert!(
        rendered.contains("press Y to bring drop/high back on the next bar"),
        "{rendered}"
    );
}

fn mc202_committed_shell_state(fixture: &Mc202RegressionFixture) -> JamShellState {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.action_log.actions.clear();
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;
    session.runtime_state.lane_state.mc202.role = Some(fixture.initial_role.clone());
    session.runtime_state.lane_state.mc202.phrase_ref = None;
    session.runtime_state.macro_state.mc202_touch = 0.4;

    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );

    let queue_result = match fixture.action {
        Mc202RegressionAction::SetRole => shell.app.queue_mc202_role_toggle(fixture.requested_at),
        Mc202RegressionAction::GenerateFollower => shell
            .app
            .queue_mc202_generate_follower(fixture.requested_at),
        Mc202RegressionAction::GenerateAnswer => {
            shell.app.queue_mc202_generate_answer(fixture.requested_at)
        }
        Mc202RegressionAction::GeneratePressure => shell
            .app
            .queue_mc202_generate_pressure(fixture.requested_at),
        Mc202RegressionAction::GenerateInstigator => shell
            .app
            .queue_mc202_generate_instigator(fixture.requested_at),
    };
    assert_eq!(
        queue_result,
        crate::jam_app::QueueControlResult::Enqueued,
        "{} did not enqueue",
        fixture.name
    );

    let committed = shell.app.commit_ready_actions(
        fixture.boundary.to_commit_boundary_state(),
        fixture.committed_at,
    );
    assert_eq!(
        committed.len(),
        1,
        "{} did not commit exactly one action",
        fixture.name
    );
    assert_eq!(
        shell
            .app
            .session
            .runtime_state
            .lane_state
            .mc202
            .role
            .as_deref(),
        Some(fixture.expected.role.as_str()),
        "{} role drifted",
        fixture.name
    );
    assert_eq!(
        shell
            .app
            .session
            .runtime_state
            .lane_state
            .mc202
            .phrase_ref
            .as_deref(),
        Some(fixture.expected.phrase_ref.as_str()),
        "{} phrase ref drifted",
        fixture.name
    );
    assert_eq!(
        shell.app.session.runtime_state.macro_state.mc202_touch, fixture.expected.touch,
        "{} touch drifted",
        fixture.name
    );
    assert_eq!(
        shell
            .app
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some(fixture.expected.result_summary.as_str()),
        "{} result summary drifted",
        fixture.name
    );

    shell
}

fn scene_committed_shell_state(fixture: &SceneRegressionFixture) -> JamShellState {
    let sample_shell = sample_shell_state();
    let graph = scene_regression_graph(&fixture.section_labels);
    let mut session = sample_shell.app.session.clone();
    session.runtime_state.transport.current_scene = None;
    session.runtime_state.scene_state.active_scene = None;
    session.runtime_state.scene_state.scenes.clear();

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    seed_scene_fixture_state(&mut shell, fixture);

    match fixture.action {
        SceneRegressionAction::ProjectCandidates => {}
        SceneRegressionAction::SelectNextScene => {
            assert_eq!(
                shell
                    .app
                    .queue_scene_select(fixture.requested_at.expect("scene select requested_at")),
                crate::jam_app::QueueControlResult::Enqueued,
                "{} did not enqueue",
                fixture.name
            );

            let committed = shell.app.commit_ready_actions(
                fixture
                    .boundary
                    .as_ref()
                    .expect("scene select boundary")
                    .to_commit_boundary_state(),
                fixture.committed_at.expect("scene select committed_at"),
            );
            assert_eq!(
                committed.len(),
                1,
                "{} did not commit exactly one action",
                fixture.name
            );
        }
        SceneRegressionAction::RestoreScene => {
            assert_eq!(
                shell
                    .app
                    .queue_scene_restore(fixture.requested_at.expect("scene restore requested_at")),
                crate::jam_app::QueueControlResult::Enqueued,
                "{} did not enqueue",
                fixture.name
            );

            let committed = shell.app.commit_ready_actions(
                fixture
                    .boundary
                    .as_ref()
                    .expect("scene restore boundary")
                    .to_commit_boundary_state(),
                fixture.committed_at.expect("scene restore committed_at"),
            );
            assert_eq!(
                committed.len(),
                1,
                "{} did not commit exactly one action",
                fixture.name
            );
        }
    }

    assert_eq!(
        shell.app.jam_view.scene.active_scene.as_deref(),
        Some(fixture.expected.active_scene.as_str()),
        "{} active scene drifted",
        fixture.name
    );
    if let Some(expected_summary) = &fixture.expected.result_summary {
        assert_eq!(
            shell
                .app
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some(expected_summary.as_str()),
            "{} result summary drifted",
            fixture.name
        );
    }

    shell
}

#[test]
fn mc202_fixture_backed_shell_regressions_hold() {
    let fixtures: Vec<Mc202RegressionFixture> =
        serde_json::from_str(include_str!("../../../tests/fixtures/mc202_regression.json"))
            .expect("parse MC-202 regression fixtures");

    for fixture in fixtures {
        let mut shell = mc202_committed_shell_state(&fixture);
        shell.active_screen = ShellScreen::Jam;
        let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.jam_contains {
            assert!(
                jam_rendered.contains(needle),
                "{} jam snapshot missing {needle}\n{jam_rendered}",
                fixture.name,
                jam_rendered = jam_rendered
            );
        }

        shell.active_screen = ShellScreen::Log;
        let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.log_contains {
            assert!(
                log_rendered.contains(needle),
                "{} log snapshot missing {needle}",
                fixture.name
            );
        }
    }
}

#[test]
fn scene_fixture_backed_shell_regressions_hold() {
    let fixtures: Vec<SceneRegressionFixture> =
        serde_json::from_str(include_str!("../../../tests/fixtures/scene_regression.json"))
            .expect("parse Scene Brain regression fixtures");

    for fixture in fixtures {
        let mut shell = scene_committed_shell_state(&fixture);
        shell.active_screen = ShellScreen::Jam;
        let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.jam_contains {
            assert!(
                jam_rendered.contains(needle),
                "{} jam snapshot missing {needle}\n{jam_rendered}",
                fixture.name,
                jam_rendered = jam_rendered
            );
        }

        shell.active_screen = ShellScreen::Log;
        let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.log_contains {
            assert!(
                log_rendered.contains(needle),
                "{} log snapshot missing {needle}\n{log_rendered}",
                fixture.name,
                log_rendered = log_rendered
            );
        }
    }
}

fn w30_committed_shell_state(fixture: &W30RegressionFixture) -> JamShellState {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.action_log.actions.clear();
    session.runtime_state.macro_state.w30_grit = fixture.initial_w30_grit.unwrap_or(0.0);
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from(
        fixture
            .initial_active_bank
            .clone()
            .unwrap_or_else(|| fixture.capture_bank.clone()),
    ));
    session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from(
        fixture
            .initial_focused_pad
            .clone()
            .unwrap_or_else(|| fixture.capture_pad.clone()),
    ));
    session.runtime_state.lane_state.w30.last_capture =
        fixture.initial_last_capture.clone().map(CaptureId::from);
    session.runtime_state.lane_state.w30.preview_mode = fixture
        .initial_preview_mode
        .as_deref()
        .map(w30_preview_mode_state);
    session.captures[0].assigned_target =
        fixture
            .capture_assigned
            .then(|| riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: fixture.capture_bank.clone().into(),
                pad_id: fixture.capture_pad.clone().into(),
            });
    session.captures[0].is_pinned = fixture.capture_pinned;
    session.captures[0].source_window = fixture.source_window.as_ref().map(|source_window| {
        riotbox_core::session::CaptureSourceWindow {
            source_id: SourceId::from(source_window.source_id.clone()),
            start_seconds: source_window.start_seconds,
            end_seconds: source_window.end_seconds,
            start_frame: source_window.start_frame,
            end_frame: source_window.end_frame,
        }
    });
    for extra in &fixture.extra_captures {
        session.captures.push(riotbox_core::session::CaptureRef {
            capture_id: extra.capture_id.clone().into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["fixture-extra".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: format!("captures/{}.wav", extra.capture_id),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: extra.bank.clone().into(),
                pad_id: extra.pad.clone().into(),
            }),
            is_pinned: extra.pinned,
            notes: extra.notes.clone(),
        });
    }

    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );

    let queue_result = match fixture.action {
        W30RegressionAction::LiveRecall => shell.app.queue_w30_live_recall(fixture.requested_at),
        W30RegressionAction::RawCaptureAudition => {
            shell.app.queue_w30_audition(fixture.requested_at)
        }
        W30RegressionAction::PromotedAudition => {
            shell.app.queue_w30_promoted_audition(fixture.requested_at)
        }
        W30RegressionAction::TriggerPad => shell.app.queue_w30_trigger_pad(fixture.requested_at),
        W30RegressionAction::SwapBank => shell.app.queue_w30_swap_bank(fixture.requested_at),
        W30RegressionAction::ApplyDamageProfile => shell
            .app
            .queue_w30_apply_damage_profile(fixture.requested_at),
        W30RegressionAction::LoopFreeze => shell.app.queue_w30_loop_freeze(fixture.requested_at),
        W30RegressionAction::BrowseSlicePool => {
            shell.app.queue_w30_browse_slice_pool(fixture.requested_at)
        }
    };
    assert_eq!(
        queue_result,
        Some(crate::jam_app::QueueControlResult::Enqueued),
        "{} did not enqueue",
        fixture.name
    );

    let committed = shell.app.commit_ready_actions(
        fixture.boundary.to_commit_boundary_state(),
        fixture.committed_at,
    );
    assert_eq!(
        committed.len(),
        1,
        "{} did not commit exactly one action",
        fixture.name
    );

    shell
}

#[test]
fn w30_fixture_backed_shell_regressions_hold() {
    let fixtures: Vec<W30RegressionFixture> =
        serde_json::from_str(include_str!("../../../tests/fixtures/w30_regression.json"))
            .expect("parse W-30 regression fixtures");

    for fixture in fixtures {
        let mut shell = w30_committed_shell_state(&fixture);
        shell.active_screen = ShellScreen::Jam;
        let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.jam_contains {
            assert!(
                jam_rendered.contains(needle),
                "{} jam snapshot missing {needle}\n{jam_rendered}",
                fixture.name,
                jam_rendered = jam_rendered
            );
        }

        shell.active_screen = ShellScreen::Capture;
        let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.capture_contains {
            assert!(
                capture_rendered.contains(needle),
                "{} capture snapshot missing {needle}\n{capture_rendered}",
                fixture.name,
                capture_rendered = capture_rendered
            );
        }

        shell.active_screen = ShellScreen::Log;
        let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.log_contains {
            assert!(
                log_rendered.contains(needle),
                "{} log snapshot missing {needle}\n{log_rendered}",
                fixture.name,
                log_rendered = log_rendered
            );
        }
    }
}

