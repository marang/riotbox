use super::*;

fn metrics(buffer: &[f32]) -> (usize, f32, f32) {
    let active = buffer.iter().filter(|sample| sample.abs() > 0.0001).count();
    let peak = buffer
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let rms =
        (buffer.iter().map(|sample| sample * sample).sum::<f32>() / buffer.len() as f32).sqrt();
    (active, peak, rms)
}

fn low_band_rms(buffer: &[f32]) -> f32 {
    let mut low = 0.0_f32;
    let mut energy = 0.0_f32;
    let alpha = 0.018_f32;
    for sample in buffer {
        low += (*sample - low) * alpha;
        energy += low * low;
    }
    (energy / buffer.len() as f32).sqrt()
}

fn transient_energy(buffer: &[f32]) -> f32 {
    if buffer.len() < 2 {
        return 0.0;
    }
    let mut energy = 0.0_f32;
    for window in buffer.windows(2) {
        let delta = window[1] - window[0];
        energy += delta * delta;
    }
    (energy / (buffer.len() - 1) as f32).sqrt()
}

fn perceptible_bass_rms(buffer: &[f32], sample_rate: u32) -> f32 {
    let low_alpha = 1.0 - (-std::f32::consts::TAU * 160.0 / sample_rate as f32).exp();
    let sub_alpha = 1.0 - (-std::f32::consts::TAU * 35.0 / sample_rate as f32).exp();
    let mut low = 0.0_f32;
    let mut sub = 0.0_f32;
    let mut energy = 0.0_f32;
    for sample in buffer {
        low += (*sample - low) * low_alpha;
        sub += (*sample - sub) * sub_alpha;
        energy += (low - sub).powi(2);
    }
    (energy / buffer.len() as f32).sqrt()
}

#[test]
fn source_phrase_articulation_separates_bass_pressure_from_answer_stab() {
    let mut bass_pressure = vec![0.0; 44_100 * 2];
    let mut answer_stab = vec![0.0; 44_100 * 2];
    let scaffold = Mc202SourcePhraseRenderPlan {
        active_mask: 0b0001_0001_0001_0001,
        semitones: [-12, 0, 0, 0, -10, 0, 0, 0, -15, 0, 0, 0, -7, 0, 0, 0],
        accent_mask: 0b0001_0001_0001_0001,
        destructive_mask: 0,
        pressure: 0.62,
        contrast: 0.55,
        bass_weight: 0.0,
        stab_bite: 0.0,
        gate_snap: 0.0,
    };
    let base = Mc202RenderState {
        mode: Mc202RenderMode::Answer,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::RootPulse,
        touch: 0.78,
        tempo_bpm: 128.0,
        is_transport_running: true,
        ..Mc202RenderState::default()
    };

    render_mc202_buffer(
        &mut bass_pressure,
        44_100,
        2,
        &Mc202RenderState {
            source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
                bass_weight: 0.96,
                stab_bite: 0.10,
                gate_snap: 0.10,
                ..scaffold
            }),
            ..base
        },
    );
    render_mc202_buffer(
        &mut answer_stab,
        44_100,
        2,
        &Mc202RenderState {
            source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
                bass_weight: 0.12,
                stab_bite: 0.96,
                gate_snap: 0.86,
                ..scaffold
            }),
            ..base
        },
    );

    let bass_metrics = metrics(&bass_pressure);
    let stab_metrics = metrics(&answer_stab);
    let max_delta = bass_pressure
        .iter()
        .zip(answer_stab.iter())
        .map(|(left, right)| (left - right).abs())
        .fold(0.0_f32, f32::max);

    assert!(
        bass_metrics.0 > stab_metrics.0,
        "bass articulation should hold notes longer than stab articulation: bass={bass_metrics:?} stab={stab_metrics:?}"
    );
    assert!(bass_metrics.2 > 0.001, "{bass_metrics:?}");
    assert!(stab_metrics.2 > 0.001, "{stab_metrics:?}");
    assert!(
        max_delta > 0.01,
        "bass/stab articulation collapsed to the same render: {max_delta}"
    );
}

