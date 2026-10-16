//! Best-effort metadata construction, deliberately not validated target replay.
use std::collections::BTreeMap;

use riotbox_core::{
    action::{ActionCommand, ActionStatus},
    replay::{ReplayPlanEntry, apply_replay_entry_with_mc202_legacy_resolver},
    session::SessionFile,
};

use super::{
    JamAppState, derive_legacy_mc202_source_phrase_plan, mc202_source_phrase_plan_is_still_trusted,
};
use crate::jam_app::helpers::is_mc202_phrase_action;

impl JamAppState {
    pub(in crate::jam_app) fn reconstruct_mc202_source_phrase_plan_for_cursor(
        &mut self,
        target_action_cursor: usize,
    ) {
        let entries = mc202_reconstruction_entries(&self.session, target_action_cursor);
        if !entries
            .iter()
            .any(|entry| is_mc202_phrase_action(entry.action.command))
        {
            // No phrase history: retain opaque caller-supplied runtime metadata.
            return;
        }
        let mut candidate = self.session.clone();
        candidate.runtime_state.lane_state.mc202.source_phrase_plan = None;
        candidate.runtime_state.lane_state.mc202.role = None;
        candidate.runtime_state.lane_state.mc202.phrase_ref = None;
        candidate.runtime_state.lane_state.mc202.phrase_variant = None;
        candidate.runtime_state.macro_state.mc202_touch = 0.0;
        let graph = self.source_graph.as_ref();
        for entry in entries {
            if apply_replay_entry_with_mc202_legacy_resolver(
                &mut candidate,
                &entry,
                |state, entry| derive_legacy_mc202_source_phrase_plan(state, entry, graph),
            )
            .is_err()
            {
                if is_mc202_phrase_action(entry.action.command) {
                    // from_parts historically accepts persisted metadata even
                    // for an unvalidated phrase action. This is not admission.
                    candidate.runtime_state.lane_state.mc202.source_phrase_plan =
                        entry.commit_record.mc202_source_phrase_plan.clone();
                } else {
                    // Malformed timing cannot establish chronological trust.
                    candidate.runtime_state.lane_state.mc202.source_phrase_plan = None;
                    break;
                }
            }
        }
        self.session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan = candidate
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .filter(|plan| mc202_source_phrase_plan_is_still_trusted(&self.session, plan));
    }
}

pub(super) fn mc202_reconstruction_entries(
    session: &SessionFile,
    cursor: usize,
) -> Vec<ReplayPlanEntry<'_>> {
    let prefix = &session.action_log.actions[..cursor.min(session.action_log.actions.len())];
    let mut first_action_by_id = BTreeMap::new();
    for action in prefix {
        // Keep the original scan's first identity, even when it masks a later
        // eligible duplicate. Never validate or reorder this unvalidated input.
        first_action_by_id.entry(action.id).or_insert(action);
    }
    session
        .action_log
        .commit_records
        .iter()
        .filter_map(|record| {
            let action = first_action_by_id.get(&record.action_id).copied()?;
            (action.status == ActionStatus::Committed
                && action.result.as_ref().is_none_or(|result| result.accepted)
                && (is_mc202_phrase_action(action.command)
                    || matches!(
                        action.command,
                        ActionCommand::SourceTimingConfirmGrid
                            | ActionCommand::SourceTimingRevertGrid
                    )))
            .then_some(ReplayPlanEntry {
                action,
                commit_record: record,
            })
        })
        .collect()
}
