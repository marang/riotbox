use riotbox_core::session::{
    Mc202RoleState, Mc202SourcePhraseCandidateFamilyState, Mc202SourcePhraseCandidateScoreState,
    Mc202SourcePhraseExpressionState, Mc202SourcePhrasePlanState,
};

use super::Mc202SourcePhraseCandidate;

pub(super) fn candidate_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    role: Mc202RoleState,
    expression: &Mc202SourcePhraseExpressionState,
    phrase_memory: f32,
    rejection_reason: Option<&'static str>,
) -> f32 {
    if rejection_reason.is_some() {
        return -1.0;
    }

    let source_fit = source_family_fit_score(family, expression);
    let production_impact =
        candidate_production_impact_score(family, role, expression, phrase_memory);

    (source_fit * 0.42
        + production_impact * 0.44
        + phrase_memory.clamp(0.0, 1.0) * 0.06
        + expression.confidence.clamp(0.0, 1.0) * 0.08)
        .clamp(0.0, 1.0)
}

pub(super) fn candidate_production_impact_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    role: Mc202RoleState,
    expression: &Mc202SourcePhraseExpressionState,
    phrase_memory: f32,
) -> f32 {
    if !family.is_source_derived() {
        return 0.0;
    }

    let weights = production_weights(role);
    (low_end_impact_score(family, expression) * weights.low_end
        + answer_contrast_score(family, expression) * weights.answer
        + hook_avoidance_score(family, expression) * weights.hook
        + destructive_usefulness_score(family, expression) * weights.destructive
        + expression.confidence.clamp(0.0, 1.0) * weights.source_grid
        + phrase_memory.clamp(0.0, 1.0) * weights.phrase_memory
        + role_fit_score(role, family) * weights.role_fit)
        .clamp(0.0, 1.0)
}

pub(super) fn selected_candidate_dimension_refs(
    family: Mc202SourcePhraseCandidateFamilyState,
    role: Mc202RoleState,
    expression: &Mc202SourcePhraseExpressionState,
    phrase_memory: f32,
) -> Vec<String> {
    vec![
        format!(
            "candidate_production_impact_score:{:.3}",
            candidate_production_impact_score(family, role, expression, phrase_memory)
        ),
        format!(
            "candidate_selected_dimensions:low={:.3}:grid={:.3}:answer={:.3}:hook={:.3}:memory={:.3}:destructive={:.3}:role={:.3}",
            low_end_impact_score(family, expression),
            expression.confidence.clamp(0.0, 1.0),
            answer_contrast_score(family, expression),
            hook_avoidance_score(family, expression),
            phrase_memory.clamp(0.0, 1.0),
            destructive_usefulness_score(family, expression),
            role_fit_score(role, family),
        ),
    ]
}

fn source_family_fit_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    expression: &Mc202SourcePhraseExpressionState,
) -> f32 {
    match family {
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => {
            expression.bass_pressure * 0.50
                + expression.low_pressure_contour * 0.16
                + expression.phrase_density * 0.16
                + (1.0 - expression.stay_out_pressure) * 0.10
        }
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer => {
            expression.offbeat_answer_space * 0.50
                + expression.transient_backbeat * 0.14
                + (1.0 - expression.hook_restraint).min(0.55) * 0.14
        }
        Mc202SourcePhraseCandidateFamilyState::CallBackStab => {
            expression.transient_backbeat * 0.34
                + expression.stab_bite * 0.30
                + expression.offbeat_answer_space * 0.12
        }
        Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer => {
            expression.hook_restraint * 0.42
                + expression.stab_bite * 0.14
                + (1.0 - expression.bass_pressure).max(0.0) * 0.08
        }
        Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            expression.transient_backbeat * 0.24
                + expression.offbeat_answer_space * 0.26
                + expression.phrase_density * 0.14
                + expression.stab_bite * 0.08
        }
        Mc202SourcePhraseCandidateFamilyState::StayOut => {
            0.10 + expression.stay_out_pressure * 0.26 + expression.hook_restraint * 0.16
        }
        Mc202SourcePhraseCandidateFamilyState::FallbackControl => -1.0,
    }
    .clamp(0.0, 1.0)
}