#[test]
fn source_phrase_production_sound_design_separates_body_from_bite_without_clipping() {
    let mut bass_pressure = vec![0.0; 44_100 * 2];
    let mut answer_stab = vec![0.0; 44_100 * 2];
    let scaffold = Mc202SourcePhraseRenderPlan {
        active_mask: 0b0001_0001_0001_0001,
        semitones: [-19, 0, 0, 0, -17, 0, 0, 0, -22, 0, 0, 0, -14, 0, 0, 0],
        accent_mask: 0b0001_0001_0001_0001,
        destructive_mask: 0b0000_0000_0001_0000,
        pressure: 0.72,
        contrast: 0.62,
        bass_weight: 0.0,
        stab_bite: 0.0,
        gate_snap: 0.0,
    };
    let base = Mc202RenderState {
        mode: Mc202RenderMode::Pressure,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::RootPulse,
        touch: 0.84,
        tempo_bpm: 128.0,
        is_transport_running: true,
        ..Mc202RenderState::default()
    };

    render_mc202_buffer(
        &mut bass_pressure,
        44_100,
        2,
        &Mc202RenderState {
            source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
                bass_weight: 0.96,
                stab_bite: 0.08,
                gate_snap: 0.12,
                ..scaffold
            }),
            ..base
        },
    );
    render_mc202_buffer(
        &mut answer_stab,
        44_100,
        2,
        &Mc202RenderState {
            source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
                bass_weight: 0.10,
                stab_bite: 0.96,
                gate_snap: 0.90,
                ..scaffold
            }),
            ..base
        },
    );

    let bass_metrics = metrics(&bass_pressure);
    let stab_metrics = metrics(&answer_stab);
    let bass_low = low_band_rms(&bass_pressure);
    let stab_low = low_band_rms(&answer_stab);
    let bass_transient = transient_energy(&bass_pressure);
    let stab_transient = transient_energy(&answer_stab);
    let bass_transient_sharpness = bass_transient / bass_metrics.2.max(f32::EPSILON);
    let stab_transient_sharpness = stab_transient / stab_metrics.2.max(f32::EPSILON);

    assert!(
        bass_metrics.2 > 0.001,
        "bass should remain audible: {bass_metrics:?}"
    );
    assert!(
        stab_metrics.2 > 0.001,
        "stab should remain audible: {stab_metrics:?}"
    );
    assert!(bass_metrics.1 <= 0.985, "bass clipped: {bass_metrics:?}");
    assert!(stab_metrics.1 <= 0.985, "stab clipped: {stab_metrics:?}");
    assert!(
        bass_low > stab_low * 1.30,
        "pressure body did not exceed stab low-band enough: bass_low={bass_low:.6} stab_low={stab_low:.6}"
    );
    assert!(
        stab_transient_sharpness > bass_transient_sharpness * 1.05,
        "answer stab did not exceed pressure transient sharpness enough: bass_sharpness={bass_transient_sharpness:.6} stab_sharpness={stab_transient_sharpness:.6}"
    );
}

#[test]
fn callback_punctuation_uses_source_marked_two_bar_omission_variation() {
    let sample_rate = 48_000;
    let channel_count = 2;
    let tempo_bpm = 130.0;
    let bar_frames = (sample_rate as f32 * 60.0 / tempo_bpm * 4.0).round() as usize;
    let mut rendered = vec![0.0; bar_frames * channel_count * 2];

    render_mc202_buffer(
        &mut rendered,
        sample_rate,
        channel_count,
        &callback_punctuation_render(tempo_bpm),
    );

    let split = bar_frames * channel_count;
    let delta_rms = (rendered[..split]
        .iter()
        .zip(&rendered[split..])
        .map(|(first, second)| (first - second).powi(2))
        .sum::<f32>()
        / split as f32)
        .sqrt();

    assert!(
        delta_rms > 0.020,
        "callback punctuation collapsed to identical bars: delta_rms={delta_rms:.6}"
    );
    assert!(metrics(&rendered[..split]).2 > 0.020);
    assert!(metrics(&rendered[split..]).2 > 0.005);
}

