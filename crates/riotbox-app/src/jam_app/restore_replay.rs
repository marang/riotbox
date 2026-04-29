use riotbox_core::replay::{
    ReplayTargetExecutionError, ReplayTargetExecutionReport, apply_replay_target_suffix_to_session,
};

use super::lifecycle::latest_commit_boundary_from_log;
use super::*;

impl JamAppState {
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
