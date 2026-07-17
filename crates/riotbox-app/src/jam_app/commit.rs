use riotbox_core::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
        CommitBoundary, Quantization, TargetScope, TypedUndoStateDomain, UndoPolicy,
    },
    ids::ActionId,
    queue::CommittedActionRef,
    session::{
        ActionCommitRecord, CaptureRef, Mc202UndoSnapshotState, SourceMonitorUndoSnapshotState,
        Tr909FillUndoSnapshotState,
    },
    source_graph::SourceGraph,
    transport::CommitBoundaryState,
};

use super::{
    JamAppState, apply_capture_promotion_side_effects, apply_capture_side_effects,
    apply_ghost_side_effects, apply_mc202_side_effects, apply_preset_side_effects,
    apply_scene_side_effects, apply_source_monitor_side_effects, apply_source_timing_side_effects,
    apply_tr909_side_effects, apply_transport_side_effects, apply_w30_side_effects,
    capture_promotion_summary, capture_ref_from_action, is_mc202_phrase_action,
    update_logged_action_result,
};
use crate::jam_app::helpers::{
    action_has_typed_undo_snapshot, append_capture_note, normalize_missing_typed_undo_policies,
};

impl JamAppState {
    pub fn undo_last_action(&mut self, requested_at: TimestampMs) -> Option<Action> {
        let normalized_missing_snapshots = normalize_missing_typed_undo_policies(&mut self.session);
        let undone_index = latest_trusted_typed_undo_index(&self.session);
        let Some(undone_index) = undone_index else {
            if normalized_missing_snapshots {
                self.refresh_view();
            }
            return None;
        };

        let undone_action_id = self.session.action_log.actions[undone_index].id;
        let undo_boundary = CommitBoundaryState {
            kind: CommitBoundary::Immediate,
            beat_index: self.runtime.transport.beat_index,
            bar_index: self.runtime.transport.bar_index,
            phrase_index: self.runtime.transport.phrase_index,
            scene_id: self.runtime.transport.current_scene.clone(),
        };
        let undone_command = self.session.action_log.actions[undone_index].command;
        let is_mc202_undo = is_mc202_phrase_action(undone_command);
        let is_source_monitor_undo = undone_command == ActionCommand::SourceMonitorSetMode;
        let is_tr909_fill_undo = undone_command == ActionCommand::Tr909FillNext;
        let mc202_restored = if is_mc202_undo {
            self.restore_mc202_undo_snapshot(undone_action_id)
        } else {
            false
        };
        let source_monitor_restored = if is_source_monitor_undo {
            self.restore_source_monitor_undo_snapshot(undone_action_id)
        } else {
            false
        };
        let tr909_fill_restored = if is_tr909_fill_undo {
            self.restore_tr909_fill_undo_snapshot(undone_action_id)
        } else {
            false
        };
        if (is_mc202_undo && !mc202_restored)
            || (is_source_monitor_undo && !source_monitor_restored)
            || (is_tr909_fill_undo && !tr909_fill_restored)
        {
            return None;
        }

        let undo_summary = match (mc202_restored, source_monitor_restored, tr909_fill_restored) {
            (true, false, false) => {
                format!("undone by user at {requested_at}; restored MC-202 lane state")
            }
            (false, true, false) => {
                format!("undone by user at {requested_at}; restored source monitor mode")
            }
            (false, false, true) => {
                format!("undone by user at {requested_at}; restored TR-909 Fill state")
            }
            _ => format!("undone by user at {requested_at}"),
        };

        let next_undo_action_id = self.queue.allocate_action_id();

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
            params: ActionParams::Undo {
                target_action_id: undone_action_id,
            },
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

        let mut committed_ref = CommittedActionRef {
            action_id: undo_action.id,
            boundary: undo_boundary.clone(),
            commit_sequence: 0,
        };
        self.record_committed_action(undo_action.clone(), &mut committed_ref, requested_at);
        self.runtime.last_commit_boundary = Some(undo_boundary);
        self.refresh_view();
        Some(undo_action)
    }

