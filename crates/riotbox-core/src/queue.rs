use std::{collections::VecDeque, error::Error, fmt};

use crate::{
    TimestampMs,
    action::{Action, ActionCommand, ActionDraft, ActionResult, ActionStatus, CommitBoundary},
    ids::ActionId,
    session::{ActionLog, CommitSequenceExhausted},
    transport::CommitBoundaryState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionIdExhausted;

impl fmt::Display for ActionIdExhausted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("action ID space exhausted")
    }
}

impl Error for ActionIdExhausted {}

#[derive(Clone, Debug)]
pub struct ActionQueue {
    next_id: Option<u64>,
    pending: VecDeque<Action>,
    history: Vec<Action>,
}

impl Default for ActionQueue {
    fn default() -> Self {
        // Preserve the original derived Default's first ID, distinct from new().
        Self {
            next_id: Some(0),
            pending: VecDeque::new(),
            history: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommittedActionRef {
    pub action_id: ActionId,
    pub boundary: CommitBoundaryState,
    pub commit_sequence: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueueEnqueueResult {
    Enqueued(ActionId),
    AlreadyPending { command: ActionCommand },
}

impl ActionQueue {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: Some(1),
            pending: VecDeque::new(),
            history: Vec::new(),
        }
    }

    pub fn enqueue(
        &mut self,
        draft: ActionDraft,
        requested_at: TimestampMs,
    ) -> Result<ActionId, ActionIdExhausted> {
        let id = self.allocate_action_id()?;

        let action = Action {
            id,
            actor: draft.actor,
            command: draft.command,
            params: draft.params,
            target: draft.target,
            requested_at,
            quantization: draft.quantization,
            status: ActionStatus::Queued,
            committed_at: None,
            result: None,
            undo_policy: draft.undo_policy,
            explanation: draft.explanation,
        };

        self.pending.push_back(action);
        Ok(id)
    }

    /// Reserve the next globally unique in-memory action id.
    ///
    /// Directly committed control actions such as `undo.last` use the same
    /// allocator as queued actions so pending/history ids cannot collide.
    /// Exhaustion never reuses an ID or mutates pending/history state.
    pub fn allocate_action_id(&mut self) -> Result<ActionId, ActionIdExhausted> {
        let next = self.next_id.ok_or(ActionIdExhausted)?;
        self.next_id = next.checked_add(1);
        Ok(ActionId(next))
    }

    pub fn enqueue_if_no_pending_command(
        &mut self,
        draft: ActionDraft,
        requested_at: TimestampMs,
    ) -> Result<QueueEnqueueResult, ActionIdExhausted> {
        let command = draft.command;
        if self.has_pending_command(command) {
            return Ok(QueueEnqueueResult::AlreadyPending { command });
        }

        self.enqueue(draft, requested_at)
            .map(QueueEnqueueResult::Enqueued)
    }

    #[must_use]
    pub fn has_pending_command(&self, command: ActionCommand) -> bool {
        self.pending.iter().any(|action| action.command == command)
    }

    #[must_use]
    pub fn pending_actions(&self) -> Vec<&Action> {
        self.pending.iter().collect()
    }

    #[must_use]
    pub fn history(&self) -> &[Action] {
        &self.history
    }

    /// Advance beyond persisted history; exhaustion is sticky, even for older history.
    pub fn reserve_action_ids_after(&mut self, last_action_id: Option<ActionId>) {
        if let Some(last_action_id) = last_action_id {
            self.next_id = self
                .next_id
                .zip(last_action_id.0.checked_add(1))
                .map(|(next, reserved)| next.max(reserved));
        }
    }

    #[must_use]
    pub fn history_action(&self, action_id: ActionId) -> Option<&Action> {
        self.history.iter().find(|action| action.id == action_id)
    }

    pub fn commit_ready(
        &mut self,
        history: &ActionLog,
        boundary: CommitBoundary,
        committed_at: TimestampMs,
    ) -> Result<Vec<ActionId>, CommitSequenceExhausted> {
        self.commit_ready_for_transport(
            history,
            CommitBoundaryState {
                kind: boundary,
                beat_index: 0,
                bar_index: 0,
                phrase_index: 0,
                scene_id: None,
            },
            committed_at,
        )
        .map(|committed| committed.into_iter().map(|item| item.action_id).collect())
    }

    #[must_use]
    pub fn ready_action_count_for_transport(&self, boundary: CommitBoundary) -> usize {
        self.pending
            .iter()
            .filter(|action| is_ready_for_transport(action, boundary))
            .count()
    }

    pub fn commit_ready_for_transport(
        &mut self,
        history: &ActionLog,
        boundary: CommitBoundaryState,
        committed_at: TimestampMs,
    ) -> Result<Vec<CommittedActionRef>, CommitSequenceExhausted> {
        let sequences = history.prepare_commit_sequences(
            &boundary,
            self.ready_action_count_for_transport(boundary.kind),
        )?;
        let mut remaining = VecDeque::with_capacity(self.pending.len());
        let mut committed = Vec::new();

        // The shared predicate counted exactly this stream before any mutation.
        // Filter the entire stream (including trailing non-ready actions), then
        // pair each ready action with one already checked absolute sequence.
        let ready = self.pending.drain(..).filter_map(|mut action| {
            if is_ready_for_transport(&action, boundary.kind) {
                Some(action)
            } else {
                action.status = ActionStatus::PendingCommit;
                remaining.push_back(action);
                None
            }
        });
        for (mut action, commit_sequence) in ready.zip(sequences) {
            action.status = ActionStatus::Committed;
            action.committed_at = Some(committed_at);
            action.result = Some(ActionResult {
                accepted: true,
                summary: format!(
                    "committed on {:?} boundary at beat {}, bar {}, phrase {}",
                    boundary.kind, boundary.beat_index, boundary.bar_index, boundary.phrase_index
                ),
            });
            committed.push(CommittedActionRef {
                action_id: action.id,
                boundary: boundary.clone(),
                commit_sequence,
            });
            self.history.push(action);
        }

        self.pending = remaining;
        Ok(committed)
    }

    pub fn commit_pending_after_side_effect(
        &mut self,
        history: &ActionLog,
        action_id: ActionId,
        boundary: CommitBoundaryState,
        committed_at: TimestampMs,
        result_summary: impl Into<String>,
    ) -> Result<Option<CommittedActionRef>, CommitSequenceExhausted> {
        let Some(index) = self.pending.iter().position(|action| {
            action.id == action_id && action.quantization.is_ready_for(boundary.kind)
        }) else {
            return Ok(None);
        };
        let mut sequences = history.prepare_commit_sequences(&boundary, 1)?;
        let result_summary = result_summary.into();

        Ok(sequences.next().and_then(|commit_sequence| {
            self.pending.remove(index).map(|mut action| {
                action.status = ActionStatus::Committed;
                action.committed_at = Some(committed_at);
                action.result = Some(ActionResult {
                    accepted: true,
                    summary: result_summary,
                });
                let committed = CommittedActionRef {
                    action_id: action.id,
                    boundary: boundary.clone(),
                    commit_sequence,
                };
                self.history.push(action);
                committed
            })
        }))
    }

    pub fn reject(&mut self, action_id: ActionId, reason: impl Into<String>) -> bool {
        let reason = reason.into();
        let mut remaining = VecDeque::with_capacity(self.pending.len());
        let mut rejected = false;

        while let Some(mut action) = self.pending.pop_front() {
            if action.id == action_id {
                action.status = ActionStatus::Rejected;
                action.result = Some(ActionResult {
                    accepted: false,
                    summary: reason.clone(),
                });
                self.history.push(action);
                rejected = true;
            } else {
                remaining.push_back(action);
            }
        }

        self.pending = remaining;
        rejected
    }
}

fn is_ready_for_transport(action: &Action, boundary: CommitBoundary) -> bool {
    !action.command.commits_only_after_side_effect() && action.quantization.is_ready_for(boundary)
}

#[cfg(test)]
mod exhaustion_tests;

#[cfg(test)]
mod commit_sequence_tests;

#[cfg(test)]
mod tests {
    use crate::{
        action::{ActionCommand, ActorType, Quantization, TargetScope},
        ids::SceneId,
        transport::{CommitBoundaryState, TransportClockState},
    };

    use super::*;

    #[test]
    fn commits_only_actions_that_match_boundary() {
        let mut queue = ActionQueue::new();

        queue
            .enqueue(
                ActionDraft::new(
                    ActorType::User,
                    ActionCommand::CaptureNow,
                    Quantization::NextBar,
                    crate::action::ActionTarget {
                        scope: Some(TargetScope::LaneW30),
                        ..Default::default()
                    },
                ),
                100,
            )
            .expect("generated queue ID available");

        queue
            .enqueue(
                ActionDraft::new(
                    ActorType::Ghost,
                    ActionCommand::GhostSetMode,
                    Quantization::Immediate,
                    crate::action::ActionTarget {
                        scope: Some(TargetScope::Ghost),
                        ..Default::default()
                    },
                ),
                101,
            )
            .expect("generated queue ID available");

        let committed = queue
            .commit_ready(&ActionLog::default(), CommitBoundary::Immediate, 110)
            .unwrap();
        assert_eq!(committed.len(), 1);
        assert_eq!(queue.pending_actions().len(), 1);

        let committed = queue
            .commit_ready(&ActionLog::default(), CommitBoundary::Bar, 200)
            .unwrap();
        assert_eq!(committed.len(), 1);
        assert_eq!(queue.pending_actions().len(), 0);
        assert_eq!(queue.history().len(), 2);
    }

    #[test]
    fn can_reject_pending_action() {
        let mut queue = ActionQueue::new();
        let id = queue
            .enqueue(
                ActionDraft::new(
                    ActorType::User,
                    ActionCommand::MutateScene,
                    Quantization::NextPhrase,
                    crate::action::ActionTarget {
                        scope: Some(TargetScope::Scene),
                        ..Default::default()
                    },
                ),
                100,
            )
            .expect("generated queue ID available");

        assert!(queue.reject(id, "scene locked"));
        assert_eq!(queue.pending_actions().len(), 0);
        assert_eq!(queue.history().len(), 1);
        assert_eq!(queue.history()[0].status, ActionStatus::Rejected);
    }

    #[test]
    fn commits_against_explicit_transport_boundary_state() {
        let mut queue = ActionQueue::new();

        queue
            .enqueue(
                ActionDraft::new(
                    ActorType::User,
                    ActionCommand::SceneLaunch,
                    Quantization::NextPhrase,
                    crate::action::ActionTarget {
                        scope: Some(TargetScope::Scene),
                        ..Default::default()
                    },
                ),
                100,
            )
            .expect("generated queue ID available");

        let clock = TransportClockState {
            is_playing: true,
            position_beats: 64.0,
            beat_index: 64,
            bar_index: 17,
            phrase_index: 3,
            current_scene: Some(SceneId::from("scene-a")),
        };

        let committed = queue
            .commit_ready_for_transport(
                &ActionLog::default(),
                clock.boundary_state(CommitBoundary::Phrase),
                200,
            )
            .unwrap();

        assert_eq!(committed.len(), 1);
        assert_eq!(committed[0].boundary.kind, CommitBoundary::Phrase);
        assert_eq!(committed[0].boundary.bar_index, 17);
        assert_eq!(committed[0].boundary.phrase_index, 3);
        assert_eq!(
            committed[0]
                .boundary
                .scene_id
                .as_ref()
                .map(ToString::to_string),
            Some("scene-a".into())
        );
    }

    #[test]
    fn preserves_stable_commit_sequence_within_boundary() {
        let mut queue = ActionQueue::new();

        let first = queue
            .enqueue(
                ActionDraft::new(
                    ActorType::User,
                    ActionCommand::CaptureNow,
                    Quantization::NextBar,
                    crate::action::ActionTarget {
                        scope: Some(TargetScope::LaneW30),
                        ..Default::default()
                    },
                ),
                100,
            )
            .expect("generated queue ID available");
        let second = queue
            .enqueue(
                ActionDraft::new(
                    ActorType::Ghost,
                    ActionCommand::MutateScene,
                    Quantization::NextBar,
                    crate::action::ActionTarget {
                        scope: Some(TargetScope::Scene),
                        ..Default::default()
                    },
                ),
                101,
            )
            .expect("generated queue ID available");

        let committed = queue
            .commit_ready_for_transport(
                &ActionLog::default(),
                CommitBoundaryState {
                    kind: CommitBoundary::Bar,
                    beat_index: 32,
                    bar_index: 9,
                    phrase_index: 2,
                    scene_id: None,
                },
                200,
            )
            .unwrap();

        assert_eq!(committed.len(), 2);
        assert_eq!(committed[0].action_id, first);
        assert_eq!(committed[0].commit_sequence, 1);
        assert_eq!(committed[1].action_id, second);
        assert_eq!(committed[1].commit_sequence, 2);
    }

    #[test]
    fn reserves_action_ids_after_existing_history() {
        let mut queue = ActionQueue::new();
        queue.reserve_action_ids_after(Some(ActionId(9)));

        let action_id = queue
            .enqueue(
                ActionDraft::new(
                    ActorType::User,
                    ActionCommand::CaptureNow,
                    Quantization::Immediate,
                    crate::action::ActionTarget {
                        scope: Some(TargetScope::LaneW30),
                        ..Default::default()
                    },
                ),
                100,
            )
            .expect("generated queue ID available");

        assert_eq!(action_id, ActionId(10));
    }

    #[test]
    fn commits_one_side_effect_action_only_after_caller_reports_success() {
        let mut queue = ActionQueue::new();

        let export_action = queue
            .enqueue(
                ActionDraft::new(
                    ActorType::User,
                    ActionCommand::ExportProductMix,
                    Quantization::Immediate,
                    crate::action::ActionTarget {
                        scope: Some(TargetScope::Session),
                        ..Default::default()
                    },
                ),
                100,
            )
            .expect("generated queue ID available");
        let scene_action = queue
            .enqueue(
                ActionDraft::new(
                    ActorType::User,
                    ActionCommand::SceneLaunch,
                    Quantization::NextBar,
                    crate::action::ActionTarget {
                        scope: Some(TargetScope::Scene),
                        ..Default::default()
                    },
                ),
                101,
            )
            .expect("generated queue ID available");

        let committed = queue
            .commit_pending_after_side_effect(
                &ActionLog::default(),
                export_action,
                CommitBoundaryState {
                    kind: CommitBoundary::Immediate,
                    beat_index: 0,
                    bar_index: 0,
                    phrase_index: 0,
                    scene_id: None,
                },
                200,
                "export wrote full_grid_mix",
            )
            .expect("generated commit sequence available")
            .expect("export action commits after side effect success");

        assert_eq!(committed.action_id, export_action);
        assert_eq!(queue.pending_actions().len(), 1);
        assert_eq!(queue.pending_actions()[0].id, scene_action);
        assert_eq!(queue.history().len(), 1);
        assert_eq!(
            queue.history()[0]
                .result
                .as_ref()
                .map(|result| result.summary.as_str()),
            Some("export wrote full_grid_mix")
        );
    }

    #[test]
    fn transport_boundaries_leave_file_side_effect_actions_pending() {
        let mut queue = ActionQueue::new();
        let export_commands = [
            ActionCommand::ExportProductMix,
            ActionCommand::ExportStemPackage,
            ActionCommand::ExportLiveRecording,
            ActionCommand::ExportDawSession,
        ];
        let export_ids = export_commands
            .into_iter()
            .map(|command| {
                queue
                    .enqueue(
                        ActionDraft::new(
                            ActorType::User,
                            command,
                            Quantization::Immediate,
                            crate::action::ActionTarget {
                                scope: Some(TargetScope::Session),
                                ..Default::default()
                            },
                        ),
                        100,
                    )
                    .expect("generated queue ID available")
            })
            .collect::<Vec<_>>();
        let scene_id = queue
            .enqueue(
                ActionDraft::new(
                    ActorType::User,
                    ActionCommand::SceneLaunch,
                    Quantization::Immediate,
                    crate::action::ActionTarget {
                        scope: Some(TargetScope::Scene),
                        ..Default::default()
                    },
                ),
                101,
            )
            .expect("generated queue ID available");

        let committed = queue
            .commit_ready_for_transport(
                &ActionLog::default(),
                CommitBoundaryState {
                    kind: CommitBoundary::Beat,
                    beat_index: 1,
                    bar_index: 1,
                    phrase_index: 1,
                    scene_id: None,
                },
                200,
            )
            .unwrap();

        assert_eq!(
            committed
                .iter()
                .map(|action| action.action_id)
                .collect::<Vec<_>>(),
            vec![scene_id]
        );
        assert_eq!(
            queue
                .pending_actions()
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            export_ids
        );
    }

    #[test]
    fn enqueue_once_blocks_duplicate_pending_stem_package_export() {
        let mut queue = ActionQueue::new();
        let draft = || {
            ActionDraft::new(
                ActorType::User,
                ActionCommand::ExportStemPackage,
                Quantization::Immediate,
                crate::action::ActionTarget {
                    scope: Some(TargetScope::Session),
                    ..Default::default()
                },
            )
        };

        let first = queue
            .enqueue_if_no_pending_command(draft(), 100)
            .expect("generated queue ID available");
        assert_eq!(first, QueueEnqueueResult::Enqueued(ActionId(1)));
        assert!(queue.has_pending_command(ActionCommand::ExportStemPackage));

        let second = queue
            .enqueue_if_no_pending_command(draft(), 101)
            .expect("generated queue ID available");
        assert_eq!(
            second,
            QueueEnqueueResult::AlreadyPending {
                command: ActionCommand::ExportStemPackage
            }
        );
        assert_eq!(queue.pending_actions().len(), 1);
    }
}
