use riotbox_core::{
    action::{Action, ActionCommand, ActionParams, CaptureLengthIntent},
    ids::{BankId, CaptureId, PadId},
    session::{
        CaptureRef, CaptureSourceWindow, CaptureTarget, CaptureType, SessionFile,
        W30HookSelectionDecision, W30HookSelectionPolicy, W30HookSelectionReason,
    },
    source_graph::{SourceGraph, W30HookCandidateEvidence, W30HookCandidateStatus},
    style::PerformancePresetId,
    transport::CommitBoundaryState,
    view::jam::source_timing_consumer_readiness,
};

pub(in crate::jam_app) fn capture_ref_from_action(
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
    let source_window = if matches!(action.command, ActionCommand::PromoteResample) {
        None
    } else {
        source_capture
            .and_then(|capture| capture.source_window.clone())
            .or_else(|| {
                source_graph
                    .and_then(|graph| capture_source_window(session, graph, action, boundary))
            })
    };
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
    let notes = Some(capture_note(action, source_window.as_ref()));

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
        notes,
    })
}

fn capture_source_window(
    session: &SessionFile,
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
    if !source_timing_consumer_readiness(Some(graph), session).can_use_source_window_grid() {
        return None;
    }

    let baseline_start_seconds = seconds_for_beat_cursor(graph, boundary.beat_index)?
        .max(0.0)
        .min(graph.source.duration_seconds);
    let beats_per_bar = graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.meter)
        .or(graph.timing.meter_hint)
        .map_or(4_u64, |meter| u64::from(meter.beats_per_bar));
    let end_beat = capture_end_beat(session, graph, action, boundary, beats_per_bar);
    let baseline_end_seconds = seconds_for_beat_cursor(graph, end_beat)
        .unwrap_or_else(|| seconds_for_beat_cursor_estimate(graph, end_beat))
        .min(graph.source.duration_seconds)
        .max(baseline_start_seconds);
    let (start_seconds, end_seconds, hook_selection) = select_w30_hook_window(
        session,
        graph,
        action,
        baseline_start_seconds,
        baseline_end_seconds,
        beats_per_bar,
    );
    let start_frame = seconds_to_frame(start_seconds, graph.source.sample_rate);
    let end_frame = seconds_to_frame(end_seconds, graph.source.sample_rate);
    if end_frame <= start_frame {
        return None;
    }

    Some(CaptureSourceWindow {
        source_id: graph.source.source_id.clone(),
        start_seconds,
        end_seconds,
        start_frame,
        end_frame,
        hook_selection,
    })
}

const W30_HOOK_BASELINE_TOLERANCE_SECONDS: f32 = 0.020;
const W30_HOOK_MINIMUM_SCORE_LIFT: f32 = 0.10;

