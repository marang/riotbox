use super::*;

pub(super) fn render_tr909_buffer(
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
        state.fill_voices.deactivate();
        state.beat_position = render.position_beats;
        return;
    }

    if matches!(render.mode, Tr909RenderMode::Fill) {
        // The playable fill owns fixed, independently decaying kick/snare/hat voices. Keeping
        // this branch fill-only preserves the established sample path for every other mode.
        state.envelope = 0.0;
        state.oscillator_phase = 0.0;
        state.oscillator_hz = 0.0;
        render_tr909_fill_buffer(data, sample_rate, channel_count, render, state);
        return;
    }

    let exited_fill = state.fill_voices.is_active();
    state.fill_voices.deactivate();

    let subdivision = render_subdivision(render);
    let current_step = (render.position_beats * f64::from(subdivision)).floor() as i64;
    if exited_fill {
        // A mode change can occur without crossing a transport step. Force a fresh legacy voice
        // trigger so no Fill subdivision or oscillator state leaks into the non-Fill path.
        state.beat_position = render.position_beats;
        state.last_step = current_step.saturating_sub(1);
        state.envelope = 0.0;
        state.oscillator_phase = 0.0;
        state.oscillator_hz = 0.0;
    }
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
                if break_performance_slam(render) > 0.0 {
                    state.oscillator_phase = 0.25;
                }
            }
        }

        let sample = if state.envelope > 0.0005 {
            let gain = render_gain(render);
            let waveform = tr909_step_waveform(render, state.last_step, state.oscillator_phase);
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

pub(super) fn render_w30_preview_buffer(
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
        state.last_transport_running = render.is_transport_running;
        state.transport_stop_latched = false;
        state.transport_stop_fade_frames_remaining = 0;
        state.envelope = 0.0;
        state.last_character_input = 0.0;
        state.character_edge_memory = 0.0;
        state.beat_position = render.position_beats;
        state.last_trigger_revision = render.trigger_revision;
        return;
    }

    let transport_stop_fade_frames = transport_stop_fade_frames(sample_rate);
    let explicitly_retriggered = render.trigger_revision > state.last_trigger_revision;
    if state.last_transport_running && !render.is_transport_running {
        state.transport_stop_latched = true;
        state.transport_stop_fade_frames_remaining = transport_stop_fade_frames;
    } else if render.is_transport_running || explicitly_retriggered {
        state.transport_stop_latched = false;
        state.transport_stop_fade_frames_remaining = 0;
    }
    state.last_transport_running = render.is_transport_running;
    if state.transport_stop_latched && state.transport_stop_fade_frames_remaining == 0 {
        state.was_active = false;
        state.envelope = 0.0;
        state.last_character_input = 0.0;
        state.character_edge_memory = 0.0;
        return;
    }

    if !state.was_active {
        state.beat_position = render.position_beats;
        state.envelope = 1.0;
        state.last_step = w30_current_step(render.position_beats, render);
        state.oscillator_phase = 0.0;
        state.lfo_phase = 0.0;
        state.source_sample_cursor = 0.0;
        state.pad_playback_cursor = w30_chop_slice_cursor(render, state.last_step);
        state.pad_playback_age_frames = 0;
        state.last_character_input = 0.0;
        state.character_edge_memory = 0.0;
        state.last_source_window_signature = w30_source_window_signature(render);
        state.last_pad_playback_signature = w30_pad_playback_signature(render);
        state.last_trigger_revision = render.trigger_revision;
        state.was_active = true;
    }

    let source_window_signature = w30_source_window_signature(render);
    if source_window_signature != state.last_source_window_signature {
        state.last_source_window_signature = source_window_signature;
        state.source_sample_cursor = 0.0;
        state.last_character_input = 0.0;
        state.character_edge_memory = 0.0;
    }
    let pad_playback_signature = w30_pad_playback_signature(render);
    if pad_playback_signature != state.last_pad_playback_signature {
        state.last_pad_playback_signature = pad_playback_signature;
        state.pad_playback_cursor = w30_chop_slice_cursor(render, state.last_step);
        state.pad_playback_age_frames = 0;
        state.last_character_input = 0.0;
        state.character_edge_memory = 0.0;
    }

    if render.trigger_revision > state.last_trigger_revision {
        state.last_trigger_revision = render.trigger_revision;
        state.envelope = state.envelope.max(
            w30_trigger_envelope(render) * (0.85 + render.trigger_velocity.clamp(0.0, 1.0) * 0.3),
        );
        state.oscillator_phase = 0.0;
        state.pad_playback_cursor = w30_chop_slice_cursor(render, state.last_step);
        state.pad_playback_age_frames = 0;
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
                    if w30_pad_playback_active(render) {
                        state.pad_playback_cursor = w30_chop_slice_cursor(render, step);
                        state.pad_playback_age_frames = 0;
                        state.last_character_input = 0.0;
                        state.character_edge_memory = 0.0;
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
        let stop_gain = transport_stop_gain(
            state.transport_stop_latched,
            &mut state.transport_stop_fade_frames_remaining,
            transport_stop_fade_frames,
        );
        let sample = waveform
            * state.envelope
            * tremolo
            * w30_render_gain(render, transport_running)
            * stop_gain;
        if transport_running && !w30_pad_playback_active(render) {
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
        let sample = w30_pad_playback_sample(&render.pad_playback, state, sample_rate);
        return w30_source_backed_character(sample, render.grit_level, state);
    }

    if w30_source_window_active(render) {
        let sample = w30_source_window_sample(&render.source_window_preview, state);
        return w30_source_backed_character(sample, render.grit_level, state);
    }

    0.0
}

/// Add source-reactive sampler bite without introducing an oscillator or fallback voice.
///
/// The differentiated edge follows actual source motion, while asymmetric saturation exposes
/// upper harmonics already implied by that motion. At zero grit the sample remains bit-for-bit
/// on the clean branch.
fn w30_source_backed_character(
    sample: f32,
    grit_level: f32,
    state: &mut W30PreviewCallbackState,
) -> f32 {
    let grit = grit_level.clamp(0.0, 1.0);
    if grit <= f32::EPSILON {
        state.last_character_input = sample;
        state.character_edge_memory = 0.0;
        return sample;
    }

    const EDGE_MEMORY: f32 = 0.72;
    const MIN_DRIVE: f32 = 1.0;
    const DRIVE_RANGE: f32 = 5.2;
    const EDGE_RANGE: f32 = 1.35;
    const TRANSIENT_DRIVE_MIN: f32 = 6.0;
    const TRANSIENT_DRIVE_RANGE: f32 = 18.0;
    const FOLD_DRIVE_MIN: f32 = 1.4;
    const FOLD_DRIVE_RANGE: f32 = 2.8;
    const SATURATED_BODY_SHARE: f32 = 0.64;
    const SOURCE_FOLD_SHARE: f32 = 0.24;
    const TRANSIENT_BITE_SHARE: f32 = 0.12;
    const WET_RANGE: f32 = 0.84;
    const ASYMMETRY_RANGE: f32 = 0.12;

    let raw_edge = sample - state.last_character_input;
    state.last_character_input = sample;
    state.character_edge_memory =
        state.character_edge_memory * EDGE_MEMORY + raw_edge * (1.0 - EDGE_MEMORY);

    let edge_emphasis = state.character_edge_memory * (grit * EDGE_RANGE);
    let driven_input = sample + edge_emphasis;
    let drive = MIN_DRIVE + grit * DRIVE_RANGE;
    let asymmetry = grit * ASYMMETRY_RANGE;
    let saturated =
        ((driven_input * drive + asymmetry).tanh() - asymmetry.tanh()) / drive.tanh().max(0.001);
    // Wavefolding remains entirely source-driven: source amplitude bends its own phase and
    // exposes a sustained hostile upper-mid edge without adding a free-running tone.
    let fold_drive = FOLD_DRIVE_MIN + grit * FOLD_DRIVE_RANGE;
    let source_fold = (driven_input * fold_drive * std::f32::consts::PI).sin();
    let transient_drive = TRANSIENT_DRIVE_MIN + grit * TRANSIENT_DRIVE_RANGE;
    let transient_bite = (raw_edge * transient_drive).tanh();
    let bitten = saturated * SATURATED_BODY_SHARE
        + source_fold * SOURCE_FOLD_SHARE
        + transient_bite * TRANSIENT_BITE_SHARE;
    let wet = grit * WET_RANGE;

    (sample * (1.0 - wet) + bitten * wet).clamp(-0.98, 0.98)
}

fn w30_source_window_active(render: &RealtimeW30PreviewRenderState) -> bool {
    !matches!(render.mode, W30PreviewRenderMode::Idle)
        && render.source_window_preview.sample_count > 0
}

fn w30_pad_playback_active(render: &RealtimeW30PreviewRenderState) -> bool {
    !matches!(render.mode, W30PreviewRenderMode::Idle) && render.pad_playback.sample_count > 0
}

pub(super) fn w30_pad_playback_sample(
    window: &RealtimeW30PadPlaybackSampleWindow,
    state: &mut W30PreviewCallbackState,
    output_sample_rate: u32,
) -> f32 {
    let sample_count = window.sample_count.min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
    if sample_count == 0 {
        return 0.0;
    }

    let source_sample_rate = window.source_sample_rate.max(1);
    let playback_frame_count = window.playback_frame_count.max(1);
    let output_frame_count = (playback_frame_count as f64 * f64::from(output_sample_rate.max(1))
        / f64::from(source_sample_rate))
    .max(1.0);
    let cursor_increment = (sample_count as f64 / output_frame_count
        * f64::from(window.playback_rate.clamp(0.25, 4.0))) as f32;
    let logical_cursor = state.pad_playback_cursor;
    if !window.loop_enabled && logical_cursor >= sample_count as f32 {
        return 0.0;
    }

    let wrapped_cursor = if window.loop_enabled {
        logical_cursor % sample_count as f32
    } else {
        logical_cursor.min(sample_count.saturating_sub(1) as f32)
    };
    let sample = interpolated_pad_sample(window, sample_count, wrapped_cursor);
    let sample = apply_pad_loop_crossfade(window, sample_count, wrapped_cursor, sample);
    let sample = apply_pad_edge_envelope(window, state, sample_count, wrapped_cursor, sample);
    state.pad_playback_cursor = if window.loop_enabled {
        (logical_cursor + cursor_increment) % sample_count as f32
    } else {
        logical_cursor + cursor_increment
    };
    state.pad_playback_age_frames = state.pad_playback_age_frames.saturating_add(1);
    sample
}

fn interpolated_pad_sample(
    window: &RealtimeW30PadPlaybackSampleWindow,
    sample_count: usize,
    cursor: f32,
) -> f32 {
    let base = cursor.floor() as usize % sample_count;
    let next = (base + 1).min(sample_count - 1);
    let fraction = cursor.fract();
    let index = if window.reverse {
        sample_count - 1 - base
    } else {
        base
    };
    let next_index = if window.reverse {
        sample_count - 1 - next
    } else {
        next
    };
    window.samples[index] + (window.samples[next_index] - window.samples[index]) * fraction
}

fn apply_pad_loop_crossfade(
    window: &RealtimeW30PadPlaybackSampleWindow,
    sample_count: usize,
    cursor: f32,
    sample: f32,
) -> f32 {
    let crossfade = window.loop_crossfade_sample_count.min(sample_count / 2);
    if !window.loop_enabled || crossfade == 0 || cursor < (sample_count - crossfade) as f32 {
        return sample;
    }

    let fade_position = cursor - (sample_count - crossfade) as f32;
    let mix = (fade_position / crossfade as f32).clamp(0.0, 1.0);
    let head = interpolated_pad_sample(window, sample_count, fade_position);
    sample * (1.0 - mix) + head * mix
}

fn apply_pad_edge_envelope(
    window: &RealtimeW30PadPlaybackSampleWindow,
    state: &W30PreviewCallbackState,
    sample_count: usize,
    cursor: f32,
    sample: f32,
) -> f32 {
    const EDGE_FRAMES: f32 = 64.0;
    let attack = (state.pad_playback_age_frames as f32 / EDGE_FRAMES).clamp(0.0, 1.0);
    let release = if window.loop_enabled {
        1.0
    } else {
        ((sample_count as f32 - cursor) / EDGE_FRAMES).clamp(0.0, 1.0)
    };
    sample * attack.min(release)
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

pub(super) fn w30_pad_playback_signature(render: &RealtimeW30PreviewRenderState) -> u64 {
    let base = render
        .pad_playback
        .source_start_frame
        .wrapping_mul(31)
        .wrapping_add(render.pad_playback.source_end_frame)
        .wrapping_add(render.pad_playback.sample_count as u64)
        .wrapping_add(u64::from(render.pad_playback.loop_enabled))
        .wrapping_add(u64::from(render.pad_playback.reverse).wrapping_mul(17))
        .wrapping_add(u64::from(render.pad_playback.playback_rate.to_bits()).wrapping_mul(19))
        .wrapping_add((render.pad_playback.chop_slice_count as u64).wrapping_mul(23));
    render
        .pad_playback
        .chop_slice_starts
        .iter()
        .take(
            render
                .pad_playback
                .chop_slice_count
                .min(W30_PAD_CHOP_SLICE_COUNT),
        )
        .fold(base, |signature, start| {
            signature.wrapping_mul(31).wrapping_add(u64::from(*start))
        })
}

pub(super) fn w30_chop_slice_cursor(render: &RealtimeW30PreviewRenderState, step: i64) -> f32 {
    let count = render
        .pad_playback
        .chop_slice_count
        .min(W30_PAD_CHOP_SLICE_COUNT);
    if count == 0 {
        return 0.0;
    }
    let sequence_index = if render.pad_playback.playback_rate < 0.95 {
        step.saturating_mul(3).rem_euclid(count as i64) as usize
    } else {
        step.rem_euclid(count as i64) as usize
    };
    render.pad_playback.chop_slice_starts[sequence_index]
        .min(render.pad_playback.sample_count.saturating_sub(1) as u32) as f32
}

pub(super) fn render_w30_resample_tap_buffer(
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
        state.last_transport_running = render.is_transport_running;
        state.transport_stop_latched = false;
        state.transport_stop_fade_frames_remaining = 0;
        state.envelope = 0.0;
        state.beat_position = 0.0;
        return;
    }

    let transport_stop_fade_frames = transport_stop_fade_frames(sample_rate);
    if state.last_transport_running && !render.is_transport_running {
        state.transport_stop_latched = true;
        state.transport_stop_fade_frames_remaining = transport_stop_fade_frames;
    } else if render.is_transport_running {
        state.transport_stop_latched = false;
        state.transport_stop_fade_frames_remaining = 0;
    } else if state.transport_stop_latched && state.transport_stop_fade_frames_remaining == 0 {
        state.was_active = false;
        state.envelope = 0.0;
        return;
    }
    state.last_transport_running = render.is_transport_running;

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
            * w30_resample_render_gain(render, transport_running)
            * transport_stop_gain(
                state.transport_stop_latched,
                &mut state.transport_stop_fade_frames_remaining,
                transport_stop_fade_frames,
            );
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

fn transport_stop_fade_frames(sample_rate: u32) -> u32 {
    (sample_rate / 200).max(1)
}

fn transport_stop_gain(latched: bool, remaining_frames: &mut u32, total_frames: u32) -> f32 {
    if !latched {
        return 1.0;
    }
    if *remaining_frames == 0 {
        return 0.0;
    }

    let gain = *remaining_frames as f32 / total_frames.max(1) as f32;
    *remaining_frames = remaining_frames.saturating_sub(1);
    gain
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
    if w30_pad_playback_active(render) {
        return if render.pad_playback.playback_rate < 0.95 {
            4
        } else {
            2
        };
    }
    match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 1,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => 2,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => 3,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => 2,
        Some(W30PreviewSourceProfile::PromotedAudition) => 4,
    }
}

pub(super) fn should_trigger_w30_step(render: &RealtimeW30PreviewRenderState, step: i64) -> bool {
    if w30_pad_playback_active(render) {
        return render.pad_playback.playback_rate >= 0.95
            || !matches!(step.rem_euclid(16), 3 | 7 | 11);
    }
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
