use super::render_types::{Mc202RenderMode, Mc202RenderState, Mc202SourcePhraseRenderPlan};

#[derive(Copy, Clone, Debug)]
pub(super) struct Mc202SourcePhraseSoundDesign {
    pub(super) gain: f32,
    pub(super) drive: f32,
    pub(super) gate_len: f32,
    pub(super) env_curve: f32,
    pub(super) sub_mix: f32,
    pub(super) body_mix: f32,
    pub(super) upper_body_mix: f32,
    pub(super) harmonic_motion: f32,
    pub(super) detuned_body_mix: f32,
    pub(super) growl_mix: f32,
    pub(super) harmonic_cap: usize,
    pub(super) clean_body_mix: f32,
    pub(super) saw_mix: f32,
    pub(super) pulse_mix: f32,
    pub(super) bite_mix: f32,
    pub(super) transient_click: f32,
    pub(super) attack_len: f32,
    pub(super) octave_drop: f64,
    pub(super) destructive_dive: f64,
    pub(super) cut_start: f32,
}

pub(super) fn mc202_source_phrase_sound_design(
    render: &Mc202RenderState,
    plan: Mc202SourcePhraseRenderPlan,
    destructive_step: bool,
) -> Mc202SourcePhraseSoundDesign {
    let touch = render.touch.clamp(0.0, 1.0);
    let pressure = plan.pressure.clamp(0.0, 1.0);
    let contrast = plan.contrast.clamp(0.0, 1.0);
    let bass_weight = plan.bass_weight.clamp(0.0, 1.0);
    let stab_bite = plan.stab_bite.clamp(0.0, 1.0);
    let gate_snap = plan.gate_snap.clamp(0.0, 1.0);
    let source_gain =
        1.0 + pressure * 0.62 + contrast * 0.20 + bass_weight * 0.28 + stab_bite * 0.12;
    let source_gate = if render.mode == Mc202RenderMode::Pressure && destructive_step {
        0.86 + contrast * 0.08
    } else if render.mode == Mc202RenderMode::Pressure {
        2.35 + bass_weight * 0.55 + pressure * 0.35
    } else {
        (0.34 + bass_weight * 0.34 + pressure * 0.08 + contrast * 0.10)
            * (1.0 - gate_snap * 0.48)
            * (1.0 - stab_bite * 0.18)
    };
    let mode_octave = match render.mode {
        Mc202RenderMode::Pressure => -1.0 - f64::from((bass_weight - 0.50).max(0.0) * 2.0),
        Mc202RenderMode::Follower => -12.0,
        Mc202RenderMode::Instigator => -2.0,
        _ => -5.0,
    };

    Mc202SourcePhraseSoundDesign {
        gain: render.music_bus_level.clamp(0.0, 1.0) * (0.075 + touch * 0.085) * source_gain,
        drive: 1.0 + pressure * 1.35 + contrast * 0.45 + bass_weight * 1.05 + stab_bite * 0.72,
        gate_len: if render.mode == Mc202RenderMode::Pressure && destructive_step {
            source_gate.clamp(0.84, 0.94)
        } else if render.mode == Mc202RenderMode::Pressure {
            source_gate.clamp(2.35, 3.05)
        } else {
            source_gate.clamp(0.12, 0.72)
        },
        env_curve: if render.mode == Mc202RenderMode::Pressure {
            (0.68 + gate_snap * 0.30 + stab_bite * 0.12).clamp(0.68, 1.05)
        } else {
            (1.18 + gate_snap * 1.65 + stab_bite * 0.95 - bass_weight * 0.45).clamp(0.85, 3.4)
        },
        sub_mix: 0.16 + bass_weight * 0.72 + pressure * 0.20,
        body_mix: if render.mode == Mc202RenderMode::Pressure {
            0.20 + bass_weight * 0.55 + pressure * 0.12
        } else {
            (0.16 + bass_weight * 0.72 + pressure * 0.20) * 0.18
        },
        upper_body_mix: if render.mode == Mc202RenderMode::Pressure {
            0.08 + bass_weight * 0.20 + contrast * 0.15
        } else {
            0.0
        },
        harmonic_motion: if render.mode == Mc202RenderMode::Pressure {
            0.48 + contrast * 0.34
        } else {
            0.0
        },
        detuned_body_mix: if render.mode == Mc202RenderMode::Pressure {
            0.02 + bass_weight * 0.12 + pressure * 0.03
        } else {
            0.0
        },
        growl_mix: if render.mode == Mc202RenderMode::Pressure {
            0.04 + bass_weight * 0.10 + pressure * 0.08 + contrast * 0.04
        } else {
            0.0
        },
        harmonic_cap: if render.mode == Mc202RenderMode::Pressure {
            2
        } else {
            5
        },
        clean_body_mix: if render.mode == Mc202RenderMode::Pressure {
            0.24 + bass_weight * 0.36 + pressure * 0.08
        } else {
            0.0
        },
        saw_mix: 0.46 + touch * 0.22 + stab_bite * 0.20 - bass_weight * 0.08,
        pulse_mix: 0.22 + touch * 0.16 + stab_bite * 0.30 + contrast * 0.08,
        bite_mix: 0.08 + stab_bite * 0.58 + gate_snap * 0.18,
        transient_click: if render.mode == Mc202RenderMode::Pressure {
            0.12 + pressure * 0.20 + contrast * 0.08
        } else if render.mode == Mc202RenderMode::Instigator {
            0.10 + stab_bite * 0.75 + gate_snap * 0.34 + contrast * 0.07
        } else {
            0.07 + stab_bite * 0.78 + gate_snap * 0.36 + contrast * 0.08
        },
        attack_len: if render.mode == Mc202RenderMode::Pressure {
            (0.028 + bass_weight * 0.010 - pressure * 0.006).clamp(0.018, 0.038)
        } else if render.mode == Mc202RenderMode::Instigator {
            (0.024 - stab_bite * 0.006 - gate_snap * 0.003).clamp(0.012, 0.024)
        } else {
            (0.045 - stab_bite * 0.034 - gate_snap * 0.006 + bass_weight * 0.018)
                .clamp(0.006, 0.060)
        },
        octave_drop: if render.mode == Mc202RenderMode::Pressure {
            mode_octave
        } else {
            mode_octave - f64::from(bass_weight * 7.0) + f64::from(stab_bite * 2.5)
        },
        destructive_dive: if destructive_step && render.mode == Mc202RenderMode::Pressure {
            -4.0 - f64::from(contrast * 2.5 + stab_bite)
        } else if destructive_step {
            -10.0 - f64::from(contrast * 5.0 + stab_bite * 3.0)
        } else {
            0.0
        },
        cut_start: (0.76 - gate_snap * 0.16 - stab_bite * 0.08).clamp(0.52, 0.82),
    }
}

