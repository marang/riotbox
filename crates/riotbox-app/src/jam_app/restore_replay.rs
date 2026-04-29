use riotbox_core::replay::{
    ReplayPlanError, ReplayTargetDryRunSummary, ReplayTargetExecutionError,
    ReplayTargetExecutionReport, apply_replay_target_suffix_to_session,
    build_replay_target_dry_run_summary, build_replay_target_plan,
};

use super::lifecycle::latest_commit_boundary_from_log;
use super::*;

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

        self.queue
            .reserve_action_ids_after(max_action_id(&self.session));
        self.runtime.transport =
            transport_clock_from_state(&self.session, self.source_graph.as_ref());
        self.runtime.last_commit_boundary = latest_commit_boundary_from_log(&self.session);
        self.refresh_view();

        Ok(report)
    }
}
