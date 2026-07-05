use riotbox_audio::mc202::Mc202SourcePhraseRenderPlan;
use riotbox_core::session::{
    Mc202RoleState as Role, Mc202SourcePhraseCandidateFamilyState as Family,
    Mc202SourcePhrasePlanState,
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
            + source_plan.touch.clamp(0.0, 1.0) * 0.20
            + role_pressure_bias(source_plan, score.low_end_impact))
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
            + score.phrase_memory * 0.15
            + role_contrast_bias(source_plan, score.destructive_usefulness))
            .clamp(0.0, 1.0)
    })
}

fn mc202_source_phrase_bass_weight(source_plan: &Mc202SourcePhrasePlanState) -> f32 {
    let selected = selected_score(source_plan);
    let bass_weight = match source_plan.candidate_family {
        Some(Family::SubPressureShove) => selected.map_or(0.70, |score| {
            let source_bass = source_expression_bass_body(source_plan, score.low_end_impact);
            (score.low_end_impact * 0.52
                + source_bass * 0.30
                + score.destructive_usefulness * 0.10
                + source_plan.touch.clamp(0.0, 1.0) * 0.08)
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
    };

    role_shaped_bass_weight(source_plan, bass_weight)
}

fn source_expression_bass_body(
    source_plan: &Mc202SourcePhrasePlanState,
    fallback_low_end_impact: f32,
) -> f32 {
    source_plan
        .source_expression
        .as_ref()
        .map_or(fallback_low_end_impact.clamp(0.0, 1.0), |expression| {
            (expression.bass_pressure * 0.72 + expression.low_pressure_contour * 0.28)
                .clamp(0.0, 1.0)
        })
}

fn mc202_source_phrase_stab_bite(source_plan: &Mc202SourcePhrasePlanState) -> f32 {
    let selected = selected_score(source_plan);
    let stab_bite = match source_plan.candidate_family {
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
    };

    role_shaped_stab_bite(source_plan, stab_bite)
}

fn mc202_source_phrase_gate_snap(source_plan: &Mc202SourcePhrasePlanState) -> f32 {
    let gate_snap = match source_plan.candidate_family {
        Some(Family::SubPressureShove) => 0.18,
        Some(Family::SparseOffbeatAnswer | Family::HookRestraintGhostAnswer) => 0.58,
        Some(Family::CallBackStab | Family::FillPickupInstigator) => 0.78,
        Some(Family::StayOut | Family::FallbackControl) | None => 0.0,
    };

    role_shaped_gate_snap(source_plan, gate_snap)
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
    mask |= role_accent_mask(source_plan, active_mask);
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
    let role_mask = role_destructive_mask(source_plan, active_mask);
    if family_mask | role_mask != 0 {
        return family_mask | role_mask;
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

fn role_pressure_bias(
    source_plan: &Mc202SourcePhrasePlanState,
    fallback_low_end_impact: f32,
) -> f32 {
    match source_plan.role {
        Role::Pressure => 0.12 + source_expression_bass_body(source_plan, fallback_low_end_impact) * 0.10,
        Role::Instigator => 0.06,
        Role::Follower | Role::Leader => 0.03,
        Role::Answer => 0.0,
    }
}

fn role_contrast_bias(
    source_plan: &Mc202SourcePhrasePlanState,
    destructive_usefulness: f32,
) -> f32 {
    let expression = source_plan.source_expression.as_ref();
    match source_plan.role {
        Role::Answer => expression.map_or(0.08, |expression| {
            0.06 + expression.offbeat_answer_space.clamp(0.0, 1.0) * 0.14
        }),
        Role::Instigator => 0.10 + destructive_usefulness.clamp(0.0, 1.0) * 0.10,
        Role::Pressure => expression.map_or(0.04, |expression| {
            expression.low_pressure_contour.clamp(0.0, 1.0) * 0.08
        }),
        Role::Follower | Role::Leader => 0.03,
    }
}

fn role_shaped_bass_weight(source_plan: &Mc202SourcePhrasePlanState, bass_weight: f32) -> f32 {
    let expression = source_plan.source_expression.as_ref();
    match source_plan.role {
        Role::Pressure => {
            let source_bass = expression.map_or(bass_weight, |expression| {
                (expression.bass_pressure * 0.70 + expression.low_pressure_contour * 0.30)
                    .clamp(0.0, 1.0)
            });
            bass_weight.max(0.58 + source_bass * 0.30).clamp(0.0, 1.0)
        }
        Role::Answer => (bass_weight * 0.54
            + expression.map_or(0.03, |expression| expression.bass_pressure * 0.06))
        .clamp(0.0, 1.0),
        Role::Instigator => (bass_weight * 0.58 + 0.12).clamp(0.0, 1.0),
        Role::Follower | Role::Leader => bass_weight.clamp(0.0, 1.0),
    }
}

fn role_shaped_stab_bite(source_plan: &Mc202SourcePhrasePlanState, stab_bite: f32) -> f32 {
    let expression = source_plan.source_expression.as_ref();
    match source_plan.role {
        Role::Pressure => (stab_bite * 0.72 + 0.06).clamp(0.0, 1.0),
        Role::Answer => {
            let answer_bite = expression.map_or(0.62, |expression| {
                0.54
                    + expression.offbeat_answer_space.clamp(0.0, 1.0) * 0.18
                    + expression.stab_bite.clamp(0.0, 1.0) * 0.10
            });
            stab_bite.max(answer_bite).clamp(0.0, 1.0)
        }
        Role::Instigator => {
            let transient_bite = expression.map_or(0.74, |expression| {
                0.64
                    + expression.transient_backbeat.clamp(0.0, 1.0) * 0.18
                    + expression.stab_bite.clamp(0.0, 1.0) * 0.12
            });
            stab_bite.max(transient_bite).clamp(0.0, 1.0)
        }
        Role::Follower | Role::Leader => stab_bite.clamp(0.0, 1.0),
    }
}

fn role_shaped_gate_snap(source_plan: &Mc202SourcePhrasePlanState, gate_snap: f32) -> f32 {
    let expression = source_plan.source_expression.as_ref();
    match source_plan.role {
        Role::Pressure => (gate_snap * 0.70 + 0.08).clamp(0.0, 1.0),
        Role::Answer => {
            let answer_snap = expression.map_or(0.62, |expression| {
                0.56 + expression.offbeat_answer_space.clamp(0.0, 1.0) * 0.18
            });
            gate_snap.max(answer_snap).clamp(0.0, 1.0)
        }
        Role::Instigator => {
            let transient_snap = expression.map_or(0.84, |expression| {
                0.74 + expression.transient_backbeat.clamp(0.0, 1.0) * 0.14
            });
            gate_snap.max(transient_snap).clamp(0.0, 1.0)
        }
        Role::Follower | Role::Leader => gate_snap.clamp(0.0, 1.0),
    }
}

fn role_accent_mask(source_plan: &Mc202SourcePhrasePlanState, active_mask: u16) -> u16 {
    match source_plan.role {
        Role::Pressure => active_mask
            & source_plan
                .rhythm_cells
                .iter()
                .enumerate()
                .filter_map(|(index, cell)| {
                    matches!(cell, Some(semitone) if *semitone <= -7).then_some(1_u16 << index)
                })
                .fold(0_u16, |mask, bit| mask | bit),
        Role::Answer => active_mask & 0b1010_1010_1010_1010,
        Role::Instigator => highest_active_bit(active_mask),
        Role::Follower | Role::Leader => 0,
    }
}

fn role_destructive_mask(source_plan: &Mc202SourcePhrasePlanState, active_mask: u16) -> u16 {
    match source_plan.role {
        Role::Pressure => active_mask
            & source_plan
                .rhythm_cells
                .iter()
                .enumerate()
                .filter_map(|(index, cell)| {
                    matches!(cell, Some(semitone) if *semitone <= -16).then_some(1_u16 << index)
                })
                .fold(0_u16, |mask, bit| mask | bit),
        Role::Instigator => highest_active_bit(active_mask),
        Role::Answer | Role::Follower | Role::Leader => 0,
    }
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
