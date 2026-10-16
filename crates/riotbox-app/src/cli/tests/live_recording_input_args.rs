use crate::cli::args::parse_args;
use crate::cli::model::{LaunchMode, LiveRecordingInputMode};
use riotbox_core::action::LiveRecordingDuration;

fn recording_args() -> Vec<String> {
    [
        "--live-master-recording-execute",
        "--session",
        "session.json",
        "--live-recording-destination",
        "exports/master.wav",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[test]
fn terminal_fill_input_is_reachable_through_the_real_recording_parser() {
    let mut args = recording_args();
    args.push("--live-recording-terminal-input".into());
    let result = parse_args(args);
    assert!(
        result.is_ok(),
        "explicit recording input opt-in must parse: {result:?}"
    );
    assert!(matches!(
        result.unwrap().mode,
        LaunchMode::LiveMasterRecordingExecute {
            input_mode: LiveRecordingInputMode::TerminalFill,
            ..
        }
    ));
}

#[test]
fn recording_input_omission_preserves_disabled_default_for_every_bounded_window() {
    assert_eq!(
        LiveRecordingInputMode::default(),
        LiveRecordingInputMode::Disabled
    );
    for (bars, duration) in [
        (None, LiveRecordingDuration::TwoBars),
        (Some("2"), LiveRecordingDuration::TwoBars),
        (Some("8"), LiveRecordingDuration::EightBars),
        (Some("16"), LiveRecordingDuration::SixteenBars),
    ] {
        let mut args = recording_args();
        if let Some(bars) = bars {
            args.extend(["--live-recording-bars".into(), bars.into()]);
        }
        let launch = parse_args(args).unwrap();
        assert!(launch.observer_path.is_none());
        let LaunchMode::LiveMasterRecordingExecute {
            input_mode,
            duration: actual,
            ..
        } = launch.mode
        else {
            panic!("expected recording launch");
        };
        assert_eq!(input_mode, LiveRecordingInputMode::Disabled);
        assert_eq!(actual, duration);
    }
}

#[test]
fn terminal_input_opt_in_preserves_each_bounded_duration_and_existing_optional_paths() {
    for (bars, duration) in [
        ("2", LiveRecordingDuration::TwoBars),
        ("8", LiveRecordingDuration::EightBars),
        ("16", LiveRecordingDuration::SixteenBars),
    ] {
        let mut args = vec!["--live-recording-terminal-input".into()];
        args.extend(recording_args());
        args.extend([
            "--live-recording-bars".into(),
            bars.into(),
            "--graph".into(),
            "generated-graph.json".into(),
            "--observer".into(),
            "events.ndjson".into(),
        ]);
        let launch = parse_args(args).unwrap();
        assert_eq!(launch.observer_path, Some("events.ndjson".into()));
        let LaunchMode::LiveMasterRecordingExecute {
            input_mode,
            duration: actual,
            source_graph_path,
            ..
        } = launch.mode
        else {
            panic!("expected recording launch");
        };
        assert_eq!(input_mode, LiveRecordingInputMode::TerminalFill);
        assert_eq!(actual, duration);
        assert_eq!(source_graph_path, Some("generated-graph.json".into()));
    }
}

#[test]
fn terminal_input_flag_requires_recording_execute_before_other_mode_dispatch() {
    for mode in [
        None,
        Some("--live-recording-readiness-report"),
        Some("--daw-export-readiness-report"),
        Some("--stem-package-local-ci-report"),
        Some("--live-master-dawproject-execute"),
    ] {
        let mut args = vec![
            "--session".into(),
            "session.json".into(),
            "--live-recording-terminal-input".into(),
        ];
        args.extend(mode.map(str::to_owned));
        assert_eq!(
            parse_args(args).unwrap_err(),
            "--live-recording-terminal-input requires --live-master-recording-execute"
        );
    }
}

#[test]
fn terminal_input_opt_in_does_not_relax_recording_mode_conflicts_or_required_paths() {
    for conflict in [
        vec!["--live-recording-readiness-report"],
        vec!["--source", "not-opened.wav"],
        vec!["--stem-package-local-ci-report"],
        vec!["--live-master-dawproject-execute"],
    ] {
        let mut args = recording_args();
        args.push("--live-recording-terminal-input".into());
        args.extend(conflict.into_iter().map(str::to_owned));
        assert!(parse_args(args).unwrap_err().contains("cannot be combined"));
    }
    for args in [
        vec![
            "--live-master-recording-execute",
            "--live-recording-terminal-input",
        ],
        vec![
            "--live-master-recording-execute",
            "--session",
            "session.json",
            "--live-recording-terminal-input",
        ],
    ] {
        assert!(parse_args(args.into_iter().map(str::to_owned)).is_err());
    }
}

#[test]
fn terminal_input_is_a_single_use_flag_without_an_attached_value() {
    let mut duplicate = recording_args();
    duplicate.extend([
        "--live-recording-terminal-input".into(),
        "--live-recording-terminal-input".into(),
    ]);
    assert!(parse_args(duplicate).unwrap_err().contains("at most once"));
    for value in ["true", "false", "terminal_fill", "disabled"] {
        let mut args = recording_args();
        args.extend(["--live-recording-terminal-input".into(), value.into()]);
        assert!(parse_args(args).unwrap_err().contains("unknown argument"));
    }
}

#[test]
fn typed_recording_input_has_stable_wire_names_and_capture_contexts() {
    assert_eq!(
        serde_json::to_value(LiveRecordingInputMode::Disabled).unwrap(),
        "disabled"
    );
    assert_eq!(
        serde_json::to_value(LiveRecordingInputMode::TerminalFill).unwrap(),
        "terminal_fill"
    );
    assert_eq!(
        LiveRecordingInputMode::Disabled.capture_context(),
        "non_interactive_real_audio_cli"
    );
    assert_eq!(
        LiveRecordingInputMode::TerminalFill.capture_context(),
        "terminal_input_real_audio_cli"
    );
    let help = parse_args(["--help".into()]).unwrap_err();
    assert!(help.contains("[--live-recording-terminal-input]"));
    assert!(help.contains("Input is disabled by default"));
}
