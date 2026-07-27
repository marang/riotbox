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
    let pad_grid_gate = w30_pad_grid_gate(render, sample_rate);

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
        let waveform = w30_preview_waveform_for_frame(render, state, sample_rate, pad_grid_gate);
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
    pad_grid_gate: Option<W30PadGridGate>,
) -> f32 {
    if w30_pad_playback_active(render) {
        let sample = w30_pad_playback_sample(&render.pad_playback, state, sample_rate);
        let characterized = w30_source_backed_character(sample, render.grit_level, state);
        return characterized * w30_pad_grid_gate_gain(pad_grid_gate, state);
    }

    if w30_source_window_active(render) {
        let sample = w30_source_window_sample(&render.source_window_preview, state);
        return w30_source_backed_character(sample, render.grit_level, state);
    }

    0.0
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct W30PadGridGate {
    fade_start_frames: f32,
    gate_end_frames: f32,
    fade_frames: f32,
}

pub(super) fn w30_pad_grid_gate(
    render: &RealtimeW30PreviewRenderState,
    sample_rate: u32,
) -> Option<W30PadGridGate> {
    let gate_fraction = render.pad_playback.gate_step_fraction.clamp(0.0, 1.0);
    if !gate_fraction.is_finite()
        || gate_fraction <= f32::EPSILON
        || !render.tempo_bpm.is_finite()
        || render.tempo_bpm <= 0.0
    {
        return None;
    }

    const GATE_FADE_STEP_FRACTION: f32 = 0.10;
    let subdivision = w30_preview_subdivision(render).max(1) as f32;
    let step_frames = sample_rate.max(1) as f32 * 60.0 / render.tempo_bpm.max(1.0) / subdivision;
    let gate_end = step_frames * gate_fraction;
    let fade_frames = (step_frames * GATE_FADE_STEP_FRACTION).max(1.0);
    let fade_start = (gate_end - fade_frames).max(0.0);
    Some(W30PadGridGate {
        fade_start_frames: fade_start,
        gate_end_frames: gate_end,
        fade_frames,
    })
}

pub(super) fn w30_pad_grid_gate_gain(
    gate: Option<W30PadGridGate>,
    state: &W30PreviewCallbackState,
) -> f32 {
    let Some(gate) = gate else {
        return 1.0;
    };
    let age = state.pad_playback_age_frames as f32;

    if age <= gate.fade_start_frames {
        1.0
    } else if age >= gate.gate_end_frames {
        0.0
    } else {
        1.0 - (age - gate.fade_start_frames) / gate.fade_frames
    }
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
        .wrapping_add(u64::from(render.pad_playback.gate_step_fraction.to_bits()).wrapping_mul(29))
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
        && render.source_audio.sample_count > 0
        && render.music_bus_level > 0.0;

    if !active {
        state.was_active = false;
        state.last_transport_running = render.is_transport_running;
        state.transport_stop_latched = false;
        state.transport_stop_fade_frames_remaining = 0;
        state.envelope = 0.0;
        state.beat_position = render.position_beats.max(0.0);
        state.source_sample_cursor = 0.0;
        state.hard_attack_sample_cursor = 0.0;
        state.hard_attack_frames_remaining = 0;
        state.hard_attack_total_frames = 0;
        state.hard_attack_fade_in_frames = 0;
        state.hard_attack_mix = 0.0;
        state.hard_attack_head_mix = 0.0;
        state.hard_hit_preservation_frames_remaining = 0;
        state.hard_hit_preservation_total_frames = 0;
        state.hard_reverse_pickup_cursor = 0.0;
        state.hard_reverse_pickup_delay_frames_remaining = 0;
        state.hard_reverse_pickup_frames_remaining = 0;
        state.hard_reverse_pickup_total_frames = 0;
        state.hard_impact_active = false;
        state.hard_impact_frames_remaining = 0;
        state.hard_impact_total_frames = 0;
        state.hard_bite_filter_initialized = false;
        state.hard_low_impact_filter_initialized = false;
        state.hard_grit_held_sample = 0.0;
        state.hard_grit_hold_frames_remaining = 0;
        state.last_character_input = 0.0;
        state.character_edge_memory = 0.0;
        state.last_source_revision = render.source_audio.source_revision;
        state.last_variation_revision = 0;
        state.variation_transition_frames_remaining = 0;
        state.variation_transition_total_frames = 0;
        state.variation_transition_start_sample = 0.0;
        state.last_output_sample = 0.0;
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

    if !render.is_transport_running && !state.transport_stop_latched {
        state.was_active = false;
        state.envelope = 0.0;
        state.beat_position = render.position_beats.max(0.0);
        state.source_sample_cursor = 0.0;
        state.hard_attack_sample_cursor = 0.0;
        state.hard_attack_frames_remaining = 0;
        state.hard_attack_total_frames = 0;
        state.hard_attack_fade_in_frames = 0;
        state.hard_attack_mix = 0.0;
        state.hard_attack_head_mix = 0.0;
        state.hard_hit_preservation_frames_remaining = 0;
        state.hard_hit_preservation_total_frames = 0;
        state.hard_reverse_pickup_cursor = 0.0;
        state.hard_reverse_pickup_delay_frames_remaining = 0;
        state.hard_reverse_pickup_frames_remaining = 0;
        state.hard_reverse_pickup_total_frames = 0;
        state.hard_impact_active = false;
        state.hard_impact_frames_remaining = 0;
        state.hard_impact_total_frames = 0;
        state.hard_bite_filter_initialized = false;
        state.hard_low_impact_filter_initialized = false;
        state.hard_grit_held_sample = 0.0;
        state.hard_grit_hold_frames_remaining = 0;
        state.last_character_input = 0.0;
        state.character_edge_memory = 0.0;
        state.last_source_revision = render.source_audio.source_revision;
        state.last_variation_revision = render.variation_revision;
        state.variation_transition_frames_remaining = 0;
        state.variation_transition_total_frames = 0;
        state.variation_transition_start_sample = 0.0;
        state.last_output_sample = 0.0;
        return;
    }

    let source_changed =
        state.was_active && render.source_audio.source_revision != state.last_source_revision;
    let variation_changed =
        state.was_active && render.variation_revision != state.last_variation_revision;
    let beats_per_sample =
        if render.is_transport_running && render.tempo_bpm.is_finite() && render.tempo_bpm > 0.0 {
            f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1))
        } else {
            0.0
        };
    let position_tolerance = (beats_per_sample * 2.0).max(f64::EPSILON);
    let transport_seek = state.was_active
        && render.is_transport_running
        && (state.beat_position - render.position_beats.max(0.0)).abs() > position_tolerance;

    if !state.was_active {
        let preserve_frozen_base_start = (render.variation == W30ResampleTapVariation::Base
            || render.hard_policy == W30ResampleTapHardPolicy::Unavailable)
            && render.position_beats.abs() <= f64::EPSILON;
        sync_w30_resample_playhead(render, state, sample_rate, !preserve_frozen_base_start);
        state.last_source_revision = render.source_audio.source_revision;
        state.last_variation_revision = render.variation_revision;
        state.variation_transition_frames_remaining = 0;
        state.variation_transition_total_frames = 0;
        state.variation_transition_start_sample = 0.0;
        state.last_output_sample = 0.0;
        state.was_active = true;
    } else if source_changed || variation_changed || transport_seek {
        state.last_source_revision = render.source_audio.source_revision;
        state.last_variation_revision = render.variation_revision;
        let transition_frames = (sample_rate / 200).max(1);
        state.variation_transition_frames_remaining = transition_frames;
        state.variation_transition_total_frames = transition_frames;
        state.variation_transition_start_sample = state.last_output_sample;
        sync_w30_resample_playhead(render, state, sample_rate, true);
    }

    let transport_running = render.is_transport_running;
    let frame_count = data.len() / channel_count.max(1);
    let envelope_decay = w30_resample_decay(render, sample_rate);

    for frame_index in 0..frame_count {
        if transport_running {
            let step =
                (state.beat_position * f64::from(w30_resample_subdivision(render))).floor() as i64;
            if step != state.last_step {
                state.last_step = step;
                if should_trigger_w30_resample_step(render, step) {
                    state.envelope = w30_resample_trigger_envelope(render);
                    trigger_w30_resample_attack(render, state, sample_rate, step);
                }
            }
        } else {
            state.envelope = state.envelope.max(0.42) * 0.99975;
        }

        let source_sample = w30_resample_source_sample(render, state, sample_rate);
        let body = w30_resample_source_character(source_sample, render, state);
        let pickup_and_body = w30_resample_reverse_pickup_sample(render, state, sample_rate, body);
        let attack_and_body =
            w30_resample_attack_body_sample(render, state, sample_rate, pickup_and_body);
        let voice = match render.hard_low_impact.recipe {
            W30ResampleLowImpactRecipe::SourceKickImpactV2 => {
                w30_resample_kick_impact_v2_sample(render, state, attack_and_body)
            }
            W30ResampleLowImpactRecipe::SourceHitShaperV3 => {
                w30_resample_hit_shaper_v3_sample(render, state, attack_and_body)
            }
            W30ResampleLowImpactRecipe::Unavailable
            | W30ResampleLowImpactRecipe::SourceLowTransientPunchV1 => {
                let bitten = w30_resample_hard_gesture_bite_sample(render, state, attack_and_body);
                let gritted = w30_resample_hard_grit_sample(render, state, sample_rate, bitten);
                w30_resample_low_impact_sample(render, state, attack_and_body, gritted)
            }
        };
        let voice = w30_resample_hard_impact_articulation_sample(render, state, sample_rate, voice);
        let voice =
            w30_resample_calibrated_hit_preservation_sample(render, state, sample_rate, voice);
        const RESAMPLE_TAP_VOICE_CEILING: f32 = 0.92;
        let hard_output_gain = if render.variation == W30ResampleTapVariation::HardDamage {
            render.hard_output_gain.clamp(0.25, 1.25)
        } else {
            1.0
        };
        let target_sample = (voice
            * state.envelope
            * w30_resample_render_gain(render, transport_running)
            * hard_output_gain
            * transport_stop_gain(
                state.transport_stop_latched,
                &mut state.transport_stop_fade_frames_remaining,
                transport_stop_fade_frames,
            ))
        .clamp(-RESAMPLE_TAP_VOICE_CEILING, RESAMPLE_TAP_VOICE_CEILING);
        let sample = if state.variation_transition_frames_remaining > 0 {
            let progress = 1.0
                - state.variation_transition_frames_remaining as f32
                    / state.variation_transition_total_frames.max(1) as f32;
            state.variation_transition_frames_remaining = state
                .variation_transition_frames_remaining
                .saturating_sub(1);
            state.variation_transition_start_sample * (1.0 - progress) + target_sample * progress
        } else {
            target_sample
        };
        state.last_output_sample = sample;
        if transport_running {
            state.envelope *= envelope_decay;
        }

        let base = frame_index * channel_count;
        for channel in 0..channel_count {
            data[base + channel] += sample;
        }

        state.beat_position += beats_per_sample;
    }
}

