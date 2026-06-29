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

fn source_plan() -> Mc202SourcePhraseRenderPlan {
    Mc202SourcePhraseRenderPlan {
        active_mask: 0b0001_0001_0010_0101,
        semitones: [-12, 0, -7, 0, 0, -5, 0, 0, -10, 0, 0, 0, -3, 0, 0, 0],
        accent_mask: 0b0001_0000_0000_0001,
        destructive_mask: 0b0000_0000_0001_0000,
        pressure: 0.68,
        contrast: 0.52,
        bass_weight: 0.70,
        stab_bite: 0.24,
        gate_snap: 0.20,
    }
}

#[test]
fn renderer_stays_silent_without_source_phrase_plan_for_all_modes() {
    for mode in [
        Mc202RenderMode::Follower,
        Mc202RenderMode::Answer,
        Mc202RenderMode::Pressure,
        Mc202RenderMode::Instigator,
    ] {
        let mut rendered = vec![0.0; 44_100 * 2];
        render_mc202_buffer(
            &mut rendered,
            44_100,
            2,
            &Mc202RenderState {
                mode,
                routing: Mc202RenderRouting::MusicBusBass,
                phrase_shape: Mc202PhraseShape::FollowerDrive,
                touch: 0.78,
                is_transport_running: true,
                ..Mc202RenderState::default()
            },
        );

        let rendered_metrics = metrics(&rendered);
        assert_eq!(rendered_metrics.0, 0, "{mode:?} leaked hardcoded fallback");
        assert_eq!(
            rendered_metrics.2, 0.0,
            "{mode:?} leaked hardcoded fallback"
        );
    }
}

#[test]
fn touch_changes_render_energy_on_same_phrase() {
    let mut low_touch = vec![0.0; 44_100 * 2];
    let mut high_touch = vec![0.0; 44_100 * 2];
    let base = Mc202RenderState {
        mode: Mc202RenderMode::Follower,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::FollowerDrive,
        source_phrase_plan: Some(source_plan()),
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        ..Mc202RenderState::default()
    };

    render_mc202_buffer(
        &mut low_touch,
        44_100,
        2,
        &Mc202RenderState {
            touch: 0.08,
            ..base
        },
    );
    render_mc202_buffer(
        &mut high_touch,
        44_100,
        2,
        &Mc202RenderState {
            touch: 0.92,
            ..base
        },
    );

    let low_metrics = metrics(&low_touch);
    let high_metrics = metrics(&high_touch);
    let max_delta = low_touch
        .iter()
        .zip(high_touch.iter())
        .map(|(low, high)| (low - high).abs())
        .fold(0.0_f32, f32::max);

    assert!(low_metrics.0 > 10_000);
    assert!(high_metrics.0 > 10_000);
    assert!(
        high_metrics.2 > low_metrics.2 + 0.006,
        "low RMS {:.6}, high RMS {:.6}",
        low_metrics.2,
        high_metrics.2
    );
    assert!(max_delta > 0.02, "max touch delta {max_delta}");
}

#[test]
fn source_phrase_plan_changes_render_output() {
    let mut sparse = vec![0.0; 44_100 * 2];
    let mut pushed = vec![0.0; 44_100 * 2];
    let base = Mc202RenderState {
        mode: Mc202RenderMode::Follower,
        routing: Mc202RenderRouting::MusicBusBass,
        touch: 0.78,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        ..Mc202RenderState::default()
    };

    render_mc202_buffer(
        &mut sparse,
        44_100,
        2,
        &Mc202RenderState {
            source_phrase_plan: Some(source_plan()),
            ..base
        },
    );
    render_mc202_buffer(
        &mut pushed,
        44_100,
        2,
        &Mc202RenderState {
            source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
                active_mask: 0b1010_0101_0010_1001,
                semitones: [-19, 0, 4, 0, 7, 0, -12, 0, 0, 0, -5, 0, 9, 0, 0, -7],
                accent_mask: 0b1000_0001_0010_0001,
                pressure: 0.88,
                contrast: 0.74,
                bass_weight: 0.90,
                stab_bite: 0.34,
                gate_snap: 0.28,
                ..source_plan()
            }),
            ..base
        },
    );

    let sparse_metrics = metrics(&sparse);
    let pushed_metrics = metrics(&pushed);
    let delta_rms = (sparse
        .iter()
        .zip(pushed.iter())
        .map(|(sparse, pushed)| (sparse - pushed).powi(2))
        .sum::<f32>()
        / sparse.len() as f32)
        .sqrt();
    let max_delta = sparse
        .iter()
        .zip(pushed.iter())
        .map(|(sparse, pushed)| (sparse - pushed).abs())
        .fold(0.0_f32, f32::max);

    assert!(sparse_metrics.0 > 10_000);
    assert!(pushed_metrics.0 > 10_000);
    assert!(delta_rms > 0.005, "source phrase delta RMS {delta_rms}");
    assert!(max_delta > 0.02, "source phrase max delta {max_delta}");
}

