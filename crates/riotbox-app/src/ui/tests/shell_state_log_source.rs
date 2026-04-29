#[test]
fn shell_state_handles_help_refresh_and_action_keys() {
    let mut shell = sample_shell_state();

    assert_eq!(
        shell.handle_key_code(KeyCode::Char('?')),
        ShellKeyOutcome::Continue
    );
    assert!(shell.show_help);
    assert_eq!(shell.status_message, "help overlay opened");

    assert_eq!(
        shell.handle_key_code(KeyCode::Char('r')),
        ShellKeyOutcome::RequestRefresh
    );
    assert_eq!(shell.status_message, "re-ingest source requested");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('2')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Log);
    assert_eq!(shell.status_message, "switched to log screen");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('3')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Source);
    assert_eq!(shell.status_message, "switched to source screen");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('4')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Capture);
    assert_eq!(shell.status_message, "switched to capture screen");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(
        shell.status_message,
        "open Jam first if you want to use inspect"
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Tab),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Jam);
    assert_eq!(shell.status_message, "switched to jam screen");
    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.jam_mode, JamViewMode::Inspect);
    assert_eq!(
        shell.status_message,
        "opened Jam inspect | press i to return to perform"
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(shell.status_message, "returned Jam to perform mode");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('m')),
        ShellKeyOutcome::QueueSceneMutation
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('y')),
        ShellKeyOutcome::QueueSceneSelect
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('Y')),
        ShellKeyOutcome::QueueSceneRestore
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('b')),
        ShellKeyOutcome::QueueMc202RoleToggle
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('g')),
        ShellKeyOutcome::QueueMc202GenerateFollower
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('a')),
        ShellKeyOutcome::QueueMc202GenerateAnswer
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('P')),
        ShellKeyOutcome::QueueMc202GeneratePressure
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('I')),
        ShellKeyOutcome::QueueMc202GenerateInstigator
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('G')),
        ShellKeyOutcome::QueueMc202MutatePhrase
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('f')),
        ShellKeyOutcome::QueueTr909Fill
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('d')),
        ShellKeyOutcome::QueueTr909Reinforce
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('s')),
        ShellKeyOutcome::QueueTr909Slam
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('t')),
        ShellKeyOutcome::QueueTr909Takeover
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('k')),
        ShellKeyOutcome::QueueTr909SceneLock
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('x')),
        ShellKeyOutcome::QueueTr909Release
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('c')),
        ShellKeyOutcome::QueueCaptureBar
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('p')),
        ShellKeyOutcome::PromoteLastCapture
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('w')),
        ShellKeyOutcome::QueueW30TriggerPad
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('n')),
        ShellKeyOutcome::QueueW30StepFocus
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('B')),
        ShellKeyOutcome::QueueW30SwapBank
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('j')),
        ShellKeyOutcome::QueueW30BrowseSlicePool
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('D')),
        ShellKeyOutcome::QueueW30ApplyDamageProfile
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('z')),
        ShellKeyOutcome::QueueW30LoopFreeze
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('l')),
        ShellKeyOutcome::QueueW30LiveRecall
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('o')),
        ShellKeyOutcome::QueueW30Audition
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('e')),
        ShellKeyOutcome::QueueW30Resample
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('v')),
        ShellKeyOutcome::TogglePinLatestCapture
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('[')),
        ShellKeyOutcome::LowerDrumBusLevel
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char(']')),
        ShellKeyOutcome::RaiseDrumBusLevel
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('<')),
        ShellKeyOutcome::LowerMc202Touch
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('>')),
        ShellKeyOutcome::RaiseMc202Touch
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('u')),
        ShellKeyOutcome::UndoLast
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char(' ')),
        ShellKeyOutcome::ToggleTransport
    );

    assert_eq!(shell.handle_key_code(KeyCode::Esc), ShellKeyOutcome::Quit);
}

#[test]
fn first_run_shell_blocks_jam_inspect_toggle() {
    let mut shell = first_run_shell_state();

    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(
        shell.status_message,
        "finish the first guided move before opening inspect"
    );
}

#[test]
fn renders_log_shell_snapshot_with_action_trust_history() {
    let mut shell = sample_shell_state();
    shell.active_screen = ShellScreen::Log;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("[2 Log]"));
    assert!(rendered.contains("Queued / Pending"));
    assert!(rendered.contains("Accepted / Committed"));
    assert!(rendered.contains("Rejected / Undone"));
    assert!(rendered.contains("MC-202 Lane"));
    assert!(rendered.contains("W-30 Lane"));
    assert!(rendered.contains("role leader"));
    assert!(rendered.contains("cue idle"));
    assert!(rendered.contains("cue idle | none"));
    assert!(rendered.contains("prev recall/fallback"));
    assert!(rendered.contains("mix 0.64/0.50 idle"));
    assert!(rendered.contains("cap cap-01 | pending"));
    assert!(rendered.contains("ghost"));
    assert!(rendered.contains("mutate.scene"));
    assert!(rendered.contains("TR-909 Render"));
    assert!(rendered.contains("accent off"));
    assert!(rendered.contains("takeover"));
    assert!(rendered.contains("scene lock blocked ghost"));
    assert!(rendered.contains("undid most recent musical"));
}

