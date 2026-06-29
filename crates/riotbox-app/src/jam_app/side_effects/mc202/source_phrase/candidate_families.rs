use riotbox_core::{
    session::{
        Mc202RoleState, Mc202SourcePhraseCandidateFamilyState,
        Mc202SourcePhraseCandidateScoreState, Mc202SourcePhraseExpressionState,
        Mc202SourcePhraseNoteBudgetState, Mc202SourcePhrasePlanState,
    },
    source_graph::{Mc202SourcePhraseFeatureVector, PhraseSpan, Section, SourceGraph},
};

use super::{
    add_source_phrase_accent, source_phrase_contour_offset, source_phrase_fingerprint,
    source_phrase_note_budget,
};

mod groove_map;
mod scoring;

use groove_map::SourcePhraseGrooveMap;
use scoring::{
    candidate_score, candidate_scorecards, phrase_memory_distance, phrase_memory_rejection_reason,
};

pub(super) struct Mc202SourcePhraseCandidateSelection {
    pub rhythm_cells: [Option<i8>; 16],
    pub note_budget: Mc202SourcePhraseNoteBudgetState,
    pub candidate_family: Mc202SourcePhraseCandidateFamilyState,
    pub candidate_count: u8,
    pub rejected_candidate_count: u8,
    pub provenance_refs: Vec<String>,
    pub scorecards: Vec<Mc202SourcePhraseCandidateScoreState>,
    pub phrase_memory_distance: f32,
    pub fallback_reason: Option<String>,
}

struct Mc202SourcePhraseCandidate {
    family: Mc202SourcePhraseCandidateFamilyState,
    cells: [Option<i8>; 16],
    score: f32,
    rejection_reason: Option<&'static str>,
    phrase_memory: f32,
}

pub(super) fn choose_source_phrase_candidate(
    graph: &SourceGraph,
    role: Mc202RoleState,
    section: Option<&Section>,
    phrase_slot: &PhraseSpan,
    features: &Mc202SourcePhraseFeatureVector,
    expression: &Mc202SourcePhraseExpressionState,
    previous_plan: Option<&Mc202SourcePhrasePlanState>,
) -> Mc202SourcePhraseCandidateSelection {
    let contour = source_phrase_contour_offset(section, expression);
    let fingerprint = source_phrase_fingerprint(graph, section, phrase_slot);
    let groove = SourcePhraseGrooveMap::from_graph(graph, phrase_slot, expression, fingerprint);
    let mut candidates = [
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::SubPressureShove,
            expression,
            groove,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer,
            expression,
            groove,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::CallBackStab,
            expression,
            groove,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer,
            expression,
            groove,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator,
            expression,
            groove,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::StayOut,
            expression,
            groove,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::FallbackControl,
            expression,
            groove,
        ),
    ];

    for candidate in &mut candidates {
        candidate.phrase_memory = phrase_memory_distance(previous_plan, candidate);
        candidate.rejection_reason = candidate
            .rejection_reason
            .or_else(|| phrase_memory_rejection_reason(previous_plan, candidate));
        candidate.score = candidate_score(
            candidate.family,
            role,
            expression,
            candidate.phrase_memory,
            candidate.rejection_reason,
        );
    }

    let candidate_count = candidates.len() as u8;
    let rejected_candidate_count = candidates
        .iter()
        .filter(|candidate| candidate.rejection_reason.is_some())
        .count() as u8;
    let selected_index = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.rejection_reason.is_none())
        .max_by(|(_, left), (_, right)| left.score.total_cmp(&right.score))
        .map(|(index, _)| index)
        .unwrap_or_else(|| {
            candidates
                .iter()
                .position(|candidate| {
                    candidate.family == Mc202SourcePhraseCandidateFamilyState::FallbackControl
                })
                .unwrap_or(0)
        });
    let (family, rhythm_cells, selected_rejection_reason) = {
        let selected = &mut candidates[selected_index];
        let family = selected.family;

        if family.is_source_derived() {
            for cell in selected.cells.iter_mut().flatten() {
                *cell = (*cell + contour).clamp(-24, 24);
            }
            add_source_phrase_accent(role, &mut selected.cells, expression, fingerprint);
        }

        (family, selected.cells, selected.rejection_reason)
    };

    Mc202SourcePhraseCandidateSelection {
        rhythm_cells,
        note_budget: candidate_note_budget(role, section, expression, family),
        candidate_family: family,
        candidate_count,
        rejected_candidate_count,
        provenance_refs: candidate_provenance_refs(
            features,
            expression,
            groove,
            &candidates,
            selected_index,
        ),
        scorecards: candidate_scorecards(role, expression, &candidates, selected_index),
        phrase_memory_distance: candidates[selected_index].phrase_memory,
        fallback_reason: candidate_fallback_reason(family, selected_rejection_reason),
    }
}