fn select_w30_hook_window(
    session: &SessionFile,
    graph: &SourceGraph,
    action: &Action,
    baseline_start_seconds: f32,
    baseline_end_seconds: f32,
    beats_per_bar: u64,
) -> (f32, f32, Option<W30HookSelectionDecision>) {
    if action.command != ActionCommand::CaptureBarGroup
        || session.runtime_state.style.active_preset != Some(PerformancePresetId::FeralBreakAlphaV2)
        || !is_one_bar_capture(session, action, beats_per_bar)
    {
        return (baseline_start_seconds, baseline_end_seconds, None);
    }

    let policy = session.runtime_state.style.w30_hook_selection_policy;
    let baseline = graph.w30_hook_candidates.iter().find(|candidate| {
        candidate.status == W30HookCandidateStatus::Eligible
            && (candidate.start_seconds - baseline_start_seconds).abs()
                <= W30_HOOK_BASELINE_TOLERANCE_SECONDS
    });
    if policy == W30HookSelectionPolicy::TransportBoundaryV1 {
        return (
            baseline_start_seconds,
            baseline_end_seconds,
            Some(selection_decision(
                policy,
                W30HookSelectionReason::TransportBoundaryPolicy,
                baseline,
                baseline,
                baseline_start_seconds,
                baseline_end_seconds,
            )),
        );
    }

    let eligible = graph
        .w30_hook_candidates
        .iter()
        .filter(|candidate| candidate.status == W30HookCandidateStatus::Eligible)
        .collect::<Vec<_>>();
    if eligible.len() < 2 {
        return retained_hook_window(
            policy,
            W30HookSelectionReason::InsufficientEligibleBars,
            baseline,
            baseline_start_seconds,
            baseline_end_seconds,
        );
    }
    let Some(baseline) = baseline else {
        return retained_hook_window(
            policy,
            W30HookSelectionReason::BaselineEvidenceUnavailable,
            None,
            baseline_start_seconds,
            baseline_end_seconds,
        );
    };
    let Some(best) = eligible
        .into_iter()
        .filter(|candidate| score(candidate, policy).is_some())
        .fold(
            None,
            |best: Option<&W30HookCandidateEvidence>, candidate| match best {
                Some(current)
                    if score(current, policy) > score(candidate, policy)
                        || (score(current, policy) == score(candidate, policy)
                            && current.start_seconds <= candidate.start_seconds) =>
                {
                    Some(current)
                }
                _ => Some(candidate),
            },
        )
    else {
        return retained_hook_window(
            policy,
            W30HookSelectionReason::InsufficientEligibleBars,
            Some(baseline),
            baseline_start_seconds,
            baseline_end_seconds,
        );
    };
    let baseline_score = score(baseline, policy).unwrap_or(0.0);
    let best_score = score(best, policy).unwrap_or(0.0);
    if best_score - baseline_score < W30_HOOK_MINIMUM_SCORE_LIFT {
        return retained_hook_window(
            policy,
            W30HookSelectionReason::ScoreLiftBelowMinimum,
            Some(baseline),
            baseline_start_seconds,
            baseline_end_seconds,
        );
    }

    (
        best.start_seconds,
        best.end_seconds,
        Some(selection_decision(
            policy,
            W30HookSelectionReason::CandidateSelected,
            Some(baseline),
            Some(best),
            baseline_start_seconds,
            baseline_end_seconds,
        )),
    )
}

fn is_one_bar_capture(session: &SessionFile, action: &Action, beats_per_bar: u64) -> bool {
    match action.params {
        ActionParams::Capture { bars: Some(bars) } => u64::from(bars) == 1,
        _ => {
            session.runtime_state.capture.length_intent == CaptureLengthIntent::OneBar
                && beats_per_bar > 0
        }
    }
}

fn retained_hook_window(
    policy: W30HookSelectionPolicy,
    reason: W30HookSelectionReason,
    baseline: Option<&W30HookCandidateEvidence>,
    baseline_start_seconds: f32,
    baseline_end_seconds: f32,
) -> (f32, f32, Option<W30HookSelectionDecision>) {
    (
        baseline_start_seconds,
        baseline_end_seconds,
        Some(selection_decision(
            policy,
            reason,
            baseline,
            baseline,
            baseline_start_seconds,
            baseline_end_seconds,
        )),
    )
}

fn selection_decision(
    policy: W30HookSelectionPolicy,
    reason: W30HookSelectionReason,
    baseline: Option<&W30HookCandidateEvidence>,
    selected: Option<&W30HookCandidateEvidence>,
    baseline_start_seconds: f32,
    baseline_end_seconds: f32,
) -> W30HookSelectionDecision {
    let baseline_score = baseline.and_then(|candidate| score(candidate, policy));
    let selected_score = selected.and_then(|candidate| score(candidate, policy));
    W30HookSelectionDecision {
        policy,
        reason,
        baseline_bar_index: baseline.map(|candidate| candidate.bar_index),
        baseline_start_seconds,
        baseline_end_seconds,
        selected_bar_index: selected.map(|candidate| candidate.bar_index),
        selected_evidence: selected.and_then(|candidate| candidate.normalized_features),
        baseline_score,
        selected_score,
        score_lift: baseline_score
            .zip(selected_score)
            .map(|(baseline, selected)| selected - baseline),
    }
}

fn score(candidate: &W30HookCandidateEvidence, policy: W30HookSelectionPolicy) -> Option<f32> {
    match policy {
        W30HookSelectionPolicy::TransportBoundaryV1 => None,
        W30HookSelectionPolicy::AttackBodyContrastV1 => candidate.attack_body_contrast_score,
        W30HookSelectionPolicy::RepetitionSalienceV1 => candidate.repetition_salience_score,
    }
}

