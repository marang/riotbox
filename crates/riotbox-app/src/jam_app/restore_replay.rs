use riotbox_core::{
    replay::{
        ReplayPlanError, ReplayTargetDryRunSummary, ReplayTargetExecutionError,
        ReplayTargetExecutionReport, SnapshotPayloadHydrationError,
        apply_replay_target_suffix_with_mc202_legacy_resolver, build_replay_target_dry_run_summary,
        build_replay_target_plan,
        hydrate_replay_target_from_snapshot_payload_with_mc202_legacy_resolver,
    },
    session::Mc202SourcePhrasePlanState,
};

use super::lifecycle::latest_commit_boundary_from_log;
use super::side_effects::derive_legacy_mc202_source_phrase_plan;
use super::{JamAppState, max_action_id, transport_clock_from_state};

mod legacy_mc202;

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
        let source_graph = self.source_graph.as_ref();
        let report = apply_replay_target_suffix_with_mc202_legacy_resolver(
            &mut self.session,
            target_action_cursor,
            source_graph,
            |session, entry| derive_legacy_mc202_source_phrase_plan(session, entry, source_graph),
        )?;

        self.refresh_after_restore_replay();

        Ok(report)
    }

    pub fn apply_restore_target_from_snapshot_payload(
        &mut self,
        target_action_cursor: usize,
    ) -> Result<ReplayTargetExecutionReport, SnapshotPayloadHydrationError> {
        let source_graph = self.source_graph.as_ref();
        let hydration_report =
            hydrate_replay_target_from_snapshot_payload_with_mc202_legacy_resolver(
                &self.session,
                target_action_cursor,
                source_graph,
                |session, entry| {
                    derive_legacy_mc202_source_phrase_plan(session, entry, source_graph)
                },
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
        self.runtime.transport_driver.pending_audio_position = Some(
            riotbox_audio::runtime::AudioTransportPositionRequest::from_beats(
                self.runtime.transport.position_beats,
            ),
        );
        self.runtime.transport_driver.pending_audio_is_playing =
            Some(self.runtime.transport.is_playing);
        self.runtime.transport_driver.last_audio_position_beats = self
            .runtime
            .transport
            .is_playing
            .then_some(self.runtime.transport.beat_index);
        self.runtime.last_commit_boundary = latest_commit_boundary_from_log(&self.session);
        if self
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .as_ref()
            .is_some_and(|plan| !mc202_source_phrase_plan_is_still_trusted(&self.session, plan))
        {
            self.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan = None;
        }
        self.refresh_view();
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

#[cfg(test)]
mod tests;
