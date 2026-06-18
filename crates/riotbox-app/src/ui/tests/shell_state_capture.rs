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
    assert!(rendered.contains("target 4 bars @ listen first"));
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
        rendered.contains("hear cap-01 unavailable: recapture"),
        "{rendered}"
    );
    assert!(
        rendered.contains("forge idle | tap ready/raw"),
        "{rendered}"
    );
    assert!(rendered.contains("g0"), "{rendered}");
    assert!(rendered.contains("latest promoted none"));
}

#[test]
fn capture_screen_shows_selected_length_before_first_capture() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.captures.clear();
    shell.app.session.runtime_state.capture.length_intent = CaptureLengthIntent::OneBar;
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("target 1 bar @ listen first"), "{rendered}");
    assert!(rendered.contains("1 [c] 1 bar @ listen first"), "{rendered}");
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