fn sync_w30_resample_playhead(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    output_sample_rate: u32,
    prime_edge_history: bool,
) {
    state.hard_grit_held_sample = 0.0;
    state.hard_grit_hold_frames_remaining = 0;
    state.hard_attack_mix = 0.0;
    state.hard_attack_head_mix = 0.0;
    state.hard_hit_preservation_frames_remaining = 0;
    state.hard_hit_preservation_total_frames = 0;
    state.hard_reverse_pickup_cursor = 0.0;
    state.hard_reverse_pickup_delay_frames_remaining = 0;
    state.hard_reverse_pickup_frames_remaining = 0;
    state.hard_reverse_pickup_total_frames = 0;
    state.hard_impact_active = false;
    state.hard_impact_frames_remaining = 0;
    state.hard_impact_total_frames = 0;
    state.hard_low_impact_filter_initialized = false;
    let position_beats = render.position_beats.max(0.0);
    let step = (position_beats * f64::from(w30_resample_subdivision(render))).floor() as i64;
    state.beat_position = position_beats;
    state.last_step = step;
    if render.variation == W30ResampleTapVariation::HardDamage
        && render.hard_policy == W30ResampleTapHardPolicy::SourceTransientChop
    {
        state.envelope = 1.0;
        state.source_sample_cursor =
            w30_resample_phase_cursor(render, output_sample_rate, position_beats);
        if should_trigger_w30_resample_step(render, step) {
            trigger_w30_resample_attack(render, state, output_sample_rate, step);
        } else {
            state.hard_attack_frames_remaining = 0;
            state.hard_attack_total_frames = 0;
            state.hard_attack_fade_in_frames = 0;
            state.hard_bite_filter_initialized = false;
        }
    } else {
        state.envelope = 1.0;
        state.source_sample_cursor =
            w30_resample_phase_cursor(render, output_sample_rate, position_beats);
        state.hard_attack_frames_remaining = 0;
        state.hard_attack_total_frames = 0;
        state.hard_attack_fade_in_frames = 0;
        state.hard_bite_filter_initialized = false;
    }
    if prime_edge_history {
        prime_w30_resample_edge_history(render, state, output_sample_rate);
    } else {
        state.last_character_input = 0.0;
        state.character_edge_memory = 0.0;
    }
    configure_w30_resample_hard_bite(render, state, output_sample_rate);
    configure_w30_resample_low_impact(render, state, output_sample_rate);
}

