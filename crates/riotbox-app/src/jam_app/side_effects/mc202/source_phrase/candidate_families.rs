use riotbox_core::{
    session::{
        Mc202RoleState, Mc202SourcePhraseCandidateFamilyState, Mc202SourcePhraseNoteBudgetState,
    },
    source_graph::{Mc202SourcePhraseFeatureVector, PhraseSpan, Section, SourceGraph},
};

use super::{
    Mc202SourcePhraseFingerprint, add_source_phrase_accent, feature_step,
    source_phrase_contour_offset, source_phrase_fingerprint, source_phrase_note_budget,
};

pub(super) struct Mc202SourcePhraseCandidateSelection {
    pub rhythm_cells: [Option<i8>; 16],
    pub note_budget: Mc202SourcePhraseNoteBudgetState,
    pub candidate_family: Mc202SourcePhraseCandidateFamilyState,
    pub candidate_count: u8,
    pub rejected_candidate_count: u8,
    pub provenance_refs: Vec<String>,
    pub fallback_reason: Option<String>,
}

struct Mc202SourcePhraseCandidate {
    family: Mc202SourcePhraseCandidateFamilyState,
    cells: [Option<i8>; 16],
    score: f32,
    rejection_reason: Option<&'static str>,
}

pub(super) fn choose_source_phrase_candidate(
    graph: &SourceGraph,
    role: Mc202RoleState,
    section: Option<&Section>,
    phrase_slot: &PhraseSpan,
    features: &Mc202SourcePhraseFeatureVector,
) -> Mc202SourcePhraseCandidateSelection {
    let contour = source_phrase_contour_offset(section, features);
    let fingerprint = source_phrase_fingerprint(graph, section, phrase_slot);
    let mut candidates = [
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::SubPressureShove,
            role,
            features,
            fingerprint,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer,
            role,
            features,
            fingerprint,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::CallBackStab,
            role,
            features,
            fingerprint,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer,
            role,
            features,
            fingerprint,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator,
            role,
            features,
            fingerprint,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::StayOut,
            role,
            features,
            fingerprint,
        ),
        build_candidate(
            Mc202SourcePhraseCandidateFamilyState::FallbackControl,
            role,
            features,
            fingerprint,
        ),
    ];

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
            add_source_phrase_accent(role, &mut selected.cells, features, fingerprint);
        }

        (family, selected.cells, selected.rejection_reason)
    };

    Mc202SourcePhraseCandidateSelection {
        rhythm_cells,
        note_budget: candidate_note_budget(role, section, features, family),
        candidate_family: family,
        candidate_count,
        rejected_candidate_count,
        provenance_refs: candidate_provenance_refs(features, &candidates, selected_index),
        fallback_reason: candidate_fallback_reason(family, selected_rejection_reason),
    }
}

fn build_candidate(
    family: Mc202SourcePhraseCandidateFamilyState,
    role: Mc202RoleState,
    features: &Mc202SourcePhraseFeatureVector,
    fingerprint: Mc202SourcePhraseFingerprint,
) -> Mc202SourcePhraseCandidate {
    let rejection_reason = candidate_rejection_reason(family, features);
    let mut cells = [None; 16];

    match family {
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => {
            cells[feature_step(features.low_band_pressure, fingerprint.step_rotation, 0)] =
                Some(-12);
            cells[feature_step(features.source_strength, fingerprint.accent_step, 8)] =
                Some(if features.low_band_pressure > 0.72 {
                    -15
                } else {
                    -10
                });
            if features.offbeat_density > 0.38 {
                cells[feature_step(features.offbeat_density, fingerprint.accent_step, 5)] =
                    Some(-7);
            }
        }
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer => {
            cells[feature_step(
                features.offbeat_density.max(0.25),
                fingerprint.accent_step,
                3,
            )] = Some(if features.hook_restraint > 0.62 { 7 } else { 5 });
            if features.transient_density > 0.35 {
                cells[feature_step(features.transient_density, fingerprint.step_rotation, 10)] =
                    Some(3);
            }
            if features.low_band_pressure > 0.64 && features.hook_restraint < 0.70 {
                cells[feature_step(features.low_band_pressure, fingerprint.accent_step, 13)] =
                    Some(0);
            }
        }
        Mc202SourcePhraseCandidateFamilyState::CallBackStab => {
            cells[feature_step(features.transient_density, fingerprint.accent_step, 2)] = Some(0);
            cells[feature_step(features.source_strength, fingerprint.step_rotation, 9)] = Some(5);
            if features.offbeat_density > 0.32 {
                cells[feature_step(features.offbeat_density, fingerprint.accent_step, 12)] =
                    Some(7);
            }
        }
        Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer => {
            cells[feature_step(features.hook_restraint, fingerprint.accent_step, 11)] = Some(7);
            if features.transient_density > 0.52 {
                cells[feature_step(features.transient_density, fingerprint.step_rotation, 15)] =
                    Some(12);
            }
        }
        Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            cells[feature_step(features.transient_density, fingerprint.accent_step, 14)] = Some(19);
            cells[feature_step(features.offbeat_density, fingerprint.step_rotation, 6)] = Some(24);
            if features.low_band_pressure > 0.55 {
                cells[feature_step(features.low_band_pressure, fingerprint.accent_step, 0)] =
                    Some(12);
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
        score: candidate_score(family, role, features, rejection_reason),
        rejection_reason,
    }
}

fn candidate_rejection_reason(
    family: Mc202SourcePhraseCandidateFamilyState,
    features: &Mc202SourcePhraseFeatureVector,
) -> Option<&'static str> {
    match family {
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove => {
            (features.low_band_pressure < 0.48 || features.source_strength < 0.35)
                .then_some("insufficient_low_band_source_pressure")
        }
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer => {
            (features.offbeat_density < 0.30 || features.hook_restraint >= 0.80)
                .then_some("insufficient_offbeat_answer_space")
        }
        Mc202SourcePhraseCandidateFamilyState::CallBackStab => (features.transient_density < 0.28
            || features.source_strength < 0.36)
            .then_some("insufficient_transient_callback"),
        Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer => {
            (features.hook_restraint < 0.55 || features.source_strength < 0.35)
                .then_some("insufficient_hook_restraint_context")
        }
        Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator => {
            (features.transient_density < 0.42 && features.offbeat_density < 0.45)
                .then_some("insufficient_pickup_or_fill_energy")
        }
        Mc202SourcePhraseCandidateFamilyState::StayOut => (!features.stay_out
            && features.hook_restraint < 0.82
            && features.source_strength >= 0.32)
            .then_some("source_context_wants_audible_phrase"),
        Mc202SourcePhraseCandidateFamilyState::FallbackControl => {
            Some("control_template_not_source_derived")
        }
    }
}

fn candidate_score(
    family: Mc202SourcePhraseCandidateFamilyState,
    role: Mc202RoleState,
    features: &Mc202SourcePhraseFeatureVector,
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

    (base + role_family_bias(role, family)).clamp(0.0, 1.0)
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

fn candidate_note_budget(
    role: Mc202RoleState,
    section: Option<&Section>,
    features: &Mc202SourcePhraseFeatureVector,
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
            source_phrase_note_budget(role, section, features)
        }
    }
}

fn candidate_provenance_refs(
    features: &Mc202SourcePhraseFeatureVector,
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
        "candidate_source_features:low={:.3}:transient={:.3}:offbeat={:.3}:hook={:.3}:strength={:.3}",
        features.low_band_pressure,
        features.transient_density,
        features.offbeat_density,
        features.hook_restraint,
        features.source_strength
    ));

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