fn capture_end_beat(
    session: &SessionFile,
    graph: &SourceGraph,
    action: &Action,
    boundary: &CommitBoundaryState,
    beats_per_bar: u64,
) -> u64 {
    if let ActionParams::Capture { bars: Some(bars) } = action.params {
        return boundary
            .beat_index
            .saturating_add(u64::from(bars).saturating_mul(beats_per_bar));
    }

    match session.runtime_state.capture.length_intent {
        CaptureLengthIntent::OneBeat => boundary.beat_index.saturating_add(1),
        CaptureLengthIntent::OneBar => boundary.beat_index.saturating_add(beats_per_bar),
        CaptureLengthIntent::FourBars => boundary
            .beat_index
            .saturating_add(4_u64.saturating_mul(beats_per_bar)),
        CaptureLengthIntent::Phrase => phrase_capture_end_beat(graph, boundary, beats_per_bar)
            .unwrap_or_else(|| {
                boundary
                    .beat_index
                    .saturating_add(4_u64.saturating_mul(beats_per_bar))
            }),
    }
}

fn phrase_capture_end_beat(
    graph: &SourceGraph,
    boundary: &CommitBoundaryState,
    beats_per_bar: u64,
) -> Option<u64> {
    let start_bar = boundary.bar_index;
    let phrase_grid = graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.phrase_grid.as_slice())
        .filter(|phrases| !phrases.is_empty())
        .unwrap_or(graph.timing.phrase_grid.as_slice());

    phrase_grid
        .iter()
        .find(|phrase| {
            u64::from(phrase.start_bar) <= start_bar && start_bar <= u64::from(phrase.end_bar)
        })
        .or_else(|| {
            phrase_grid
                .iter()
                .find(|phrase| u64::from(phrase.start_bar) >= start_bar)
        })
        .and_then(|phrase| {
            let bar_after_phrase = u64::from(phrase.end_bar).saturating_add(1);
            if let Some(primary) = graph.timing.primary_hypothesis()
                && !primary.bar_grid.is_empty()
            {
                primary.bar_start_beat_cursor(bar_after_phrase)
            } else {
                Some(u64::from(phrase.end_bar).saturating_mul(beats_per_bar))
            }
        })
        .filter(|end_beat| *end_beat > boundary.beat_index)
}

fn seconds_for_beat_cursor(graph: &SourceGraph, beat_cursor: u64) -> Option<f32> {
    let source_graph_beat_index = beat_cursor.saturating_add(1);
    let beat_grid = graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.beat_grid.as_slice())
        .filter(|grid| !grid.is_empty())
        .unwrap_or(graph.timing.beat_grid.as_slice());
    beat_grid
        .iter()
        .find(|beat| u64::from(beat.beat_index) == source_graph_beat_index)
        .map(|beat| beat.time_seconds)
        .or_else(|| {
            preferred_beat_cursor_bpm(graph)
                .map(|_| seconds_for_beat_cursor_estimate(graph, beat_cursor))
        })
}

fn seconds_for_beat_cursor_estimate(graph: &SourceGraph, beat_cursor: u64) -> f32 {
    let bpm = preferred_beat_cursor_bpm(graph).unwrap_or(120.0);
    beat_cursor as f32 * 60.0 / bpm
}

fn preferred_beat_cursor_bpm(graph: &SourceGraph) -> Option<f32> {
    graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.bpm)
        .filter(|bpm| bpm.is_finite() && *bpm > 0.0)
        .or_else(|| {
            graph
                .timing
                .bpm_estimate
                .filter(|bpm| bpm.is_finite() && *bpm > 0.0)
        })
}

fn seconds_to_frame(seconds: f32, sample_rate: u32) -> u64 {
    (seconds.max(0.0) * sample_rate as f32).floor() as u64
}

