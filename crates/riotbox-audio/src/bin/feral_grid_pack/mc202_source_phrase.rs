use super::{
    grid::Grid,
    mc202_source_contour::{Mc202SourceContourProfile, mc202_source_low_dominance},
    source_aware_tr909::SourceAwareTr909Profile,
};
use riotbox_audio::{
    mc202::{
        Mc202ContourHint, Mc202NoteBudget, Mc202PhraseShape, Mc202RenderMode, Mc202RenderRouting,
        Mc202RenderState, Mc202SourcePhraseRenderPlan,
    },
    tr909::Tr909SourceSupportProfile,
};

pub(super) const MC202_SOURCE_EXPRESSION_ROLE_BASS_PRESSURE: &str = "bass_pressure";

pub(super) const MC202_SOURCE_EXPRESSION_ROLE_ANSWER_LIFT: &str = "answer_lift";

pub(super) const MC202_SOURCE_EXPRESSION_ROLE_HOOK_RESTRAINT_HOLD: &str = "hook_restraint_hold";

pub(super) fn mc202_source_expression_role(
    source_contour: Mc202SourceContourProfile,
) -> &'static str {
    match source_contour.contour_hint {
        Mc202ContourHint::Drop => MC202_SOURCE_EXPRESSION_ROLE_BASS_PRESSURE,
        Mc202ContourHint::Lift => MC202_SOURCE_EXPRESSION_ROLE_ANSWER_LIFT,
        Mc202ContourHint::Hold | Mc202ContourHint::Neutral => {
            MC202_SOURCE_EXPRESSION_ROLE_HOOK_RESTRAINT_HOLD
        }
    }
}

fn mc202_source_expression_render_plan(
    source_contour: Mc202SourceContourProfile,
    support_profile: Tr909SourceSupportProfile,
    bar: u32,
) -> Mc202SourcePhraseRenderPlan {
    let density = source_contour.event_density_per_bar.clamp(0.0, 8.0) / 8.0;
    let low = source_contour.low_band_energy_ratio.clamp(0.0, 1.0);
    let mid = source_contour.mid_band_energy_ratio.clamp(0.0, 1.0);
    let high = source_contour.high_band_energy_ratio.clamp(0.0, 1.0);
    let low_dominance = mc202_source_low_dominance(source_contour);
    let dense_transient_drop = source_contour.event_density_per_bar >= 48.0 && low_dominance < 0.58;
    let low_heavy_drop = low_dominance >= 0.58;
    let bar_alt = if bar.is_multiple_of(2) { 0 } else { 1 };

    let mut semitones = [0_i8; 16];
    let (
        active_mask,
        accent_mask,
        destructive_mask,
        pressure,
        contrast,
        bass_weight,
        stab_bite,
        gate_snap,
    ) = match source_contour.contour_hint {
        Mc202ContourHint::Drop => {
            let dense_transient_trim = if dense_transient_drop { 0.045 } else { 0.0 };
            semitones[0] = -24;
            semitones[4] = -19 + bar_alt;
            semitones[8] = -22;
            semitones[12] = -17 - bar_alt;
            if dense_transient_drop {
                semitones[6] = -12 + bar_alt;
                semitones[14] = -24;
            } else if low_heavy_drop {
                semitones[10] = -24;
            } else if density > 0.18 {
                semitones[14] = -24;
            }
            let mut active_mask = (1_u16 << 0) | (1_u16 << 4) | (1_u16 << 8) | (1_u16 << 12);
            if dense_transient_drop {
                active_mask |= (1_u16 << 6) | (1_u16 << 14);
            } else if low_heavy_drop {
                active_mask |= 1_u16 << 10;
            } else if density > 0.18 {
                active_mask |= 1_u16 << 14;
            }
            let mut accent_mask = (1_u16 << 0) | (1_u16 << 4) | (1_u16 << 12);
            if dense_transient_drop {
                accent_mask |= 1_u16 << 6;
            }
            (
                active_mask,
                accent_mask,
                if low_dominance > 0.42 {
                    0b0000_0001_0000_0000
                } else {
                    0
                },
                (0.72 + low * 0.22 + low_dominance * 0.18 - dense_transient_trim).clamp(0.0, 1.0),
                (0.42 + density * 0.24).clamp(0.0, 1.0),
                (0.74 + low * 0.24 - dense_transient_trim).clamp(0.0, 1.0),
                (0.12 + high * 0.18).clamp(0.0, 1.0),
                (0.10 + density * 0.18).clamp(0.0, 1.0),
            )
        }
        Mc202ContourHint::Lift => {
            semitones[2] = -7 + bar_alt;
            semitones[6] = 0;
            semitones[10] = 5;
            semitones[14] = 7 - bar_alt;
            if density > 0.42 {
                semitones[15] = 12;
            }
            (
                if density > 0.42 {
                    0b1100_0100_0100_0100
                } else {
                    0b0100_0100_0100_0100
                },
                0b0100_0100_0000_0100,
                if high > 0.38 {
                    0b0000_0100_0000_0000
                } else {
                    0
                },
                (0.38 + low * 0.18).clamp(0.0, 1.0),
                (0.58 + high * 0.28 + density * 0.16).clamp(0.0, 1.0),
                (0.24 + low * 0.18).clamp(0.0, 1.0),
                (0.54 + high * 0.36 + density * 0.12).clamp(0.0, 1.0),
                (0.42 + high * 0.30 + density * 0.18).clamp(0.0, 1.0),
            )
        }
        Mc202ContourHint::Hold | Mc202ContourHint::Neutral => {
            semitones[0] = -12;
            semitones[8] = -7 + bar_alt;
            if mid > 0.58 || density > 0.20 {
                semitones[11] = -5;
            }
            (
                if mid > 0.58 || density > 0.20 {
                    0b0000_1001_0000_0001
                } else {
                    0b0000_0001_0000_0001
                },
                0b0000_0001_0000_0001,
                0,
                (0.46 + low * 0.18).clamp(0.0, 1.0),
                (0.34 + mid * 0.18).clamp(0.0, 1.0),
                (0.38 + low * 0.18).clamp(0.0, 1.0),
                (0.22 + high * 0.24).clamp(0.0, 1.0),
                (0.18 + mid * 0.18).clamp(0.0, 1.0),
            )
        }
    };

    let profile_pressure = match support_profile {
        Tr909SourceSupportProfile::DropDrive => 0.10,
        Tr909SourceSupportProfile::BreakLift => 0.04,
        Tr909SourceSupportProfile::SteadyPulse => 0.06,
    };

    Mc202SourcePhraseRenderPlan {
        active_mask,
        semitones,
        accent_mask,
        destructive_mask,
        pressure: (pressure + profile_pressure).clamp(0.0, 1.0),
        contrast,
        bass_weight,
        stab_bite,
        gate_snap,
    }
}

