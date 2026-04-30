use crate::{
    action::ActionCommand,
    ids::ActionId,
    replay::{
        ReplayPlanEntry, ReplayPlanError, ReplayTargetPlan, build_replay_target_plan,
        replay_supported_action_commands,
    },
    session::{ActionLog, Snapshot},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayTargetDryRunSummary {
    pub target_action_cursor: usize,
    pub origin_action_count: usize,
    pub suffix_action_count: usize,
    pub needs_replay: bool,
    pub anchor_snapshot_id: Option<String>,
    pub anchor_action_cursor: Option<usize>,
    pub suffix_action_ids: Vec<ActionId>,
    pub suffix_commands: Vec<ActionCommand>,
    pub origin_unsupported_action_count: usize,
    pub origin_unsupported_action_ids: Vec<ActionId>,
    pub origin_unsupported_commands: Vec<ActionCommand>,
    pub suffix_unsupported_action_count: usize,
    pub suffix_unsupported_action_ids: Vec<ActionId>,
    pub suffix_unsupported_commands: Vec<ActionCommand>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnapshotPayloadReadiness {
    NoAnchor,
    Missing,
    Ready,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LatestSnapshotReplayConvergenceSummary {
    /// End-of-log cursor targeted by the latest-snapshot convergence check.
    pub target_action_cursor: usize,
    /// Total persisted actions, including non-committed diagnostic or queued entries.
    pub origin_action_count: usize,
    /// Committed replay entries that form the deterministic origin plan.
    pub origin_replay_entry_count: usize,
    /// Persisted snapshots considered while selecting the latest valid anchor.
    pub snapshot_count: usize,
    pub anchor_snapshot_id: Option<String>,
    pub anchor_action_cursor: Option<usize>,
    pub anchor_payload_readiness: SnapshotPayloadReadiness,
    /// Committed entries that must replay after the selected snapshot anchor.
    pub suffix_action_count: usize,
    pub needs_replay: bool,
    /// True when no snapshot anchor exists and replay must start from origin.
    pub needs_full_replay: bool,
    pub suffix_action_ids: Vec<ActionId>,
    pub suffix_commands: Vec<ActionCommand>,
    /// Unsupported committed entries in the full origin plan.
    pub origin_unsupported_action_count: usize,
    pub origin_unsupported_action_ids: Vec<ActionId>,
    pub origin_unsupported_commands: Vec<ActionCommand>,
    /// Unsupported committed entries that would be replayed after the selected anchor.
    pub suffix_unsupported_action_count: usize,
    pub suffix_unsupported_action_ids: Vec<ActionId>,
    pub suffix_unsupported_commands: Vec<ActionCommand>,
}

#[must_use]
pub fn build_replay_target_dry_run_summary(
    plan: &ReplayTargetPlan<'_>,
) -> ReplayTargetDryRunSummary {
    let origin_unsupported = unsupported_replay_entries(&plan.origin);
    let suffix_unsupported = unsupported_replay_entries(&plan.suffix);

    ReplayTargetDryRunSummary {
        target_action_cursor: plan.target_action_cursor,
        origin_action_count: plan.origin.len(),
        suffix_action_count: plan.suffix.len(),
        needs_replay: !plan.suffix.is_empty(),
        anchor_snapshot_id: plan
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str().to_owned()),
        anchor_action_cursor: plan.anchor.map(|snapshot| snapshot.action_cursor),
        suffix_action_ids: plan.suffix.iter().map(|entry| entry.action.id).collect(),
        suffix_commands: plan
            .suffix
            .iter()
            .map(|entry| entry.action.command)
            .collect(),
        origin_unsupported_action_count: origin_unsupported.len(),
        origin_unsupported_action_ids: origin_unsupported
            .iter()
            .map(|entry| entry.action.id)
            .collect(),
        origin_unsupported_commands: origin_unsupported
            .iter()
            .map(|entry| entry.action.command)
            .collect(),
        suffix_unsupported_action_count: suffix_unsupported.len(),
        suffix_unsupported_action_ids: suffix_unsupported
            .iter()
            .map(|entry| entry.action.id)
            .collect(),
        suffix_unsupported_commands: suffix_unsupported
            .iter()
            .map(|entry| entry.action.command)
            .collect(),
    }
}

pub fn build_latest_snapshot_replay_convergence_summary(
    action_log: &ActionLog,
    snapshots: &[Snapshot],
) -> Result<LatestSnapshotReplayConvergenceSummary, ReplayPlanError> {
    let target_action_cursor = action_log.actions.len();
    let plan = build_replay_target_plan(action_log, snapshots, target_action_cursor)?;
    let dry_run_summary = build_replay_target_dry_run_summary(&plan);

    Ok(LatestSnapshotReplayConvergenceSummary {
        target_action_cursor,
        origin_action_count: action_log.actions.len(),
        origin_replay_entry_count: plan.origin.len(),
        snapshot_count: snapshots.len(),
        anchor_snapshot_id: dry_run_summary.anchor_snapshot_id,
        anchor_action_cursor: dry_run_summary.anchor_action_cursor,
        anchor_payload_readiness: anchor_payload_readiness(&plan),
        suffix_action_count: dry_run_summary.suffix_action_count,
        needs_replay: dry_run_summary.needs_replay,
        needs_full_replay: plan.anchor.is_none() && !plan.origin.is_empty(),
        suffix_action_ids: dry_run_summary.suffix_action_ids,
        suffix_commands: dry_run_summary.suffix_commands,
        origin_unsupported_action_count: dry_run_summary.origin_unsupported_action_count,
        origin_unsupported_action_ids: dry_run_summary.origin_unsupported_action_ids,
        origin_unsupported_commands: dry_run_summary.origin_unsupported_commands,
        suffix_unsupported_action_count: dry_run_summary.suffix_unsupported_action_count,
        suffix_unsupported_action_ids: dry_run_summary.suffix_unsupported_action_ids,
        suffix_unsupported_commands: dry_run_summary.suffix_unsupported_commands,
    })
}

fn anchor_payload_readiness(plan: &ReplayTargetPlan<'_>) -> SnapshotPayloadReadiness {
    let Some(anchor) = plan.anchor else {
        return SnapshotPayloadReadiness::NoAnchor;
    };

    let Some(payload) = anchor.payload.as_ref() else {
        return SnapshotPayloadReadiness::Missing;
    };

    if payload.snapshot_id == anchor.snapshot_id && payload.action_cursor == anchor.action_cursor {
        SnapshotPayloadReadiness::Ready
    } else {
        SnapshotPayloadReadiness::Invalid
    }
}

fn unsupported_replay_entries<'a>(entries: &[ReplayPlanEntry<'a>]) -> Vec<ReplayPlanEntry<'a>> {
    let supported = replay_supported_action_commands();
    entries
        .iter()
        .filter(|entry| !supported.contains(&entry.action.command))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        TimestampMs,
        action::{
            Action, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
            CommitBoundary, Quantization, UndoPolicy,
        },
        ids::{SceneId, SnapshotId},
        replay::build_replay_target_plan,
        session::{
            ActionCommitRecord, ActionLog, ReplayPolicy, RuntimeState, Snapshot, SnapshotPayload,
        },
        transport::CommitBoundaryState,
    };

    fn action(id: u64, command: ActionCommand, committed_at: TimestampMs) -> Action {
        Action {
            id: ActionId(id),
            actor: ActorType::User,
            command,
            params: ActionParams::Empty,
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
        bar_index: u64,
        commit_sequence: u32,
        committed_at: TimestampMs,
    ) -> ActionCommitRecord {
        ActionCommitRecord {
            action_id: ActionId(action_id),
            boundary: CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index,
                bar_index,
                phrase_index: bar_index / 4,
                scene_id: Some(SceneId::from("scene-1")),
            },
            commit_sequence,
            committed_at,
        }
    }

    fn snapshot(snapshot_id: &str, action_cursor: usize) -> Snapshot {
        Snapshot {
            snapshot_id: SnapshotId::from(snapshot_id),
            created_at: "2026-04-29T19:30:00Z".into(),
            label: "test snapshot".into(),
            action_cursor,
            payload: None,
        }
    }

    fn snapshot_with_payload(snapshot_id: &str, action_cursor: usize) -> Snapshot {
        let mut snapshot = snapshot(snapshot_id, action_cursor);
        snapshot.payload = Some(SnapshotPayload::from_runtime_state(
            &snapshot.snapshot_id,
            snapshot.action_cursor,
            &RuntimeState::default(),
        ));
        snapshot
    }

    fn action_log() -> ActionLog {
        ActionLog {
            actions: vec![
                action(1, ActionCommand::MutateScene, 200),
                action(2, ActionCommand::Tr909FillNext, 210),
                action(3, ActionCommand::Mc202GenerateAnswer, 300),
            ],
            commit_records: vec![
                commit_record(3, 12, 3, 1, 300),
                commit_record(2, 8, 2, 2, 210),
                commit_record(1, 8, 2, 1, 200),
            ],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        }
    }

    #[test]
    fn dry_run_summary_reports_unanchored_origin_replay_scope() {
        let action_log = action_log();
        let plan = build_replay_target_plan(&action_log, &[], 2).expect("valid target plan");

        let summary = build_replay_target_dry_run_summary(&plan);

        assert_eq!(summary.target_action_cursor, 2);
        assert_eq!(summary.origin_action_count, 3);
        assert_eq!(summary.suffix_action_count, 2);
        assert!(summary.needs_replay);
        assert_eq!(summary.anchor_snapshot_id, None);
        assert_eq!(summary.anchor_action_cursor, None);
        assert_eq!(summary.suffix_action_ids, vec![ActionId(1), ActionId(2)]);
        assert_eq!(
            summary.suffix_commands,
            vec![ActionCommand::MutateScene, ActionCommand::Tr909FillNext]
        );
    }

    #[test]
    fn dry_run_summary_reports_exact_snapshot_anchor_without_replay_scope() {
        let action_log = action_log();
        let snapshots = vec![snapshot("snap-2", 2)];
        let plan = build_replay_target_plan(&action_log, &snapshots, 2).expect("valid target plan");

        let summary = build_replay_target_dry_run_summary(&plan);

        assert_eq!(summary.target_action_cursor, 2);
        assert_eq!(summary.origin_action_count, 3);
        assert_eq!(summary.suffix_action_count, 0);
        assert!(!summary.needs_replay);
        assert_eq!(summary.anchor_snapshot_id.as_deref(), Some("snap-2"));
        assert_eq!(summary.anchor_action_cursor, Some(2));
        assert!(summary.suffix_action_ids.is_empty());
        assert!(summary.suffix_commands.is_empty());
    }

    #[test]
    fn latest_snapshot_convergence_summary_reports_full_replay_when_no_snapshot_exists() {
        let action_log = action_log();

        let summary = build_latest_snapshot_replay_convergence_summary(&action_log, &[])
            .expect("valid convergence summary");

        assert_eq!(summary.target_action_cursor, 3);
        assert_eq!(summary.origin_action_count, 3);
        assert_eq!(summary.origin_replay_entry_count, 3);
        assert_eq!(summary.snapshot_count, 0);
        assert_eq!(summary.anchor_snapshot_id, None);
        assert_eq!(summary.anchor_action_cursor, None);
        assert_eq!(
            summary.anchor_payload_readiness,
            SnapshotPayloadReadiness::NoAnchor
        );
        assert_eq!(summary.suffix_action_count, 3);
        assert!(summary.needs_replay);
        assert!(summary.needs_full_replay);
        assert_eq!(
            summary.suffix_action_ids,
            vec![ActionId(1), ActionId(2), ActionId(3)]
        );
        assert_eq!(
            summary.suffix_commands,
            vec![
                ActionCommand::MutateScene,
                ActionCommand::Tr909FillNext,
                ActionCommand::Mc202GenerateAnswer
            ]
        );
    }

    #[test]
    fn latest_snapshot_convergence_summary_distinguishes_log_count_from_replay_count() {
        let mut action_log = action_log();
        action_log.actions.push(Action {
            id: ActionId(4),
            actor: ActorType::User,
            command: ActionCommand::SceneLaunch,
            params: ActionParams::Empty,
            target: ActionTarget::default(),
            requested_at: 400,
            quantization: Quantization::NextBar,
            status: ActionStatus::Queued,
            committed_at: None,
            result: None,
            undo_policy: UndoPolicy::Undoable,
            explanation: None,
        });

        let summary = build_latest_snapshot_replay_convergence_summary(&action_log, &[])
            .expect("valid convergence summary");

        assert_eq!(summary.target_action_cursor, 4);
        assert_eq!(summary.origin_action_count, 4);
        assert_eq!(summary.origin_replay_entry_count, 3);
        assert_eq!(summary.suffix_action_count, 3);
    }

    #[test]
    fn latest_snapshot_convergence_summary_reports_partial_suffix_after_snapshot() {
        let action_log = action_log();
        let snapshots = vec![snapshot("snap-1", 1)];

        let summary = build_latest_snapshot_replay_convergence_summary(&action_log, &snapshots)
            .expect("valid convergence summary");

        assert_eq!(summary.target_action_cursor, 3);
        assert_eq!(summary.origin_action_count, 3);
        assert_eq!(summary.origin_replay_entry_count, 3);
        assert_eq!(summary.snapshot_count, 1);
        assert_eq!(summary.anchor_snapshot_id.as_deref(), Some("snap-1"));
        assert_eq!(summary.anchor_action_cursor, Some(1));
        assert_eq!(
            summary.anchor_payload_readiness,
            SnapshotPayloadReadiness::Missing
        );
        assert_eq!(summary.suffix_action_count, 2);
        assert!(summary.needs_replay);
        assert!(!summary.needs_full_replay);
        assert_eq!(summary.suffix_action_ids, vec![ActionId(2), ActionId(3)]);
        assert_eq!(
            summary.suffix_commands,
            vec![
                ActionCommand::Tr909FillNext,
                ActionCommand::Mc202GenerateAnswer
            ]
        );
    }

    #[test]
    fn latest_snapshot_convergence_summary_reports_payload_readiness() {
        let action_log = action_log();
        let snapshots = vec![snapshot_with_payload("snap-3", 3)];

        let summary = build_latest_snapshot_replay_convergence_summary(&action_log, &snapshots)
            .expect("valid convergence summary");

        assert_eq!(summary.anchor_snapshot_id.as_deref(), Some("snap-3"));
        assert_eq!(
            summary.anchor_payload_readiness,
            SnapshotPayloadReadiness::Ready
        );
    }

    #[test]
    fn latest_snapshot_convergence_summary_reports_invalid_payload_readiness() {
        let action_log = action_log();
        let mut invalid_snapshot = snapshot_with_payload("snap-3", 3);
        invalid_snapshot
            .payload
            .as_mut()
            .expect("payload exists")
            .action_cursor = 2;

        let summary =
            build_latest_snapshot_replay_convergence_summary(&action_log, &[invalid_snapshot])
                .expect("valid convergence summary");

        assert_eq!(
            summary.anchor_payload_readiness,
            SnapshotPayloadReadiness::Invalid
        );
    }

    #[test]
    fn latest_snapshot_convergence_summary_reports_unsupported_replay_commands() {
        let mut action_log = action_log();
        action_log
            .actions
            .push(action(4, ActionCommand::PromoteResample, 400));
        action_log
            .commit_records
            .push(commit_record(4, 16, 4, 1, 400));
        let snapshots = vec![snapshot("snap-before-artifact", 3)];

        let summary = build_latest_snapshot_replay_convergence_summary(&action_log, &snapshots)
            .expect("valid convergence summary");

        assert_eq!(summary.origin_action_count, 4);
        assert_eq!(summary.origin_replay_entry_count, 4);
        assert_eq!(summary.origin_unsupported_action_count, 2);
        assert_eq!(
            summary.origin_unsupported_action_ids,
            vec![ActionId(1), ActionId(4)]
        );
        assert_eq!(
            summary.origin_unsupported_commands,
            vec![ActionCommand::MutateScene, ActionCommand::PromoteResample]
        );
        assert_eq!(summary.suffix_action_count, 1);
        assert_eq!(summary.suffix_unsupported_action_count, 1);
        assert_eq!(summary.suffix_unsupported_action_ids, vec![ActionId(4)]);
        assert_eq!(
            summary.suffix_unsupported_commands,
            vec![ActionCommand::PromoteResample]
        );
    }

    #[test]
    fn latest_snapshot_convergence_summary_uses_latest_valid_snapshot() {
        let action_log = action_log();
        let snapshots = vec![
            snapshot("snap-1", 1),
            snapshot("snap-3a", 3),
            snapshot("snap-3b", 3),
        ];

        let summary = build_latest_snapshot_replay_convergence_summary(&action_log, &snapshots)
            .expect("valid convergence summary");

        assert_eq!(summary.target_action_cursor, 3);
        assert_eq!(summary.snapshot_count, 3);
        assert_eq!(summary.anchor_snapshot_id.as_deref(), Some("snap-3b"));
        assert_eq!(summary.anchor_action_cursor, Some(3));
        assert_eq!(summary.suffix_action_count, 0);
        assert!(!summary.needs_replay);
        assert!(!summary.needs_full_replay);
        assert!(summary.suffix_action_ids.is_empty());
        assert!(summary.suffix_commands.is_empty());
    }

    #[test]
    fn latest_snapshot_convergence_summary_rejects_out_of_bounds_snapshot() {
        let action_log = action_log();
        let snapshots = vec![snapshot("snap-invalid", 4)];

        let error = build_latest_snapshot_replay_convergence_summary(&action_log, &snapshots)
            .expect_err("invalid snapshot cursor should fail");

        assert_eq!(
            error,
            ReplayPlanError::SnapshotCursorOutOfBounds {
                action_cursor: 4,
                action_count: 3
            }
        );
    }
}