pub(super) fn phrase_memory_distance(
    previous_plan: Option<&Mc202SourcePhrasePlanState>,
    candidate: &Mc202SourcePhraseCandidate,
) -> f32 {
    let Some(previous) = previous_plan else {
        return 1.0;
    };

    let previous_family = previous.candidate_family;
    let family_distance = if previous_family == Some(candidate.family) {
        0.0
    } else {
        0.45
    };
    let cell_distance = previous
        .rhythm_cells
        .iter()
        .zip(candidate.cells.iter())
        .filter(|(left, right)| left != right)
        .count() as f32
        / 16.0;
    let activity_delta = (previous
        .rhythm_cells
        .iter()
        .filter(|cell| cell.is_some())
        .count() as f32
        - candidate.cells.iter().filter(|cell| cell.is_some()).count() as f32)
        .abs()
        / 8.0;

    (family_distance + cell_distance * 0.45 + activity_delta * 0.10).clamp(0.0, 1.0)
}

pub(super) fn phrase_memory_rejection_reason(
    previous_plan: Option<&Mc202SourcePhrasePlanState>,
    candidate: &Mc202SourcePhraseCandidate,
) -> Option<&'static str> {
    let previous = previous_plan?;
    if !candidate.family.is_source_derived() || previous.candidate_family != Some(candidate.family)
    {
        return None;
    }
    if candidate.phrase_memory < 0.12 {
        return Some("phrase_memory_static_repeat");
    }
    (candidate.phrase_memory < 0.35).then_some("phrase_memory_too_close_to_previous")
}

pub(super) fn candidate_scorecards(
    role: Mc202RoleState,
    expression: &Mc202SourcePhraseExpressionState,
    candidates: &[Mc202SourcePhraseCandidate],
    selected_index: usize,
) -> Vec<Mc202SourcePhraseCandidateScoreState> {
    candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let family = candidate.family;
            Mc202SourcePhraseCandidateScoreState {
                family,
                selected: index == selected_index,
                total_score: candidate.score.clamp(0.0, 1.0),
                low_end_impact: low_end_impact_score(family, expression),
                source_grid_lock: expression.confidence.clamp(0.0, 1.0),
                answer_contrast: answer_contrast_score(family, expression),
                hook_avoidance: hook_avoidance_score(family, expression),
                phrase_memory: candidate.phrase_memory.clamp(0.0, 1.0),
                destructive_usefulness: destructive_usefulness_score(family, expression),
                role_fit: role_fit_score(role, family),
                rejection_reason: candidate.rejection_reason.map(str::to_owned),
            }
        })
        .collect()
}

#[derive(Copy, Clone)]
struct ProductionWeights {
    low_end: f32,
    answer: f32,
    hook: f32,
    destructive: f32,
    source_grid: f32,
    phrase_memory: f32,
    role_fit: f32,
}

fn production_weights(role: Mc202RoleState) -> ProductionWeights {
    match role {
        Mc202RoleState::Pressure => ProductionWeights {
            low_end: 0.34,
            answer: 0.04,
            hook: 0.10,
            destructive: 0.20,
            source_grid: 0.14,
            phrase_memory: 0.08,
            role_fit: 0.10,
        },
        Mc202RoleState::Answer => ProductionWeights {
            low_end: 0.06,
            answer: 0.32,
            hook: 0.12,
            destructive: 0.14,
            source_grid: 0.14,
            phrase_memory: 0.10,
            role_fit: 0.12,
        },
        Mc202RoleState::Instigator => ProductionWeights {
            low_end: 0.06,
            answer: 0.10,
            hook: 0.02,
            destructive: 0.38,
            source_grid: 0.10,
            phrase_memory: 0.06,
            role_fit: 0.28,
        },
        Mc202RoleState::Leader | Mc202RoleState::Follower => ProductionWeights {
            low_end: 0.10,
            answer: 0.22,
            hook: 0.10,
            destructive: 0.22,
            source_grid: 0.16,
            phrase_memory: 0.08,
            role_fit: 0.12,
        },
    }
}

