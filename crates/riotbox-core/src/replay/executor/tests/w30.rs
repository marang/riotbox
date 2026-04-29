use super::*;
use crate::{
    action::{ActionTarget, TargetScope},
    ids::{BankId, CaptureId, PadId},
    replay::build_replay_target_plan,
    session::{SessionFile, W30PreviewModeState},
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
    let artifact_actions = vec![
        (
            ActionCommand::W30LoopFreeze,
            w30_capture_params("cap-01", 0.74),
        ),
        (
            ActionCommand::PromoteResample,
            w30_capture_params("cap-01", 0.86),
        ),
        (
            ActionCommand::W30CaptureToPad,
            ActionParams::Capture { bars: Some(2) },
        ),
        (
            ActionCommand::CaptureBarGroup,
            ActionParams::Capture { bars: Some(4) },
        ),
    ];

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
            .expect_err("artifact-producing W-30 actions stay outside replay subset");

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