#[test]
fn callback_punctuation_has_no_click_sized_sample_discontinuities() {
    let sample_rate = 48_000;
    let channel_count = 2;
    let tempo_bpm = 130.0;
    let mut rendered = vec![0.0; sample_rate as usize * channel_count * 4];

    render_mc202_buffer(
        &mut rendered,
        sample_rate,
        channel_count,
        &callback_punctuation_render(tempo_bpm),
    );

    let max_sample_jump = rendered
        .chunks_exact(channel_count)
        .map(|frame| frame[0])
        .collect::<Vec<_>>()
        .windows(2)
        .map(|window| (window[1] - window[0]).abs())
        .fold(0.0_f32, f32::max);

    assert!(
        max_sample_jump < 0.050,
        "callback punctuation retained click-sized waveform edges: max_sample_jump={max_sample_jump:.6}"
    );
}

#[test]
fn fill_pickup_instigator_varies_across_bars_without_click_sized_edges() {
    let sample_rate = 48_000;
    let channel_count = 2;
    let tempo_bpm = 130.0;
    let bar_frames = (sample_rate as f32 * 60.0 / tempo_bpm * 4.0).round() as usize;
    let mut rendered = vec![0.0; bar_frames * channel_count * 2];
    let render = Mc202RenderState {
        mode: Mc202RenderMode::Instigator,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::InstigatorSpike,
        note_budget: Mc202NoteBudget::Push,
        contour_hint: Mc202ContourHint::Hold,
        source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
            active_mask: 208,
            semitones: [0, 0, 0, 0, 19, 0, 24, -10, 0, 0, 0, 0, 0, 0, 0, 0],
            accent_mask: 144,
            destructive_mask: 192,
            pressure: 0.5015595,
            contrast: 0.5508757,
            bass_weight: 0.6559019,
            stab_bite: 0.4881247,
            gate_snap: 0.6259999,
        }),
        touch: 0.84,
        music_bus_level: 0.70,
        tempo_bpm,
        is_transport_running: true,
        ..Mc202RenderState::default()
    };

    render_mc202_buffer(&mut rendered, sample_rate, channel_count, &render);

    let split = bar_frames * channel_count;
    let delta_rms = (rendered[..split]
        .iter()
        .zip(&rendered[split..])
        .map(|(first, second)| (first - second).powi(2))
        .sum::<f32>()
        / split as f32)
        .sqrt();
    let max_sample_jump = rendered
        .chunks_exact(channel_count)
        .map(|frame| frame[0])
        .collect::<Vec<_>>()
        .windows(2)
        .map(|window| (window[1] - window[0]).abs())
        .fold(0.0_f32, f32::max);
    let instigator_metrics = metrics(&rendered);

    assert!(
        delta_rms > 0.020,
        "fill pickup collapsed to identical bars: delta_rms={delta_rms:.6}"
    );
    assert!(
        instigator_metrics.1 > 0.47,
        "fill pickup lost forceful transient impact: {instigator_metrics:?}"
    );
    assert!(
        instigator_metrics.2 > 0.030,
        "fill pickup lost audible body: {instigator_metrics:?}"
    );
    assert!(
        max_sample_jump < 0.050,
        "fill pickup retained click-sized waveform edges: max_sample_jump={max_sample_jump:.6}"
    );
}

