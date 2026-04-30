use super::*;
use crate::{
    action::GhostMode,
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
            ActionCommand::LockObject,
            ActionParams::Lock {
                object_id: "pad-a1".into(),
            },
            300,
        ),
        action(
            4,
            ActionCommand::GhostSetMode,
            ActionParams::Ghost {
                mode: Some(GhostMode::Assist),
                proposal_id: None,
            },
            400,
        ),
        action(5, ActionCommand::TransportPause, ActionParams::Empty, 500),
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
            ActionId(5)
        ]
    );
    assert!(!session.runtime_state.transport.is_playing);
    assert_eq!(session.runtime_state.transport.position_beats, 16.0);
    assert_eq!(
        session.runtime_state.lock_state.locked_object_ids,
        vec!["pad-a1".to_string()]
    );
    assert_eq!(session.ghost_state.mode, GhostMode::Assist);
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
fn plan_executor_rejects_unsupported_actions_without_mutating_session() {
    let action_log = action_log(vec![
        action(1, ActionCommand::TransportPlay, ActionParams::Empty, 100),
        action(2, ActionCommand::MutateScene, ActionParams::Empty, 200),
    ]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("unsupported");

    assert_eq!(
        error,
        ReplayExecutionError::UnsupportedAction {
            action_id: ActionId(2),
            command: ActionCommand::MutateScene
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
            ActionCommand::LockObject,
            ActionParams::Lock {
                object_id: "scene-drop".into(),
            },
            300,
        ),
        action(
            4,
            ActionCommand::GhostSetMode,
            ActionParams::Ghost {
                mode: Some(GhostMode::Assist),
                proposal_id: None,
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
        action(6, ActionCommand::TransportPause, ActionParams::Empty, 600),
    ]);
    let snapshots = vec![snapshot("snap-after-lock", 3)];
    let origin_plan = build_replay_target_plan(&action_log, &[], 6).expect("origin plan");
    let anchor_plan = build_replay_target_plan(&action_log, &[], 3).expect("anchor plan");
    let snapshot_plan =
        build_replay_target_plan(&action_log, &snapshots, 6).expect("snapshot plan");

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
        Some("snap-after-lock")
    );
    assert_eq!(
        origin_report.applied_action_ids,
        vec![
            ActionId(1),
            ActionId(2),
            ActionId(3),
            ActionId(4),
            ActionId(5),
            ActionId(6)
        ]
    );
    assert_eq!(
        anchor_report.applied_action_ids,
        vec![ActionId(1), ActionId(2), ActionId(3)]
    );
    assert_eq!(
        suffix_report.applied_action_ids,
        vec![ActionId(4), ActionId(5), ActionId(6)]
    );
    assert_eq!(
        snapshot_session.runtime_state.transport,
        origin_session.runtime_state.transport
    );
    assert_eq!(
        snapshot_session.runtime_state.lock_state,
        origin_session.runtime_state.lock_state
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
            ActionCommand::LockObject,
            ActionCommand::UnlockObject,
            ActionCommand::GhostSetMode,
            ActionCommand::SceneLaunch,
            ActionCommand::SceneRestore,
            ActionCommand::PromoteCaptureToPad,
            ActionCommand::PromoteCaptureToScene,
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