#[test]
fn pressure_source_plan_differs_from_follower_source_plan() {
    let mut follower = vec![0.0; 44_100 * 2];
    let mut pressure = vec![0.0; 44_100 * 2];
    let base = Mc202RenderState {
        routing: Mc202RenderRouting::MusicBusBass,
        touch: 0.84,
        source_phrase_plan: Some(source_plan()),
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        ..Mc202RenderState::default()
    };

    render_mc202_buffer(
        &mut follower,
        44_100,
        2,
        &Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            ..base
        },
    );
    render_mc202_buffer(
        &mut pressure,
        44_100,
        2,
        &Mc202RenderState {
            mode: Mc202RenderMode::Pressure,
            phrase_shape: Mc202PhraseShape::PressureCell,
            source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
                active_mask: 0b0001_0001_0001_0001,
                semitones: [-19, 0, 0, 0, -17, 0, 0, 0, -22, 0, 0, 0, -14, 0, 0, 0],
                accent_mask: 0b0001_0001_0001_0001,
                pressure: 0.92,
                contrast: 0.60,
                bass_weight: 0.96,
                stab_bite: 0.10,
                gate_snap: 0.12,
                ..source_plan()
            }),
            ..base
        },
    );

    let follower_metrics = metrics(&follower);
    let pressure_metrics = metrics(&pressure);
    let delta_rms = (follower
        .iter()
        .zip(pressure.iter())
        .map(|(follower, pressure)| (follower - pressure).powi(2))
        .sum::<f32>()
        / follower.len() as f32)
        .sqrt();
    let max_delta = follower
        .iter()
        .zip(pressure.iter())
        .map(|(follower, pressure)| (follower - pressure).abs())
        .fold(0.0_f32, f32::max);

    assert!(follower_metrics.0 > 10_000);
    assert!(pressure_metrics.0 > 10_000);
    assert!(delta_rms > 0.004, "pressure phrase delta RMS {delta_rms}");
    assert!(max_delta > 0.02, "pressure phrase max delta {max_delta}");
}

#[test]
fn empty_source_phrase_plan_is_silent() {
    let mut rendered = vec![0.0; 44_100 * 2 * 2];

    render_mc202_buffer(
        &mut rendered,
        44_100,
        2,
        &Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
                active_mask: 0,
                ..source_plan()
            }),
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        },
    );

    let rendered_metrics = metrics(&rendered);
    assert_eq!(rendered_metrics.0, 0);
    assert_eq!(rendered_metrics.2, 0.0);
}

#[test]
fn contour_hint_changes_phrase_without_silencing_it() {
    let mut neutral = vec![0.0; 44_100 * 2];
    let mut lift = vec![0.0; 44_100 * 2];
    let base = Mc202RenderState {
        mode: Mc202RenderMode::Follower,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::FollowerDrive,
        note_budget: Mc202NoteBudget::Balanced,
        source_phrase_plan: Some(source_plan()),
        touch: 0.78,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        ..Mc202RenderState::default()
    };

    render_mc202_buffer(&mut neutral, 44_100, 2, &base);
    render_mc202_buffer(
        &mut lift,
        44_100,
        2,
        &Mc202RenderState {
            contour_hint: Mc202ContourHint::Lift,
            ..base
        },
    );

    let neutral_metrics = metrics(&neutral);
    let lift_metrics = metrics(&lift);
    let delta_rms = (neutral
        .iter()
        .zip(lift.iter())
        .map(|(neutral, lift)| (neutral - lift).powi(2))
        .sum::<f32>()
        / neutral.len() as f32)
        .sqrt();

    assert!(neutral_metrics.0 > 10_000);
    assert!(lift_metrics.0 > 10_000);
    assert!(delta_rms > 0.004, "contour hint delta RMS {delta_rms}");
}