fn trigger_w30_resample_attack(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    output_sample_rate: u32,
    step: i64,
) {
    let slot = step.rem_euclid(W30_RESAMPLE_HARD_SLICE_COUNT as i64) as usize;
    let sample_count = render
        .source_audio
        .sample_count
        .min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    if sample_count == 0 {
        state.hard_attack_frames_remaining = 0;
        state.hard_attack_total_frames = 0;
        state.hard_attack_fade_in_frames = 0;
        state.hard_attack_mix = 0.0;
        state.hard_attack_head_mix = 0.0;
        state.hard_reverse_pickup_delay_frames_remaining = 0;
        state.hard_reverse_pickup_frames_remaining = 0;
        state.hard_reverse_pickup_total_frames = 0;
        state.hard_impact_active = false;
        state.hard_impact_frames_remaining = 0;
        state.hard_impact_total_frames = 0;
        state.hard_bite_filter_initialized = false;
        return;
    }
    let cursor_increment =
        sample_count as f64 / w30_resample_cycle_output_frames(render, output_sample_rate);
    if render.hard_gesture.recipe == W30ResampleHardGestureRecipe::SourceReverseIntoImpactV1
        && slot == usize::from(render.hard_gesture.pickup_slot)
    {
        let pickup_frames = render
            .hard_gesture
            .recipe
            .pickup_duration_frames(output_sample_rate)
            .max(1);
        let step_frames = if render.tempo_bpm.is_finite() && render.tempo_bpm > 0.0 {
            (f64::from(output_sample_rate.max(1)) * 30.0 / f64::from(render.tempo_bpm))
                .round()
                .max(1.0) as u32
        } else {
            pickup_frames
        };
        let impact_slot =
            usize::from(render.hard_gesture.impact_slot).min(W30_RESAMPLE_HARD_SLICE_COUNT - 1);
        let impact_cursor = f64::from(render.hard_slice_cursors[impact_slot])
            .min(sample_count.saturating_sub(1) as f64);
        state.hard_reverse_pickup_cursor = (impact_cursor
            + cursor_increment * f64::from(pickup_frames.saturating_sub(1)))
        .rem_euclid(sample_count as f64);
        state.hard_reverse_pickup_delay_frames_remaining =
            step_frames.saturating_sub(pickup_frames);
        state.hard_reverse_pickup_frames_remaining = pickup_frames.min(step_frames);
        state.hard_reverse_pickup_total_frames = pickup_frames.min(step_frames);
        state.hard_attack_frames_remaining = 0;
        state.hard_attack_total_frames = 0;
        state.hard_attack_fade_in_frames = 0;
        state.hard_attack_mix = 0.0;
        state.hard_attack_head_mix = 0.0;
        state.hard_impact_active = false;
        state.hard_impact_frames_remaining = 0;
        state.hard_impact_total_frames = 0;
        state.hard_bite_filter_initialized = false;
        return;
    }
    state.hard_reverse_pickup_delay_frames_remaining = 0;
    state.hard_reverse_pickup_frames_remaining = 0;
    state.hard_reverse_pickup_total_frames = 0;
    state.hard_impact_active = render.hard_gesture.recipe
        == W30ResampleHardGestureRecipe::SourceReverseIntoImpactV1
        && slot == usize::from(render.hard_gesture.impact_slot);
    state.hard_impact_total_frames = if state.hard_impact_active {
        render
            .hard_gesture
            .recipe
            .body_end_frames(output_sample_rate)
            .max(1)
    } else {
        0
    };
    state.hard_impact_frames_remaining = state.hard_impact_total_frames;
    let onset_cursor = w30_resample_step_cursor(render, step);
    let proxy_attack_length = u32::from(render.hard_attack_lengths[slot].max(1));
    let mut attack_frames = (f64::from(proxy_attack_length) / cursor_increment)
        .round()
        .clamp(1.0, f64::from(output_sample_rate.max(1)) * 0.08) as u32;
    attack_frames = attack_frames.max(
        render
            .hard_low_impact
            .recipe
            .minimum_hit_window_frames(output_sample_rate),
    );
    let fade_in_frames = if state.hard_impact_active {
        0
    } else {
        (output_sample_rate / 2_000)
            .max(1)
            .min((attack_frames / 4).max(1))
    };
    state.hard_attack_sample_cursor = if state.hard_impact_active {
        onset_cursor
    } else {
        (onset_cursor - cursor_increment * f64::from(fade_in_frames))
            .rem_euclid(sample_count as f64)
    };
    state.hard_attack_total_frames = attack_frames.saturating_add(fade_in_frames);
    state.hard_attack_frames_remaining = state.hard_attack_total_frames;
    state.hard_attack_fade_in_frames = fade_in_frames;
    let preservation_frames = render
        .hard_low_impact
        .recipe
        .calibrated_hit_preservation_frames(output_sample_rate);
    let preservation_fade_frames = render
        .hard_low_impact
        .recipe
        .calibrated_hit_preservation_fade_frames(output_sample_rate);
    state.hard_hit_preservation_total_frames =
        preservation_frames.saturating_add(preservation_fade_frames);
    state.hard_hit_preservation_frames_remaining = state.hard_hit_preservation_total_frames;
    if render.hard_low_impact.recipe == W30ResampleLowImpactRecipe::SourceHitShaperV3 {
        state.hard_body_eq_z1 = 0.0;
        state.hard_body_eq_z2 = 0.0;
        state.hard_low_impact_filter_initialized = false;
    }
}

