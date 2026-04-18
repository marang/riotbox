use riotbox_core::{
    action::{Action, ActionCommand, ActionParams},
    ids::{BankId, CaptureId, PadId},
    session::{CaptureRef, CaptureTarget, CaptureType, SessionFile},
    source_graph::SourceGraph,
};

pub(super) fn capture_ref_from_action(
    session: &SessionFile,
    source_graph: Option<&SourceGraph>,
    action: &Action,
) -> Option<CaptureRef> {
    let capture_type = match action.command {
        ActionCommand::CaptureNow | ActionCommand::CaptureLoop => CaptureType::Loop,
        ActionCommand::CaptureBarGroup | ActionCommand::W30CaptureToPad => CaptureType::Pad,
        ActionCommand::PromoteResample => CaptureType::Resample,
        ActionCommand::W30LoopFreeze => matches!(action.params, ActionParams::Promotion { .. })
            .then(|| promotion_capture_id(session, action))
            .flatten()
            .and_then(|capture_id| {
                session
                    .captures
                    .iter()
                    .find(|capture| capture.capture_id == capture_id)
                    .map(|capture| capture.capture_type)
            })
            .unwrap_or(CaptureType::Loop),
        _ => return None,
    };

    let assigned_target = match action.command {
        ActionCommand::W30CaptureToPad => session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .clone()
            .zip(session.runtime_state.lane_state.w30.focused_pad.clone())
            .map(|(bank_id, pad_id)| CaptureTarget::W30Pad { bank_id, pad_id }),
        ActionCommand::W30LoopFreeze => action
            .target
            .bank_id
            .clone()
            .zip(action.target.pad_id.clone())
            .map(|(bank_id, pad_id)| CaptureTarget::W30Pad { bank_id, pad_id }),
        _ => None,
    };

    let capture_id = next_capture_id(session);
    let source_capture = matches!(
        action.command,
        ActionCommand::PromoteResample | ActionCommand::W30LoopFreeze
    )
    .then(|| promotion_capture_id(session, action))
    .flatten()
    .and_then(|capture_id| {
        session
            .captures
            .iter()
            .find(|capture| capture.capture_id == capture_id)
    });
    let source_origin_refs = source_capture
        .map(|capture| capture.source_origin_refs.clone())
        .or_else(|| source_graph.map(capture_origin_refs))
        .unwrap_or_else(|| vec!["source-graph-unavailable".into()]);
    let mut lineage_capture_refs = source_capture
        .map(|capture| capture.lineage_capture_refs.clone())
        .unwrap_or_default();
    if let Some(source_capture) = source_capture
        && !lineage_capture_refs.contains(&source_capture.capture_id)
    {
        lineage_capture_refs.push(source_capture.capture_id.clone());
    }
    let resample_generation_depth = source_capture
        .map(|capture| {
            if matches!(action.command, ActionCommand::PromoteResample) {
                capture.resample_generation_depth.saturating_add(1)
            } else {
                capture.resample_generation_depth
            }
        })
        .unwrap_or(0);

    Some(CaptureRef {
        storage_path: format!("captures/{capture_id}.wav"),
        capture_id,
        capture_type,
        source_origin_refs,
        lineage_capture_refs,
        resample_generation_depth,
        created_from_action: Some(action.id),
        assigned_target,
        is_pinned: matches!(action.command, ActionCommand::W30LoopFreeze),
        notes: Some(capture_note(action)),
    })
}

pub(super) fn apply_capture_promotion_side_effects(
    session: &mut SessionFile,
    action: &Action,
) -> bool {
    if !matches!(
        action.command,
        ActionCommand::PromoteCaptureToPad | ActionCommand::PromoteCaptureToScene
    ) {
        return false;
    }

    let target = match promotion_target_from_action(session, action) {
        Some(target) => target,
        None => return false,
    };
    let capture_id = match promotion_capture_id(session, action) {
        Some(capture_id) => capture_id,
        None => return false,
    };

    let Some(capture) = session
        .captures
        .iter_mut()
        .find(|capture| capture.capture_id == capture_id)
    else {
        return false;
    };

    capture.assigned_target = Some(target.clone());
    capture.notes = Some(updated_capture_note(capture.notes.as_deref(), &target));

    session.runtime_state.lane_state.w30.last_capture = Some(capture.capture_id.clone());
    if let CaptureTarget::W30Pad { bank_id, pad_id } = target {
        session.runtime_state.lane_state.w30.active_bank = Some(bank_id);
        session.runtime_state.lane_state.w30.focused_pad = Some(pad_id);
    }

    true
}

