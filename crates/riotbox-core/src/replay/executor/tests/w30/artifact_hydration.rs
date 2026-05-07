use super::*;
use crate::{replay::plan_w30_artifact_replay_hydration, session::SessionFile};

#[test]
fn w30_loop_freeze_replay_hydrates_persisted_artifact_capture() {
    let action_log = action_log(vec![
        w30_action(
            1,
            ActionCommand::W30LiveRecall,
            w30_capture_params("cap-01", 0.62),
            100,
        ),
        w30_action(
            2,
            ActionCommand::W30LoopFreeze,
            w30_promotion_params("cap-01", "w30:loop_freeze"),
            200,
        ),
    ]);
    let plan = build_replay_target_plan(&action_log, &[], 2).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T10:20:00Z");
    let mut loop_freeze = loop_freeze_capture_for_action(2, "cap-02");
    loop_freeze.assigned_target = Some(crate::session::CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-05"),
    });
    session.captures.push(source_capture("cap-01"));
    session.captures.push(loop_freeze);

    let report = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect("W-30 loop-freeze replay hydrates artifact capture");

    assert_eq!(report.applied_action_ids, vec![ActionId(1), ActionId(2)]);
    assert_eq!(
        session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .as_ref()
            .map(ToString::to_string),
        Some("bank-b".into())
    );
    assert_eq!(
        session
            .runtime_state
            .lane_state
            .w30
            .focused_pad
            .as_ref()
            .map(ToString::to_string),
        Some("pad-05".into())
    );
    assert_eq!(
        session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .map(ToString::to_string),
        Some("cap-02".into())
    );
    assert_eq!(
        session.runtime_state.lane_state.w30.preview_mode,
        Some(W30PreviewModeState::LiveRecall)
    );
    assert!((session.runtime_state.macro_state.w30_grit - 0.78).abs() < f32::EPSILON);
}

#[test]
fn w30_artifact_replay_hydration_contract_accepts_explicit_resample_artifact() {
    let action_log = action_log(vec![
        w30_action(
            1,
            ActionCommand::W30LiveRecall,
            w30_capture_params("cap-01", 0.62),
            100,
        ),
        targeted_action(
            2,
            ActionCommand::PromoteResample,
            w30_promotion_params("cap-01", "w30:resample"),
            ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
            200,
        ),
    ]);
    let plan = build_replay_target_plan(&action_log, &[], 2).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T09:55:00Z");
    let mut source_capture = source_capture("cap-01");
    source_capture.assigned_target = Some(crate::session::CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    session.captures.push(source_capture);
    session.captures.push(resample_capture_for_action(2));

    let hydration_plan = plan_w30_artifact_replay_hydration(&session, &plan.suffix[1])
        .expect("explicit artifact identity is accepted");

    assert_eq!(hydration_plan.action_id, ActionId(2));
    assert_eq!(hydration_plan.command, ActionCommand::PromoteResample);
    assert_eq!(
        hydration_plan.produced_capture_id,
        CaptureId::from("cap-02")
    );
    assert_eq!(hydration_plan.source_capture_id, CaptureId::from("cap-01"));
    assert_eq!(hydration_plan.capture_type, CaptureType::Resample);
    assert_eq!(hydration_plan.storage_path, "captures/cap-02.wav");
    assert_eq!(hydration_plan.resample_generation_depth, 1);

    let report = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect("promote.resample replay hydrates explicit artifact identity");
    assert_eq!(report.applied_action_ids, vec![ActionId(1), ActionId(2)]);
    assert_eq!(
        session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .as_ref()
            .map(ToString::to_string),
        Some("bank-b".into())
    );
    assert_eq!(
        session
            .runtime_state
            .lane_state
            .w30
            .focused_pad
            .as_ref()
            .map(ToString::to_string),
        Some("pad-03".into())
    );
    assert_eq!(
        session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .map(ToString::to_string),
        Some("cap-02".into())
    );
    assert_eq!(
        session.runtime_state.lane_state.w30.preview_mode,
        Some(W30PreviewModeState::LiveRecall)
    );
    assert!((session.runtime_state.macro_state.w30_grit - 0.78).abs() < f32::EPSILON);
}

#[test]
fn w30_artifact_replay_hydration_contract_rejects_missing_artifact_identity() {
    let action_log = action_log(vec![w30_action(
        7,
        ActionCommand::PromoteResample,
        w30_promotion_params("cap-01", "w30:resample"),
        700,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T09:55:00Z");
    let mut capture = resample_capture_for_action(7);
    capture.storage_path.clear();
    session.captures.push(source_capture("cap-01"));
    session.captures.push(capture);

    let error = plan_w30_artifact_replay_hydration(&session, &plan.suffix[0])
        .expect_err("missing artifact path blocks hydration planning");

    assert_eq!(
        error,
        W30ArtifactReplayHydrationError::MissingStoragePath {
            capture_id: CaptureId::from("cap-02"),
        }
    );
}

#[test]
fn w30_artifact_replay_hydration_contract_rejects_ambiguous_produced_capture() {
    let action_log = action_log(vec![w30_action(
        8,
        ActionCommand::W30LoopFreeze,
        w30_promotion_params("cap-01", "w30:loop_freeze"),
        800,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T09:55:00Z");
    let mut first = loop_freeze_capture_for_action(8, "cap-02");
    let mut second = loop_freeze_capture_for_action(8, "cap-03");
    first.storage_path = "captures/cap-02.wav".into();
    second.storage_path = "captures/cap-03.wav".into();
    session.captures.push(source_capture("cap-01"));
    session.captures.push(first);
    session.captures.push(second);

    let error = plan_w30_artifact_replay_hydration(&session, &plan.suffix[0])
        .expect_err("one action may not hydrate from multiple produced captures");

    assert_eq!(
        error,
        W30ArtifactReplayHydrationError::AmbiguousProducedCapture {
            action_id: ActionId(8),
            command: ActionCommand::W30LoopFreeze,
            capture_count: 2,
        }
    );
}

#[test]
fn w30_artifact_replay_hydration_contract_rejects_missing_source_capture() {
    let action_log = action_log(vec![w30_action(
        9,
        ActionCommand::PromoteResample,
        w30_promotion_params("cap-missing", "w30:resample"),
        900,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T09:55:00Z");
    session.captures.push(resample_capture_for_action(9));

    let error = plan_w30_artifact_replay_hydration(&session, &plan.suffix[0])
        .expect_err("missing source capture blocks hydration planning");

    assert_eq!(
        error,
        W30ArtifactReplayHydrationError::MissingSourceCapture {
            capture_id: CaptureId::from("cap-missing"),
        }
    );
}
