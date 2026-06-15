use riotbox_audio::mc202::Mc202SourcePhraseRenderPlan;
use riotbox_core::session::{
    Mc202SourcePhraseCandidateFamilyState as Family, Mc202SourcePhrasePlanState,
};

pub(super) fn mc202_source_phrase_render_plan(
    source_plan: Option<&Mc202SourcePhrasePlanState>,
) -> Option<Mc202SourcePhraseRenderPlan> {
    let source_plan = source_plan.filter(|plan| plan.is_source_derived())?;
    let mut active_mask = 0_u16;
    let mut semitones = [0_i8; 16];
    for (index, cell) in source_plan.rhythm_cells.iter().enumerate() {
        let Some(semitone) = cell else {
            continue;
        };
        active_mask |= 1_u16 << index;
        semitones[index] = *semitone;
    }

    (active_mask != 0).then_some(Mc202SourcePhraseRenderPlan {
        active_mask,
        semitones,
        accent_mask: mc202_source_phrase_accent_mask(source_plan, active_mask),
        destructive_mask: mc202_source_phrase_destructive_mask(source_plan, active_mask),
        pressure: mc202_source_phrase_render_pressure(source_plan),
        contrast: mc202_source_phrase_render_contrast(source_plan),
        bass_weight: mc202_source_phrase_bass_weight(source_plan),
        stab_bite: mc202_source_phrase_stab_bite(source_plan),
        gate_snap: mc202_source_phrase_gate_snap(source_plan),
    })
}

fn mc202_source_phrase_render_pressure(source_plan: &Mc202SourcePhrasePlanState) -> f32 {
    let selected = source_plan
        .candidate_scorecards
        .iter()
        .find(|score| score.selected);
    selected.map_or(0.35, |score| {
        (score.low_end_impact * 0.62
            + score.destructive_usefulness * 0.18
            + source_plan.touch.clamp(0.0, 1.0) * 0.20)
            .clamp(0.0, 1.0)
    })
}

fn mc202_source_phrase_render_contrast(source_plan: &Mc202SourcePhrasePlanState) -> f32 {
    let selected = source_plan
        .candidate_scorecards
        .iter()
        .find(|score| score.selected);
    selected.map_or(0.35, |score| {
        (score.answer_contrast * 0.40
            + score.hook_avoidance * 0.20
            + score.destructive_usefulness * 0.25
            + score.phrase_memory * 0.15)
            .clamp(0.0, 1.0)
    })
}

fn mc202_source_phrase_bass_weight(source_plan: &Mc202SourcePhrasePlanState) -> f32 {
    let selected = selected_score(source_plan);
    match source_plan.candidate_family {
        Some(Family::SubPressureShove) => selected.map_or(0.70, |score| {
            (score.low_end_impact * 0.78
                + score.destructive_usefulness * 0.12
                + source_plan.touch.clamp(0.0, 1.0) * 0.10)
                .clamp(0.0, 1.0)
        }),
        Some(Family::SparseOffbeatAnswer | Family::HookRestraintGhostAnswer) => selected
            .map_or(0.28, |score| {
                (score.low_end_impact * 0.30 + score.answer_contrast * 0.12 + 0.16)
                    .clamp(0.0, 1.0)
            }),
        Some(Family::CallBackStab | Family::FillPickupInstigator) => selected.map_or(0.22, |score| {
            (score.low_end_impact * 0.24 + score.destructive_usefulness * 0.10 + 0.12)
                .clamp(0.0, 1.0)
        }),
        Some(Family::StayOut | Family::FallbackControl) | None => 0.0,
    }
}

fn mc202_source_phrase_stab_bite(source_plan: &Mc202SourcePhrasePlanState) -> f32 {
    let selected = selected_score(source_plan);
    match source_plan.candidate_family {
        Some(Family::SubPressureShove) => selected.map_or(0.18, |score| {
            (score.answer_contrast * 0.12 + score.destructive_usefulness * 0.10 + 0.12)
                .clamp(0.0, 1.0)
        }),
        Some(Family::SparseOffbeatAnswer | Family::HookRestraintGhostAnswer) => selected
            .map_or(0.58, |score| {
                (score.answer_contrast * 0.50
                    + score.hook_avoidance * 0.12
                    + score.phrase_memory * 0.10
                    + 0.20)
                    .clamp(0.0, 1.0)
            }),
        Some(Family::CallBackStab | Family::FillPickupInstigator) => selected.map_or(0.76, |score| {
            (score.destructive_usefulness * 0.42
                + score.answer_contrast * 0.24
                + score.source_grid_lock * 0.08
                + 0.26)
                .clamp(0.0, 1.0)
        }),
        Some(Family::StayOut | Family::FallbackControl) | None => 0.0,
    }
}

fn mc202_source_phrase_gate_snap(source_plan: &Mc202SourcePhrasePlanState) -> f32 {
    match source_plan.candidate_family {
        Some(Family::SubPressureShove) => 0.18,
        Some(Family::SparseOffbeatAnswer | Family::HookRestraintGhostAnswer) => 0.58,
        Some(Family::CallBackStab | Family::FillPickupInstigator) => 0.78,
        Some(Family::StayOut | Family::FallbackControl) | None => 0.0,
    }
}

fn mc202_source_phrase_accent_mask(source_plan: &Mc202SourcePhrasePlanState, active_mask: u16) -> u16 {
    let mut mask = 0_u16;
    for (index, cell) in source_plan.rhythm_cells.iter().enumerate() {
        let bit = 1_u16 << index;
        if active_mask & bit == 0 {
            continue;
        }
        if matches!(cell, Some(semitone) if *semitone <= 0) || index.is_multiple_of(4) {
            mask |= bit;
        }
    }
    if mask == 0 { active_mask } else { mask }
}

fn mc202_source_phrase_destructive_mask(
    source_plan: &Mc202SourcePhrasePlanState,
    active_mask: u16,
) -> u16 {
    let family = source_plan.candidate_family;
    let family_mask = match family {
        Some(Family::SubPressureShove) => active_mask & 0b1111_0000_1111_0000,
        Some(Family::SparseOffbeatAnswer | Family::HookRestraintGhostAnswer) => {
            active_mask & 0b1010_1010_1010_1010
        }
        Some(Family::CallBackStab | Family::FillPickupInstigator) => {
            active_mask & 0b1100_0000_1100_0000
        }
        Some(Family::StayOut | Family::FallbackControl) | None => 0,
    };
    if family_mask != 0 {
        return family_mask;
    }

    let selected = source_plan
        .candidate_scorecards
        .iter()
        .find(|score| score.selected);
    if selected.is_some_and(|score| score.destructive_usefulness > 0.42) {
        return highest_active_bit(active_mask);
    }
    0
}

fn selected_score(
    source_plan: &Mc202SourcePhrasePlanState,
) -> Option<&riotbox_core::session::Mc202SourcePhraseCandidateScoreState> {
    source_plan
        .candidate_scorecards
        .iter()
        .find(|score| score.selected)
}

fn highest_active_bit(mask: u16) -> u16 {
    (0..16)
        .rev()
        .find_map(|index| {
            let bit = 1_u16 << index;
            (mask & bit != 0).then_some(bit)
        })
        .unwrap_or(0)
}