pub(super) fn mc202_bass_pressure_state(
    grid: &Grid,
    profile: SourceAwareTr909Profile,
    source_contour: Option<Mc202SourceContourProfile>,
    bar: u32,
) -> Mc202RenderState {
    let (mode, primary_shape, note_budget, touch, music_bus_level, contour_hint) =
        match profile.support_profile {
            Tr909SourceSupportProfile::DropDrive => (
                Mc202RenderMode::Pressure,
                Mc202PhraseShape::FollowerDrive,
                Mc202NoteBudget::Balanced,
                0.46,
                0.12,
                Mc202ContourHint::Drop,
            ),
            Tr909SourceSupportProfile::BreakLift => (
                Mc202RenderMode::Follower,
                Mc202PhraseShape::FollowerDrive,
                Mc202NoteBudget::Sparse,
                0.48,
                0.11,
                Mc202ContourHint::Lift,
            ),
            Tr909SourceSupportProfile::SteadyPulse => (
                Mc202RenderMode::Follower,
                Mc202PhraseShape::RootPulse,
                Mc202NoteBudget::Balanced,
                0.44,
                0.10,
                Mc202ContourHint::Neutral,
            ),
        };
    let note_budget = source_contour
        .map(|contour| contour.note_budget)
        .unwrap_or(note_budget);
    let contour_hint = source_contour
        .map(|contour| contour.contour_hint)
        .unwrap_or(contour_hint);
    let touch = source_contour
        .map(|contour| (touch + contour.touch_boost).clamp(0.0, 1.0))
        .unwrap_or(touch);
    let music_bus_level = source_contour
        .map(|contour| (music_bus_level + contour.music_bus_boost).clamp(0.0, 1.0))
        .unwrap_or(music_bus_level);
    let music_bus_level = if let Some(contour) = source_contour {
        let source_expression_bus_scale = match contour.contour_hint {
            Mc202ContourHint::Drop => 0.28,
            Mc202ContourHint::Lift => 0.34,
            Mc202ContourHint::Hold | Mc202ContourHint::Neutral => 0.46,
        };
        music_bus_level * source_expression_bus_scale
    } else {
        music_bus_level
    };
    let phrase_shape = if bar % 2 == 1 {
        match primary_shape {
            Mc202PhraseShape::RootPulse => Mc202PhraseShape::FollowerDrive,
            _ => Mc202PhraseShape::MutatedDrive,
        }
    } else {
        primary_shape
    };

    Mc202RenderState {
        mode,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape,
        note_budget,
        contour_hint,
        source_phrase_plan: source_contour.map(|contour| {
            mc202_source_expression_render_plan(contour, profile.support_profile, bar)
        }),
        touch,
        music_bus_level,
        tempo_bpm: grid.bpm,
        position_beats: 0.0,
        is_transport_running: true,
        ..Mc202RenderState::default()
    }
}
