use super::*;
use crate::{
    action::{ActionReplayCoverage, CaptureLengthIntent, GhostMode, SourceMonitorMode},
    replay::{build_committed_replay_plan, build_replay_target_plan},
    session::SessionFile,
};

#[test]
fn plan_executor_applies_supported_structural_actions_in_commit_order() {
    let action_log = action_log(vec![
        action(1, ActionCommand::TransportPlay, ActionParams::Empty, 100),
        action(
            2,
            ActionCommand::TransportSeek,
            ActionParams::Transport {
                position_beats: Some(16),
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
            ActionCommand::SourceTimingRevertGrid,
            ActionParams::SourceTimingGrid {
                source_id: Some(SourceId::from("src-1")),
                hypothesis_id: Some("primary-grid".into()),
            },
            375,
        ),
        action(
            6,
            ActionCommand::CaptureSetLength,
            ActionParams::CaptureLength {
                intent: Some(CaptureLengthIntent::OneBar),
            },
            390,
        ),
        action(
            7,
            ActionCommand::LockObject,
            ActionParams::Lock {
                object_id: "pad-a1".into(),
            },
            400,
        ),
        action(
            8,
            ActionCommand::GhostSetMode,
            ActionParams::Ghost {
                mode: Some(GhostMode::Assist),
                proposal_id: None,
            },
            500,
        ),
        action(9, ActionCommand::TransportPause, ActionParams::Empty, 600),
    ]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");

    let report = apply_replay_plan_to_session(&mut session, &plan).expect("supported replay plan");

    assert_eq!(
        report.applied_action_ids,
        vec![
            ActionId(1),
            ActionId(2),
            ActionId(3),
            ActionId(4),
            ActionId(5),
            ActionId(6),
            ActionId(7),
            ActionId(8),
            ActionId(9)
        ]
    );
    assert!(!session.runtime_state.transport.is_playing);
    assert_eq!(session.runtime_state.transport.position_beats, 16.0);
    assert_eq!(
        session.runtime_state.lock_state.locked_object_ids,
        vec!["pad-a1".to_string()]
    );
    assert_eq!(
        session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Blend
    );
    assert!(session.runtime_state.source_timing.confirmed_grid.is_none());
    assert_eq!(
        session.runtime_state.capture.length_intent,
        CaptureLengthIntent::OneBar
    );
    assert_eq!(
        session.runtime_state.capture.length_set_by_action,
        Some(ActionId(6))
    );
    assert_eq!(session.runtime_state.capture.length_set_at, Some(390));
    assert_eq!(session.ghost_state.mode, GhostMode::Assist);
}

#[test]
fn plan_executor_rejects_capture_length_without_intent_param() {
    let action_log = action_log(vec![action(
        1,
        ActionCommand::CaptureSetLength,
        ActionParams::CaptureLength { intent: None },
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T08:10:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("invalid params");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::CaptureSetLength,
            expected: "ActionParams::CaptureLength { intent: Some(_) }"
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn plan_executor_rejects_source_monitor_mode_without_mode_param() {
    let action_log = action_log(vec![action(
        1,
        ActionCommand::SourceMonitorSetMode,
        ActionParams::SourceMonitor { mode: None },
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T08:10:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("invalid params");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::SourceMonitorSetMode,
            expected: "ActionParams::SourceMonitor { mode: Some(_) }"
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn plan_executor_rejects_source_timing_confirmation_without_source_id() {
    let action_log = action_log(vec![action(
        1,
        ActionCommand::SourceTimingConfirmGrid,
        ActionParams::SourceTimingGrid {
            source_id: None,
            hypothesis_id: Some("primary-grid".into()),
        },
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T12:10:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("invalid params");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::SourceTimingConfirmGrid,
            expected: "ActionParams::SourceTimingGrid { source_id: Some(_) }"
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn plan_executor_handles_stop_unlock_and_duplicate_lock_deterministically() {
    let action_log = action_log(vec![
        action(
            1,
            ActionCommand::TransportSeek,
            ActionParams::Transport {
                position_beats: Some(32),
            },
            100,
        ),
        action(2, ActionCommand::TransportPlay, ActionParams::Empty, 200),
        action(
            3,
            ActionCommand::LockObject,
            ActionParams::Lock {
                object_id: "scene-drop".into(),
            },
            300,
        ),
        action(
            4,
            ActionCommand::LockObject,
            ActionParams::Lock {
                object_id: "scene-drop".into(),
            },
            400,
        ),
        action(
            5,
            ActionCommand::UnlockObject,
            ActionParams::Lock {
                object_id: "scene-drop".into(),
            },
            500,
        ),
        action(6, ActionCommand::TransportStop, ActionParams::Empty, 600),
    ]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");

    apply_replay_plan_to_session(&mut session, &plan).expect("supported replay plan");

    assert!(!session.runtime_state.transport.is_playing);
    assert_eq!(session.runtime_state.transport.position_beats, 0.0);
    assert!(
        session
            .runtime_state
            .lock_state
            .locked_object_ids
            .is_empty()
    );
}

#[test]
fn plan_executor_applies_scene_mutation_to_session_macro_state() {
    let action_log = action_log(vec![action(
        1,
        ActionCommand::MutateScene,
        ActionParams::Mutation {
            intensity: 0.8,
            target_id: Some("scene-1".into()),
        },
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-26T12:00:00Z");
    session.runtime_state.macro_state.scene_aggression = 0.4;

    apply_replay_plan_to_session(&mut session, &plan).expect("supported replay plan");

    assert!((session.runtime_state.macro_state.scene_aggression - 0.6).abs() < f32::EPSILON);
}

#[test]
fn plan_executor_rejects_scene_mutation_without_mutation_params() {
    let action_log = action_log(vec![action(
        1,
        ActionCommand::MutateScene,
        ActionParams::Empty,
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-26T12:05:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("invalid params");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::MutateScene,
            expected: "ActionParams::Mutation { intensity, .. }"
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn plan_executor_rejects_unsupported_actions_without_mutating_session() {
    let action_log = action_log(vec![
        action(1, ActionCommand::TransportPlay, ActionParams::Empty, 100),
        action(2, ActionCommand::MutateLane, ActionParams::Empty, 200),
    ]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("unsupported");

    assert_eq!(
        error,
        ReplayExecutionError::UnsupportedAction {
            action_id: ActionId(2),
            command: ActionCommand::MutateLane
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn plan_executor_rejects_invalid_params_without_mutating_session() {
    let action_log = action_log(vec![action(
        1,
        ActionCommand::GhostSetMode,
        ActionParams::Ghost {
            mode: None,
            proposal_id: None,
        },
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("invalid params");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::GhostSetMode,
            expected: "ActionParams::Ghost { mode: Some(_) }"
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn snapshot_suffix_replay_converges_with_origin_for_supported_structural_actions() {
    let action_log = action_log(vec![
        action(1, ActionCommand::TransportPlay, ActionParams::Empty, 100),
        action(
            2,
            ActionCommand::TransportSeek,
            ActionParams::Transport {
                position_beats: Some(16),
            },
            200,
        ),
        action(
            3,
            ActionCommand::SourceMonitorSetMode,
            ActionParams::SourceMonitor {
                mode: Some(SourceMonitorMode::Riotbox),
            },
            300,
        ),
        action(
            4,
            ActionCommand::LockObject,
            ActionParams::Lock {
                object_id: "scene-drop".into(),
            },
            400,
        ),
        action(
            5,
            ActionCommand::GhostSetMode,
            ActionParams::Ghost {
                mode: Some(GhostMode::Assist),
                proposal_id: None,
            },
            500,
        ),
        action(
            6,
            ActionCommand::UnlockObject,
            ActionParams::Lock {
                object_id: "scene-drop".into(),
            },
            600,
        ),
        action(7, ActionCommand::TransportPause, ActionParams::Empty, 700),
    ]);
    let snapshots = vec![snapshot("snap-after-monitor", 3)];
    let origin_plan = build_replay_target_plan(&action_log, &[], 7).expect("origin plan");
    let anchor_plan = build_replay_target_plan(&action_log, &[], 3).expect("anchor plan");
    let snapshot_plan =
        build_replay_target_plan(&action_log, &snapshots, 7).expect("snapshot plan");

    let mut origin_session =
        SessionFile::new("origin-session", "riotbox-test", "2026-04-29T20:30:00Z");
    let mut snapshot_session =
        SessionFile::new("snapshot-session", "riotbox-test", "2026-04-29T20:30:00Z");

    let origin_report = apply_replay_plan_to_session(&mut origin_session, &origin_plan.suffix)
        .expect("origin replay succeeds");
    let anchor_report = apply_replay_plan_to_session(&mut snapshot_session, &anchor_plan.suffix)
        .expect("anchor replay succeeds");
    let suffix_report = apply_replay_plan_to_session(&mut snapshot_session, &snapshot_plan.suffix)
        .expect("snapshot suffix replay succeeds");

    assert_eq!(
        snapshot_plan
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("snap-after-monitor")
    );
    assert_eq!(
        origin_report.applied_action_ids,
        vec![
            ActionId(1),
            ActionId(2),
            ActionId(3),
            ActionId(4),
            ActionId(5),
            ActionId(6),
            ActionId(7)
        ]
    );
    assert_eq!(
        anchor_report.applied_action_ids,
        vec![ActionId(1), ActionId(2), ActionId(3)]
    );
    assert_eq!(
        suffix_report.applied_action_ids,
        vec![ActionId(4), ActionId(5), ActionId(6), ActionId(7)]
    );
    assert_eq!(
        snapshot_session.runtime_state.transport,
        origin_session.runtime_state.transport
    );
    assert_eq!(
        snapshot_session.runtime_state.lock_state,
        origin_session.runtime_state.lock_state
    );
    assert_eq!(
        snapshot_session.runtime_state.source_monitor,
        origin_session.runtime_state.source_monitor
    );
    assert_eq!(snapshot_session.ghost_state, origin_session.ghost_state);
}

#[test]
fn supported_action_list_documents_the_initial_executor_subset() {
    assert_eq!(
        replay_supported_action_commands(),
        &[
            ActionCommand::TransportPlay,
            ActionCommand::TransportPause,
            ActionCommand::TransportStop,
            ActionCommand::TransportSeek,
            ActionCommand::SourceMonitorSetMode,
            ActionCommand::SourceTimingConfirmGrid,
            ActionCommand::SourceTimingRevertGrid,
            ActionCommand::LockObject,
            ActionCommand::UnlockObject,
            ActionCommand::GhostSetMode,
            ActionCommand::MutateScene,
            ActionCommand::SceneLaunch,
            ActionCommand::SceneRestore,
            ActionCommand::PromoteCaptureToPad,
            ActionCommand::PromoteCaptureToScene,
            ActionCommand::CaptureSetLength,
            ActionCommand::CaptureNow,
            ActionCommand::CaptureLoop,
            ActionCommand::CaptureBarGroup,
            ActionCommand::Mc202SetRole,
            ActionCommand::Mc202GenerateFollower,
            ActionCommand::Mc202GenerateAnswer,
            ActionCommand::Mc202GeneratePressure,
            ActionCommand::Mc202GenerateInstigator,
            ActionCommand::Mc202MutatePhrase,
            ActionCommand::Tr909SetSlam,
            ActionCommand::Tr909FillNext,
            ActionCommand::Tr909ReinforceBreak,
            ActionCommand::Tr909Takeover,
            ActionCommand::Tr909SceneLock,
            ActionCommand::Tr909Release,
            ActionCommand::W30LiveRecall,
            ActionCommand::W30TriggerPad,
            ActionCommand::W30AuditionRawCapture,
            ActionCommand::W30AuditionPromoted,
            ActionCommand::W30SwapBank,
            ActionCommand::W30BrowseSlicePool,
            ActionCommand::W30StepFocus,
            ActionCommand::W30ApplyDamageProfile,
            ActionCommand::W30CaptureToPad,
            ActionCommand::W30LoopFreeze,
            ActionCommand::PromoteResample,
        ]
    );
}

#[test]
fn replay_supported_action_list_matches_command_coverage_contract() {
    for command in replay_supported_action_commands() {
        assert_eq!(
            command.replay_coverage(),
            ActionReplayCoverage::Supported,
            "{command} is in the executor supported list but the command contract marks it differently"
        );
    }

    let supported_from_contract = ActionCommand::all()
        .iter()
        .copied()
        .filter(|command| command.replay_coverage() == ActionReplayCoverage::Supported)
        .collect::<Vec<_>>();

    assert_eq!(
        replay_supported_action_commands().len(),
        supported_from_contract.len()
    );
    for command in supported_from_contract {
        assert!(
            replay_supported_action_commands().contains(&command),
            "{command} is marked replay-supported but is missing from the executor list"
        );
    }
}
