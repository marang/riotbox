fn render_tr909_buffer(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeTr909RenderState,
    state: &mut Tr909CallbackState,
) {
    if !render.is_transport_running
        || matches!(render.mode, Tr909RenderMode::Idle)
        || render.tempo_bpm <= 0.0
    {
        state.was_running = false;
        state.envelope = 0.0;
        state.beat_position = render.position_beats;
        return;
    }

    let subdivision = render_subdivision(render);
    let current_step = (render.position_beats * f64::from(subdivision)).floor() as i64;
    if !state.was_running || (state.beat_position - render.position_beats).abs() > 0.125 {
        state.beat_position = render.position_beats;
        state.last_step = current_step.saturating_sub(1);
        state.was_running = true;
    }

    let beats_per_sample = f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1));
    let frame_count = data.len() / channel_count.max(1);

    for frame_index in 0..frame_count {
        let step = (state.beat_position * f64::from(subdivision)).floor() as i64;
        if step != state.last_step {
            state.last_step = step;
            if should_trigger_step(render, step) {
                state.envelope = trigger_envelope(render);
                state.oscillator_hz = trigger_frequency(render, step);
            }
        }

        let sample = if state.envelope > 0.0005 {
            let gain = render_gain(render);
            let waveform = (std::f32::consts::TAU * state.oscillator_phase).sin();
            state.oscillator_phase =
                (state.oscillator_phase + state.oscillator_hz / sample_rate.max(1) as f32).fract();
            let rendered = waveform * state.envelope * gain;
            state.envelope *= envelope_decay(render);
            rendered
        } else {
            0.0
        };

        let base = frame_index * channel_count;
        for channel in 0..channel_count {
            data[base + channel] += sample;
        }

        state.beat_position += beats_per_sample;
    }
}

fn render_w30_preview_buffer(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeW30PreviewRenderState,
    state: &mut W30PreviewCallbackState,
) {
    let active = !matches!(render.mode, W30PreviewRenderMode::Idle)
        && matches!(render.routing, W30PreviewRenderRouting::MusicBusPreview)
        && render.music_bus_level > 0.0;

    if !active {
        state.was_active = false;
        state.envelope = 0.0;
        state.beat_position = render.position_beats;
        state.last_trigger_revision = render.trigger_revision;
        return;
    }

    if !state.was_active {
        state.beat_position = render.position_beats;
        state.envelope = 1.0;
        state.last_step = w30_current_step(render.position_beats, render);
        state.oscillator_phase = 0.0;
        state.lfo_phase = 0.0;
        state.source_sample_cursor = 0.0;
        state.pad_playback_cursor = 0.0;
        state.last_source_window_signature = w30_source_window_signature(render);
        state.last_pad_playback_signature = w30_pad_playback_signature(render);
        state.last_trigger_revision = render.trigger_revision;
        state.was_active = true;
    }

    let source_window_signature = w30_source_window_signature(render);
    if source_window_signature != state.last_source_window_signature {
        state.last_source_window_signature = source_window_signature;
        state.source_sample_cursor = 0.0;
    }
    let pad_playback_signature = w30_pad_playback_signature(render);
    if pad_playback_signature != state.last_pad_playback_signature {
        state.last_pad_playback_signature = pad_playback_signature;
        state.pad_playback_cursor = 0.0;
    }

    if render.trigger_revision > state.last_trigger_revision {
        state.last_trigger_revision = render.trigger_revision;
        state.envelope = state.envelope.max(
            w30_trigger_envelope(render) * (0.85 + render.trigger_velocity.clamp(0.0, 1.0) * 0.3),
        );
        state.oscillator_phase = 0.0;
        state.pad_playback_cursor = 0.0;
    }

    let frame_count = data.len() / channel_count.max(1);
    let transport_running = render.is_transport_running && render.tempo_bpm > 0.0;
    let beats_per_sample = if transport_running {
        f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1))
    } else {
        f64::from(w30_preview_idle_bpm(render)) / 60.0 / f64::from(sample_rate.max(1))
    };

    for frame_index in 0..frame_count {
        if transport_running {
            let step = w30_current_step(state.beat_position, render);
            if step != state.last_step {
                state.last_step = step;
                if should_trigger_w30_step(render, step) {
                    state.envelope = w30_trigger_envelope(render);
                    if w30_source_window_active(render) {
                        state.source_sample_cursor = 0.0;
                    }
                }
            }
        } else {
            state.envelope = (state.envelope * 0.9998).max(0.35);
        }

        let tremolo = if transport_running {
            1.0
        } else {
            state.lfo_phase = (state.lfo_phase + 1.8 / sample_rate.max(1) as f32).fract();
            0.45 + 0.55 * ((std::f32::consts::TAU * state.lfo_phase).sin() * 0.5 + 0.5)
        };
        let waveform = w30_preview_waveform_for_frame(render, state, sample_rate);
        let sample =
            waveform * state.envelope * tremolo * w30_render_gain(render, transport_running);
        if transport_running {
            state.envelope *= w30_envelope_decay(render);
        }

        let base = frame_index * channel_count;
        for channel in 0..channel_count {
            data[base + channel] += sample;
        }

        state.beat_position += beats_per_sample;
    }
}