fn w30_resample_reverse_pickup_sample(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    output_sample_rate: u32,
    body: f32,
) -> f32 {
    if render.variation != W30ResampleTapVariation::HardDamage
        || render.hard_policy != W30ResampleTapHardPolicy::SourceTransientChop
        || render.hard_gesture.recipe != W30ResampleHardGestureRecipe::SourceReverseIntoImpactV1
    {
        state.hard_reverse_pickup_delay_frames_remaining = 0;
        state.hard_reverse_pickup_frames_remaining = 0;
        state.hard_reverse_pickup_total_frames = 0;
        return body;
    }
    if state.hard_reverse_pickup_delay_frames_remaining > 0 {
        state.hard_reverse_pickup_delay_frames_remaining = state
            .hard_reverse_pickup_delay_frames_remaining
            .saturating_sub(1);
        return body;
    }
    if state.hard_reverse_pickup_frames_remaining == 0 {
        return body;
    }
    let sample_count = render
        .source_audio
        .sample_count
        .min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    if sample_count == 0 {
        state.hard_reverse_pickup_frames_remaining = 0;
        return body;
    }
    let reverse = w30_resample_source_sample_at(render, state.hard_reverse_pickup_cursor);
    let cursor_increment =
        sample_count as f64 / w30_resample_cycle_output_frames(render, output_sample_rate);
    state.hard_reverse_pickup_cursor =
        (state.hard_reverse_pickup_cursor - cursor_increment).rem_euclid(sample_count as f64);
    let elapsed = state
        .hard_reverse_pickup_total_frames
        .saturating_sub(state.hard_reverse_pickup_frames_remaining);
    let pickup_mix =
        smoothstep(elapsed as f32 / state.hard_reverse_pickup_total_frames.max(1) as f32);
    let boundary_ramp_frames = (output_sample_rate / 100).max(1);
    let boundary_progress = if state.hard_reverse_pickup_frames_remaining <= boundary_ramp_frames {
        1.0 - smoothstep(
            state.hard_reverse_pickup_frames_remaining as f32 / boundary_ramp_frames as f32,
        )
    } else {
        0.0
    };
    let pickup_gain = render
        .hard_gesture
        .pickup_gain
        .clamp(W30_RESAMPLE_H13_MIN_PICKUP_GAIN, 1.0);
    let impact_gain = render
        .hard_gesture
        .impact_level_compensation
        .clamp(W30_RESAMPLE_H13_MIN_IMPACT_LEVEL_COMPENSATION, 1.0);
    let reverse_gain = pickup_gain * (1.0 - boundary_progress) + impact_gain * boundary_progress;
    state.hard_reverse_pickup_frames_remaining =
        state.hard_reverse_pickup_frames_remaining.saturating_sub(1);
    body * (1.0 - pickup_mix) + reverse * reverse_gain * pickup_mix
}

fn w30_resample_attack_body_sample(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    output_sample_rate: u32,
    body: f32,
) -> f32 {
    if render.variation != W30ResampleTapVariation::HardDamage
        || render.hard_policy != W30ResampleTapHardPolicy::SourceTransientChop
        || state.hard_attack_frames_remaining == 0
    {
        state.hard_attack_mix = 0.0;
        state.hard_attack_head_mix = 0.0;
        return body;
    }

    let sample_count = render
        .source_audio
        .sample_count
        .min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    if sample_count == 0 {
        state.hard_attack_frames_remaining = 0;
        return body;
    }
    let attack = w30_resample_source_sample_at(render, state.hard_attack_sample_cursor);
    let cursor_increment =
        sample_count as f64 / w30_resample_cycle_output_frames(render, output_sample_rate);
    state.hard_attack_sample_cursor =
        (state.hard_attack_sample_cursor + cursor_increment).rem_euclid(sample_count as f64);

    let elapsed = state
        .hard_attack_total_frames
        .saturating_sub(state.hard_attack_frames_remaining);
    let fade_in_frames = state.hard_attack_fade_in_frames;
    let attack_frames = state
        .hard_attack_total_frames
        .saturating_sub(state.hard_attack_fade_in_frames)
        .max(1);
    let attack_elapsed = elapsed.saturating_sub(state.hard_attack_fade_in_frames);
    let release_start = attack_frames * 3 / 5;
    let attack_mix = if fade_in_frames > 0 && elapsed < fade_in_frames {
        smoothstep(elapsed as f32 / fade_in_frames as f32)
    } else if attack_elapsed <= release_start {
        1.0
    } else {
        let release_frames = attack_frames.saturating_sub(release_start).max(1);
        1.0 - smoothstep(
            attack_elapsed.saturating_sub(release_start) as f32 / release_frames as f32,
        )
    };
    state.hard_attack_mix = attack_mix;
    let head_frames = (output_sample_rate / 55).max(1);
    state.hard_attack_head_mix = if fade_in_frames > 0 && elapsed < fade_in_frames {
        smoothstep(elapsed as f32 / fade_in_frames as f32)
    } else {
        1.0 - smoothstep(attack_elapsed as f32 / head_frames as f32)
    };
    state.hard_attack_frames_remaining = state.hard_attack_frames_remaining.saturating_sub(1);
    body * (1.0 - attack_mix) + attack * attack_mix
}

