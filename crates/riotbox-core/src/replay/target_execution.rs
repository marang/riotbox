use crate::{
    ids::ActionId,
    replay::{
        ReplayExecutionError, ReplayPlanError, apply_graph_aware_replay_plan_to_session,
        apply_replay_plan_to_session, build_replay_target_plan,
    },
    session::SessionFile,
    source_graph::SourceGraph,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayTargetExecutionError {
    Plan(ReplayPlanError),
    Execution(ReplayExecutionError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayTargetExecutionReport {
    pub target_action_cursor: usize,
    pub anchor_snapshot_id: Option<String>,
    pub anchor_action_cursor: Option<usize>,
    pub applied_action_ids: Vec<ActionId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotPayloadHydrationError {
    Plan(ReplayPlanError),
    MissingSnapshotAnchor {
        target_action_cursor: usize,
    },
    MissingSnapshotPayload {
        snapshot_id: String,
    },
    PayloadSnapshotIdMismatch {
        snapshot_id: String,
        payload_snapshot_id: String,
    },
    PayloadActionCursorMismatch {
        snapshot_id: String,
        snapshot_action_cursor: usize,
        payload_action_cursor: usize,
    },
    Execution(ReplayTargetExecutionError),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotPayloadHydrationReport {
    pub session: SessionFile,
    pub replay_report: ReplayTargetExecutionReport,
}

pub fn hydrate_replay_target_from_snapshot_payload(
    session: &SessionFile,
    target_action_cursor: usize,
    source_graph: Option<&SourceGraph>,
) -> Result<SnapshotPayloadHydrationReport, SnapshotPayloadHydrationError> {
    let plan = build_replay_target_plan(
        &session.action_log,
        &session.snapshots,
        target_action_cursor,
    )
    .map_err(SnapshotPayloadHydrationError::Plan)?;
    let Some(anchor) = plan.anchor else {
        return Err(SnapshotPayloadHydrationError::MissingSnapshotAnchor {
            target_action_cursor,
        });
    };
    let snapshot_id = anchor.snapshot_id.as_str().to_owned();
    let payload = anchor.payload.as_ref().ok_or_else(|| {
        SnapshotPayloadHydrationError::MissingSnapshotPayload {
            snapshot_id: snapshot_id.clone(),
        }
    })?;

    if payload.snapshot_id != anchor.snapshot_id {
        return Err(SnapshotPayloadHydrationError::PayloadSnapshotIdMismatch {
            snapshot_id,
            payload_snapshot_id: payload.snapshot_id.as_str().to_owned(),
        });
    }

    if payload.action_cursor != anchor.action_cursor {
        return Err(SnapshotPayloadHydrationError::PayloadActionCursorMismatch {
            snapshot_id,
            snapshot_action_cursor: anchor.action_cursor,
            payload_action_cursor: payload.action_cursor,
        });
    }

    let mut hydrated_session = session.clone();
    hydrated_session.runtime_state = payload.runtime_state.clone();
    let replay_report = apply_replay_target_suffix_to_session(
        &mut hydrated_session,
        target_action_cursor,
        source_graph,
    )
    .map_err(SnapshotPayloadHydrationError::Execution)?;

    Ok(SnapshotPayloadHydrationReport {
        session: hydrated_session,
        replay_report,
    })
}

pub fn apply_replay_target_suffix_to_session(
    session: &mut SessionFile,
    target_action_cursor: usize,
    source_graph: Option<&SourceGraph>,
) -> Result<ReplayTargetExecutionReport, ReplayTargetExecutionError> {
    let action_log = session.action_log.clone();
    let snapshots = session.snapshots.clone();
    let plan = build_replay_target_plan(&action_log, &snapshots, target_action_cursor)
        .map_err(ReplayTargetExecutionError::Plan)?;
    let anchor_snapshot_id = plan
        .anchor
        .map(|snapshot| snapshot.snapshot_id.as_str().to_owned());
    let anchor_action_cursor = plan.anchor.map(|snapshot| snapshot.action_cursor);

    let execution_report = match source_graph {
        Some(source_graph) => {
            apply_graph_aware_replay_plan_to_session(session, &plan.suffix, source_graph)
        }
        None => apply_replay_plan_to_session(session, &plan.suffix),
    }
    .map_err(ReplayTargetExecutionError::Execution)?;

    Ok(ReplayTargetExecutionReport {
        target_action_cursor,
        anchor_snapshot_id,
        anchor_action_cursor,
        applied_action_ids: execution_report.applied_action_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        TimestampMs,
        action::{
            Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget,
            ActorType, CommitBoundary, Quantization, SourceMonitorMode, UndoPolicy,
        },
        ids::SnapshotId,
        session::{
            ActionCommitRecord, ActionLog, ReplayPolicy, Snapshot, SnapshotPayload,
            SnapshotPayloadVersion,
        },
        transport::CommitBoundaryState,
    };

    fn action(
        id: u64,
        command: ActionCommand,
        params: ActionParams,
        committed_at: TimestampMs,
    ) -> Action {
        Action {
            id: ActionId(id),
            actor: ActorType::User,
            command,
            params,
            target: ActionTarget::default(),
            requested_at: committed_at - 10,
            quantization: Quantization::NextBar,
            status: ActionStatus::Committed,
            committed_at: Some(committed_at),
            result: Some(ActionResult {
                accepted: true,
                summary: "committed".into(),
            }),
            undo_policy: UndoPolicy::Undoable,
            explanation: None,
        }
    }

    fn commit_record(
        action_id: u64,
        beat_index: u64,
        commit_sequence: u32,
        committed_at: TimestampMs,
    ) -> ActionCommitRecord {
        ActionCommitRecord {
            action_id: ActionId(action_id),
            boundary: CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index,
                bar_index: beat_index / 4,
                phrase_index: beat_index / 16,
                scene_id: None,
            },
            commit_sequence,
            committed_at,
            mc202_source_phrase_plan: None,
        }
    }

    fn action_log(actions: Vec<Action>) -> ActionLog {
        let commit_records = actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let mut record = commit_record(
                    action.id.0,
                    ((index + 1) * 4) as u64,
                    1,
                    action.committed_at.expect("test action committed"),
                );
                if action.command == ActionCommand::UndoLast {
                    record.boundary.kind = CommitBoundary::Immediate;
                }
                record
            })
            .collect();

        ActionLog {
            actions,
            commit_records,
            replay_policy: ReplayPolicy::DeterministicPreferred,
        }
    }

    fn snapshot(snapshot_id: &str, action_cursor: usize) -> Snapshot {
        Snapshot {
            snapshot_id: SnapshotId::from(snapshot_id),
            created_at: "2026-04-29T22:20:00Z".into(),
            label: "test snapshot".into(),
            action_cursor,
            payload: None,
        }
    }

    fn snapshot_with_session_payload(
        snapshot_id: &str,
        action_cursor: usize,
        session: &SessionFile,
    ) -> Snapshot {
        let snapshot_id = SnapshotId::from(snapshot_id);
        Snapshot {
            snapshot_id: snapshot_id.clone(),
            created_at: "2026-04-29T22:20:00Z".into(),
            label: "test snapshot".into(),
            action_cursor,
            payload: Some(SnapshotPayload {
                payload_version: SnapshotPayloadVersion::V1,
                snapshot_id,
                action_cursor,
                runtime_state: session.runtime_state.clone(),
            }),
        }
    }

    fn session_with_log(action_log: ActionLog, snapshots: Vec<Snapshot>) -> SessionFile {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T22:20:00Z");
        session.action_log = action_log;
        session.snapshots = snapshots;
        session
    }

    #[test]
    fn target_suffix_execution_is_noop_when_anchor_matches_target() {
        let action_log = action_log(vec![action(
            1,
            ActionCommand::TransportPlay,
            ActionParams::Empty,
            100,
        )]);
        let mut session = session_with_log(action_log, vec![snapshot("snap-after-play", 1)]);
        session.runtime_state.transport.is_playing = true;
        let original_session = session.clone();

        let report = apply_replay_target_suffix_to_session(&mut session, 1, None)
            .expect("exact anchor suffix succeeds");

        assert_eq!(
            report,
            ReplayTargetExecutionReport {
                target_action_cursor: 1,
                anchor_snapshot_id: Some("snap-after-play".into()),
                anchor_action_cursor: Some(1),
                applied_action_ids: Vec::new(),
            }
        );
        assert_eq!(session, original_session);
    }

    #[test]
    fn target_suffix_execution_applies_supported_suffix_to_hydrated_anchor_state() {
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
        ]);
        let mut session = session_with_log(action_log, vec![snapshot("snap-after-play", 1)]);
        session.runtime_state.transport.is_playing = true;

        let report = apply_replay_target_suffix_to_session(&mut session, 2, None)
            .expect("supported suffix succeeds");

        assert_eq!(
            report.anchor_snapshot_id.as_deref(),
            Some("snap-after-play")
        );
        assert_eq!(report.anchor_action_cursor, Some(1));
        assert_eq!(report.applied_action_ids, vec![ActionId(2)]);
        assert!(session.runtime_state.transport.is_playing);
        assert_eq!(session.runtime_state.transport.position_beats, 16.0);
    }

    #[test]
    fn target_suffix_execution_rejects_unsupported_suffix_without_mutating_session() {
        let action_log = action_log(vec![
            action(1, ActionCommand::TransportPlay, ActionParams::Empty, 100),
            action(2, ActionCommand::MutateLane, ActionParams::Empty, 200),
        ]);
        let mut session = session_with_log(action_log, vec![snapshot("snap-after-play", 1)]);
        session.runtime_state.transport.is_playing = true;
        let original_session = session.clone();

        let error = apply_replay_target_suffix_to_session(&mut session, 2, None)
            .expect_err("unsupported suffix should reject");

        assert_eq!(
            error,
            ReplayTargetExecutionError::Execution(ReplayExecutionError::UnsupportedAction {
                action_id: ActionId(2),
                command: ActionCommand::MutateLane,
            })
        );
        assert_eq!(session, original_session);
    }

    #[test]
    fn snapshot_payload_hydration_clones_anchor_state_and_applies_suffix() {
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
        ]);
        let mut latest_session = session_with_log(action_log.clone(), Vec::new());
        apply_replay_target_suffix_to_session(&mut latest_session, 2, None)
            .expect("latest runtime state materializes");

        let mut anchor_session = session_with_log(action_log.clone(), Vec::new());
        apply_replay_target_suffix_to_session(&mut anchor_session, 1, None)
            .expect("anchor runtime state materializes");
        let snapshot = snapshot_with_session_payload("snap-after-play", 1, &anchor_session);
        let mut session = latest_session.clone();
        session.snapshots = vec![snapshot];
        let original_session = session.clone();

        let report = hydrate_replay_target_from_snapshot_payload(&session, 2, None)
            .expect("snapshot payload hydration succeeds");

        assert_eq!(
            report.replay_report.anchor_snapshot_id.as_deref(),
            Some("snap-after-play")
        );
        assert_eq!(report.replay_report.anchor_action_cursor, Some(1));
        assert_eq!(report.replay_report.applied_action_ids, vec![ActionId(2)]);
        assert_eq!(report.session.runtime_state, latest_session.runtime_state);
        assert_eq!(session, original_session);
    }

    #[test]
    fn snapshot_payload_hydration_rejects_missing_payload() {
        let action_log = action_log(vec![action(
            1,
            ActionCommand::TransportPlay,
            ActionParams::Empty,
            100,
        )]);
        let session = session_with_log(action_log, vec![snapshot("snap-after-play", 1)]);

        let error = hydrate_replay_target_from_snapshot_payload(&session, 1, None)
            .expect_err("missing payload should reject");

        assert_eq!(
            error,
            SnapshotPayloadHydrationError::MissingSnapshotPayload {
                snapshot_id: "snap-after-play".into()
            }
        );
    }

    #[test]
    fn snapshot_payload_hydration_rejects_payload_cursor_mismatch() {
        let action_log = action_log(vec![action(
            1,
            ActionCommand::TransportPlay,
            ActionParams::Empty,
            100,
        )]);
        let anchor_session = session_with_log(action_log.clone(), Vec::new());
        let mut snapshot = snapshot_with_session_payload("snap-after-play", 1, &anchor_session);
        snapshot
            .payload
            .as_mut()
            .expect("test payload exists")
            .action_cursor = 0;
        let session = session_with_log(action_log, vec![snapshot]);

        let error = hydrate_replay_target_from_snapshot_payload(&session, 1, None)
            .expect_err("cursor mismatch should reject");

        assert_eq!(
            error,
            SnapshotPayloadHydrationError::PayloadActionCursorMismatch {
                snapshot_id: "snap-after-play".into(),
                snapshot_action_cursor: 1,
                payload_action_cursor: 0,
            }
        );
    }

    #[test]
    fn snapshot_payload_hydration_does_not_resurrect_state_from_before_undo() {
        let mut monitor_action = action(
            1,
            ActionCommand::SourceMonitorSetMode,
            ActionParams::SourceMonitor {
                mode: Some(SourceMonitorMode::Blend),
            },
            100,
        );
        monitor_action.status = ActionStatus::Undone;
        let undo_marker = action(2, ActionCommand::UndoLast, ActionParams::Empty, 200);
        let action_log = ActionLog {
            actions: vec![monitor_action, undo_marker],
            commit_records: vec![commit_record(1, 4, 1, 100)],
            replay_policy: crate::session::ReplayPolicy::DeterministicPreferred,
        };
        let mut stale_anchor_session = session_with_log(action_log.clone(), Vec::new());
        stale_anchor_session.runtime_state.source_monitor.mode = SourceMonitorMode::Blend;
        let stale_snapshot =
            snapshot_with_session_payload("snap-before-monitor-undo", 1, &stale_anchor_session);
        let session = session_with_log(action_log, vec![stale_snapshot]);

        let error = hydrate_replay_target_from_snapshot_payload(&session, 2, None)
            .expect_err("stale pre-undo payload must not be hydrated");

        assert_eq!(
            error,
            SnapshotPayloadHydrationError::MissingSnapshotAnchor {
                target_action_cursor: 2,
            }
        );
    }

    #[test]
    fn snapshot_payload_hydration_accepts_post_undo_anchor_and_replays_tail() {
        let mut monitor_action = action(
            1,
            ActionCommand::SourceMonitorSetMode,
            ActionParams::SourceMonitor {
                mode: Some(SourceMonitorMode::Blend),
            },
            100,
        );
        monitor_action.status = ActionStatus::Undone;
        let mut undo_marker = action(
            2,
            ActionCommand::UndoLast,
            ActionParams::Undo {
                target_action_id: ActionId(1),
            },
            200,
        );
        undo_marker.quantization = Quantization::Immediate;
        undo_marker.undo_policy = UndoPolicy::NotUndoable {
            reason: "undo marker".into(),
        };
        let tail = action(3, ActionCommand::TransportPlay, ActionParams::Empty, 300);
        let action_log = action_log(vec![monitor_action, undo_marker, tail]);
        let mut anchor_session = session_with_log(action_log.clone(), Vec::new());
        anchor_session.runtime_state.source_monitor.mode = SourceMonitorMode::Source;
        anchor_session.runtime_state.transport.is_playing = false;
        let post_undo_snapshot =
            snapshot_with_session_payload("snap-after-monitor-undo", 2, &anchor_session);
        let session = session_with_log(action_log, vec![post_undo_snapshot]);

        let hydrated = hydrate_replay_target_from_snapshot_payload(&session, 3, None)
            .expect("post-undo payload and tail should hydrate");

        assert_eq!(
            hydrated.replay_report.anchor_snapshot_id.as_deref(),
            Some("snap-after-monitor-undo")
        );
        assert_eq!(hydrated.replay_report.applied_action_ids, vec![ActionId(3)]);
        assert_eq!(
            hydrated.session.runtime_state.source_monitor.mode,
            SourceMonitorMode::Source
        );
        assert!(hydrated.session.runtime_state.transport.is_playing);
    }
}