fn w30_preview_waveform_for_frame(
    render: &RealtimeW30PreviewRenderState,
    state: &mut W30PreviewCallbackState,
    sample_rate: u32,
) -> f32 {
    if w30_pad_playback_active(render) {
        let sample = w30_pad_playback_sample(&render.pad_playback, state);
        let grit = render.grit_level.clamp(0.0, 1.0);
        return (sample * (1.0 + grit * 0.35)).clamp(-1.0, 1.0);
    }

    if w30_source_window_active(render) {
        let sample = w30_source_window_sample(&render.source_window_preview, state);
        let grit = render.grit_level.clamp(0.0, 1.0);
        return (sample * (1.0 + grit * 0.35)).clamp(-1.0, 1.0);
    }

    let frequency = w30_preview_frequency(render, state.last_step);
    let waveform = w30_preview_waveform(state.oscillator_phase, render.grit_level);
    state.oscillator_phase =
        (state.oscillator_phase + frequency / sample_rate.max(1) as f32).fract();
    waveform
}

fn w30_source_window_active(render: &RealtimeW30PreviewRenderState) -> bool {
    !matches!(render.mode, W30PreviewRenderMode::Idle)
        && render.source_window_preview.sample_count > 0
}

fn w30_pad_playback_active(render: &RealtimeW30PreviewRenderState) -> bool {
    !matches!(render.mode, W30PreviewRenderMode::Idle) && render.pad_playback.sample_count > 0
}

fn w30_pad_playback_sample(
    window: &RealtimeW30PadPlaybackSampleWindow,
    state: &mut W30PreviewCallbackState,
) -> f32 {
    let sample_count = window.sample_count.min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
    if sample_count == 0 {
        return 0.0;
    }

    let cursor = state.pad_playback_cursor as usize;
    let clamped_cursor = if window.loop_enabled {
        cursor % sample_count
    } else {
        cursor.min(sample_count - 1)
    };
    state.pad_playback_cursor = if window.loop_enabled {
        (state.pad_playback_cursor + 1.0) % sample_count as f32
    } else {
        (state.pad_playback_cursor + 1.0).min(sample_count.saturating_sub(1) as f32)
    };
    window.samples[clamped_cursor]
}

fn w30_source_window_sample(
    window: &RealtimeW30PreviewSampleWindow,
    state: &mut W30PreviewCallbackState,
) -> f32 {
    let sample_count = window.sample_count.min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    if sample_count == 0 {
        return 0.0;
    }

    let cursor = state.source_sample_cursor as usize % sample_count;
    state.source_sample_cursor = (state.source_sample_cursor + 0.5) % sample_count as f32;
    window.samples[cursor]
}

