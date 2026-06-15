#[derive(Copy, Clone, Debug)]
struct Mc202SourcePhraseSoundDesign {
    gain: f32,
    drive: f32,
    gate_len: f32,
    env_curve: f32,
    sub_mix: f32,
    saw_mix: f32,
    pulse_mix: f32,
    bite_mix: f32,
    transient_click: f32,
    attack_len: f32,
    octave_drop: f64,
    destructive_dive: f64,
    cut_start: f32,
}

fn mc202_source_phrase_sound_design(
    render: &Mc202RenderState,
    plan: Option<Mc202SourcePhraseRenderPlan>,
    destructive_step: bool,
) -> Mc202SourcePhraseSoundDesign {
    let touch = render.touch.clamp(0.0, 1.0);
    let pressure = plan.map_or(0.0, |plan| plan.pressure.clamp(0.0, 1.0));
    let contrast = plan.map_or(0.0, |plan| plan.contrast.clamp(0.0, 1.0));
    let bass_weight = plan.map_or(0.0, |plan| plan.bass_weight.clamp(0.0, 1.0));
    let stab_bite = plan.map_or(0.0, |plan| plan.stab_bite.clamp(0.0, 1.0));
    let gate_snap = plan.map_or(0.0, |plan| plan.gate_snap.clamp(0.0, 1.0));
    let source_gain = 1.0
        + pressure * 0.62
        + contrast * 0.20
        + bass_weight * 0.28
        + stab_bite * 0.12;
    let primitive_gate: f32 = match render.phrase_shape {
        Mc202PhraseShape::PressureCell => 0.50,
        Mc202PhraseShape::InstigatorSpike => 0.30,
        _ => 0.62,
    };
    let source_gate = (0.34 + bass_weight * 0.34 + pressure * 0.08 + contrast * 0.10)
        * (1.0 - gate_snap * 0.48)
        * (1.0 - stab_bite * 0.18);
    let mode_octave = match render.mode {
        Mc202RenderMode::Follower | Mc202RenderMode::Pressure => -12.0,
        Mc202RenderMode::Instigator => -2.0,
        _ => -5.0,
    };
    if plan.is_none() {
        return Mc202SourcePhraseSoundDesign {
            gain: render.music_bus_level.clamp(0.0, 1.0) * (0.08 + touch * 0.08),
            drive: 1.0,
            gate_len: primitive_gate.clamp(0.18, 0.68),
            env_curve: 1.8,
            sub_mix: 0.0,
            saw_mix: 0.58 + touch * 0.25,
            pulse_mix: 0.24 + touch * 0.18,
            bite_mix: 0.0,
            transient_click: 0.0,
            attack_len: 0.001,
            octave_drop: mode_octave,
            destructive_dive: if destructive_step { -10.0 } else { 0.0 },
            cut_start: 0.70,
        };
    }

    Mc202SourcePhraseSoundDesign {
        gain: render.music_bus_level.clamp(0.0, 1.0) * (0.075 + touch * 0.085) * source_gain,
        drive: 1.0
            + pressure * 1.35
            + contrast * 0.45
            + bass_weight * 1.05
            + stab_bite * 0.72,
        gate_len: if plan.is_some() {
            source_gate
        } else {
            primitive_gate
        }
        .clamp(0.12, 0.72),
        env_curve: (1.18 + gate_snap * 1.65 + stab_bite * 0.95 - bass_weight * 0.45)
            .clamp(0.85, 3.4),
        sub_mix: 0.16 + bass_weight * 0.72 + pressure * 0.20,
        saw_mix: 0.46 + touch * 0.22 + stab_bite * 0.20 - bass_weight * 0.08,
        pulse_mix: 0.22 + touch * 0.16 + stab_bite * 0.30 + contrast * 0.08,
        bite_mix: 0.08 + stab_bite * 0.58 + gate_snap * 0.18,
        transient_click: if plan.is_some() {
            0.04 + stab_bite * 0.42 + gate_snap * 0.24 + contrast * 0.08
        } else {
            0.0
        },
        attack_len: 0.045,
        octave_drop: mode_octave - f64::from(bass_weight * 7.0) + f64::from(stab_bite * 2.5),
        destructive_dive: if destructive_step {
            -10.0 - f64::from(contrast * 5.0 + stab_bite * 3.0)
        } else {
            0.0
        },
        cut_start: (0.76 - gate_snap * 0.16 - stab_bite * 0.08).clamp(0.52, 0.82),
    }
}

fn mc202_source_phrase_sample(
    phase: f64,
    step_phase: f32,
    accent: f32,
    design: Mc202SourcePhraseSoundDesign,
) -> f32 {
    if step_phase > design.gate_len {
        return 0.0;
    }

    let gate_position = (step_phase / design.gate_len).clamp(0.0, 1.0);
    let env = (1.0 - gate_position).powf(design.env_curve);
    let attack = (step_phase / design.attack_len)
        .clamp(0.0, 1.0)
        .powf(0.55);
    let click_env = (1.0 - (step_phase / 0.055).clamp(0.0, 1.0)).powf(2.4);
    let saw = (phase as f32 * 2.0) - 1.0;
    let pulse_width = (0.42 + design.bite_mix * 0.08 - design.sub_mix * 0.04).clamp(0.30, 0.58);
    let pulse = if phase < f64::from(pulse_width) {
        1.0
    } else {
        -1.0
    };
    let sub = (phase as f32 * std::f32::consts::TAU).sin();
    let second = (phase as f32 * std::f32::consts::TAU * 2.0).sin();
    let bite_edge = (saw - sub * 0.28 + pulse * 0.18).tanh();
    let tone = sub * design.sub_mix
        + second * design.sub_mix * 0.18
        + saw * design.saw_mix
        + pulse * design.pulse_mix
        + bite_edge * design.bite_mix;
    let transient = bite_edge * design.transient_click * click_env;
    let cut = if step_phase > design.cut_start {
        (1.0 - (step_phase - design.cut_start) / (1.0 - design.cut_start)).clamp(0.0, 1.0)
    } else {
        1.0
    };

    ((tone * design.drive + transient) * env * attack * accent * design.gain * cut).tanh()
}