pub(in crate::jam_app) fn apply_capture_promotion_side_effects(
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

fn capture_note(action: &Action, source_window: Option<&CaptureSourceWindow>) -> String {
    let base = match &action.explanation {
        Some(explanation) if !explanation.is_empty() => explanation.clone(),
        _ => format!("capture committed from {}", action.command),
    };
    match source_window.and_then(|window| window.hook_selection.as_ref()) {
        Some(selection) => format!(
            "{base} | w30 hook policy={:?} reason={:?} bar={:?} lift={:?}",
            selection.policy, selection.reason, selection.selected_bar_index, selection.score_lift
        ),
        None => base,
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

pub(in crate::jam_app) fn capture_promotion_summary(
    session: &SessionFile,
    action: &Action,
) -> Option<String> {
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

pub(in crate::jam_app) fn capture_targets_w30_pad(capture: &CaptureRef) -> bool {
    matches!(capture.assigned_target, Some(CaptureTarget::W30Pad { .. }))
}

pub(in crate::jam_app) fn capture_targets_specific_w30_pad(
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

#[cfg(test)]
mod hook_selection_tests {
    use riotbox_core::{
        action::{
            Action, ActionCommand, ActionParams, ActionStatus, ActionTarget, ActorType,
            Quantization, UndoPolicy,
        },
        ids::ActionId,
        session::{SessionFile, W30HookSelectionPolicy, W30HookSelectionReason},
        source_graph::{
            DecodeProfile, GraphProvenance, SourceDescriptor, SourceGraph,
            W30HookCandidateEvidence, W30HookCandidateStatus, W30HookFeatures,
        },
        style::PerformancePresetId,
    };

    use super::select_w30_hook_window;

    #[test]
    fn frozen_policy_selects_earliest_candidate_only_above_lift_gate() {
        let mut session = session(W30HookSelectionPolicy::AttackBodyContrastV1);
        let graph = graph_with_scores(&[(1, 0.20), (3, 0.75), (2, 0.75)]);
        let action = capture_action();

        let (start, end, decision) = select_w30_hook_window(&session, &graph, &action, 0.0, 2.0, 4);
        let decision = decision.expect("selection decision");

        assert_eq!((start, end), (2.0, 4.0));
        assert_eq!(decision.reason, W30HookSelectionReason::CandidateSelected);
        assert_eq!(decision.selected_bar_index, Some(2));
        assert!((decision.score_lift.expect("lift") - 0.55).abs() < 1.0e-6);

        session.runtime_state.style.w30_hook_selection_policy =
            W30HookSelectionPolicy::RepetitionSalienceV1;
        let (start, end, decision) = select_w30_hook_window(&session, &graph, &action, 0.0, 2.0, 4);
        assert_eq!((start, end), (0.0, 2.0));
        assert_eq!(
            decision.expect("retained decision").reason,
            W30HookSelectionReason::ScoreLiftBelowMinimum
        );
    }

    #[test]
    fn policy_never_changes_non_bar_group_capture() {
        let session = session(W30HookSelectionPolicy::AttackBodyContrastV1);
        let graph = graph_with_scores(&[(1, 0.10), (2, 0.90)]);
        let mut action = capture_action();
        action.command = ActionCommand::CaptureNow;

        let (start, end, decision) = select_w30_hook_window(&session, &graph, &action, 0.0, 2.0, 4);

        assert_eq!((start, end, decision), (0.0, 2.0, None));
    }

    fn session(policy: W30HookSelectionPolicy) -> SessionFile {
        let mut session = SessionFile::new("session", "test", "2026-08-14T00:00:00Z");
        session.runtime_state.style.active_preset = Some(PerformancePresetId::FeralBreakAlphaV2);
        session.runtime_state.style.w30_hook_selection_policy = policy;
        session
    }

    fn capture_action() -> Action {
        Action {
            id: ActionId(1),
            actor: ActorType::User,
            command: ActionCommand::CaptureBarGroup,
            params: ActionParams::Capture { bars: Some(1) },
            target: ActionTarget::default(),
            requested_at: 1,
            quantization: Quantization::NextBar,
            committed_at: None,
            status: ActionStatus::Queued,
            result: None,
            undo_policy: UndoPolicy::Undoable,
            explanation: None,
        }
    }

    fn graph_with_scores(scores: &[(u32, f32)]) -> SourceGraph {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: "source".into(),
                path: "ignored.wav".into(),
                content_hash: "hash".into(),
                duration_seconds: 8.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "test".into(),
                provider_set: vec![],
                generated_at: "2026-08-14T00:00:00Z".into(),
                source_hash: "hash".into(),
                analysis_seed: 1,
                run_notes: None,
            },
        );
        graph.w30_hook_candidates = scores
            .iter()
            .map(|(bar_index, attack_score)| W30HookCandidateEvidence {
                bar_index: *bar_index,
                start_seconds: (*bar_index - 1) as f32 * 2.0,
                end_seconds: *bar_index as f32 * 2.0,
                downbeat_confidence: 1.0,
                bar_rms: 0.1,
                status: W30HookCandidateStatus::Eligible,
                raw_features: W30HookFeatures::default(),
                normalized_features: Some(W30HookFeatures::default()),
                attack_body_contrast_score: Some(*attack_score),
                repetition_salience_score: Some(0.5),
            })
            .collect();
        graph
    }
}