fn w30_source_window_signature(render: &RealtimeW30PreviewRenderState) -> u64 {
    render
        .source_window_preview
        .source_start_frame
        .wrapping_mul(31)
        .wrapping_add(render.source_window_preview.source_end_frame)
        .wrapping_add(render.source_window_preview.sample_count as u64)
}

fn w30_pad_playback_signature(render: &RealtimeW30PreviewRenderState) -> u64 {
    render
        .pad_playback
        .source_start_frame
        .wrapping_mul(31)
        .wrapping_add(render.pad_playback.source_end_frame)
        .wrapping_add(render.pad_playback.sample_count as u64)
        .wrapping_add(u64::from(render.pad_playback.loop_enabled))
}

fn render_w30_resample_tap_buffer(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
) {
    let active = !matches!(render.mode, W30ResampleTapMode::Idle)
        && matches!(render.routing, W30ResampleTapRouting::InternalCaptureTap)
        && render.music_bus_level > 0.0;

    if !active {
        state.was_active = false;
        state.envelope = 0.0;
        state.beat_position = 0.0;
        return;
    }

    if !state.was_active {
        state.beat_position = 0.0;
        state.envelope = 1.0;
        state.last_step = 0;
        state.oscillator_phase = 0.0;
        state.shimmer_phase = 0.0;
        state.was_active = true;
    }

    let transport_running = render.is_transport_running;
    let beats_per_sample = if transport_running {
        124.0_f64 / 60.0 / f64::from(sample_rate.max(1))
    } else {
        92.0_f64 / 60.0 / f64::from(sample_rate.max(1))
    };
    let frame_count = data.len() / channel_count.max(1);

    for frame_index in 0..frame_count {
        if transport_running {
            let step =
                (state.beat_position * f64::from(w30_resample_subdivision(render))).floor() as i64;
            if step != state.last_step {
                state.last_step = step;
                if should_trigger_w30_resample_step(render, step) {
                    state.envelope = w30_resample_trigger_envelope(render);
                }
            }
        } else {
            state.envelope = state.envelope.max(0.42) * 0.99975;
        }

        let frequency = w30_resample_frequency(render, state.last_step);
        let shimmer_rate = 0.35 + f32::from(render.generation_depth) * 0.18;
        state.shimmer_phase =
            (state.shimmer_phase + shimmer_rate / sample_rate.max(1) as f32).fract();
        let shimmer =
            0.72 + 0.28 * ((std::f32::consts::TAU * state.shimmer_phase).sin() * 0.5 + 0.5);
        let waveform = w30_resample_waveform(state.oscillator_phase, render.grit_level);
        let sample = waveform
            * state.envelope
            * shimmer
            * w30_resample_render_gain(render, transport_running);
        state.oscillator_phase =
            (state.oscillator_phase + frequency / sample_rate.max(1) as f32).fract();
        if transport_running {
            state.envelope *= w30_resample_decay(render);
        }

        let base = frame_index * channel_count;
        for channel in 0..channel_count {
            data[base + channel] += sample;
        }

        state.beat_position += beats_per_sample;
    }
}

fn w30_resample_subdivision(render: &RealtimeW30ResampleTapState) -> u32 {
    let base = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) => 1,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 2,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 4,
        None => 1,
    };
    (base + u32::from(render.lineage_capture_count >= 2)).min(4)
}

fn should_trigger_w30_resample_step(render: &RealtimeW30ResampleTapState, step: i64) -> bool {
    match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => step.rem_euclid(2) == 0,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => !matches!(step.rem_euclid(4), 1),
        Some(W30ResampleTapSourceProfile::PinnedCapture) => true,
    }
}

fn w30_resample_trigger_envelope(render: &RealtimeW30ResampleTapState) -> f32 {
    let profile_boost = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => 0.0,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 0.05,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 0.1,
    };
    let lineage_boost = f32::from(render.lineage_capture_count.min(4)) * 0.03;
    let generation_boost = f32::from(render.generation_depth.min(4)) * 0.04;
    (0.24 + profile_boost + lineage_boost + generation_boost + render.grit_level * 0.12)
        .clamp(0.0, 0.9)
}

