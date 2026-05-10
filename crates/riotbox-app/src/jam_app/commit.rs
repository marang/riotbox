use riotbox_core::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
        Quantization, TargetScope,
    },
    ids::ActionId,
    queue::CommittedActionRef,
    session::{ActionCommitRecord, CaptureRef, Mc202UndoSnapshotState, SessionFile},
    source_graph::SourceGraph,
    transport::CommitBoundaryState,
};

use super::{
    JamAppState, apply_capture_promotion_side_effects, apply_ghost_side_effects,
    apply_mc202_side_effects, apply_scene_side_effects, apply_tr909_side_effects,
    apply_w30_side_effects, capture_promotion_summary, capture_ref_from_action,
    is_mc202_phrase_action, max_action_id, next_action_id_from_session,
};
use crate::jam_app::helpers::append_capture_note;

impl JamAppState {
    pub fn undo_last_action(&mut self, requested_at: TimestampMs) -> Option<Action> {
        let next_undo_action_id = next_action_id_from_session(&self.session);

        let undone_index = self.session.action_log.actions.iter().rposition(|action| {
            action.status == ActionStatus::Committed
                && matches!(
                    action.undo_policy,
                    riotbox_core::action::UndoPolicy::Undoable
                )
        })?;

        let undone_action_id = self.session.action_log.actions[undone_index].id;
        let undone_command = self.session.action_log.actions[undone_index].command;
        let is_mc202_undo = is_mc202_phrase_action(undone_command);
        let mc202_restored = if is_mc202_undo {
            self.restore_mc202_undo_snapshot(undone_action_id)
        } else {
            false
        };
        if is_mc202_undo && !mc202_restored {
            return None;
        }

        let undo_summary = if mc202_restored {
            format!("undone by user at {requested_at}; restored MC-202 lane state")
        } else {
            format!("undone by user at {requested_at}")
        };

        {
            let undone = &mut self.session.action_log.actions[undone_index];
            undone.status = ActionStatus::Undone;
            undone.result = Some(ActionResult {
                accepted: true,
                summary: undo_summary,
            });
        }

        let undo_action = Action {
            id: next_undo_action_id,
            actor: ActorType::User,
            command: ActionCommand::UndoLast,
            params: ActionParams::Empty,
            target: ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
            requested_at,
            quantization: Quantization::Immediate,
            status: ActionStatus::Committed,
            committed_at: Some(requested_at),
            result: Some(ActionResult {
                accepted: true,
                summary: "undid most recent undoable action".into(),
            }),
            undo_policy: riotbox_core::action::UndoPolicy::NotUndoable {
                reason: "undo marker actions are not themselves undoable".into(),
            },
            explanation: Some("undo most recent committed action".into()),
        };

        self.session.action_log.actions.push(undo_action.clone());
        self.queue
            .reserve_action_ids_after(max_action_id(&self.session));
        self.refresh_view();
        Some(undo_action)
    }

    fn restore_mc202_undo_snapshot(&mut self, action_id: ActionId) -> bool {
        let Some(snapshot_index) = self
            .session
            .runtime_state
            .undo_state
            .mc202_snapshots
            .iter()
            .rposition(|snapshot| snapshot.action_id == action_id)
        else {
            return false;
        };
        let snapshot = self
            .session
            .runtime_state
            .undo_state
            .mc202_snapshots
            .remove(snapshot_index);
        snapshot.apply_to_session(&mut self.session);
        true
    }

    pub fn commit_ready_actions(
        &mut self,
        boundary: CommitBoundaryState,
        committed_at: TimestampMs,
    ) -> Vec<CommittedActionRef> {
        let committed = self
            .queue
            .commit_ready_for_transport(boundary.clone(), committed_at);

        for committed_ref in &committed {
            if let Some(action) = self.queue.history_action(committed_ref.action_id) {
                let action =
                    self.record_committed_action(action.clone(), committed_ref, committed_at);
                self.apply_committed_action_pipeline(&action, &boundary);
            }
        }

        self.runtime.last_commit_boundary = Some(boundary);
        self.refresh_view();
        committed
    }

    fn record_committed_action(
        &mut self,
        action: Action,
        committed_ref: &CommittedActionRef,
        committed_at: TimestampMs,
    ) -> Action {
        self.session.action_log.actions.push(action.clone());
        self.session
            .action_log
            .commit_records
            .push(ActionCommitRecord {
                action_id: committed_ref.action_id,
                boundary: committed_ref.boundary.clone(),
                commit_sequence: committed_ref.commit_sequence,
                committed_at,
            });
        action
    }

