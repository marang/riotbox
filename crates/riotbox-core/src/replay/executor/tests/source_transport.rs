use crate::{
    action::{CaptureLengthIntent, SourceMonitorMode},
    replay::build_committed_replay_plan,
    session::SessionFile,
};

use super::*;

#[test]
fn plan_executor_replays_source_transport_capture_state_for_restore_projection() {
    let action_log = action_log(vec![
        action(1, ActionCommand::TransportPlay, ActionParams::Empty, 100),
        action(
            2,
            ActionCommand::TransportSeek,
            ActionParams::Transport {
                position_beats: Some(21),
            },
            200,
        ),
        action(
            3,
            ActionCommand::SourceMonitorSetMode,
            ActionParams::SourceMonitor {
                mode: Some(SourceMonitorMode::Blend),
            },
            300,
        ),
        action(
            4,
            ActionCommand::SourceTimingConfirmGrid,
            ActionParams::SourceTimingGrid {
                source_id: Some(SourceId::from("src-1")),
                hypothesis_id: Some("primary-grid".into()),
            },
            350,
        ),
        action(
            5,
            ActionCommand::CaptureSetLength,
            ActionParams::CaptureLength {
                intent: Some(CaptureLengthIntent::OneBar),
            },
            390,
        ),
    ]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T17:39:00Z");

    apply_replay_plan_to_session(&mut session, &plan).expect("source transport replay succeeds");

    assert!(session.runtime_state.transport.is_playing);
    assert_eq!(session.runtime_state.transport.position_beats, 21.0);
    assert_eq!(
        session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Blend
    );
    assert_eq!(
        session.runtime_state.capture.length_intent,
        CaptureLengthIntent::OneBar
    );
    let confirmed = session
        .runtime_state
        .source_timing
        .confirmed_grid
        .expect("confirmed source grid replay state");
    assert_eq!(confirmed.source_id, SourceId::from("src-1"));
    assert_eq!(confirmed.hypothesis_id.as_deref(), Some("primary-grid"));
    assert_eq!(confirmed.confirmed_by_action, ActionId(4));
    assert_eq!(confirmed.confirmed_at, 350);
}