pub(super) fn mc202_source_phrase_sample(
    phase: f64,
    phase_increment: f32,
    step_phase: f32,
    accent: f32,
    design: Mc202SourcePhraseSoundDesign,
) -> f32 {
    if step_phase > design.gate_len {
        return 0.0;
    }

    let gate_position = (step_phase / design.gate_len).clamp(0.0, 1.0);
    let env = (1.0 - gate_position).powf(design.env_curve);
    let attack = (step_phase / design.attack_len).clamp(0.0, 1.0).powf(0.55);
    let click_env = (1.0 - (step_phase / 0.055).clamp(0.0, 1.0)).powf(2.4);
    let oscillator_phase = phase as f32;
    let saw = rounded_saw(oscillator_phase, phase_increment, design.harmonic_cap);
    let detuned_saw = rounded_saw(
        oscillator_phase * 1.007,
        phase_increment * 1.007,
        design.harmonic_cap,
    );
    let pulse = rounded_pulse(oscillator_phase, phase_increment, design.harmonic_cap);
    let sub = (phase as f32 * std::f32::consts::TAU).sin();
    let second = (phase as f32 * std::f32::consts::TAU * 2.0).sin();
    let third = (phase as f32 * std::f32::consts::TAU * 3.0).sin();
    let bite_edge = (saw - sub * 0.28 + pulse * 0.18).tanh();
    let harmonic_envelope = (1.0 - gate_position).powf(0.65);
    let harmonic_scale =
        1.0 - design.harmonic_motion + design.harmonic_motion * (0.42 + harmonic_envelope * 0.78);
    let body_scale = if design.harmonic_motion > 0.0 {
        0.82 + harmonic_envelope * 0.24
    } else {
        1.0
    };
    let internal_motion =
        1.0 + design.harmonic_motion * 0.14 * (gate_position * std::f32::consts::TAU * 1.5).sin();
    let growl =
        ((-saw * 0.82 - detuned_saw * 0.36 + pulse * 0.30) * (1.6 + design.drive * 0.15)).tanh();
    let tone = sub * design.sub_mix
        + second * design.body_mix * body_scale
        + third * design.upper_body_mix * harmonic_scale
        + saw * design.saw_mix * harmonic_scale
        - detuned_saw * design.detuned_body_mix * harmonic_scale * internal_motion
        + pulse * design.pulse_mix * harmonic_scale
        + bite_edge * design.bite_mix * harmonic_scale
        + growl * design.growl_mix * internal_motion;
    let transient = bite_edge * design.transient_click * click_env;
    let cut_start = design.cut_start * design.gate_len;
    let cut = if step_phase > cut_start {
        (1.0 - (step_phase - cut_start) / (design.gate_len - cut_start).max(f32::EPSILON))
            .clamp(0.0, 1.0)
    } else {
        1.0
    };

    let amplitude = env * attack * accent * design.gain * cut;
    let driven = ((tone * design.drive + transient) * amplitude).tanh();
    if design.clean_body_mix > 0.0 {
        let clean_body = (sub * 0.84 + second * 0.16) * amplitude * design.clean_body_mix;
        (driven + clean_body).tanh()
    } else {
        driven
    }
}

fn rounded_saw(phase: f32, phase_increment: f32, harmonic_cap: usize) -> f32 {
    let harmonics = harmonic_limit(phase_increment, harmonic_cap);
    let angle = phase * std::f32::consts::TAU;
    let mut sample = 0.0_f32;
    for harmonic in 1..=harmonics {
        sample += (angle * harmonic as f32).sin() / harmonic as f32;
    }
    (-2.0 / std::f32::consts::PI * sample).clamp(-1.0, 1.0)
}

fn rounded_pulse(phase: f32, phase_increment: f32, harmonic_cap: usize) -> f32 {
    let harmonics = harmonic_limit(phase_increment, harmonic_cap);
    let angle = phase * std::f32::consts::TAU;
    let mut sample = 0.0_f32;
    for harmonic in (1..=harmonics).step_by(2) {
        sample += (angle * harmonic as f32).sin() / harmonic as f32;
    }
    (4.0 / std::f32::consts::PI * sample).clamp(-1.0, 1.0)
}

fn harmonic_limit(phase_increment: f32, harmonic_cap: usize) -> usize {
    (0.45 / phase_increment.max(f32::EPSILON))
        .floor()
        .clamp(1.0, harmonic_cap.max(1) as f32) as usize
}
