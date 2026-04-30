use riotbox_core::replay::{
    ReplayPlanError, ReplayTargetDryRunSummary, ReplayTargetExecutionError,
    ReplayTargetExecutionReport, SnapshotPayloadHydrationError,
    apply_replay_target_suffix_to_session, build_replay_target_dry_run_summary,
    build_replay_target_plan, hydrate_replay_target_from_snapshot_payload,
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

        self.refresh_after_restore_replay();

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
        self.refresh_after_restore_replay();

        Ok(replay_report)
    }

    fn refresh_after_restore_replay(&mut self) {
        self.queue
            .reserve_action_ids_after(max_action_id(&self.session));
        self.runtime.transport =
            transport_clock_from_state(&self.session, self.source_graph.as_ref());
        self.runtime.last_commit_boundary = latest_commit_boundary_from_log(&self.session);
        self.refresh_view();
    }
}
