use riotbox_audio::mc202::Mc202SourcePhraseRenderPlan;
use riotbox_core::session::Mc202SourcePhrasePlanState;

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
    use riotbox_core::session::Mc202SourcePhraseCandidateFamilyState as Family;

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

fn highest_active_bit(mask: u16) -> u16 {
    (0..16)
        .rev()
        .find_map(|index| {
            let bit = 1_u16 << index;
            (mask & bit != 0).then_some(bit)
        })
        .unwrap_or(0)
}