    fn apply_committed_action_pipeline(&mut self, action: &Action, boundary: &CommitBoundaryState) {
        self.snapshot_undo_state_before_side_effects(action);
        self.materialize_capture_before_lane_side_effects(action, boundary);
        self.apply_lane_scene_and_ghost_side_effects(action, boundary);
        self.mirror_scene_commit_to_runtime_transport(action);
    }

    fn snapshot_undo_state_before_side_effects(&mut self, action: &Action) {
        if is_mc202_phrase_action(action.command) {
            self.session.runtime_state.undo_state.mc202_snapshots.push(
                Mc202UndoSnapshotState::from_session(action.id, &self.session),
            );
        }
    }

    fn materialize_capture_before_lane_side_effects(
        &mut self,
        action: &Action,
        boundary: &CommitBoundaryState,
    ) {
        if let Some(mut capture) =
            capture_ref_from_action(&self.session, self.source_graph.as_ref(), action, boundary)
        {
            if matches!(action.command, ActionCommand::PromoteResample) {
                self.persist_w30_bus_print_artifact(&mut capture);
                if let Some(summary) =
                    feral_resample_policy_summary(action, &capture, self.source_graph.as_ref())
                {
                    append_capture_note(&mut capture, &summary);
                    update_logged_action_result(&mut self.session, action.id, summary);
                }
            } else {
                self.persist_capture_audio_artifact(&mut capture);
            }
            self.session.runtime_state.lane_state.w30.last_capture =
                Some(capture.capture_id.clone());
            self.session.captures.push(capture);
        } else if apply_capture_promotion_side_effects(&mut self.session, action) {
            let result_summary = capture_promotion_summary(&self.session, action)
                .unwrap_or_else(|| "promotion committed".into());
            if let Some(logged_action) = self
                .session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary: result_summary,
                });
            }
        }
    }

    fn apply_lane_scene_and_ghost_side_effects(
        &mut self,
        action: &Action,
        boundary: &CommitBoundaryState,
    ) {
        apply_w30_side_effects(&mut self.session, action, Some(boundary));
        apply_mc202_side_effects(&mut self.session, action, Some(boundary));
        apply_tr909_side_effects(&mut self.session, action, Some(boundary));
        apply_scene_side_effects(
            &mut self.session,
            action,
            Some(boundary),
            self.source_graph.as_ref(),
        );
        apply_ghost_side_effects(&mut self.session, action);
    }

    fn mirror_scene_commit_to_runtime_transport(&mut self, action: &Action) {
        if matches!(
            action.command,
            ActionCommand::SceneLaunch | ActionCommand::SceneRestore
        ) {
            self.runtime.transport.current_scene =
                self.session.runtime_state.transport.current_scene.clone();
        }
    }
}

fn feral_resample_policy_summary(
    action: &Action,
    capture: &CaptureRef,
    source_graph: Option<&SourceGraph>,
) -> Option<String> {
    let source_graph = source_graph?;
    if !matches!(action.command, ActionCommand::PromoteResample)
        || capture.capture_type != riotbox_core::session::CaptureType::Resample
        || capture.lineage_capture_refs.is_empty()
        || !source_graph.has_feral_break_support_evidence()
    {
        return None;
    }

    let quote_risk_count = source_graph
        .relationships
        .iter()
        .filter(|relationship| {
            relationship.relation_type
                == riotbox_core::source_graph::RelationshipType::HighQuoteRiskWith
        })
        .count();
    if quote_risk_count > 0 {
        return Some(format!(
            "feral rebake held: quote risk {}, lineage-safe W-30 reuse, gen {}, lineage {}",
            quote_risk_count,
            capture.resample_generation_depth,
            capture.lineage_capture_refs.len()
        ));
    }

    Some(format!(
        "feral rebake approved: lineage-safe W-30 reuse, gen {}, lineage {}",
        capture.resample_generation_depth,
        capture.lineage_capture_refs.len()
    ))
}

fn update_logged_action_result(
    session: &mut SessionFile,
    action_id: ActionId,
    summary: impl Into<String>,
) {
    if let Some(logged_action) = session
        .action_log
        .actions
        .iter_mut()
        .rev()
        .find(|logged_action| logged_action.id == action_id)
    {
        logged_action.result = Some(ActionResult {
            accepted: true,
            summary: summary.into(),
        });
    }
}