fn w30_resample_frequency(render: &RealtimeW30ResampleTapState, step: i64) -> f32 {
    let base = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => 130.81,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 164.81,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 196.0,
    };
    let step_offset = match step.rem_euclid(4) {
        0 => 0.0,
        1 => 5.0,
        2 => 12.0,
        _ => 7.0,
    };
    let lineage_offset = f32::from(render.lineage_capture_count.min(5)) * 3.0;
    let generation_offset = f32::from(render.generation_depth.min(5)) * 5.0;
    let grit_offset = render.grit_level * 18.0;
    base + step_offset + lineage_offset + generation_offset + grit_offset
}

fn w30_resample_waveform(phase: f32, grit_level: f32) -> f32 {
    let sine = (std::f32::consts::TAU * phase).sin();
    let saw = ((phase * 2.0) - 1.0).clamp(-1.0, 1.0);
    let shimmer = (std::f32::consts::TAU * phase * 3.0).sin();
    let grit = grit_level.clamp(0.0, 1.0);
    (sine * (1.0 - grit * 0.35) + saw * 0.22 + shimmer * (0.12 + grit * 0.22)).clamp(-1.0, 1.0)
}

fn w30_resample_render_gain(render: &RealtimeW30ResampleTapState, transport_running: bool) -> f32 {
    let profile_gain = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => 0.08,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 0.11,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 0.14,
    };
    let transport_gain = if transport_running { 1.0 } else { 0.7 };
    (profile_gain
        * transport_gain
        * render.music_bus_level.clamp(0.0, 1.0)
        * (1.0 + render.grit_level.clamp(0.0, 1.0) * 0.18))
        .clamp(0.0, 0.22)
}

fn w30_resample_decay(render: &RealtimeW30ResampleTapState) -> f32 {
    let generation_offset = f32::from(render.generation_depth.min(4)) * 0.00003;
    let lineage_offset = f32::from(render.lineage_capture_count.min(4)) * 0.00002;
    let grit_offset = render.grit_level.clamp(0.0, 1.0) * 0.00005;
    (0.99978 - generation_offset - lineage_offset - grit_offset).clamp(0.0, 1.0)
}

fn w30_current_step(position_beats: f64, render: &RealtimeW30PreviewRenderState) -> i64 {
    (position_beats * f64::from(w30_preview_subdivision(render))).floor() as i64
}

fn w30_preview_subdivision(render: &RealtimeW30PreviewRenderState) -> u32 {
    match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 1,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => 2,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => 3,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => 2,
        Some(W30PreviewSourceProfile::PromotedAudition) => 4,
    }
}

fn should_trigger_w30_step(render: &RealtimeW30PreviewRenderState, step: i64) -> bool {
    match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => true,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => step.rem_euclid(2) == 0,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => step.rem_euclid(3) != 1,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => step.rem_euclid(2) == 0,
        Some(W30PreviewSourceProfile::PromotedAudition) => {
            !matches!(step.rem_euclid(4), 1) || render.grit_level >= 0.65
        }
    }
}

fn w30_trigger_envelope(render: &RealtimeW30PreviewRenderState) -> f32 {
    let mode_boost = match render.mode {
        W30PreviewRenderMode::Idle => 0.0,
        W30PreviewRenderMode::LiveRecall => 0.16,
        W30PreviewRenderMode::RawCaptureAudition => 0.2,
        W30PreviewRenderMode::PromotedAudition => 0.24,
    };
    let profile_boost = match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 0.0,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => 0.05,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => 0.07,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => 0.08,
        Some(W30PreviewSourceProfile::PromotedAudition) => 0.1,
    };
    (0.32 + mode_boost + profile_boost + render.grit_level.clamp(0.0, 1.0) * 0.18).clamp(0.0, 0.9)
}

