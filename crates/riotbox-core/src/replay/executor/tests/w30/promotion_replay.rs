use super::*;
use crate::session::SessionFile;

#[test]
fn promote_capture_to_pad_replay_assigns_existing_capture_and_w30_focus() {
    let action_log = action_log(vec![w30_action(
        2,
        ActionCommand::PromoteCaptureToPad,
        w30_promotion_params("cap-01", "w30:bank-a/pad-01"),
        200,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:05:00Z");
    let mut capture = source_capture("cap-01");
    capture.notes = Some("source capture".into());
    session.captures.push(capture);

    let report = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect("promote.capture_to_pad replays existing capture assignment");

    assert_eq!(report.applied_action_ids, vec![ActionId(2)]);
    let capture = session
        .captures
        .iter()
        .find(|capture| capture.capture_id == CaptureId::from("cap-01"))
        .expect("promoted capture");
    assert_eq!(
        capture.assigned_target,
        Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        })
    );
    assert_eq!(
        capture.notes.as_deref(),
        Some("source capture | promoted to pad bank-a/pad-01")
    );
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
        Some(CaptureId::from("cap-01"))
    );
    assert_eq!(
        session.runtime_state.lane_state.w30.preview_mode,
        Some(W30PreviewModeState::LiveRecall)
    );
}

#[test]
fn promote_capture_to_pad_replay_rejects_missing_capture_without_mutation() {
    let action_log = action_log(vec![w30_action(
        2,
        ActionCommand::PromoteCaptureToPad,
        w30_promotion_params("cap-missing", "w30:bank-a/pad-01"),
        200,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:05:00Z");
    session.captures.push(source_capture("cap-01"));
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("promote.capture_to_pad requires existing capture identity");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(2),
            command: ActionCommand::PromoteCaptureToPad,
            expected: "existing CaptureRef for ActionParams::Promotion.capture_id",
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn promote_capture_to_scene_replay_assigns_existing_capture() {
    let action_log = action_log(vec![targeted_action(
        2,
        ActionCommand::PromoteCaptureToScene,
        w30_promotion_params("cap-01", "scene:drop-1"),
        ActionTarget {
            scope: Some(TargetScope::LaneW30),
            scene_id: Some(SceneId::from("drop-1")),
            ..Default::default()
        },
        200,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:05:00Z");
    let mut capture = source_capture("cap-01");
    capture.notes = Some("source capture".into());
    session.captures.push(capture);

    let report = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect("promote.capture_to_scene replays existing capture assignment");

    assert_eq!(report.applied_action_ids, vec![ActionId(2)]);
    let capture = session
        .captures
        .iter()
        .find(|capture| capture.capture_id == CaptureId::from("cap-01"))
        .expect("promoted capture");
    assert_eq!(
        capture.assigned_target,
        Some(CaptureTarget::Scene(SceneId::from("drop-1")))
    );
    assert_eq!(
        capture.notes.as_deref(),
        Some("source capture | promoted to scene drop-1")
    );
    assert_eq!(
        session.runtime_state.lane_state.w30.last_capture,
        Some(CaptureId::from("cap-01"))
    );
}

#[test]
fn promote_capture_to_scene_replay_rejects_missing_scene_target_without_mutation() {
    let action_log = action_log(vec![w30_action(
        2,
        ActionCommand::PromoteCaptureToScene,
        w30_promotion_params("cap-01", "scene:drop-1"),
        200,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:05:00Z");
    session.captures.push(source_capture("cap-01"));
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("promote.capture_to_scene requires explicit scene target");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(2),
            command: ActionCommand::PromoteCaptureToScene,
            expected: "ActionTarget { scene_id: Some(_) }",
        }
    );
    assert_eq!(session, original_session);
}
