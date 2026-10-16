use riotbox_core::{
    TimestampMs,
    action::{Action, ActionCommand, ActionDraft, ActionResult, CommitBoundary},
    queue::CommittedActionRef,
    session::{
        ActionCommitRecord, ActionLog, CaptureRef, CommitSequenceExhausted, Mc202UndoSnapshotState,
        SourceMonitorUndoSnapshotState, Tr909FillUndoSnapshotState,
    },
    source_graph::SourceGraph,
    transport::CommitBoundaryState,
};

use super::side_effects::apply_committed_side_effects;
use super::{
    JamAppError, JamAppState, apply_capture_promotion_side_effects, capture_promotion_summary,
    capture_ref_from_action, is_mc202_phrase_action, update_logged_action_result,
};
use crate::jam_app::helpers::append_capture_note;

mod admission;
mod typed_undo;

// Actual and prospective histories share the same record construction owner.
fn record_committed_action_in_log(
    log: &mut ActionLog,
    action: Action,
    committed_ref: &CommittedActionRef,
    committed_at: TimestampMs,
) -> Action {
    log.actions.push(action.clone());
    log.commit_records.push(ActionCommitRecord {
        action_id: committed_ref.action_id,
        boundary: committed_ref.boundary.clone(),
        commit_sequence: committed_ref.commit_sequence,
        committed_at,
        mc202_source_phrase_plan: None,
    });
    action
}

impl JamAppState {
    pub fn commit_ready_actions(
        &mut self,
        boundary: CommitBoundaryState,
        committed_at: TimestampMs,
    ) -> Result<Vec<CommittedActionRef>, JamAppError> {
        self.preflight_ready_actions(&boundary, committed_at)?;
        let mut committed = self.queue.commit_ready_for_transport(
            &self.session.action_log,
            boundary.clone(),
            committed_at,
        )?;

        for committed_ref in &mut committed {
            if let Some(action) = self.queue.history_action(committed_ref.action_id) {
                let action =
                    self.record_committed_action(action.clone(), committed_ref, committed_at);
                self.apply_committed_action_pipeline(&action, &boundary);
            }
        }

        self.runtime.last_commit_boundary = Some(boundary);
        self.refresh_view();
        Ok(committed)
    }

    pub(in crate::jam_app) fn record_committed_action(
        &mut self,
        action: Action,
        committed_ref: &mut CommittedActionRef,
        committed_at: TimestampMs,
    ) -> Action {
        record_committed_action_in_log(
            &mut self.session.action_log,
            action,
            committed_ref,
            committed_at,
        )
    }

    pub(in crate::jam_app) fn preflight_immediate_commit(
        &self,
    ) -> Result<(), CommitSequenceExhausted> {
        let boundary = self
            .runtime
            .transport
            .boundary_state(CommitBoundary::Immediate);
        self.session
            .action_log
            .prepare_commit_sequences(&boundary, 1)?;
        Ok(())
    }

    pub(in crate::jam_app) fn preflight_immediate_enqueue(&self) -> Result<(), JamAppError> {
        let boundary = self
            .runtime
            .transport
            .boundary_state(CommitBoundary::Immediate);
        let count = self
            .queue
            .ready_action_count_for_transport(boundary.kind)
            .checked_add(1)
            .ok_or_else(|| {
                JamAppError::InvalidSession("ready action count exceeds address space".into())
            })?;
        self.session
            .action_log
            .prepare_commit_sequences(&boundary, count)?;
        Ok(())
    }

    pub(in crate::jam_app) fn preflight_ready_actions(
        &self,
        boundary: &CommitBoundaryState,
        committed_at: TimestampMs,
    ) -> Result<(), JamAppError> {
        let count = self.queue.ready_action_count_for_transport(boundary.kind);
        self.session
            .action_log
            .prepare_commit_sequences(boundary, count)?;
        admission::preflight_ready_legacy_commits(
            &self.queue,
            &self.session.action_log,
            boundary,
            committed_at,
        )
    }

    pub(in crate::jam_app) fn preflight_legacy_immediate_draft(
        &self,
        draft: &ActionDraft,
        boundary: &CommitBoundaryState,
        requested_at: TimestampMs,
    ) -> Result<(), JamAppError> {
        admission::preflight_legacy_draft(
            &self.queue,
            &self.session.action_log,
            draft,
            boundary,
            requested_at,
        )
    }

    fn apply_committed_action_pipeline(&mut self, action: &Action, boundary: &CommitBoundaryState) {
        self.snapshot_undo_state_before_side_effects(action);
        self.materialize_capture_before_lane_side_effects(action, boundary);
        apply_committed_side_effects(
            &mut self.session,
            action,
            boundary,
            self.source_graph.as_ref(),
        );
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
                if matches!(
                    action.command,
                    ActionCommand::TransportSeek | ActionCommand::TransportStop
                ) {
                    self.runtime.transport_driver.pending_audio_position = Some(
                        riotbox_audio::runtime::AudioTransportPositionRequest::from_beats(
                            self.runtime.transport.position_beats,
                        ),
                    );
                }
                if matches!(
                    action.command,
                    ActionCommand::TransportPlay
                        | ActionCommand::TransportPause
                        | ActionCommand::TransportStop
                ) {
                    self.runtime.transport_driver.pending_audio_is_playing =
                        Some(self.runtime.transport.is_playing);
                }
            }
            ActionCommand::SceneLaunch | ActionCommand::SceneRestore => {
                self.runtime.transport.current_scene =
                    self.session.runtime_state.transport.current_scene.clone();
            }
            _ => {}
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
