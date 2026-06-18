use riotbox_core::{
    action::Action,
    replay::{
        ReplayPlanError, ReplayTargetDryRunSummary, ReplayTargetExecutionError,
        ReplayTargetExecutionReport, SnapshotPayloadHydrationError,
        apply_replay_target_suffix_to_session, build_replay_target_dry_run_summary,
        build_replay_target_plan, hydrate_replay_target_from_snapshot_payload,
    },
    session::Mc202SourcePhrasePlanState,
    transport::CommitBoundaryState,
};

use super::helpers::is_mc202_phrase_action;
use super::lifecycle::latest_commit_boundary_from_log;
use super::side_effects::apply_mc202_side_effects;
use super::{JamAppState, max_action_id, transport_clock_from_state};

impl JamAppState {
    pub fn restore_target_dry_run_summary(
        &self,
        target_action_cursor: usize,
    ) -> Result<ReplayTargetDryRunSummary, ReplayPlanError> {
        let plan = build_replay_target_plan(
            &self.session.action_log,
            &self.session.snapshots,
            target_action_cursor,
        )?;
        Ok(build_replay_target_dry_run_summary(&plan))
    }

    pub fn apply_restore_target_suffix(
        &mut self,
        target_action_cursor: usize,
    ) -> Result<ReplayTargetExecutionReport, ReplayTargetExecutionError> {
        let report = apply_replay_target_suffix_to_session(
            &mut self.session,
            target_action_cursor,
            self.source_graph.as_ref(),
        )?;

        self.refresh_after_restore_replay(Some(target_action_cursor));

        Ok(report)
    }

    pub fn apply_restore_target_from_snapshot_payload(
        &mut self,
        target_action_cursor: usize,
    ) -> Result<ReplayTargetExecutionReport, SnapshotPayloadHydrationError> {
        let hydration_report = hydrate_replay_target_from_snapshot_payload(
            &self.session,
            target_action_cursor,
            self.source_graph.as_ref(),
        )?;
        let replay_report = hydration_report.replay_report;

        self.session = hydration_report.session;
        self.refresh_after_restore_replay(Some(target_action_cursor));

        Ok(replay_report)
    }

    fn refresh_after_restore_replay(&mut self, target_action_cursor: Option<usize>) {
        self.queue
            .reserve_action_ids_after(max_action_id(&self.session));
        self.runtime.transport =
            transport_clock_from_state(&self.session, self.source_graph.as_ref());
        self.runtime.last_commit_boundary = latest_commit_boundary_from_log(&self.session);
        self.reconstruct_mc202_source_phrase_plan_for_cursor(
            target_action_cursor.unwrap_or(self.session.action_log.actions.len()),
        );
        self.refresh_view();
    }

    pub(in crate::jam_app) fn reconstruct_mc202_source_phrase_plan_for_cursor(
        &mut self,
        target_action_cursor: usize,
    ) {
        let replay_inputs = mc202_phrase_replay_inputs(&self.session, target_action_cursor);
        if replay_inputs.is_empty() {
            return;
        }
        if let Some(persisted_plan) = replay_inputs
            .last()
            .and_then(|input| input.source_phrase_plan.clone())
        {
            if !mc202_source_phrase_plan_is_still_trusted(&self.session, &persisted_plan) {
                self.session
                    .runtime_state
                    .lane_state
                    .mc202
                    .source_phrase_plan = None;
                return;
            }
            self.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan = Some(persisted_plan);
            return;
        }

        let Some(source_graph) = self.source_graph.as_ref() else {
            self.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan = None;
            return;
        };

        let mut planning_session = self.session.clone();
        planning_session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan = None;
        planning_session.runtime_state.lane_state.mc202.role = None;
        planning_session.runtime_state.lane_state.mc202.phrase_ref = None;
        planning_session
            .runtime_state
            .lane_state
            .mc202
            .phrase_variant = None;
        planning_session.runtime_state.macro_state.mc202_touch = 0.0;

        for input in replay_inputs {
            apply_mc202_side_effects(
                &mut planning_session,
                &input.action,
                Some(&input.boundary),
                Some(source_graph),
            );
        }

        self.session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan = planning_session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan;
    }
}

fn mc202_source_phrase_plan_is_still_trusted(
    session: &riotbox_core::session::SessionFile,
    plan: &Mc202SourcePhrasePlanState,
) -> bool {
    session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .is_some_and(|confirmed| confirmed.source_id == plan.source_id)
}

struct Mc202PhraseReplayInput {
    action: Action,
    boundary: CommitBoundaryState,
    source_phrase_plan: Option<Mc202SourcePhrasePlanState>,
}

fn mc202_phrase_replay_inputs(
    session: &riotbox_core::session::SessionFile,
    target_action_cursor: usize,
) -> Vec<Mc202PhraseReplayInput> {
    let target_action_cursor = target_action_cursor.min(session.action_log.actions.len());
    let committed_actions = &session.action_log.actions[..target_action_cursor];

    session
        .action_log
        .commit_records
        .iter()
        .filter_map(|record| {
            let action = committed_actions
                .iter()
                .find(|action| action.id == record.action_id)?;
            is_mc202_phrase_action(action.command).then(|| Mc202PhraseReplayInput {
                action: action.clone(),
                boundary: record.boundary.clone(),
                source_phrase_plan: record.mc202_source_phrase_plan.clone(),
            })
        })
        .collect()
}
