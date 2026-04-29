use crate::{action::ActionCommand, ids::ActionId, replay::ReplayTargetPlan};

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
}

#[must_use]
pub fn build_replay_target_dry_run_summary(
    plan: &ReplayTargetPlan<'_>,
) -> ReplayTargetDryRunSummary {
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
    }
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
        session::{ActionCommitRecord, ActionLog, ReplayPolicy, Snapshot},
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
        }
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
}
