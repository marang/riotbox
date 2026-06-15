use riotbox_core::{
    session::{
        Mc202RoleState, Mc202SourcePhraseCandidateFamilyState,
        Mc202SourcePhraseCandidateScoreState, Mc202SourcePhrasePlanState,
    },
    source_graph::Mc202SourcePhraseFeatureVector,
};

use super::Mc202SourcePhraseCandidate;

pub(super) fn candidate_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    role: Mc202RoleState,
    features: &Mc202SourcePhraseFeatureVector,
    phrase_memory: f32,
    rejection_reason: Option<&'static str>,
) -> f32 {
    if rejection_reason.is_some() {
        return -1.0;
    }

    let base = match family {
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => {
            features.low_band_pressure * 0.52
                + features.source_strength * 0.28
                + (1.0 - features.hook_restraint) * 0.10
        }
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer => {
            features.offbeat_density * 0.48
                + features.transient_density * 0.18
                + (1.0 - features.hook_restraint).min(0.55) * 0.16
        }
        Mc202SourcePhraseCandidateFamilyState::CallBackStab => {
            features.transient_density * 0.42
                + features.source_strength * 0.26
                + features.offbeat_density * 0.14
        }
        Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer => {
            features.hook_restraint * 0.44
                + features.transient_density * 0.14
                + (1.0 - features.low_band_pressure).max(0.0) * 0.08
        }
        Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            features.transient_density * 0.44
                + features.offbeat_density * 0.24
                + features.source_strength * 0.12
        }
        Mc202SourcePhraseCandidateFamilyState::StayOut => 0.10 + features.hook_restraint * 0.30,
        Mc202SourcePhraseCandidateFamilyState::FallbackControl => -1.0,
    };

    (base
        + role_family_bias(role, family)
        + phrase_memory.clamp(0.0, 1.0) * 0.12
        + features.confidence.clamp(0.0, 1.0) * 0.08)
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
    features: &Mc202SourcePhraseFeatureVector,
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
                low_end_impact: low_end_impact_score(family, features),
                source_grid_lock: features.confidence.clamp(0.0, 1.0),
                answer_contrast: answer_contrast_score(family, features),
                hook_avoidance: hook_avoidance_score(family, features),
                phrase_memory: candidate.phrase_memory.clamp(0.0, 1.0),
                destructive_usefulness: destructive_usefulness_score(family, features),
                role_fit: (0.50 + role_family_bias(role, family)).clamp(0.0, 1.0),
                rejection_reason: candidate.rejection_reason.map(str::to_owned),
            }
        })
        .collect()
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
    features: &Mc202SourcePhraseFeatureVector,
) -> f32 {
    match family {
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => features.low_band_pressure,
        Mc202SourcePhraseCandidateFamilyState::CallBackStab
        | Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            features.low_band_pressure * 0.45
        }
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer
        | Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer
        | Mc202SourcePhraseCandidateFamilyState::StayOut
        | Mc202SourcePhraseCandidateFamilyState::FallbackControl => {
            features.low_band_pressure * 0.20
        }
    }
    .clamp(0.0, 1.0)
}

fn answer_contrast_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    features: &Mc202SourcePhraseFeatureVector,
) -> f32 {
    match family {
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer => {
            features.offbeat_density * (1.0 - features.hook_restraint * 0.45)
        }
        Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer => {
            features.hook_restraint * 0.70 + features.transient_density * 0.15
        }
        Mc202SourcePhraseCandidateFamilyState::CallBackStab => {
            features.transient_density * 0.45 + features.offbeat_density * 0.25
        }
        Mc202SourcePhraseCandidateFamilyState::StayOut => features.hook_restraint,
        _ => features.offbeat_density * 0.20,
    }
    .clamp(0.0, 1.0)
}

fn hook_avoidance_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    features: &Mc202SourcePhraseFeatureVector,
) -> f32 {
    match family {
        Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer
        | Mc202SourcePhraseCandidateFamilyState::StayOut => features.hook_restraint,
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => {
            (1.0 - features.hook_restraint * 0.55).clamp(0.0, 1.0)
        }
        _ => (1.0 - features.hook_restraint * 0.35).clamp(0.0, 1.0),
    }
}

fn destructive_usefulness_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    features: &Mc202SourcePhraseFeatureVector,
) -> f32 {
    match family {
        Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            features.transient_density * 0.70 + features.offbeat_density * 0.20
        }
        Mc202SourcePhraseCandidateFamilyState::CallBackStab => {
            features.transient_density * 0.45 + features.source_strength * 0.25
        }
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => {
            features.low_band_pressure * 0.35 + features.source_strength * 0.20
        }
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer
        | Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer => {
            features.offbeat_density * 0.25 + features.transient_density * 0.25
        }
        Mc202SourcePhraseCandidateFamilyState::StayOut
        | Mc202SourcePhraseCandidateFamilyState::FallbackControl => 0.0,
    }
    .clamp(0.0, 1.0)
}