fn build_candidate(
    family: Mc202SourcePhraseCandidateFamilyState,
    expression: &Mc202SourcePhraseExpressionState,
    groove: SourcePhraseGrooveMap,
) -> Mc202SourcePhraseCandidate {
    let rejection_reason = candidate_rejection_reason(family, expression);
    let mut cells = [None; 16];

    match family {
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => {
            cells[groove.pressure_step] = Some(source_pressure_root(expression));
            cells[groove.secondary_pressure_step()] = Some(source_pressure_secondary(expression));
            if expression.low_pressure_contour > 0.54 && expression.bass_pressure > 0.56 {
                cells[groove.pressure_movement_step()] = Some(source_pressure_movement(expression));
            }
            if expression.offbeat_answer_space > 0.42 {
                cells[groove.answer_step] = Some(-7);
            }
        }
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer => {
            cells[groove.answer_step] = Some(if expression.hook_restraint > 0.62 {
                7
            } else {
                5
            });
            if expression.transient_backbeat > 0.35 || expression.offbeat_answer_space > 0.62 {
                cells[groove.backbeat_answer_step()] = Some(3);
            }
            if expression.bass_pressure > 0.64 && expression.hook_restraint < 0.70 {
                cells[groove.secondary_pressure_step()] = Some(0);
            }
        }
        Mc202SourcePhraseCandidateFamilyState::CallBackStab => {
            cells[groove.callback_step] = Some(0);
            cells[groove.callback_tail_step()] =
                Some(if expression.stab_bite > 0.55 { 7 } else { 5 });
            if expression.offbeat_answer_space > 0.32 {
                cells[groove.answer_tail_step()] =
                    Some(if expression.stab_bite > 0.50 { 10 } else { 7 });
            }
        }
        Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer => {
            cells[groove.hook_safe_step] = Some(7);
            if expression.transient_backbeat > 0.52 || expression.stab_bite > 0.54 {
                cells[groove.hook_safe_step.wrapping_add(3) % 16] = Some(12);
            }
        }
        Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            cells[groove.fill_pickup_step] = Some(19);
            cells[groove.answer_step] = Some(24);
            if expression.bass_pressure > 0.55 {
                cells[groove.pressure_step] = Some(12);
            }
        }
        Mc202SourcePhraseCandidateFamilyState::StayOut => {}
        Mc202SourcePhraseCandidateFamilyState::FallbackControl => {
            cells[0] = Some(0);
            cells[8] = Some(7);
        }
    }

    Mc202SourcePhraseCandidate {
        family,
        cells,
        score: 0.0,
        rejection_reason,
        phrase_memory: 1.0,
    }
}

fn source_pressure_root(expression: &Mc202SourcePhraseExpressionState) -> i8 {
    if expression.bass_pressure > 0.78 && expression.low_pressure_contour > 0.52 {
        -14
    } else {
        -12
    }
}

fn source_pressure_secondary(expression: &Mc202SourcePhraseExpressionState) -> i8 {
    if expression.low_pressure_contour > 0.78 {
        -19
    } else if expression.low_pressure_contour > 0.54 {
        -16
    } else if expression.bass_pressure > 0.72 {
        -14
    } else {
        -10
    }
}

fn source_pressure_movement(expression: &Mc202SourcePhraseExpressionState) -> i8 {
    if expression.low_pressure_contour > 0.82 {
        -22
    } else if expression.low_pressure_contour > 0.66 {
        -19
    } else {
        -16
    }
}

