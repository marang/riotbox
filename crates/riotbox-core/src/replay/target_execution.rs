use crate::{
    ids::ActionId,
    replay::{ReplayExecutionError, ReplayPlanEntry, ReplayPlanError, build_replay_target_plan},
    session::{Mc202SourcePhrasePlanState, SessionFile},
    source_graph::SourceGraph,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayTargetExecutionError {
    Plan(ReplayPlanError),
    Execution(ReplayExecutionError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayTargetExecutionReport {
    pub target_action_cursor: usize,
    pub anchor_snapshot_id: Option<String>,
    pub anchor_action_cursor: Option<usize>,
    pub applied_action_ids: Vec<ActionId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotPayloadHydrationError {
    Plan(ReplayPlanError),
    MissingSnapshotAnchor {
        target_action_cursor: usize,
    },
    MissingSnapshotPayload {
        snapshot_id: String,
    },
    PayloadSnapshotIdMismatch {
        snapshot_id: String,
        payload_snapshot_id: String,
    },
    PayloadActionCursorMismatch {
        snapshot_id: String,
        snapshot_action_cursor: usize,
        payload_action_cursor: usize,
    },
    RuntimeProvenance {
        snapshot_id: String,
        reason: super::SnapshotRuntimeValidationError,
    },
    Execution(ReplayTargetExecutionError),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotPayloadHydrationReport {
    pub session: SessionFile,
    pub replay_report: ReplayTargetExecutionReport,
}

/// Hydrates the selected anchor after metadata admission.
///
/// Any supplied graph must already have passed identity admission
/// against this Session's graph reference. Embedded references resolve only
/// against their persisted graph; this function never loads graph/audio files.
pub fn hydrate_replay_target_from_snapshot_payload(
    session: &SessionFile,
    target_action_cursor: usize,
    source_graph: Option<&SourceGraph>,
) -> Result<SnapshotPayloadHydrationReport, SnapshotPayloadHydrationError> {
    hydrate_replay_target_from_snapshot_payload_with_mc202_legacy_resolver(
        session,
        target_action_cursor,
        source_graph,
        |_, _| None,
    )
}

/// Hydrates the validated anchor, resolving missing legacy MC-202 suffix plans.
///
/// Only actual suffix entries invoke the resolver; an empty suffix preserves
/// the payload state without deriving any skipped historical phrase.
/// Any supplied graph must already have passed identity admission
/// against this Session's graph reference; embedded metadata remains authoritative.
pub fn hydrate_replay_target_from_snapshot_payload_with_mc202_legacy_resolver(
    session: &SessionFile,
    target_action_cursor: usize,
    source_graph: Option<&SourceGraph>,
    resolver: impl FnMut(&SessionFile, &ReplayPlanEntry<'_>) -> Option<Mc202SourcePhrasePlanState>,
) -> Result<SnapshotPayloadHydrationReport, SnapshotPayloadHydrationError> {
    let plan = build_replay_target_plan(
        &session.action_log,
        &session.snapshots,
        target_action_cursor,
    )
    .map_err(SnapshotPayloadHydrationError::Plan)?;
    let Some(anchor) = plan.anchor else {
        return Err(SnapshotPayloadHydrationError::MissingSnapshotAnchor {
            target_action_cursor,
        });
    };
    let snapshot_id = anchor.snapshot_id.as_str().to_owned();
    let payload = anchor.payload.as_ref().ok_or_else(|| {
        SnapshotPayloadHydrationError::MissingSnapshotPayload {
            snapshot_id: snapshot_id.clone(),
        }
    })?;

    if payload.snapshot_id != anchor.snapshot_id {
        return Err(SnapshotPayloadHydrationError::PayloadSnapshotIdMismatch {
            snapshot_id,
            payload_snapshot_id: payload.snapshot_id.as_str().to_owned(),
        });
    }

    if payload.action_cursor != anchor.action_cursor {
        return Err(SnapshotPayloadHydrationError::PayloadActionCursorMismatch {
            snapshot_id,
            snapshot_action_cursor: anchor.action_cursor,
            payload_action_cursor: payload.action_cursor,
        });
    }

    super::snapshot_runtime::validate(session, payload, source_graph).map_err(|reason| {
        SnapshotPayloadHydrationError::RuntimeProvenance {
            snapshot_id,
            reason,
        }
    })?;

    let mut hydrated_session = session.clone();
    hydrated_session.runtime_state = payload.runtime_state.clone();
    let replay_report = apply_replay_target_suffix_with_mc202_legacy_resolver(
        &mut hydrated_session,
        target_action_cursor,
        source_graph,
        resolver,
    )
    .map_err(SnapshotPayloadHydrationError::Execution)?;

    Ok(SnapshotPayloadHydrationReport {
        session: hydrated_session,
        replay_report,
    })
}

pub fn apply_replay_target_suffix_to_session(
    session: &mut SessionFile,
    target_action_cursor: usize,
    source_graph: Option<&SourceGraph>,
) -> Result<ReplayTargetExecutionReport, ReplayTargetExecutionError> {
    apply_replay_target_suffix_with_mc202_legacy_resolver(
        session,
        target_action_cursor,
        source_graph,
        |_, _| None,
    )
}

/// Applies the Core-selected suffix, resolving only missing legacy MC-202 plans.
///
/// The resolver receives immutable pre-entry state. Persisted plans bypass it;
/// Core retains ordinary execution, Undo snapshots and transactional installation.
pub fn apply_replay_target_suffix_with_mc202_legacy_resolver(
    session: &mut SessionFile,
    target_action_cursor: usize,
    source_graph: Option<&SourceGraph>,
    resolver: impl FnMut(&SessionFile, &ReplayPlanEntry<'_>) -> Option<Mc202SourcePhrasePlanState>,
) -> Result<ReplayTargetExecutionReport, ReplayTargetExecutionError> {
    let action_log = session.action_log.clone();
    let snapshots = session.snapshots.clone();
    let plan = build_replay_target_plan(&action_log, &snapshots, target_action_cursor)
        .map_err(ReplayTargetExecutionError::Plan)?;
    let anchor_snapshot_id = plan
        .anchor
        .map(|snapshot| snapshot.snapshot_id.as_str().to_owned());
    let anchor_action_cursor = plan.anchor.map(|snapshot| snapshot.action_cursor);

    let execution_report = super::traversal::apply_replay_plan_with_mc202_legacy_resolver(
        session,
        &plan.suffix,
        source_graph,
        resolver,
    )
    .map_err(ReplayTargetExecutionError::Execution)?;

    Ok(ReplayTargetExecutionReport {
        target_action_cursor,
        anchor_snapshot_id,
        anchor_action_cursor,
        applied_action_ids: execution_report.applied_action_ids,
    })
}

#[cfg(test)]
mod tests;
