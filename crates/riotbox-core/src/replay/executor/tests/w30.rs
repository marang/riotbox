use super::*;
use crate::{
    action::{ActionTarget, TargetScope},
    ids::{BankId, CaptureId, PadId, SceneId},
    replay::{
        W30ArtifactReplayHydrationError, build_replay_target_plan,
        plan_w30_artifact_replay_hydration,
    },
    session::{
        CaptureRef, CaptureSourceWindow, CaptureTarget, CaptureType, SessionFile,
        W30PreviewModeState,
    },
};

#[test]
fn snapshot_suffix_replay_converges_with_origin_for_w30_cue_moves() {
    let action_log = action_log(vec![
        w30_action(1, ActionCommand::W30StepFocus, ActionParams::Empty, 100),
        w30_action(
            2,
            ActionCommand::W30BrowseSlicePool,
            w30_capture_params("cap-02", 1.0),
            200,
        ),
        w30_action(
            3,
            ActionCommand::W30AuditionPromoted,
            w30_capture_params("cap-02", 0.68),
            300,
        ),
        w30_action(
            4,
            ActionCommand::W30TriggerPad,
            w30_capture_params("cap-02", 0.84),
            400,
        ),
    ]);
    let snapshots = vec![snapshot("snap-after-browse", 2)];
    let origin_plan = build_replay_target_plan(&action_log, &[], 4).expect("origin plan");
    let anchor_plan = build_replay_target_plan(&action_log, &[], 2).expect("anchor plan");
    let snapshot_plan =
        build_replay_target_plan(&action_log, &snapshots, 4).expect("snapshot plan");

    let mut origin_session =
        SessionFile::new("origin-session", "riotbox-test", "2026-04-29T21:35:00Z");
    let mut snapshot_session =
        SessionFile::new("snapshot-session", "riotbox-test", "2026-04-29T21:35:00Z");

    apply_replay_plan_to_session(&mut origin_session, &origin_plan.suffix)
        .expect("origin replay succeeds");
    apply_replay_plan_to_session(&mut snapshot_session, &anchor_plan.suffix)
        .expect("anchor replay succeeds");
    apply_replay_plan_to_session(&mut snapshot_session, &snapshot_plan.suffix)
        .expect("snapshot suffix replay succeeds");

    assert_eq!(
        snapshot_plan
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("snap-after-browse")
    );
    assert_eq!(
        snapshot_session.runtime_state.lane_state.w30,
        origin_session.runtime_state.lane_state.w30
    );
    assert_eq!(
        snapshot_session.runtime_state.macro_state.w30_grit,
        origin_session.runtime_state.macro_state.w30_grit
    );
    assert_eq!(
        origin_session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .as_ref()
            .map(ToString::to_string),
        Some("bank-a".into())
    );
    assert_eq!(
        origin_session
            .runtime_state
            .lane_state
            .w30
            .focused_pad
            .as_ref()
            .map(ToString::to_string),
        Some("pad-01".into())
    );
    assert_eq!(
        origin_session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .map(ToString::to_string),
        Some("cap-02".into())
    );
    assert_eq!(
        origin_session.runtime_state.lane_state.w30.preview_mode,
        Some(W30PreviewModeState::LiveRecall)
    );
    assert!((origin_session.runtime_state.macro_state.w30_grit - 0.6888).abs() < 0.0001);
}

