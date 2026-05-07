use super::*;
use crate::session::SessionFile;

#[test]
fn w30_capture_to_pad_replay_hydrates_persisted_source_window_capture() {
    let action_log = action_log(vec![w30_action(
        2,
        ActionCommand::W30CaptureToPad,
        ActionParams::Capture { bars: Some(2) },
        200,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:05:00Z");
    session
        .captures
        .push(w30_capture_to_pad_capture_for_action(2));

    let report = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect("w30.capture_to_pad hydrates persisted capture artifact");

    assert_eq!(report.applied_action_ids, vec![ActionId(2)]);
    assert_eq!(
        session.runtime_state.lane_state.w30.active_bank,
        Some(BankId::from("bank-a"))
    );
    assert_eq!(
        session.runtime_state.lane_state.w30.focused_pad,
        Some(PadId::from("pad-01"))
    );
    assert_eq!(
        session.runtime_state.lane_state.w30.last_capture,
        Some(CaptureId::from("cap-02"))
    );
    assert_eq!(
        session.runtime_state.lane_state.w30.preview_mode,
        Some(W30PreviewModeState::LiveRecall)
    );
}

#[test]
fn capture_now_and_loop_replay_hydrate_persisted_source_window_captures() {
    for (action_id, command, bars) in [
        (2, ActionCommand::CaptureNow, 1),
        (3, ActionCommand::CaptureLoop, 2),
    ] {
        let action_log = action_log(vec![w30_action(
            action_id,
            command,
            ActionParams::Capture { bars: Some(bars) },
            200,
        )]);
        let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:05:00Z");
        session.captures.push(loop_capture_for_action(action_id));

        let report = apply_replay_plan_to_session(&mut session, &plan.suffix)
            .expect("source-window loop capture hydrates persisted artifact");

        assert_eq!(report.applied_action_ids, vec![ActionId(action_id)]);
        assert_eq!(
            session.runtime_state.lane_state.w30.last_capture,
            Some(CaptureId::from("cap-02"))
        );
        assert_eq!(
            session.runtime_state.lane_state.w30.preview_mode,
            Some(W30PreviewModeState::LiveRecall)
        );
    }
}

#[test]
fn capture_loop_replay_rejects_non_loop_capture_without_mutation() {
    let action_log = action_log(vec![w30_action(
        2,
        ActionCommand::CaptureLoop,
        ActionParams::Capture { bars: Some(2) },
        200,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:05:00Z");
    let mut capture = loop_capture_for_action(2);
    capture.capture_type = CaptureType::Pad;
    session.captures.push(capture);
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("capture.loop requires loop capture identity");

    assert_eq!(
        error,
        ReplayExecutionError::ArtifactHydration {
            action_id: ActionId(2),
            command: ActionCommand::CaptureLoop,
            reason: W30ArtifactReplayHydrationError::InvalidLoopCaptureIdentity {
                capture_id: CaptureId::from("cap-02"),
            },
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn capture_bar_group_replay_hydrates_persisted_source_window_capture() {
    let action_log = action_log(vec![w30_action(
        2,
        ActionCommand::CaptureBarGroup,
        ActionParams::Capture { bars: Some(4) },
        200,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:05:00Z");
    session
        .captures
        .push(capture_bar_group_capture_for_action(2));

    let report = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect("capture.bar_group hydrates persisted source-window capture");

    assert_eq!(report.applied_action_ids, vec![ActionId(2)]);
    assert_eq!(session.runtime_state.lane_state.w30.active_bank, None);
    assert_eq!(session.runtime_state.lane_state.w30.focused_pad, None);
    assert_eq!(
        session.runtime_state.lane_state.w30.last_capture,
        Some(CaptureId::from("cap-02"))
    );
    assert_eq!(
        session.runtime_state.lane_state.w30.preview_mode,
        Some(W30PreviewModeState::LiveRecall)
    );
}

#[test]
fn capture_bar_group_replay_rejects_missing_source_window_without_mutation() {
    let action_log = action_log(vec![w30_action(
        2,
        ActionCommand::CaptureBarGroup,
        ActionParams::Capture { bars: Some(4) },
        200,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:05:00Z");
    let mut capture = capture_bar_group_capture_for_action(2);
    capture.source_window = None;
    session.captures.push(capture);
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("capture.bar_group requires source-window identity");

    assert_eq!(
        error,
        ReplayExecutionError::ArtifactHydration {
            action_id: ActionId(2),
            command: ActionCommand::CaptureBarGroup,
            reason: W30ArtifactReplayHydrationError::MissingSourceWindowForSourceBackedCapture {
                capture_id: CaptureId::from("cap-02"),
            },
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn w30_capture_to_pad_replay_rejects_missing_source_window_without_mutation() {
    let action_log = action_log(vec![w30_action(
        2,
        ActionCommand::W30CaptureToPad,
        ActionParams::Capture { bars: Some(2) },
        200,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:05:00Z");
    let mut capture = w30_capture_to_pad_capture_for_action(2);
    capture.source_window = None;
    session.captures.push(capture);
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("w30.capture_to_pad requires source-window identity");

    assert_eq!(
        error,
        ReplayExecutionError::ArtifactHydration {
            action_id: ActionId(2),
            command: ActionCommand::W30CaptureToPad,
            reason: W30ArtifactReplayHydrationError::MissingSourceWindowForSourceBackedCapture {
                capture_id: CaptureId::from("cap-02"),
            },
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn w30_capture_to_pad_replay_rejects_non_pad_capture_without_mutation() {
    let action_log = action_log(vec![w30_action(
        2,
        ActionCommand::W30CaptureToPad,
        ActionParams::Capture { bars: Some(2) },
        200,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:05:00Z");
    let mut capture = w30_capture_to_pad_capture_for_action(2);
    capture.capture_type = CaptureType::Resample;
    session.captures.push(capture);
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("w30.capture_to_pad requires a pad capture identity");

    assert_eq!(
        error,
        ReplayExecutionError::ArtifactHydration {
            action_id: ActionId(2),
            command: ActionCommand::W30CaptureToPad,
            reason: W30ArtifactReplayHydrationError::InvalidPadCaptureIdentity {
                capture_id: CaptureId::from("cap-02"),
            },
        }
    );
    assert_eq!(session, original_session);
}