fn role_fit_score(role: Mc202RoleState, family: Mc202SourcePhraseCandidateFamilyState) -> f32 {
    (0.50 + role_family_bias(role, family)).clamp(0.0, 1.0)
}

fn role_family_bias(role: Mc202RoleState, family: Mc202SourcePhraseCandidateFamilyState) -> f32 {
    match (role, family) {
        (Mc202RoleState::Pressure, Mc202SourcePhraseCandidateFamilyState::SubPressureShove)
        | (Mc202RoleState::Answer, Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer)
        | (
            Mc202RoleState::Instigator,
            Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator,
        ) => 0.30,
        (
            Mc202RoleState::Leader | Mc202RoleState::Follower,
            Mc202SourcePhraseCandidateFamilyState::CallBackStab,
        ) => 0.12,
        (Mc202RoleState::Answer, Mc202SourcePhraseCandidateFamilyState::SubPressureShove)
        | (Mc202RoleState::Pressure, Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer) => {
            -0.18
        }
        (_, Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer) => 0.04,
        (_, Mc202SourcePhraseCandidateFamilyState::StayOut) => -0.10,
        (_, Mc202SourcePhraseCandidateFamilyState::FallbackControl) => -1.0,
        _ => 0.0,
    }
}

fn low_end_impact_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    expression: &Mc202SourcePhraseExpressionState,
) -> f32 {
    match family {
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => expression.bass_pressure,
        Mc202SourcePhraseCandidateFamilyState::CallBackStab
        | Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            expression.bass_pressure * 0.45
        }
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer
        | Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer
        | Mc202SourcePhraseCandidateFamilyState::StayOut
        | Mc202SourcePhraseCandidateFamilyState::FallbackControl => expression.bass_pressure * 0.20,
    }
    .clamp(0.0, 1.0)
}

fn answer_contrast_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    expression: &Mc202SourcePhraseExpressionState,
) -> f32 {
    match family {
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer => {
            expression.offbeat_answer_space * (1.0 - expression.hook_restraint * 0.35)
        }
        Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer => {
            expression.hook_restraint * 0.70 + expression.stab_bite * 0.15
        }
        Mc202SourcePhraseCandidateFamilyState::CallBackStab => {
            expression.transient_backbeat * 0.36 + expression.offbeat_answer_space * 0.25
        }
        Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            expression.offbeat_answer_space * 0.32 + expression.transient_backbeat * 0.20
        }
        Mc202SourcePhraseCandidateFamilyState::StayOut => expression.stay_out_pressure,
        _ => expression.offbeat_answer_space * 0.20,
    }
    .clamp(0.0, 1.0)
}

fn hook_avoidance_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    expression: &Mc202SourcePhraseExpressionState,
) -> f32 {
    match family {
        Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer
        | Mc202SourcePhraseCandidateFamilyState::StayOut => expression.hook_restraint,
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => {
            (1.0 - expression.hook_restraint * 0.55).clamp(0.0, 1.0)
        }
        _ => (1.0 - expression.hook_restraint * 0.35).clamp(0.0, 1.0),
    }
}

fn destructive_usefulness_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    expression: &Mc202SourcePhraseExpressionState,
) -> f32 {
    match family {
        Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            expression.transient_backbeat * 0.56
                + expression.offbeat_answer_space * 0.18
                + expression.stab_bite * 0.16
        }
        Mc202SourcePhraseCandidateFamilyState::CallBackStab => {
            expression.transient_backbeat * 0.40 + expression.stab_bite * 0.26
        }
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => {
            expression.bass_pressure * 0.30 + expression.low_pressure_contour * 0.18
        }
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer
        | Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer => {
            expression.offbeat_answer_space * 0.45
                + expression.stab_bite * 0.24
                + expression.transient_backbeat * 0.10
        }
        Mc202SourcePhraseCandidateFamilyState::StayOut
        | Mc202SourcePhraseCandidateFamilyState::FallbackControl => 0.0,
    }
    .clamp(0.0, 1.0)
}