fn w30_resample_hard_impact_articulation_sample(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    output_sample_rate: u32,
    voice: f32,
) -> f32 {
    if render.variation != W30ResampleTapVariation::HardDamage
        || render.hard_policy != W30ResampleTapHardPolicy::SourceTransientChop
        || render.hard_gesture.recipe != W30ResampleHardGestureRecipe::SourceReverseIntoImpactV1
        || !state.hard_impact_active
        || state.hard_impact_frames_remaining == 0
    {
        return voice;
    }
    let elapsed = state
        .hard_impact_total_frames
        .saturating_sub(state.hard_impact_frames_remaining);
    state.hard_impact_frames_remaining = state.hard_impact_frames_remaining.saturating_sub(1);
    if state.hard_impact_frames_remaining == 0 {
        state.hard_impact_active = false;
    }
    let impact_compensation = render
        .hard_gesture
        .impact_level_compensation
        .clamp(W30_RESAMPLE_H13_MIN_IMPACT_LEVEL_COMPENSATION, 1.0);
    let body_start = render
        .hard_gesture
        .recipe
        .body_start_frames(output_sample_rate);
    let body_end = render
        .hard_gesture
        .recipe
        .body_end_frames(output_sample_rate)
        .max(body_start + 1);
    if elapsed < body_start || elapsed >= body_end {
        return voice * impact_compensation;
    }
    let body_gain = render
        .hard_gesture
        .body_gain
        .clamp(1.0, W30_RESAMPLE_H13_MAX_BODY_GAIN);
    let transition_frames = (output_sample_rate / 400).max(1);
    let fade_in = smoothstep(elapsed.saturating_sub(body_start) as f32 / transition_frames as f32);
    let fade_out = smoothstep(body_end.saturating_sub(elapsed) as f32 / transition_frames as f32);
    let articulation = fade_in.min(fade_out);
    let gain = impact_compensation + (body_gain - impact_compensation) * articulation;
    (voice * gain).clamp(-0.98, 0.98)
}

fn configure_w30_resample_hard_bite(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    output_sample_rate: u32,
) {
    state.hard_bite_filter_initialized = false;
    if let Some((low_hz, high_hz)) = render.hard_attack_bite.band.cutoff_hz() {
        let sample_rate = output_sample_rate.max(1) as f32;
        state.hard_bite_low_alpha = 1.0 - (-std::f32::consts::TAU * low_hz / sample_rate).exp();
        state.hard_bite_high_alpha = 1.0 - (-std::f32::consts::TAU * high_hz / sample_rate).exp();
    } else {
        state.hard_bite_low_alpha = 0.0;
        state.hard_bite_high_alpha = 0.0;
    }
}

fn w30_resample_hard_gesture_bite_sample(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    sample: f32,
) -> f32 {
    // H4 replaces only the source-selected band with its RMS-matched nonlinear residual.
    // The complementary dry spectrum stays at unity while dry-band masking no longer hides
    // the distortion component.
    if render.variation != W30ResampleTapVariation::HardDamage
        || render.hard_policy != W30ResampleTapHardPolicy::SourceTransientChop
        || render.hard_attack_bite.band == W30ResampleAttackBiteBand::Unavailable
        || state.hard_bite_low_alpha <= 0.0
        || state.hard_bite_high_alpha <= 0.0
    {
        return sample;
    }
    let input_gain = render.hard_attack_bite.input_gain.clamp(1.0, 12.0);
    let output_gain = render.hard_attack_bite.output_gain.clamp(0.25, 12.0);
    if !state.hard_bite_filter_initialized {
        state.hard_bite_lowpass_low = sample;
        state.hard_bite_lowpass_high = sample;
        state.hard_bite_filter_initialized = true;
        return sample;
    }

    state.hard_bite_lowpass_low +=
        state.hard_bite_low_alpha * (sample - state.hard_bite_lowpass_low);
    state.hard_bite_lowpass_high +=
        state.hard_bite_high_alpha * (sample - state.hard_bite_lowpass_high);
    let selected_band = state.hard_bite_lowpass_high - state.hard_bite_lowpass_low;
    let drive_normalization = W30_RESAMPLE_HARD_BITE_NONLINEAR_DRIVE
        .tanh()
        .max(f32::EPSILON);
    let shaped_band = (selected_band * input_gain * W30_RESAMPLE_HARD_BITE_NONLINEAR_DRIVE).tanh()
        / drive_normalization
        / input_gain;
    let residual_band = (shaped_band - selected_band) * output_gain;
    (sample - selected_band + residual_band).clamp(-0.98, 0.98)
}

pub(super) fn configure_w30_resample_low_impact(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    output_sample_rate: u32,
) {
    state.hard_low_impact_filter_initialized = false;
    if let Some((low_hz, high_hz)) = render.hard_low_impact.recipe.cutoff_hz() {
        let sample_rate = output_sample_rate.max(1) as f32;
        state.hard_low_impact_low_alpha =
            1.0 - (-std::f32::consts::TAU * low_hz / sample_rate).exp();
        state.hard_low_impact_high_alpha =
            1.0 - (-std::f32::consts::TAU * high_hz / sample_rate).exp();
    } else {
        state.hard_low_impact_low_alpha = 0.0;
        state.hard_low_impact_high_alpha = 0.0;
    }
    if let Some((low_hz, high_hz)) = render.hard_low_impact.recipe.presence_cutoff_hz() {
        let sample_rate = output_sample_rate.max(1) as f32;
        state.hard_impact_presence_low_alpha =
            1.0 - (-std::f32::consts::TAU * low_hz / sample_rate).exp();
        state.hard_impact_presence_high_alpha =
            1.0 - (-std::f32::consts::TAU * high_hz / sample_rate).exp();
    } else {
        state.hard_impact_presence_low_alpha = 0.0;
        state.hard_impact_presence_high_alpha = 0.0;
    }
    let recipe = render.hard_low_impact.recipe;
    if recipe == W30ResampleLowImpactRecipe::SourceHitShaperV3 {
        let sample_rate = output_sample_rate.max(1) as f32;
        let omega = std::f32::consts::TAU * recipe.body_eq_center_hz().max(1.0) / sample_rate;
        let alpha = omega.sin() / (2.0 * recipe.body_eq_q().max(0.01));
        let amplitude = 10.0_f32.powf(recipe.body_eq_gain_db() / 40.0);
        let a0 = 1.0 + alpha / amplitude;
        state.hard_body_eq_b0 = (1.0 + alpha * amplitude) / a0;
        state.hard_body_eq_b1 = -2.0 * omega.cos() / a0;
        state.hard_body_eq_b2 = (1.0 - alpha * amplitude) / a0;
        state.hard_body_eq_a1 = -2.0 * omega.cos() / a0;
        state.hard_body_eq_a2 = (1.0 - alpha / amplitude) / a0;
    } else {
        state.hard_body_eq_b0 = 1.0;
        state.hard_body_eq_b1 = 0.0;
        state.hard_body_eq_b2 = 0.0;
        state.hard_body_eq_a1 = 0.0;
        state.hard_body_eq_a2 = 0.0;
    }
    state.hard_body_eq_z1 = 0.0;
    state.hard_body_eq_z2 = 0.0;
}