#[test]
fn bass_pressure_translates_into_audible_body_and_two_bar_movement() {
    let sample_rate = 48_000;
    let channel_count = 2;
    let tempo_bpm = 128.0;
    let bar_frames = (sample_rate as f32 * 60.0 / tempo_bpm * 4.0).round() as usize;
    let mut rendered = vec![0.0; bar_frames * channel_count * 2];
    let render = Mc202RenderState {
        mode: Mc202RenderMode::Pressure,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::PressureCell,
        note_budget: Mc202NoteBudget::Sparse,
        contour_hint: Mc202ContourHint::Hold,
        source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
            active_mask: 4353,
            semitones: [-12, 0, 0, 0, 0, 0, 0, 0, -10, 0, 0, 0, -7, 0, 0, 0],
            accent_mask: 4353,
            destructive_mask: 4096,
            pressure: 0.6678762,
            contrast: 0.37953687,
            bass_weight: 0.69874275,
            stab_bite: 0.16190626,
            gate_snap: 0.206,
        }),
        touch: 0.84,
        music_bus_level: 0.83937943,
        tempo_bpm,
        is_transport_running: true,
        ..Mc202RenderState::default()
    };

    render_mc202_buffer(&mut rendered, sample_rate, channel_count, &render);

    let split = bar_frames * channel_count;
    let active_ratio = metrics(&rendered).0 as f32 / rendered.len() as f32;
    let body_rms = perceptible_bass_rms(&rendered, sample_rate);
    let delta_rms = (rendered[..split]
        .iter()
        .zip(&rendered[split..])
        .map(|(first, second)| (first - second).powi(2))
        .sum::<f32>()
        / split as f32)
        .sqrt();
    let first_bar_rms = metrics(&rendered[..split]).2;
    let second_bar_rms = metrics(&rendered[split..]).2;
    let mono = rendered
        .chunks_exact(channel_count)
        .map(|frame| frame[0])
        .collect::<Vec<_>>();
    let step_frames = bar_frames / 16;
    let destructive_tail = &mono[(12 * step_frames)..(13 * step_frames)];
    let second_bar_tail = &mono[((16 + 12) * step_frames)..((16 + 15) * step_frames)];
    let destructive_tail_body = perceptible_bass_rms(destructive_tail, sample_rate);
    let second_bar_tail_rms = metrics(second_bar_tail).2;
    let dc_offset = mono.iter().sum::<f32>() / mono.len() as f32;
    let max_sample_jump = rendered
        .chunks_exact(channel_count)
        .map(|frame| frame[0])
        .collect::<Vec<_>>()
        .windows(2)
        .map(|window| (window[1] - window[0]).abs())
        .fold(0.0_f32, f32::max);

    assert!(
        active_ratio > 0.38,
        "bass pressure left too much empty space: active_ratio={active_ratio:.3}"
    );
    assert!(
        body_rms > 0.045,
        "bass pressure did not translate above the sub-only band: body_rms={body_rms:.6}"
    );
    assert!(
        delta_rms > 0.025,
        "bass pressure collapsed to identical bars: delta_rms={delta_rms:.6}"
    );
    assert!(
        first_bar_rms > 0.10,
        "first bar lost pressure: {first_bar_rms:.6}"
    );
    assert!(
        second_bar_rms > 0.10,
        "second bar lost pressure: {second_bar_rms:.6}"
    );
    assert!(
        destructive_tail_body > 0.10,
        "destructive pressure tail collapsed below the perceptible bass band: body_rms={destructive_tail_body:.6}"
    );
    assert!(
        second_bar_tail_rms < 0.0001,
        "source-marked tail was not omitted before the two-bar loop reset: rms={second_bar_tail_rms:.6}"
    );
    assert!(
        dc_offset.abs() < 0.01,
        "destructive pressure tail retained excessive sub-cycle DC: dc_offset={dc_offset:.6}"
    );
    assert!(
        max_sample_jump < 0.050,
        "bass pressure retained click-sized waveform edges: max_sample_jump={max_sample_jump:.6}"
    );
}

fn callback_punctuation_render(tempo_bpm: f32) -> Mc202RenderState {
    Mc202RenderState {
        mode: Mc202RenderMode::Answer,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::RootPulse,
        note_budget: Mc202NoteBudget::Sparse,
        contour_hint: Mc202ContourHint::Hold,
        source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
            active_mask: 17024,
            semitones: [0, 0, 0, 0, 0, 0, 0, -10, 0, 0, 0, 0, 0, 0, 5, 0],
            accent_mask: 640,
            destructive_mask: 16512,
            pressure: 0.48163605,
            contrast: 0.5643454,
            bass_weight: 0.6559019,
            stab_bite: 0.47242627,
            gate_snap: 0.6259999,
        }),
        touch: 0.84,
        music_bus_level: 0.62,
        tempo_bpm,
        is_transport_running: true,
        ..Mc202RenderState::default()
    }
}
