use riotbox_core::{
    action::{Action, ActionCommand, ActionParams},
    ids::{BankId, CaptureId, PadId},
    session::{CaptureRef, CaptureSourceWindow, CaptureTarget, CaptureType, SessionFile},
    source_graph::SourceGraph,
    transport::CommitBoundaryState,
};

pub(super) fn capture_ref_from_action(
    session: &SessionFile,
    source_graph: Option<&SourceGraph>,
    action: &Action,
    boundary: &CommitBoundaryState,
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
    let source_window = source_capture
        .and_then(|capture| capture.source_window.clone())
        .or_else(|| source_graph.and_then(|graph| capture_source_window(graph, action, boundary)));
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
        source_window,
        lineage_capture_refs,
        resample_generation_depth,
        created_from_action: Some(action.id),
        assigned_target,
        is_pinned: matches!(action.command, ActionCommand::W30LoopFreeze),
        notes: Some(capture_note(action)),
    })
}

fn capture_source_window(
    graph: &SourceGraph,
    action: &Action,
    boundary: &CommitBoundaryState,
) -> Option<CaptureSourceWindow> {
    if !matches!(
        action.command,
        ActionCommand::CaptureNow
            | ActionCommand::CaptureLoop
            | ActionCommand::CaptureBarGroup
            | ActionCommand::W30CaptureToPad
    ) {
        return None;
    }

    let start_seconds = seconds_for_beat(graph, boundary.beat_index)?
        .max(0.0)
        .min(graph.source.duration_seconds);
    let bars = match action.params {
        ActionParams::Capture { bars } => bars.unwrap_or(1),
        _ => 1,
    };
    let beats_per_bar = graph
        .timing
        .meter_hint
        .map_or(4_u64, |meter| u64::from(meter.beats_per_bar));
    let end_beat = boundary
        .beat_index
        .saturating_add(u64::from(bars).saturating_mul(beats_per_bar));
    let end_seconds = seconds_for_beat(graph, end_beat)
        .unwrap_or_else(|| seconds_for_beat_estimate(graph, end_beat))
        .min(graph.source.duration_seconds)
        .max(start_seconds);

    Some(CaptureSourceWindow {
        source_id: graph.source.source_id.clone(),
        start_seconds,
        end_seconds,
        start_frame: seconds_to_frame(start_seconds, graph.source.sample_rate),
        end_frame: seconds_to_frame(end_seconds, graph.source.sample_rate),
    })
}

fn seconds_for_beat(graph: &SourceGraph, beat_index: u64) -> Option<f32> {
    graph
        .timing
        .beat_grid
        .iter()
        .find(|beat| u64::from(beat.beat_index) == beat_index)
        .map(|beat| beat.time_seconds)
        .or_else(|| {
            graph
                .timing
                .bpm_estimate
                .map(|_| seconds_for_beat_estimate(graph, beat_index))
        })
}

fn seconds_for_beat_estimate(graph: &SourceGraph, beat_index: u64) -> f32 {
    let bpm = graph.timing.bpm_estimate.unwrap_or(120.0).max(1.0);
    beat_index as f32 * 60.0 / bpm
}

fn seconds_to_frame(seconds: f32, sample_rate: u32) -> u64 {
    (seconds.max(0.0) * sample_rate as f32).floor() as u64
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