fn candidate_rejection_reason(
    family: Mc202SourcePhraseCandidateFamilyState,
    expression: &Mc202SourcePhraseExpressionState,
) -> Option<&'static str> {
    match family {
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => {
            (expression.bass_pressure < 0.48 || expression.stay_out_pressure > 0.78)
                .then_some("insufficient_low_band_source_pressure")
        }
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer => {
            (expression.offbeat_answer_space < 0.25 || expression.hook_restraint >= 0.80)
                .then_some("insufficient_offbeat_answer_space")
        }
        Mc202SourcePhraseCandidateFamilyState::CallBackStab => (expression.transient_backbeat
            < 0.24
            || expression.stab_bite < 0.12
            || expression.phrase_density < 0.22)
            .then_some("insufficient_transient_callback"),
        Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer => {
            (expression.hook_restraint < 0.55 || expression.phrase_density < 0.22)
                .then_some("insufficient_hook_restraint_context")
        }
        Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            (expression.transient_backbeat < 0.42 && expression.offbeat_answer_space < 0.45)
                .then_some("insufficient_pickup_or_fill_energy")
        }
        Mc202SourcePhraseCandidateFamilyState::StayOut => (expression.stay_out_pressure < 0.80
            && expression.hook_restraint < 0.82
            && (expression.phrase_density >= 0.22
                || expression.bass_pressure >= 0.45
                || expression.offbeat_answer_space >= 0.30
                || expression.transient_backbeat >= 0.30))
            .then_some("source_context_wants_audible_phrase"),
        Mc202SourcePhraseCandidateFamilyState::FallbackControl => {
            Some("control_template_not_source_derived")
        }
    }
}

fn candidate_note_budget(
    role: Mc202RoleState,
    section: Option<&Section>,
    expression: &Mc202SourcePhraseExpressionState,
    family: Mc202SourcePhraseCandidateFamilyState,
) -> Mc202SourcePhraseNoteBudgetState {
    match family {
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove
        | Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer
        | Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer
        | Mc202SourcePhraseCandidateFamilyState::StayOut => {
            Mc202SourcePhraseNoteBudgetState::Sparse
        }
        Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            Mc202SourcePhraseNoteBudgetState::Push
        }
        Mc202SourcePhraseCandidateFamilyState::CallBackStab
        | Mc202SourcePhraseCandidateFamilyState::FallbackControl => {
            source_phrase_note_budget(role, section, expression)
        }
    }
}

fn candidate_provenance_refs(
    features: &Mc202SourcePhraseFeatureVector,
    expression: &Mc202SourcePhraseExpressionState,
    groove: SourcePhraseGrooveMap,
    candidates: &[Mc202SourcePhraseCandidate],
    selected_index: usize,
) -> Vec<String> {
    let mut refs = Vec::new();
    let selected = &candidates[selected_index];
    refs.push(format!("candidate_family:{}", selected.family.label()));
    refs.push(format!(
        "candidate_score:{:.3}",
        selected.score.clamp(0.0, 1.0)
    ));
    refs.push(format!(
        "phrase_memory_selected_distance:{:.3}",
        selected.phrase_memory.clamp(0.0, 1.0)
    ));
    refs.push(format!(
        "candidate_source_features:low={:.3}:transient={:.3}:offbeat={:.3}:hook={:.3}:strength={:.3}",
        features.low_band_pressure,
        features.transient_density,
        features.offbeat_density,
        features.hook_restraint,
        features.source_strength
    ));
    refs.push(format!(
        "candidate_source_expression:bass={:.3}:backbeat={:.3}:answer={:.3}:density={:.3}:hook={:.3}:bite={:.3}:stay={:.3}",
        expression.bass_pressure,
        expression.transient_backbeat,
        expression.offbeat_answer_space,
        expression.phrase_density,
        expression.hook_restraint,
        expression.stab_bite,
        expression.stay_out_pressure,
    ));
    refs.extend(groove.provenance_refs());

    refs.extend(
        features
            .provenance_refs
            .iter()
            .take(8)
            .map(|reference| format!("source_feature:{reference}")),
    );
    refs.extend(
        candidates
            .iter()
            .filter_map(|candidate| {
                candidate.rejection_reason.map(|reason| {
                    format!("candidate_rejected:{}:{reason}", candidate.family.label())
                })
            })
            .take(8),
    );
    refs
}

fn candidate_fallback_reason(
    family: Mc202SourcePhraseCandidateFamilyState,
    selected_rejection_reason: Option<&'static str>,
) -> Option<String> {
    selected_rejection_reason
        .map(str::to_owned)
        .or_else(|| match family {
            Mc202SourcePhraseCandidateFamilyState::StayOut => {
                Some("stay_out_candidate_family".into())
            }
            Mc202SourcePhraseCandidateFamilyState::FallbackControl => {
                Some("fallback_control_candidate_family".into())
            }
            _ => None,
        })
}