    fn next_commit_sequence_for_boundary(&self, boundary: &CommitBoundaryState) -> Option<u32> {
        self.session
            .action_log
            .commit_records
            .iter()
            .filter(|record| record.boundary == *boundary)
            .map(|record| record.commit_sequence)
            .max()
            .unwrap_or(0)
            .checked_add(1)
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

    fn restore_source_monitor_undo_snapshot(&mut self, action_id: ActionId) -> bool {
        let Some(snapshot_index) = self
            .session
            .runtime_state
            .undo_state
            .source_monitor_snapshots
            .iter()
            .rposition(|snapshot| snapshot.action_id == action_id)
        else {
            return false;
        };
        let snapshot = self
            .session
            .runtime_state
            .undo_state
            .source_monitor_snapshots
            .remove(snapshot_index);
        snapshot.apply_to_session(&mut self.session);
        true
    }

    fn restore_tr909_fill_undo_snapshot(&mut self, action_id: ActionId) -> bool {
        let Some(snapshot_index) = self
            .session
            .runtime_state
            .undo_state
            .tr909_fill_snapshots
            .iter()
            .rposition(|snapshot| snapshot.action_id == action_id)
        else {
            return false;
        };
        let snapshot = self
            .session
            .runtime_state
            .undo_state
            .tr909_fill_snapshots
            .remove(snapshot_index);
        snapshot.apply_to_session(&mut self.session);
        true
    }

    pub fn commit_ready_actions(
        &mut self,
        boundary: CommitBoundaryState,
        committed_at: TimestampMs,
    ) -> Vec<CommittedActionRef> {
        let mut committed = self
            .queue
            .commit_ready_for_transport(boundary.clone(), committed_at);

        for committed_ref in &mut committed {
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

    pub(in crate::jam_app) fn record_committed_action(
        &mut self,
        action: Action,
        committed_ref: &mut CommittedActionRef,
        committed_at: TimestampMs,
    ) -> Action {
        committed_ref.commit_sequence = self
            .next_commit_sequence_for_boundary(&committed_ref.boundary)
            .expect("commit sequence exhausted for one transport boundary");
        self.session.action_log.actions.push(action.clone());
        self.session
            .action_log
            .commit_records
            .push(ActionCommitRecord {
                action_id: committed_ref.action_id,
                boundary: committed_ref.boundary.clone(),
                commit_sequence: committed_ref.commit_sequence,
                committed_at,
                mc202_source_phrase_plan: None,
            });
        action
    }

    fn apply_committed_action_pipeline(&mut self, action: &Action, boundary: &CommitBoundaryState) {
        self.snapshot_undo_state_before_side_effects(action);
        self.materialize_capture_before_lane_side_effects(action, boundary);
        self.apply_lane_scene_and_ghost_side_effects(action, boundary);
        self.discard_rejected_action_undo_snapshot(action);
        self.persist_committed_action_replay_artifacts(action);
        self.mirror_committed_transport_state(action);
    }

    fn persist_committed_action_replay_artifacts(&mut self, action: &Action) {
        if !is_mc202_phrase_action(action.command) {
            return;
        }

        let accepted = self
            .session
            .action_log
            .actions
            .iter()
            .rev()
            .find(|logged_action| logged_action.id == action.id)
            .and_then(|logged_action| logged_action.result.as_ref())
            .is_some_and(|result| result.accepted);
        if !accepted {
            return;
        }

        if let Some(commit_record) = self
            .session
            .action_log
            .commit_records
            .iter_mut()
            .rev()
            .find(|record| record.action_id == action.id)
        {
            commit_record.mc202_source_phrase_plan = self
                .session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan
                .clone();
        }
    }

    fn discard_rejected_action_undo_snapshot(&mut self, action: &Action) {
        let rejected = self
            .session
            .action_log
            .actions
            .iter()
            .rev()
            .find(|logged_action| logged_action.id == action.id)
            .and_then(|logged_action| logged_action.result.as_ref())
            .is_some_and(|result| !result.accepted);
        if rejected {
            let undo_state = &mut self.session.runtime_state.undo_state;
            if is_mc202_phrase_action(action.command) {
                undo_state
                    .mc202_snapshots
                    .retain(|snapshot| snapshot.action_id != action.id);
            }
            if action.command == ActionCommand::SourceMonitorSetMode {
                undo_state
                    .source_monitor_snapshots
                    .retain(|snapshot| snapshot.action_id != action.id);
            }
            if action.command == ActionCommand::Tr909FillNext {
                undo_state
                    .tr909_fill_snapshots
                    .retain(|snapshot| snapshot.action_id != action.id);
            }
        }
    }

    fn snapshot_undo_state_before_side_effects(&mut self, action: &Action) {
        if is_mc202_phrase_action(action.command) {
            self.session.runtime_state.undo_state.mc202_snapshots.push(
                Mc202UndoSnapshotState::from_session(action.id, &self.session),
            );
        }
        if action.command == ActionCommand::SourceMonitorSetMode {
            self.session
                .runtime_state
                .undo_state
                .source_monitor_snapshots
                .push(SourceMonitorUndoSnapshotState::from_session(
                    action.id,
                    &self.session,
                ));
        }
        if action.command == ActionCommand::Tr909FillNext {
            self.session
                .runtime_state
                .undo_state
                .tr909_fill_snapshots
                .push(Tr909FillUndoSnapshotState::from_session(
                    action.id,
                    &self.session,
                ));
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
        apply_mc202_side_effects(
            &mut self.session,
            action,
            Some(boundary),
            self.source_graph.as_ref(),
        );
        apply_tr909_side_effects(&mut self.session, action, Some(boundary));
        apply_transport_side_effects(&mut self.session, action);
        apply_capture_side_effects(&mut self.session, action);
        apply_preset_side_effects(&mut self.session, action);
        apply_source_monitor_side_effects(&mut self.session, action);
        apply_source_timing_side_effects(&mut self.session, action);
        apply_scene_side_effects(
            &mut self.session,
            action,
            Some(boundary),
            self.source_graph.as_ref(),
        );
        apply_ghost_side_effects(&mut self.session, action);
    }

    fn mirror_committed_transport_state(&mut self, action: &Action) {
        match action.command {
            ActionCommand::TransportPlay
            | ActionCommand::TransportPause
            | ActionCommand::TransportStop
            | ActionCommand::TransportSeek
            | ActionCommand::SourceTimingConfirmGrid
            | ActionCommand::SourceTimingRevertGrid => {
                self.runtime.transport = super::transport_helpers::transport_clock_from_state(
                    &self.session,
                    self.source_graph.as_ref(),
                );
                self.runtime.transport_driver.last_audio_position_beats = self
                    .runtime
                    .transport
                    .is_playing
                    .then_some(self.runtime.transport.beat_index);
            }
            ActionCommand::SceneLaunch | ActionCommand::SceneRestore => {
                self.runtime.transport.current_scene =
                    self.session.runtime_state.transport.current_scene.clone();
            }
            _ => {}
        }
    }
}

fn latest_trusted_typed_undo_index(session: &riotbox_core::session::SessionFile) -> Option<usize> {
    let domains = [
        TypedUndoStateDomain::Mc202Lane,
        TypedUndoStateDomain::SourceMonitorMode,
        TypedUndoStateDomain::Tr909FillWindow,
    ];
    let mut blocked_domains = std::collections::BTreeSet::new();

    for (index, action) in session.action_log.actions.iter().enumerate().rev() {
        let committed_effect_may_exist = action.status == ActionStatus::Committed
            && !action
                .result
                .as_ref()
                .is_some_and(|result| !result.accepted);
        if let Some(domain) = action.command.typed_undo_state_domain()
            && !blocked_domains.contains(&domain)
            && matches!(action.undo_policy, UndoPolicy::Undoable)
            && action.result.as_ref().is_some_and(|result| result.accepted)
            && session
                .action_log
                .commit_records
                .iter()
                .filter(|record| record.action_id == action.id)
                .count()
                == 1
            && action_has_typed_undo_snapshot(session, action)
        {
            return Some(index);
        }

        if committed_effect_may_exist {
            blocked_domains.extend(
                domains
                    .into_iter()
                    .filter(|domain| action.command.mutates_typed_undo_state_domain(*domain)),
            );
        }
    }
    None
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
