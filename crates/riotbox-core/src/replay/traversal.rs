//! One transactional replay walk, with optional missing legacy MC-202 plans.
use crate::{
    action::{ActionCommand, TypedUndoStateDomain},
    session::{Mc202SourcePhrasePlanState, SessionFile},
    source_graph::SourceGraph,
};

use super::{
    ReplayExecutionError, ReplayExecutionReport, ReplayPlanEntry,
    executor::apply_replay_entry_to_session,
    scene_movement::derive_scene_movement_for_replay_entry,
};

/// Executes one entry, resolving only an absent legacy MC-202 plan.
///
/// The resolver sees immutable pre-entry state. Core then captures ordinary
/// Undo state and executes the entry before supplying the missing plan. Like
/// ordinary single-entry execution, this does not validate history or provide
/// a transaction; callers own their candidate Session and admission boundary.
pub fn apply_replay_entry_with_mc202_legacy_resolver(
    session: &mut SessionFile,
    entry: &ReplayPlanEntry<'_>,
    mut resolver: impl FnMut(&SessionFile, &ReplayPlanEntry<'_>) -> Option<Mc202SourcePhrasePlanState>,
) -> Result<(), ReplayExecutionError> {
    // Persisted plans, including degraded decisions, must never reselect.
    let missing_plan = if entry.action.command.typed_undo_state_domain()
        == Some(TypedUndoStateDomain::Mc202Lane)
        && entry.commit_record.mc202_source_phrase_plan.is_none()
    {
        resolver(session, entry)
    } else {
        None
    };
    apply_replay_entry_to_session(session, entry)?;
    if let Some(plan) = missing_plan {
        session.runtime_state.lane_state.mc202.source_phrase_plan = Some(plan);
    }
    Ok(())
}

pub(super) fn apply_replay_plan_with_mc202_legacy_resolver(
    session: &mut SessionFile,
    entries: &[ReplayPlanEntry<'_>],
    source_graph: Option<&SourceGraph>,
    mut resolve_missing_mc202_plan: impl FnMut(
        &SessionFile,
        &ReplayPlanEntry<'_>,
    ) -> Option<Mc202SourcePhrasePlanState>,
) -> Result<ReplayExecutionReport, ReplayExecutionError> {
    session
        .validate_live_recording_duration_contracts()
        .map_err(ReplayExecutionError::InvalidLiveRecordingDuration)?;
    let mut candidate = session.clone();
    if let Some(source_graph) = source_graph {
        candidate.migrate_scene_source_bindings_with_refs(
            source_graph,
            entries.iter().flat_map(|entry| {
                crate::session::action_scene_refs(entry.action)
                    .chain(entry.commit_record.boundary.scene_id.iter())
            }),
        );
        candidate
            .runtime_state
            .scene_state
            .validate_source_bindings(source_graph)
            .map_err(ReplayExecutionError::InvalidSceneSourceBinding)?;
    }
    let mut applied_action_ids = Vec::with_capacity(entries.len());
    for entry in entries {
        let scene_movement = source_graph
            .and_then(|graph| derive_scene_movement_for_replay_entry(&candidate, entry, graph));
        apply_replay_entry_with_mc202_legacy_resolver(
            &mut candidate,
            entry,
            &mut resolve_missing_mc202_plan,
        )?;
        if source_graph.is_some()
            && matches!(
                entry.action.command,
                ActionCommand::SceneLaunch | ActionCommand::SceneRestore
            )
        {
            if entry.action.command == ActionCommand::SceneLaunch {
                // The entry preserved the old projection in the restore slot.
                // Only the new launch receives graph-derived movement here.
                candidate
                    .runtime_state
                    .scene_state
                    .active_projection_movement = scene_movement.clone();
            }
            candidate.runtime_state.scene_state.last_movement = scene_movement;
        }
        applied_action_ids.push(entry.action.id);
    }
    *session = candidate;
    Ok(ReplayExecutionReport { applied_action_ids })
}

#[cfg(test)]
mod tests;