pub(super) fn w30_resample_kick_impact_v2_sample(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    source_attack: f32,
) -> f32 {
    if render.variation != W30ResampleTapVariation::HardDamage
        || render.hard_policy != W30ResampleTapHardPolicy::SourceTransientChop
        || render.hard_low_impact.recipe != W30ResampleLowImpactRecipe::SourceKickImpactV2
        || state.hard_attack_mix <= 0.0
    {
        return source_attack;
    }
    if !state.hard_low_impact_filter_initialized {
        state.hard_low_impact_lowpass_low = source_attack;
        state.hard_low_impact_lowpass_high = source_attack;
        state.hard_impact_presence_lowpass_low = source_attack;
        state.hard_impact_presence_lowpass_high = source_attack;
        state.hard_low_impact_filter_initialized = true;
        return source_attack;
    }

    state.hard_low_impact_lowpass_low +=
        state.hard_low_impact_low_alpha * (source_attack - state.hard_low_impact_lowpass_low);
    state.hard_low_impact_lowpass_high +=
        state.hard_low_impact_high_alpha * (source_attack - state.hard_low_impact_lowpass_high);
    state.hard_impact_presence_lowpass_low += state.hard_impact_presence_low_alpha
        * (source_attack - state.hard_impact_presence_lowpass_low);
    state.hard_impact_presence_lowpass_high += state.hard_impact_presence_high_alpha
        * (source_attack - state.hard_impact_presence_lowpass_high);

    let low_body = state.hard_low_impact_lowpass_high - state.hard_low_impact_lowpass_low;
    let head = state.hard_impact_presence_lowpass_high - state.hard_impact_presence_lowpass_low;
    (source_attack
        + low_body * render.hard_low_impact.recipe.parallel_attack_gain() * state.hard_attack_mix
        + head * render.hard_low_impact.recipe.parallel_head_gain() * state.hard_attack_head_mix)
        .clamp(-0.98, 0.98)
}

pub(super) fn w30_resample_hit_shaper_v3_sample(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    source_hit: f32,
) -> f32 {
    if render.variation != W30ResampleTapVariation::HardDamage
        || render.hard_policy != W30ResampleTapHardPolicy::SourceTransientChop
        || render.hard_low_impact.recipe != W30ResampleLowImpactRecipe::SourceHitShaperV3
        || state.hard_attack_mix <= 0.0
    {
        return source_hit;
    }
    if !state.hard_low_impact_filter_initialized {
        state.hard_low_impact_filter_initialized = true;
        return source_hit;
    }

    let recipe = render.hard_low_impact.recipe;
    let equalized_body = state.hard_body_eq_b0 * source_hit + state.hard_body_eq_z1;
    state.hard_body_eq_z1 = state.hard_body_eq_b1 * source_hit
        - state.hard_body_eq_a1 * equalized_body
        + state.hard_body_eq_z2;
    state.hard_body_eq_z2 =
        state.hard_body_eq_b2 * source_hit - state.hard_body_eq_a2 * equalized_body;
    let body_mix = state.hard_attack_mix * (1.0 - state.hard_attack_head_mix);
    let body_hit = source_hit + (equalized_body - source_hit) * body_mix;
    let shaped_head = normalized_soft_clip(body_hit, recipe.head_drive());
    let head_mix = recipe.head_wet() * state.hard_attack_head_mix;
    let shaped = body_hit + (shaped_head - body_hit) * head_mix;
    shaped.clamp(-0.98, 0.98)
}

fn normalized_soft_clip(sample: f32, drive: f32) -> f32 {
    let drive = drive.max(1.0);
    (sample * drive).tanh() / drive.tanh().max(f32::EPSILON)
}

pub(super) fn w30_resample_calibrated_hit_preservation_sample(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    output_sample_rate: u32,
    voice: f32,
) -> f32 {
    if render.variation != W30ResampleTapVariation::HardDamage
        || render.hard_policy != W30ResampleTapHardPolicy::SourceTransientChop
        || render.hard_low_impact.recipe != W30ResampleLowImpactRecipe::SourceHitShaperV3
    {
        return voice;
    }
    let recipe = render.hard_low_impact.recipe;
    let primary_hit_frames = recipe.minimum_hit_window_frames(output_sample_rate);
    let hold_frames = recipe.calibrated_hit_preservation_frames(output_sample_rate);
    let fade_frames = recipe
        .calibrated_hit_preservation_fade_frames(output_sample_rate)
        .max(1);
    let elapsed = state
        .hard_hit_preservation_total_frames
        .saturating_sub(state.hard_hit_preservation_frames_remaining);
    let active_target = if state.hard_hit_preservation_frames_remaining > 0 {
        state.hard_hit_preservation_frames_remaining = state
            .hard_hit_preservation_frames_remaining
            .saturating_sub(1);
        if elapsed < primary_hit_frames {
            W30_RESAMPLE_HIT_SHAPER_PRESERVED_OUTPUT_GAIN
        } else if elapsed < hold_frames {
            W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN
        } else {
            let fade = smoothstep(elapsed.saturating_sub(hold_frames) as f32 / fade_frames as f32);
            W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN
                + (render.hard_output_gain - W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN) * fade
        }
    } else {
        render.hard_output_gain
    };
    let step_position = state.beat_position * f64::from(w30_resample_subdivision(render));
    let next_step = step_position.floor() as i64 + 1;
    let next_slot = next_step.rem_euclid(W30_RESAMPLE_HARD_SLICE_COUNT as i64) as u8;
    let frames_per_step =
        f64::from(output_sample_rate.max(1)) * 30.0 / f64::from(render.tempo_bpm.max(f32::EPSILON));
    let frames_until_next_step = ((next_step as f64 - step_position) * frames_per_step)
        .round()
        .max(0.0) as u32;
    let preroll_frames = recipe.calibrated_hit_preroll_frames(output_sample_rate);
    let preroll_fade_frames = recipe
        .calibrated_hit_preroll_fade_frames(output_sample_rate)
        .max(1);
    let preroll_with_fade_frames = preroll_frames.saturating_add(preroll_fade_frames);
    let preroll_target = if render.hard_trigger_mask & (1_u8 << next_slot) != 0
        && frames_until_next_step <= preroll_with_fade_frames
    {
        let progress = smoothstep(
            preroll_with_fade_frames.saturating_sub(frames_until_next_step) as f32
                / preroll_fade_frames as f32,
        );
        render.hard_output_gain
            + (W30_RESAMPLE_HIT_SHAPER_PRESERVED_OUTPUT_GAIN - render.hard_output_gain) * progress
    } else {
        render.hard_output_gain
    };
    let target_output_gain = active_target.max(preroll_target);
    if target_output_gain <= render.hard_output_gain {
        return voice;
    }
    let compensation = (target_output_gain / render.hard_output_gain.max(f32::EPSILON))
        .clamp(1.0, W30_RESAMPLE_HIT_SHAPER_MAX_WINDOW_COMPENSATION_GAIN);
    (voice * compensation).clamp(-0.98, 0.98)
}