#[test]
fn w30_cue_rejects_missing_target_without_mutating_session() {
    let action_log = action_log(vec![targeted_action(
        1,
        ActionCommand::W30TriggerPad,
        w30_capture_params("cap-01", 0.84),
        ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-a")),
            ..Default::default()
        },
        100,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T21:35:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("W-30 cue requires bank and pad targets");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::W30TriggerPad,
            expected: "ActionTarget { bank_id: Some(_), pad_id: Some(_) }",
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn w30_non_focus_cue_rejects_missing_capture_without_mutating_session() {
    let action_log = action_log(vec![w30_action(
        1,
        ActionCommand::W30TriggerPad,
        ActionParams::Empty,
        100,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T21:35:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("W-30 trigger requires capture params");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::W30TriggerPad,
            expected: "ActionParams::Mutation { target_id: Some(_), .. }",
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn w30_step_focus_rejects_unexpected_params_without_mutating_session() {
    let action_log = action_log(vec![w30_action(
        1,
        ActionCommand::W30StepFocus,
        w30_capture_params("cap-01", 1.0),
        100,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T21:35:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("W-30 focus step requires empty params");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::W30StepFocus,
            expected: "ActionParams::Empty",
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn w30_artifact_producing_actions_reject_without_partial_mutation() {
    let artifact_actions = vec![(
        ActionCommand::CaptureBarGroup,
        ActionParams::Capture { bars: Some(4) },
    )];

    for (command, params) in artifact_actions {
        let action_log = action_log(vec![
            w30_action(
                1,
                ActionCommand::W30LiveRecall,
                w30_capture_params("cap-01", 0.62),
                100,
            ),
            w30_action(2, command, params, 200),
        ]);
        let plan = build_replay_target_plan(&action_log, &[], 2).expect("origin plan");
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T21:35:00Z");
        let original_session = session.clone();

        let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
            .expect_err("unsupported artifact-producing actions stay outside replay subset");

        assert_eq!(
            error,
            ReplayExecutionError::UnsupportedAction {
                action_id: ActionId(2),
                command,
            }
        );
        assert_eq!(
            session, original_session,
            "{command:?} must not leave the supported recall partially applied"
        );
    }
}

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

#[test]
fn w30_damage_profile_preserves_existing_preview_mode() {
    let action_log = action_log(vec![
        w30_action(
            1,
            ActionCommand::W30AuditionPromoted,
            w30_capture_params("cap-01", 0.68),
            100,
        ),
        w30_action(
            2,
            ActionCommand::W30ApplyDamageProfile,
            w30_capture_params("cap-01", 0.92),
            200,
        ),
    ]);
    let plan = build_replay_target_plan(&action_log, &[], 2).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T21:35:00Z");

    apply_replay_plan_to_session(&mut session, &plan.suffix).expect("W-30 replay succeeds");

    assert_eq!(
        session.runtime_state.lane_state.w30.preview_mode,
        Some(W30PreviewModeState::PromotedAudition)
    );
    assert_eq!(
        session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .map(ToString::to_string),
        Some("cap-01".into())
    );
    assert!((session.runtime_state.macro_state.w30_grit - 0.92).abs() < f32::EPSILON);
}

fn w30_action(
    id: u64,
    command: ActionCommand,
    params: ActionParams,
    committed_at: TimestampMs,
) -> Action {
    targeted_action(
        id,
        command,
        params,
        ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-a")),
            pad_id: Some(PadId::from("pad-01")),
            ..Default::default()
        },
        committed_at,
    )
}

fn w30_capture_params(capture_id: &str, intensity: f32) -> ActionParams {
    ActionParams::Mutation {
        intensity,
        target_id: Some(CaptureId::from(capture_id).to_string()),
    }
}

fn w30_promotion_params(capture_id: &str, destination: &str) -> ActionParams {
    ActionParams::Promotion {
        capture_id: Some(CaptureId::from(capture_id)),
        destination: Some(destination.into()),
    }
}

fn source_capture(capture_id: &str) -> CaptureRef {
    CaptureRef {
        capture_id: CaptureId::from(capture_id),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["source-1".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: format!("captures/{capture_id}.wav"),
        assigned_target: None,
        is_pinned: false,
        notes: None,
    }
}

fn resample_capture_for_action(action_id: u64) -> CaptureRef {
    CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Resample,
        source_origin_refs: vec!["source-1".into()],
        source_window: None,
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 1,
        created_from_action: Some(ActionId(action_id)),
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: None,
        is_pinned: false,
        notes: None,
    }
}

fn w30_capture_to_pad_capture_for_action(action_id: u64) -> CaptureRef {
    CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["source-1".into()],
        source_window: Some(CaptureSourceWindow {
            source_id: "source-1".into(),
            start_seconds: 1.0,
            end_seconds: 3.0,
            start_frame: 48_000,
            end_frame: 144_000,
        }),
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: Some(ActionId(action_id)),
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: false,
        notes: None,
    }
}

fn loop_freeze_capture_for_action(action_id: u64, capture_id: &str) -> CaptureRef {
    CaptureRef {
        capture_id: CaptureId::from(capture_id),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["source-1".into()],
        source_window: None,
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 0,
        created_from_action: Some(ActionId(action_id)),
        storage_path: format!("captures/{capture_id}.wav"),
        assigned_target: None,
        is_pinned: true,
        notes: None,
    }
}