fn capture_origin_refs(graph: &SourceGraph) -> Vec<String> {
    let mut refs = Vec::new();
    refs.push(graph.source.source_id.to_string());
    refs.extend(
        graph
            .candidates
            .iter()
            .take(2)
            .map(|candidate| candidate.asset_ref.to_string()),
    );
    refs.dedup();
    refs
}

fn capture_note(action: &Action) -> String {
    match &action.explanation {
        Some(explanation) if !explanation.is_empty() => explanation.clone(),
        _ => format!("capture committed from {}", action.command),
    }
}

fn promotion_capture_id(session: &SessionFile, action: &Action) -> Option<CaptureId> {
    match &action.params {
        ActionParams::Promotion {
            capture_id: Some(capture_id),
            ..
        } => Some(capture_id.clone()),
        _ => session
            .captures
            .last()
            .map(|capture| capture.capture_id.clone()),
    }
}

fn promotion_target_from_action(session: &SessionFile, action: &Action) -> Option<CaptureTarget> {
    match action.command {
        ActionCommand::PromoteCaptureToPad => action
            .target
            .bank_id
            .clone()
            .or_else(|| session.runtime_state.lane_state.w30.active_bank.clone())
            .zip(
                action
                    .target
                    .pad_id
                    .clone()
                    .or_else(|| session.runtime_state.lane_state.w30.focused_pad.clone()),
            )
            .map(|(bank_id, pad_id)| CaptureTarget::W30Pad { bank_id, pad_id }),
        ActionCommand::PromoteCaptureToScene => {
            action.target.scene_id.clone().map(CaptureTarget::Scene)
        }
        _ => None,
    }
}

fn promotion_note(target: &CaptureTarget) -> String {
    match target {
        CaptureTarget::W30Pad { bank_id, pad_id } => {
            format!("promoted to pad {bank_id}/{pad_id}")
        }
        CaptureTarget::Scene(scene_id) => format!("promoted to scene {scene_id}"),
    }
}

pub(super) fn capture_promotion_summary(session: &SessionFile, action: &Action) -> Option<String> {
    let capture_id = promotion_capture_id(session, action)?;
    let capture = session
        .captures
        .iter()
        .find(|capture| capture.capture_id == capture_id)?;
    capture.notes.clone()
}

fn updated_capture_note(existing_notes: Option<&str>, target: &CaptureTarget) -> String {
    let promotion = promotion_note(target);
    match existing_notes {
        Some(existing_notes) => {
            let base = existing_notes
                .split(" | promoted to ")
                .next()
                .unwrap_or(existing_notes);
            format!("{base} | {promotion}")
        }
        None => promotion,
    }
}

pub(super) fn capture_targets_w30_pad(capture: &CaptureRef) -> bool {
    matches!(capture.assigned_target, Some(CaptureTarget::W30Pad { .. }))
}

pub(super) fn capture_targets_specific_w30_pad(
    capture: &CaptureRef,
    bank_id: &BankId,
    pad_id: &PadId,
) -> bool {
    matches!(
        capture.assigned_target.as_ref(),
        Some(CaptureTarget::W30Pad {
            bank_id: target_bank_id,
            pad_id: target_pad_id,
        }) if target_bank_id == bank_id && target_pad_id == pad_id
    )
}

fn next_capture_id(session: &SessionFile) -> CaptureId {
    CaptureId::from(format!(
        "cap-{:02}",
        session.captures.len().saturating_add(1)
    ))
}