pub(super) fn w30_resample_low_impact_sample(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    source_attack: f32,
    processed: f32,
) -> f32 {
    if render.variation != W30ResampleTapVariation::HardDamage
        || render.hard_policy != W30ResampleTapHardPolicy::SourceTransientChop
        || render.hard_low_impact.recipe == W30ResampleLowImpactRecipe::Unavailable
        || state.hard_attack_mix <= 0.0
        || state.hard_low_impact_low_alpha <= 0.0
        || state.hard_low_impact_high_alpha <= 0.0
    {
        return processed;
    }
    if !state.hard_low_impact_filter_initialized {
        state.hard_low_impact_lowpass_low = source_attack;
        state.hard_low_impact_lowpass_high = source_attack;
        state.hard_low_impact_filter_initialized = true;
        return processed;
    }
    state.hard_low_impact_lowpass_low +=
        state.hard_low_impact_low_alpha * (source_attack - state.hard_low_impact_lowpass_low);
    state.hard_low_impact_lowpass_high +=
        state.hard_low_impact_high_alpha * (source_attack - state.hard_low_impact_lowpass_high);
    let source_low_band = state.hard_low_impact_lowpass_high - state.hard_low_impact_lowpass_low;
    let parallel_gain = render.hard_low_impact.recipe.parallel_attack_gain();
    (processed + source_low_band * parallel_gain * state.hard_attack_mix).clamp(-0.98, 0.98)
}

pub(super) fn w30_resample_hard_grit_sample(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    output_sample_rate: u32,
    sample: f32,
) -> f32 {
    if render.variation != W30ResampleTapVariation::HardDamage
        || render.hard_policy != W30ResampleTapHardPolicy::SourceTransientChop
    {
        state.hard_grit_held_sample = sample;
        state.hard_grit_hold_frames_remaining = 0;
        return sample;
    }

    let recipe = render.hard_policy.grit_recipe();
    let (Some(effective_sample_rate_hz), Some(quantization_levels)) = (
        recipe.effective_sample_rate_hz(),
        recipe.quantization_levels(),
    ) else {
        return sample;
    };
    let quantization_peak = f32::from((quantization_levels - 1) / 2);
    if state.hard_grit_hold_frames_remaining == 0 {
        state.hard_grit_held_sample = (sample * quantization_peak).round() / quantization_peak;
        state.hard_grit_hold_frames_remaining = (output_sample_rate.max(effective_sample_rate_hz)
            / effective_sample_rate_hz)
            .saturating_sub(1);
    } else {
        state.hard_grit_hold_frames_remaining =
            state.hard_grit_hold_frames_remaining.saturating_sub(1);
    }
    state.hard_grit_held_sample
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn w30_resample_phase_cursor(
    render: &RealtimeW30ResampleTapState,
    output_sample_rate: u32,
    position_beats: f64,
) -> f64 {
    let sample_count = render
        .source_audio
        .sample_count
        .min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    if sample_count == 0
        || !render.tempo_bpm.is_finite()
        || render.tempo_bpm <= 0.0
        || !position_beats.is_finite()
    {
        return 0.0;
    }
    let output_frame_count = w30_resample_cycle_output_frames(render, output_sample_rate);
    let elapsed_output_frames = position_beats.max(0.0) * 60.0 / f64::from(render.tempo_bpm)
        * f64::from(output_sample_rate);
    (elapsed_output_frames * sample_count as f64 / output_frame_count)
        .rem_euclid(sample_count as f64)
}

fn prime_w30_resample_edge_history(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    output_sample_rate: u32,
) {
    let sample_count = render
        .source_audio
        .sample_count
        .min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    if sample_count == 0 {
        state.last_character_input = 0.0;
        state.character_edge_memory = 0.0;
        return;
    }
    let cursor_increment =
        sample_count as f64 / w30_resample_cycle_output_frames(render, output_sample_rate);
    let previous_cursor =
        (state.source_sample_cursor - cursor_increment).rem_euclid(sample_count as f64);
    state.last_character_input = w30_resample_source_sample_at(render, previous_cursor);
    state.character_edge_memory = 0.0;
}

fn w30_resample_source_sample(
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
    output_sample_rate: u32,
) -> f32 {
    let window = &render.source_audio;
    let sample_count = window.sample_count.min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    if sample_count == 0 {
        return 0.0;
    }

    let output_frame_count = w30_resample_cycle_output_frames(render, output_sample_rate);
    let cursor_increment = sample_count as f64 / output_frame_count;
    let cursor = state.source_sample_cursor.rem_euclid(sample_count as f64);
    let sample = w30_resample_source_sample_at(render, cursor);
    state.source_sample_cursor = (cursor + cursor_increment).rem_euclid(sample_count as f64);
    sample
}

fn w30_resample_source_sample_at(render: &RealtimeW30ResampleTapState, cursor: f64) -> f32 {
    let window = &render.source_audio;
    let sample_count = window.sample_count.min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    if sample_count == 0 {
        return 0.0;
    }
    let cursor = cursor.rem_euclid(sample_count as f64);
    let base = cursor.floor() as usize % sample_count;
    let next = (base + 1) % sample_count;
    window.samples[base] + (window.samples[next] - window.samples[base]) * cursor.fract() as f32
}

fn w30_resample_cycle_output_frames(
    render: &RealtimeW30ResampleTapState,
    output_sample_rate: u32,
) -> f64 {
    const MIN_GRID_ALIGNED_CYCLE_BEATS: f64 = 1.0;
    const MAX_GRID_ALIGNED_CYCLE_BEATS: f64 = 64.0;

    let window = &render.source_audio;
    let source_duration_seconds =
        window.source_frame_count.max(1) as f64 / f64::from(window.source_sample_rate.max(1));
    let raw_output_frames =
        (source_duration_seconds * f64::from(output_sample_rate.max(1))).max(1.0);
    if !render.tempo_bpm.is_finite() || render.tempo_bpm <= 0.0 {
        return raw_output_frames;
    }

    let source_duration_beats = source_duration_seconds * f64::from(render.tempo_bpm) / 60.0;
    let aligned_beat_count = source_duration_beats
        .round()
        .clamp(MIN_GRID_ALIGNED_CYCLE_BEATS, MAX_GRID_ALIGNED_CYCLE_BEATS);
    (aligned_beat_count * 60.0 / f64::from(render.tempo_bpm) * f64::from(output_sample_rate.max(1)))
        .max(1.0)
}

pub(super) fn w30_resample_step_cursor(render: &RealtimeW30ResampleTapState, step: i64) -> f64 {
    let sample_count = render
        .source_audio
        .sample_count
        .min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    if sample_count == 0 {
        return 0.0;
    }
    if render.variation == W30ResampleTapVariation::Base {
        return 0.0;
    }
    if render.hard_policy == W30ResampleTapHardPolicy::SourceTransientChop {
        let slot = step.rem_euclid(W30_RESAMPLE_HARD_SLICE_COUNT as i64) as usize;
        return usize::from(render.hard_slice_cursors[slot]).min(sample_count.saturating_sub(1))
            as f64;
    }
    0.0
}

/// Add source-reactive resample bite without a free-running oscillator or fallback voice.
fn w30_resample_source_character(
    sample: f32,
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
) -> f32 {
    let hard_source_character = render.variation == W30ResampleTapVariation::HardDamage
        && render.hard_policy == W30ResampleTapHardPolicy::SourceTextureBite;
    w30_resample_source_character_sample(
        sample,
        render.grit_level,
        render.variation_intensity,
        render.hard_policy != W30ResampleTapHardPolicy::Unavailable,
        hard_source_character,
        &mut state.last_character_input,
        &mut state.character_edge_memory,
    )
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
    if render.variation == W30ResampleTapVariation::HardDamage
        && render.hard_policy == W30ResampleTapHardPolicy::SourceTransientChop
    {
        return 2;
    }
    let base = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) => 1,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 2,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 4,
        None => 1,
    };
    (base + u32::from(render.lineage_capture_count >= 2)).min(4)
}

