//! Metadata-only admission of the selected anchor, never a second replay engine.
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    action::{Action, ActionCommand, ActionParams, ActionStatus, TypedUndoStateDomain},
    ids::{ActionId, CaptureId, SceneId, SectionId, SourceId},
    session::{SceneMovementKindState, SceneMovementState, SessionFile, SnapshotPayload},
    source_graph::SourceGraph,
};

mod references;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnapshotRuntimeField {
    TimingConfirmation,
    CaptureLength,
    LastSceneMovement,
    ActiveSceneProjection,
    RestoreSceneProjection,
    Mc202Undo,
    SourceMonitorUndo,
    Tr909FillUndo,
    Mc202SourcePhrase,
    W30LastCapture,
    W30HookArticulation,
    SceneSourceBinding,
    CaptureLineage,
    CaptureSourceWindow,
    CaptureProducer,
    SourceGraphContext,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotRuntimeActionError {
    Missing,
    AfterAnchor {
        action_index: usize,
        action_cursor: usize,
    },
    NotCommitted,
    WrongOwner,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotRuntimeReference {
    Source(SourceId),
    Capture(CaptureId),
    Graph(SourceId),
    Section {
        source_id: SourceId,
        section_id: SectionId,
    },
    SceneBinding(SceneId),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnapshotRuntimeReferenceError {
    Missing,
    Ambiguous,
    Unavailable,
    Cycle,
    IdentityMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotRuntimeValidationError {
    Action {
        field: SnapshotRuntimeField,
        action_id: ActionId,
        reason: SnapshotRuntimeActionError,
    },
    Reference {
        field: SnapshotRuntimeField,
        reference: SnapshotRuntimeReference,
        reason: SnapshotRuntimeReferenceError,
    },
}

struct Validator<'a> {
    session: &'a SessionFile,
    graph: Option<&'a SourceGraph>,
    cursor: usize,
    actions: BTreeMap<ActionId, (usize, &'a Action)>,
}

pub(super) fn validate(
    session: &SessionFile,
    payload: &SnapshotPayload,
    graph: Option<&SourceGraph>,
) -> Result<(), SnapshotRuntimeValidationError> {
    // The existing target planner already validated history identities/records.
    // Its filtered executable entries deliberately are NOT the provenance index.
    let validator = Validator {
        session,
        graph,
        cursor: payload.action_cursor,
        actions: session
            .action_log
            .actions
            .iter()
            .enumerate()
            .map(|(index, action)| (action.id, (index, action)))
            .collect(),
    };
    let runtime = &payload.runtime_state;
    validator.graph_context()?;
    if let Some(confirmation) = &runtime.source_timing.confirmed_grid {
        let field = SnapshotRuntimeField::TimingConfirmation;
        let action = validator.action(field, confirmation.confirmed_by_action)?;
        let matches = action.command == ActionCommand::SourceTimingConfirmGrid
            && matches!(&action.params,
                ActionParams::SourceTimingGrid { source_id: Some(source_id), hypothesis_id, confirmed_bpm }
                if *source_id == confirmation.source_id && *hypothesis_id == confirmation.hypothesis_id
                    && *confirmed_bpm == runtime.source_timing.confirmed_bpm);
        validator.owner(field, action, matches)?;
        validator.source(field, &confirmation.source_id)?;
    }
    if let Some(id) = runtime.capture.length_set_by_action {
        let field = SnapshotRuntimeField::CaptureLength;
        let action = validator.action(field, id)?;
        validator.owner(
            field,
            action,
            action.command == ActionCommand::CaptureSetLength
                && matches!(action.params, ActionParams::CaptureLength { intent: Some(intent) }
                if intent == runtime.capture.length_intent),
        )?;
    }
    for (field, movement) in [
        (
            SnapshotRuntimeField::LastSceneMovement,
            &runtime.scene_state.last_movement,
        ),
        (
            SnapshotRuntimeField::ActiveSceneProjection,
            &runtime.scene_state.active_projection_movement,
        ),
        (
            SnapshotRuntimeField::RestoreSceneProjection,
            &runtime.scene_state.restore_projection_movement,
        ),
    ] {
        if let Some(movement) = movement {
            validator.movement(field, movement)?;
        }
    }
    for undo in &runtime.undo_state.mc202_snapshots {
        validator.undo(
            SnapshotRuntimeField::Mc202Undo,
            undo.action_id,
            TypedUndoStateDomain::Mc202Lane,
        )?;
        if let Some(plan) = &undo.source_phrase_plan {
            validator.phrase(SnapshotRuntimeField::Mc202Undo, plan)?;
        }
    }
    for undo in &runtime.undo_state.source_monitor_snapshots {
        validator.undo(
            SnapshotRuntimeField::SourceMonitorUndo,
            undo.action_id,
            TypedUndoStateDomain::SourceMonitorMode,
        )?;
    }
    for undo in &runtime.undo_state.tr909_fill_snapshots {
        validator.undo(
            SnapshotRuntimeField::Tr909FillUndo,
            undo.action_id,
            TypedUndoStateDomain::Tr909FillWindow,
        )?;
    }
    if let Some(plan) = &runtime.lane_state.mc202.source_phrase_plan {
        validator.phrase(SnapshotRuntimeField::Mc202SourcePhrase, plan)?;
    }
    for (field, id) in [
        (
            SnapshotRuntimeField::W30LastCapture,
            runtime.lane_state.w30.last_capture.as_ref(),
        ),
        (
            SnapshotRuntimeField::W30HookArticulation,
            runtime
                .lane_state
                .w30
                .hook_articulation
                .as_ref()
                .map(|hook| &hook.capture_id),
        ),
    ] {
        if let Some(id) = id {
            validator.capture(field, id)?;
        }
    }
    if let Some(bindings) = &runtime.scene_state.source_bindings {
        let mut seen = BTreeSet::new();
        for binding in bindings {
            let field = SnapshotRuntimeField::SceneSourceBinding;
            if !seen.insert(&binding.scene_id) {
                return Err(Validator::reference_error(
                    field,
                    SnapshotRuntimeReference::SceneBinding(binding.scene_id.clone()),
                    SnapshotRuntimeReferenceError::Ambiguous,
                ));
            }
            validator.section(field, &binding.source_id, &binding.section_id)?;
        }
    }
    Ok(())
}

impl Validator<'_> {
    fn action(
        &self,
        field: SnapshotRuntimeField,
        id: ActionId,
    ) -> Result<&Action, SnapshotRuntimeValidationError> {
        let error = |reason| SnapshotRuntimeValidationError::Action {
            field,
            action_id: id,
            reason,
        };
        let Some((index, action)) = self.actions.get(&id) else {
            return Err(error(SnapshotRuntimeActionError::Missing));
        };
        if *index >= self.cursor {
            return Err(error(SnapshotRuntimeActionError::AfterAnchor {
                action_index: *index,
                action_cursor: self.cursor,
            }));
        }
        // V2 record completeness is owned by the existing planner. Legacy
        // policies deliberately admit partial record coverage; do not silently
        // reinterpret that historical policy here.
        if action.committed_at.is_none()
            || !(action.status == ActionStatus::Committed
                || action.status == ActionStatus::Undone
                    && action.command.has_typed_undo_semantics())
            || action
                .result
                .as_ref()
                .is_some_and(|result| !result.accepted)
        {
            return Err(error(SnapshotRuntimeActionError::NotCommitted));
        }
        Ok(action)
    }

    fn owner(
        &self,
        field: SnapshotRuntimeField,
        action: &Action,
        valid: bool,
    ) -> Result<(), SnapshotRuntimeValidationError> {
        if valid {
            Ok(())
        } else {
            Err(SnapshotRuntimeValidationError::Action {
                field,
                action_id: action.id,
                reason: SnapshotRuntimeActionError::WrongOwner,
            })
        }
    }

    fn undo(
        &self,
        field: SnapshotRuntimeField,
        id: ActionId,
        domain: TypedUndoStateDomain,
    ) -> Result<(), SnapshotRuntimeValidationError> {
        let action = self.action(field, id)?;
        self.owner(
            field,
            action,
            action.command.typed_undo_state_domain() == Some(domain),
        )
    }

    fn movement(
        &self,
        field: SnapshotRuntimeField,
        movement: &SceneMovementState,
    ) -> Result<(), SnapshotRuntimeValidationError> {
        let action = self.action(field, movement.action_id)?;
        let expected = match movement.kind {
            SceneMovementKindState::Launch => ActionCommand::SceneLaunch,
            SceneMovementKindState::Restore => ActionCommand::SceneRestore,
        };
        let target = action.target.scene_id.as_ref().or(match &action.params {
            ActionParams::Scene { scene_id } => scene_id.as_ref(),
            _ => None,
        });
        self.owner(
            field,
            action,
            action.command == expected && target == Some(&movement.to_scene),
        )
    }
}

#[cfg(test)]
mod tests;