#[test]
fn hook_response_no_longer_adds_synthetic_answer_shape() {
    let mut direct = vec![0.0; 44_100 * 2 * 2];
    let mut response = vec![0.0; 44_100 * 2 * 2];
    let base = Mc202RenderState {
        mode: Mc202RenderMode::Follower,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::FollowerDrive,
        note_budget: Mc202NoteBudget::Balanced,
        contour_hint: Mc202ContourHint::Neutral,
        source_phrase_plan: Some(source_plan()),
        touch: 0.78,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        ..Mc202RenderState::default()
    };

    render_mc202_buffer(&mut direct, 44_100, 2, &base);
    render_mc202_buffer(
        &mut response,
        44_100,
        2,
        &Mc202RenderState {
            hook_response: Mc202HookResponse::Direct,
            ..base
        },
    );

    let direct_metrics = metrics(&direct);
    let response_metrics = metrics(&response);
    let delta_rms = (direct
        .iter()
        .zip(response.iter())
        .map(|(direct, response)| (direct - response).powi(2))
        .sum::<f32>()
        / direct.len() as f32)
        .sqrt();

    assert!(direct_metrics.0 > 10_000);
    assert!(response_metrics.0 > 10_000);
    assert!(
        delta_rms <= f32::EPSILON,
        "direct hook response should not inject a hardcoded answer shape: {delta_rms}"
    );
}

#[test]
fn instigator_spike_differs_from_follower_drive() {
    let mut follower = vec![0.0; 44_100 * 2];
    let mut instigator = vec![0.0; 44_100 * 2];
    let base = Mc202RenderState {
        routing: Mc202RenderRouting::MusicBusBass,
        touch: 0.90,
        source_phrase_plan: Some(source_plan()),
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        ..Mc202RenderState::default()
    };

    render_mc202_buffer(
        &mut follower,
        44_100,
        2,
        &Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            ..base
        },
    );
    render_mc202_buffer(
        &mut instigator,
        44_100,
        2,
        &Mc202RenderState {
            mode: Mc202RenderMode::Instigator,
            phrase_shape: Mc202PhraseShape::InstigatorSpike,
            ..base
        },
    );

    let follower_metrics = metrics(&follower);
    let instigator_metrics = metrics(&instigator);
    let delta_rms = (follower
        .iter()
        .zip(instigator.iter())
        .map(|(follower, instigator)| (follower - instigator).powi(2))
        .sum::<f32>()
        / follower.len() as f32)
        .sqrt();
    let max_delta = follower
        .iter()
        .zip(instigator.iter())
        .map(|(follower, instigator)| (follower - instigator).abs())
        .fold(0.0_f32, f32::max);

    assert!(follower_metrics.0 > 10_000);
    assert!(instigator_metrics.0 > 8_000);
    assert!(delta_rms > 0.010, "instigator phrase delta RMS {delta_rms}");
    assert!(max_delta > 0.04, "instigator phrase max delta {max_delta}");
}

#[test]
fn render_is_stable_across_callback_chunk_boundaries() {
    let render = Mc202RenderState {
        mode: Mc202RenderMode::Follower,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::FollowerDrive,
        source_phrase_plan: Some(source_plan()),
        touch: 0.78,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        ..Mc202RenderState::default()
    };
    let mut whole = vec![0.0; 44_100 * 2];
    let mut chunked = vec![0.0; 44_100 * 2];
    let split_frames = 2_048;
    let split_samples = split_frames * 2;

    render_mc202_buffer(&mut whole, 44_100, 2, &render);
    render_mc202_buffer(&mut chunked[..split_samples], 44_100, 2, &render);

    let mut second_render = render;
    second_render.position_beats +=
        split_frames as f64 * f64::from(render.tempo_bpm) / 60.0 / 44_100.0;
    render_mc202_buffer(&mut chunked[split_samples..], 44_100, 2, &second_render);

    let max_delta = whole
        .iter()
        .zip(chunked.iter())
        .map(|(whole, chunked)| (whole - chunked).abs())
        .fold(0.0_f32, f32::max);
    assert!(max_delta < 0.0001, "max chunk boundary delta {max_delta}");
}