#[test]
fn renders_tr909_feral_support_reason_cue() {
    let mut shell = sample_shell_state();
    shell
        .app
        .session
        .runtime_state
        .lane_state
        .tr909
        .takeover_enabled = false;
    shell
        .app
        .session
        .runtime_state
        .lane_state
        .tr909
        .takeover_profile = None;
    shell
        .app
        .session
        .runtime_state
        .lane_state
        .tr909
        .reinforcement_mode = Some(Tr909ReinforcementModeState::SourceSupport);
    shell.app.session.runtime_state.lane_state.tr909.pattern_ref =
        Some("support-feral-break".into());
    shell.app.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: 4.0,
        beat_index: 4,
        bar_index: 1,
        phrase_index: 1,
        current_scene: Some(SceneId::from("scene-a")),
    });
    shell.active_screen = ShellScreen::Log;

    assert_eq!(
        shell.app.runtime_view.tr909_render_support_reason,
        "feral break lift"
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("feral break lift"), "{rendered}");
}

#[test]
fn renders_log_shell_snapshot_with_scene_brain_diagnostics() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_scene_select(300),
        crate::jam_app::QueueControlResult::Enqueued
    );
    shell.active_screen = ShellScreen::Log;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Counts"));
    assert!(rendered.contains("scene scene-a | medium"));
    assert!(rendered.contains("restore none"));
    assert!(rendered.contains("pending"));
}

#[test]
fn renders_source_shell_snapshot_with_feral_scorecard() {
    let mut shell = sample_shell_state();
    shell.active_screen = ShellScreen::Source;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("[3 Source]"));
    assert!(rendered.contains("Identity"));
    assert!(rendered.contains("Timing"));
    assert!(rendered.contains("Sections"));
    assert!(rendered.contains("Candidates"));
    assert!(rendered.contains("Provenance"));
    assert!(rendered.contains("Source Graph Warnings"));
    assert!(rendered.contains("feral ready"));
    assert!(rendered.contains("break high"));
    assert!(rendered.contains("quote risk 1"));
    assert!(rendered.contains("use capture before quoting"));
    assert!(rendered.contains("decoded.wav_baseline"));
    assert!(rendered.contains("fixtures/input.wav"));
    assert!(rendered.contains("wav_baseline_only"));
}

#[test]
fn renders_source_shell_snapshot_with_near_miss_feral_readiness() {
    let mut shell = sample_shell_state();
    let graph = shell
        .app
        .source_graph
        .as_mut()
        .expect("sample shell should include source graph");
    graph.relationships.retain(|relationship| {
        relationship.relation_type != RelationshipType::SupportsBreakRebuild
    });
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Source;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("feral needs support"), "{rendered}");
    assert!(rendered.contains("quote risk 1 | support 0"), "{rendered}");
    assert!(rendered.contains("hooks 1 | capture 1"), "{rendered}");
}

#[test]
fn renders_capture_shell_snapshot_with_capture_context() {
    let mut shell = sample_shell_state();
    shell.active_screen = ShellScreen::Capture;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("[4 Capture]"));
    assert!(rendered.contains("Readiness"));
    assert!(rendered.contains("Latest Capture"));
    assert!(rendered.contains("Do Next"));
    assert!(rendered.contains("Provenance"));
    assert!(rendered.contains("Pending Capture Cues"));
    assert!(rendered.contains("Recent Captures"));
    assert!(rendered.contains("Advanced Routing"));
    assert!(rendered.contains("cap-01"));
    assert!(rendered.contains("promote keeper capture"));
    assert!(rendered.contains("promotion result pending"));
    assert!(rendered.contains("captures total 1"));
    assert!(rendered.contains("pinned 0 | promoted 0"));
    assert!(
        rendered.contains("queued [p] promote @ next_bar"),
        "{rendered}"
    );
    assert!(
        rendered.contains("wait, then hear with [w] hit"),
        "{rendered}"
    );
    assert!(
        rendered.contains("target lanew30:bank-a/pad-01"),
        "{rendered}"
    );
    assert!(rendered.contains("pending W-30 cue idle"));
    assert!(
        rendered.contains("hear cap-01 fallback: [o] raw"),
        "{rendered}"
    );
    assert!(rendered.contains("[p]->[w]"), "{rendered}");
    assert!(
        rendered.contains("forge idle | tap ready/raw"),
        "{rendered}"
    );
    assert!(rendered.contains("g0"), "{rendered}");
    assert!(rendered.contains("latest promoted none"));
}

#[test]
fn renders_capture_provenance_source_window_when_available() {
    let mut shell = sample_shell_state();
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

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("win src-1 1.25-3.75s"), "{rendered}");
}

#[test]
fn renders_recent_capture_source_window_shorthand_when_available() {
    let mut shell = sample_shell_state();
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

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("cap-01 | 1.25-3.75s"), "{rendered}");
}

#[test]
fn source_window_formatters_keep_surface_shapes_stable() {
    let source_window = riotbox_core::session::CaptureSourceWindow {
        source_id: SourceId::from("src-1"),
        start_seconds: 1.25,
        end_seconds: 3.75,
        start_frame: 60_000,
        end_frame: 180_000,
    };

    assert_eq!(format_source_window_span(&source_window), "1.25-3.75s");
    assert_eq!(
        format_source_window_log_compact(&source_window),
        "win 1.25-3.75s src-1"
    );
}