pub(super) fn should_trigger_w30_resample_step(
    render: &RealtimeW30ResampleTapState,
    step: i64,
) -> bool {
    if render.variation == W30ResampleTapVariation::Base {
        // Base is the recognizable full-phrase anchor. Repeated grid resets reduced the committed
        // artifact to a polite tap and prevented the callback from reaching most source material.
        return false;
    }

    // Hard damage follows the source-derived policy. Transient material retains its own selected
    // eighth-note attack pattern; sustained material keeps phrase flow and gets timbral bite
    // without imposing a generic gate template.
    if render.variation == W30ResampleTapVariation::HardDamage {
        return match render.hard_policy {
            W30ResampleTapHardPolicy::SourceTransientChop => {
                let slot = step.rem_euclid(W30_RESAMPLE_HARD_SLICE_COUNT as i64) as u8;
                render.hard_trigger_mask & (1_u8 << slot) != 0
                    || (render.hard_gesture.recipe
                        == W30ResampleHardGestureRecipe::SourceReverseIntoImpactV1
                        && slot == render.hard_gesture.pickup_slot)
            }
            W30ResampleTapHardPolicy::SourceTextureBite | W30ResampleTapHardPolicy::Unavailable => {
                false
            }
        };
    }

    match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => step.rem_euclid(2) == 0,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => !matches!(step.rem_euclid(4), 1),
        Some(W30ResampleTapSourceProfile::PinnedCapture) => true,
    }
}

fn w30_resample_trigger_envelope(render: &RealtimeW30ResampleTapState) -> f32 {
    if render.variation == W30ResampleTapVariation::HardDamage {
        return 1.0;
    }

    let profile_boost = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => 0.0,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 0.1,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 0.16,
    };
    let lineage_boost = f32::from(render.lineage_capture_count.min(4)) * 0.04;
    let generation_boost = f32::from(render.generation_depth.min(4)) * 0.06;
    (0.58 + profile_boost + lineage_boost + generation_boost + render.grit_level * 0.08)
        .clamp(0.0, 0.98)
}

fn w30_resample_render_gain(render: &RealtimeW30ResampleTapState, transport_running: bool) -> f32 {
    let profile_gain = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => 0.88,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 0.98,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 1.08,
    };
    let transport_gain = if transport_running { 1.0 } else { 0.7 };
    let grit_gain = if render.hard_policy == W30ResampleTapHardPolicy::Unavailable {
        1.0
    } else {
        1.0 + render.grit_level.clamp(0.0, 1.0) * 0.18
    };
    (profile_gain * transport_gain * render.music_bus_level.clamp(0.0, 1.0) * grit_gain)
        .clamp(0.0, 1.2)
}

pub(super) fn w30_resample_decay(_render: &RealtimeW30ResampleTapState, _sample_rate: u32) -> f32 {
    // H1 keeps the source body intact. The source-adaptive attack path crossfades
    // back to the continuous body instead of deleting it with a global gate.
    1.0
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
