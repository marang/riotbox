use crate::{
    action::{ActionCommand, ActionParams},
    ids::{ActionId, CaptureId},
    replay::ReplayPlanEntry,
    session::{CaptureRef, CaptureType, SessionFile},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct W30ArtifactReplayHydrationPlan {
    pub action_id: ActionId,
    pub command: ActionCommand,
    pub produced_capture_id: CaptureId,
    pub source_capture_id: CaptureId,
    pub capture_type: CaptureType,
    pub storage_path: String,
    pub resample_generation_depth: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct W30SourceCaptureReplayHydrationPlan {
    pub action_id: ActionId,
    pub command: ActionCommand,
    pub produced_capture_id: CaptureId,
    pub capture_type: CaptureType,
    pub storage_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum W30ArtifactReplayHydrationError {
    NotArtifactProducingW30Action {
        action_id: ActionId,
        command: ActionCommand,
    },
    MissingSourceCaptureTarget {
        action_id: ActionId,
        command: ActionCommand,
    },
    MissingProducedCapture {
        action_id: ActionId,
        command: ActionCommand,
    },
    MissingSourceCapture {
        capture_id: CaptureId,
    },
    AmbiguousProducedCapture {
        action_id: ActionId,
        command: ActionCommand,
        capture_count: usize,
    },
    MissingStoragePath {
        capture_id: CaptureId,
    },
    MissingSourceWindowForSourceBackedCapture {
        capture_id: CaptureId,
    },
    InvalidPadCaptureIdentity {
        capture_id: CaptureId,
    },
    InvalidLoopCaptureIdentity {
        capture_id: CaptureId,
    },
    InvalidResampleIdentity {
        capture_id: CaptureId,
    },
    SourceCaptureLineageMismatch {
        produced_capture_id: CaptureId,
        source_capture_id: CaptureId,
    },
}

pub fn plan_w30_artifact_replay_hydration(
    session: &SessionFile,
    entry: &ReplayPlanEntry<'_>,
) -> Result<W30ArtifactReplayHydrationPlan, W30ArtifactReplayHydrationError> {
    let action = entry.action;
    if !matches!(
        action.command,
        ActionCommand::W30LoopFreeze | ActionCommand::PromoteResample
    ) {
        return Err(
            W30ArtifactReplayHydrationError::NotArtifactProducingW30Action {
                action_id: action.id,
                command: action.command,
            },
        );
    }

    let source_capture_id = source_capture_target(action)?;
    require_source_capture(session, &source_capture_id)?;
    let produced_capture = produced_capture_for_action(session, action.id, action.command)?;
    validate_capture_identity(produced_capture, &source_capture_id)?;

    Ok(W30ArtifactReplayHydrationPlan {
        action_id: action.id,
        command: action.command,
        produced_capture_id: produced_capture.capture_id.clone(),
        source_capture_id,
        capture_type: produced_capture.capture_type,
        storage_path: produced_capture.storage_path.clone(),
        resample_generation_depth: produced_capture.resample_generation_depth,
    })
}

pub fn plan_source_window_pad_capture_replay_hydration(
    session: &SessionFile,
    entry: &ReplayPlanEntry<'_>,
) -> Result<W30SourceCaptureReplayHydrationPlan, W30ArtifactReplayHydrationError> {
    let action = entry.action;
    if !matches!(
        action.command,
        ActionCommand::CaptureBarGroup | ActionCommand::W30CaptureToPad
    ) {
        return Err(
            W30ArtifactReplayHydrationError::NotArtifactProducingW30Action {
                action_id: action.id,
                command: action.command,
            },
        );
    }

    let produced_capture = produced_capture_for_action(session, action.id, action.command)?;
    if produced_capture.storage_path.trim().is_empty() {
        return Err(W30ArtifactReplayHydrationError::MissingStoragePath {
            capture_id: produced_capture.capture_id.clone(),
        });
    }
    if produced_capture.source_window.is_none() {
        return Err(
            W30ArtifactReplayHydrationError::MissingSourceWindowForSourceBackedCapture {
                capture_id: produced_capture.capture_id.clone(),
            },
        );
    }
    if produced_capture.capture_type != CaptureType::Pad {
        return Err(W30ArtifactReplayHydrationError::InvalidPadCaptureIdentity {
            capture_id: produced_capture.capture_id.clone(),
        });
    }

    Ok(W30SourceCaptureReplayHydrationPlan {
        action_id: action.id,
        command: action.command,
        produced_capture_id: produced_capture.capture_id.clone(),
        capture_type: produced_capture.capture_type,
        storage_path: produced_capture.storage_path.clone(),
    })
}

pub fn plan_source_window_loop_capture_replay_hydration(
    session: &SessionFile,
    entry: &ReplayPlanEntry<'_>,
) -> Result<W30SourceCaptureReplayHydrationPlan, W30ArtifactReplayHydrationError> {
    let action = entry.action;
    if !matches!(
        action.command,
        ActionCommand::CaptureNow | ActionCommand::CaptureLoop
    ) {
        return Err(
            W30ArtifactReplayHydrationError::NotArtifactProducingW30Action {
                action_id: action.id,
                command: action.command,
            },
        );
    }

    let produced_capture = produced_capture_for_action(session, action.id, action.command)?;
    if produced_capture.storage_path.trim().is_empty() {
        return Err(W30ArtifactReplayHydrationError::MissingStoragePath {
            capture_id: produced_capture.capture_id.clone(),
        });
    }
    if produced_capture.source_window.is_none() {
        return Err(
            W30ArtifactReplayHydrationError::MissingSourceWindowForSourceBackedCapture {
                capture_id: produced_capture.capture_id.clone(),
            },
        );
    }
    if produced_capture.capture_type != CaptureType::Loop {
        return Err(
            W30ArtifactReplayHydrationError::InvalidLoopCaptureIdentity {
                capture_id: produced_capture.capture_id.clone(),
            },
        );
    }

    Ok(W30SourceCaptureReplayHydrationPlan {
        action_id: action.id,
        command: action.command,
        produced_capture_id: produced_capture.capture_id.clone(),
        capture_type: produced_capture.capture_type,
        storage_path: produced_capture.storage_path.clone(),
    })
}

fn source_capture_target(
    action: &crate::action::Action,
) -> Result<CaptureId, W30ArtifactReplayHydrationError> {
    match &action.params {
        ActionParams::Mutation {
            target_id: Some(target_id),
            ..
        } => Ok(CaptureId::from(target_id.clone())),
        ActionParams::Promotion {
            capture_id: Some(capture_id),
            ..
        } => Ok(capture_id.clone()),
        _ => Err(
            W30ArtifactReplayHydrationError::MissingSourceCaptureTarget {
                action_id: action.id,
                command: action.command,
            },
        ),
    }
}

fn require_source_capture(
    session: &SessionFile,
    source_capture_id: &CaptureId,
) -> Result<(), W30ArtifactReplayHydrationError> {
    if session
        .captures
        .iter()
        .any(|capture| capture.capture_id == *source_capture_id)
    {
        return Ok(());
    }

    Err(W30ArtifactReplayHydrationError::MissingSourceCapture {
        capture_id: source_capture_id.clone(),
    })
}

fn produced_capture_for_action(
    session: &SessionFile,
    action_id: ActionId,
    command: ActionCommand,
) -> Result<&CaptureRef, W30ArtifactReplayHydrationError> {
    let mut matches = session
        .captures
        .iter()
        .filter(|capture| capture.created_from_action == Some(action_id));
    let Some(capture) = matches.next() else {
        return Err(W30ArtifactReplayHydrationError::MissingProducedCapture { action_id, command });
    };
    if matches.next().is_some() {
        let capture_count = session
            .captures
            .iter()
            .filter(|capture| capture.created_from_action == Some(action_id))
            .count();
        return Err(W30ArtifactReplayHydrationError::AmbiguousProducedCapture {
            action_id,
            command,
            capture_count,
        });
    }

    Ok(capture)
}

fn validate_capture_identity(
    capture: &CaptureRef,
    source_capture_id: &CaptureId,
) -> Result<(), W30ArtifactReplayHydrationError> {
    if capture.storage_path.trim().is_empty() {
        return Err(W30ArtifactReplayHydrationError::MissingStoragePath {
            capture_id: capture.capture_id.clone(),
        });
    }

    if capture.capture_type == CaptureType::Resample {
        if capture.lineage_capture_refs.is_empty() || capture.resample_generation_depth == 0 {
            return Err(W30ArtifactReplayHydrationError::InvalidResampleIdentity {
                capture_id: capture.capture_id.clone(),
            });
        }
        require_lineage_contains_source(capture, source_capture_id)?;
        return Ok(());
    }

    if capture.source_window.is_some() {
        return Ok(());
    }

    if capture.lineage_capture_refs.is_empty() {
        return Err(
            W30ArtifactReplayHydrationError::MissingSourceWindowForSourceBackedCapture {
                capture_id: capture.capture_id.clone(),
            },
        );
    }
    require_lineage_contains_source(capture, source_capture_id)?;

    Ok(())
}

fn require_lineage_contains_source(
    capture: &CaptureRef,
    source_capture_id: &CaptureId,
) -> Result<(), W30ArtifactReplayHydrationError> {
    if capture
        .lineage_capture_refs
        .iter()
        .any(|lineage_capture_id| lineage_capture_id == source_capture_id)
    {
        return Ok(());
    }

    Err(
        W30ArtifactReplayHydrationError::SourceCaptureLineageMismatch {
            produced_capture_id: capture.capture_id.clone(),
            source_capture_id: source_capture_id.clone(),
        },
    )
}
