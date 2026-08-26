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
    render_w30_preview_buffer_with_stereo_side(
        data,
        sample_rate,
        channel_count,
        render,
        None,
        state,
    );
}

pub(super) fn render_w30_preview_buffer_with_stereo_side(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeW30PreviewRenderState,
    stereo_side_samples: Option<&[f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN]>,
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
        state.reset_character();
        state.beat_position = render.position_beats;
        state.last_trigger_revision = render.trigger_revision;
        state.pitch_dive.reset();
        state.filter_slam.reset();
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
        state.reset_character();
        state.pitch_dive.reset();
        state.filter_slam.reset();
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
        state.reset_character();
        state.last_source_window_signature = w30_source_window_signature(render);
        state.last_pad_playback_signature = w30_pad_playback_signature(render);
        state.last_trigger_revision = render.trigger_revision;
        state.pitch_dive.reset();
        state.filter_slam.reset();
        state.was_active = true;
    }

    let source_window_signature = w30_source_window_signature(render);
    if source_window_signature != state.last_source_window_signature {
        state.last_source_window_signature = source_window_signature;
        state.source_sample_cursor = 0.0;
        state.reset_character();
    }
    let pad_playback_signature = w30_pad_playback_signature(render);
    if pad_playback_signature != state.last_pad_playback_signature {
        state.last_pad_playback_signature = pad_playback_signature;
        state.pad_playback_cursor = w30_chop_slice_cursor(render, state.last_step);
        state.pad_playback_age_frames = 0;
        state.reset_character();
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
                        state.reset_character();
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
        let stop_gain = transport_stop_gain(
            state.transport_stop_latched,
            &mut state.transport_stop_fade_frames_remaining,
            transport_stop_fade_frames,
        );
        if let Some(stereo_side_samples) = w30_stereo_pad_playback_active(render)
            .then_some(stereo_side_samples)
            .flatten()
        {
            let waveform = w30_stereo_pad_playback_waveform_for_frame(
                render,
                stereo_side_samples,
                state,
                sample_rate,
            );
            let common_gain =
                state.envelope * tremolo * w30_render_gain(render, transport_running) * stop_gain;
            let rendered = W30StereoFrame {
                left: waveform.left * common_gain,
                right: waveform.right * common_gain,
            };
            state.pitch_dive.reset();
            state.filter_slam.reset();

            let base = frame_index * channel_count;
            if channel_count == 1 {
                data[base] += (rendered.left + rendered.right) / 2.0;
            } else {
                for channel in 0..channel_count {
                    data[base + channel] += if channel % 2 == 0 {
                        rendered.left
                    } else {
                        rendered.right
                    };
                }
            }
            state.beat_position += beats_per_sample;
            continue;
        }

        let waveform = w30_preview_waveform_for_frame(render, state, sample_rate);
        let control_sample = waveform
            * state.envelope
            * tremolo
            * w30_render_gain(render, transport_running)
            * stop_gain;
        let articulation_sample =
            w30_post_render_articulation_sample(control_sample, render, state, state.beat_position);
        let filter_slam_frame = w30_filter_slam_frame(render, state.beat_position, sample_rate);
        if filter_slam_frame.is_some() {
            state
                .filter_slam
                .prepare(render.pad_playback.hook_articulation_started_at_beat);
        } else {
            state.filter_slam.reset();
        }
        if transport_running && !w30_pad_playback_active(render) {
            state.envelope *= w30_envelope_decay(render);
        }

        let base = frame_index * channel_count;
        for channel in 0..channel_count {
            let sample = filter_slam_frame.map_or(articulation_sample, |frame| {
                w30_filter_slam_sample(articulation_sample, channel, frame, &mut state.filter_slam)
            });
            data[base + channel] += sample;
        }

        state.beat_position += beats_per_sample;
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
struct W30StereoFrame {
    left: f32,
    right: f32,
}

fn w30_stereo_pad_playback_waveform_for_frame(
    render: &RealtimeW30PreviewRenderState,
    stereo_side_samples: &[f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
    state: &mut W30PreviewCallbackState,
    sample_rate: u32,
) -> W30StereoFrame {
    let sample = w30_stereo_pad_playback_sample_with_reverse(
        &render.pad_playback,
        stereo_side_samples,
        state,
        sample_rate,
        render.pad_playback.reverse,
    );
    let left = w30_source_backed_character_for_channel(
        sample.left,
        render.grit_level,
        &mut state.last_character_input,
        &mut state.character_edge_memory,
    );
    let right = w30_source_backed_character_for_channel(
        sample.right,
        render.grit_level,
        &mut state.right_last_character_input,
        &mut state.right_character_edge_memory,
    );
    let gate =
        w30_pad_grid_gate_for_fraction(render, sample_rate, render.pad_playback.gate_step_fraction);
    let gain = w30_pad_grid_gate_gain(gate, state);
    W30StereoFrame {
        left: left * gain,
        right: right * gain,
    }
}

fn w30_preview_waveform_for_frame(
    render: &RealtimeW30PreviewRenderState,
    state: &mut W30PreviewCallbackState,
    sample_rate: u32,
) -> f32 {
    if w30_pad_playback_active(render) {
        let articulation = if matches!(
            render.pad_playback.hook_articulation_profile,
            Some(W30HookArticulationProfile::PitchDiveV1)
        ) {
            None
        } else {
            w30_hook_articulation_frame(render, state.beat_position)
        };
        let reverse = articulation.map_or(render.pad_playback.reverse, |frame| frame.reverse);
        let gate_fraction = articulation.map_or(render.pad_playback.gate_step_fraction, |frame| {
            frame.gate_step_fraction
        });
        let sample =
            w30_pad_playback_sample_with_reverse(&render.pad_playback, state, sample_rate, reverse);
        let characterized = w30_source_backed_character(sample, render.grit_level, state);
        let gate = w30_pad_grid_gate_for_fraction(render, sample_rate, gate_fraction);
        return characterized * w30_pad_grid_gate_gain(gate, state);
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

#[cfg(test)]
pub(super) fn w30_pad_grid_gate(
    render: &RealtimeW30PreviewRenderState,
    sample_rate: u32,
) -> Option<W30PadGridGate> {
    w30_pad_grid_gate_for_fraction(render, sample_rate, render.pad_playback.gate_step_fraction)
}

fn w30_pad_grid_gate_for_fraction(
    render: &RealtimeW30PreviewRenderState,
    sample_rate: u32,
    gate_step_fraction: f32,
) -> Option<W30PadGridGate> {
    let gate_fraction = gate_step_fraction.clamp(0.0, 1.0);
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
    w30_source_backed_character_for_channel(
        sample,
        grit_level,
        &mut state.last_character_input,
        &mut state.character_edge_memory,
    )
}

fn w30_source_backed_character_for_channel(
    sample: f32,
    grit_level: f32,
    last_character_input: &mut f32,
    character_edge_memory: &mut f32,
) -> f32 {
    let grit = grit_level.clamp(0.0, 1.0);
    if grit <= f32::EPSILON {
        *last_character_input = sample;
        *character_edge_memory = 0.0;
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

    let raw_edge = sample - *last_character_input;
    *last_character_input = sample;
    *character_edge_memory = *character_edge_memory * EDGE_MEMORY + raw_edge * (1.0 - EDGE_MEMORY);

    let edge_emphasis = *character_edge_memory * (grit * EDGE_RANGE);
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

fn w30_stereo_pad_playback_active(render: &RealtimeW30PreviewRenderState) -> bool {
    w30_pad_playback_active(render) && render.pad_playback.hook_articulation_profile.is_none()
}

#[cfg(test)]
pub(super) fn w30_pad_playback_sample(
    window: &RealtimeW30PadPlaybackSampleWindow,
    state: &mut W30PreviewCallbackState,
    output_sample_rate: u32,
) -> f32 {
    w30_pad_playback_sample_with_reverse(window, state, output_sample_rate, window.reverse)
}

fn w30_pad_playback_sample_with_reverse(
    window: &RealtimeW30PadPlaybackSampleWindow,
    state: &mut W30PreviewCallbackState,
    output_sample_rate: u32,
    reverse: bool,
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
    let sample = interpolated_pad_sample(window, sample_count, wrapped_cursor, reverse);
    let sample = apply_pad_loop_crossfade(window, sample_count, wrapped_cursor, sample, reverse);
    let sample = apply_pad_edge_envelope(window, state, sample_count, wrapped_cursor, sample);
    state.pad_playback_cursor = if window.loop_enabled {
        (logical_cursor + cursor_increment) % sample_count as f32
    } else {
        logical_cursor + cursor_increment
    };
    state.pad_playback_age_frames = state.pad_playback_age_frames.saturating_add(1);
    sample
}

fn w30_stereo_pad_playback_sample_with_reverse(
    window: &RealtimeW30PadPlaybackSampleWindow,
    stereo_side_samples: &[f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
    state: &mut W30PreviewCallbackState,
    output_sample_rate: u32,
    reverse: bool,
) -> W30StereoFrame {
    let sample_count = window.sample_count.min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
    if sample_count == 0 {
        return W30StereoFrame::default();
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
        return W30StereoFrame::default();
    }

    let wrapped_cursor = if window.loop_enabled {
        logical_cursor % sample_count as f32
    } else {
        logical_cursor.min(sample_count.saturating_sub(1) as f32)
    };
    let mid = interpolated_pad_channel(&window.samples, sample_count, wrapped_cursor, reverse);
    let side = interpolated_pad_channel(stereo_side_samples, sample_count, wrapped_cursor, reverse);
    let mid = apply_pad_loop_crossfade_to_channel(
        &window.samples,
        window,
        sample_count,
        wrapped_cursor,
        mid,
        reverse,
    );
    let side = apply_pad_loop_crossfade_to_channel(
        stereo_side_samples,
        window,
        sample_count,
        wrapped_cursor,
        side,
        reverse,
    );
    let edge_gain = pad_edge_envelope_gain(window, state, sample_count, wrapped_cursor);
    let sample = W30StereoFrame {
        left: (mid + side) * edge_gain,
        right: (mid - side) * edge_gain,
    };
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
    reverse: bool,
) -> f32 {
    let base = cursor.floor() as usize % sample_count;
    let next = (base + 1).min(sample_count - 1);
    let fraction = cursor.fract();
    let index = if reverse {
        sample_count - 1 - base
    } else {
        base
    };
    let next_index = if reverse {
        sample_count - 1 - next
    } else {
        next
    };
    window.samples[index] + (window.samples[next_index] - window.samples[index]) * fraction
}

fn interpolated_pad_channel(
    samples: &[f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
    sample_count: usize,
    cursor: f32,
    reverse: bool,
) -> f32 {
    let base = cursor.floor() as usize % sample_count;
    let next = (base + 1).min(sample_count - 1);
    let fraction = cursor.fract();
    let index = if reverse {
        sample_count - 1 - base
    } else {
        base
    };
    let next_index = if reverse {
        sample_count - 1 - next
    } else {
        next
    };
    samples[index] + (samples[next_index] - samples[index]) * fraction
}

fn apply_pad_loop_crossfade(
    window: &RealtimeW30PadPlaybackSampleWindow,
    sample_count: usize,
    cursor: f32,
    sample: f32,
    reverse: bool,
) -> f32 {
    let crossfade = window.loop_crossfade_sample_count.min(sample_count / 2);
    if !window.loop_enabled || crossfade == 0 || cursor < (sample_count - crossfade) as f32 {
        return sample;
    }

    let fade_position = cursor - (sample_count - crossfade) as f32;
    let mix = (fade_position / crossfade as f32).clamp(0.0, 1.0);
    let head = interpolated_pad_sample(window, sample_count, fade_position, reverse);
    sample * (1.0 - mix) + head * mix
}

fn apply_pad_loop_crossfade_to_channel(
    samples: &[f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
    window: &RealtimeW30PadPlaybackSampleWindow,
    sample_count: usize,
    cursor: f32,
    sample: f32,
    reverse: bool,
) -> f32 {
    let crossfade = window.loop_crossfade_sample_count.min(sample_count / 2);
    if !window.loop_enabled || crossfade == 0 || cursor < (sample_count - crossfade) as f32 {
        return sample;
    }

    let fade_position = cursor - (sample_count - crossfade) as f32;
    let mix = (fade_position / crossfade as f32).clamp(0.0, 1.0);
    let head = interpolated_pad_channel(samples, sample_count, fade_position, reverse);
    sample * (1.0 - mix) + head * mix
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct W30HookArticulationFrame {
    pub(super) reverse: bool,
    pub(super) gate_step_fraction: f32,
    pub(super) playback_rate_multiplier: f32,
    pub(super) terminal_gain: f32,
    pub(super) continuous_cursor: bool,
    pub(super) silent: bool,
}

pub(super) fn w30_hook_articulation_frame(
    render: &RealtimeW30PreviewRenderState,
    position_beats: f64,
) -> Option<W30HookArticulationFrame> {
    let profile = render.pad_playback.hook_articulation_profile?;
    let relative_beat =
        position_beats - render.pad_playback.hook_articulation_started_at_beat as f64;
    if !relative_beat.is_finite() {
        return None;
    }
    // Transport position advances in floating-point sample increments. Snap only values that are
    // already within a sub-sample tolerance of an integer beat so the frozen [1, 3, 4] boundaries
    // land on the intended callback frame instead of one frame late.
    let nearest_beat = relative_beat.round();
    let relative_beat = if (relative_beat - nearest_beat).abs() <= 1.0e-9 {
        nearest_beat
    } else {
        relative_beat
    };

    match profile {
        W30HookArticulationProfile::TurnaroundV1 if (1.0..3.0).contains(&relative_beat) => {
            Some(W30HookArticulationFrame {
                reverse: true,
                gate_step_fraction: 0.68,
                playback_rate_multiplier: 1.0,
                terminal_gain: 1.0,
                continuous_cursor: false,
                silent: false,
            })
        }
        W30HookArticulationProfile::TurnaroundV1 if (3.0..4.0).contains(&relative_beat) => {
            Some(W30HookArticulationFrame {
                reverse: false,
                gate_step_fraction: 0.34,
                playback_rate_multiplier: 1.0,
                terminal_gain: 1.0,
                continuous_cursor: false,
                silent: false,
            })
        }
        W30HookArticulationProfile::TurnaroundV1 => None,
        W30HookArticulationProfile::PitchDiveV1 if (8.0..12.0).contains(&relative_beat) => {
            let local_beat = relative_beat - 8.0;
            let progress = (local_beat / 4.0).clamp(0.0, 1.0);
            Some(W30HookArticulationFrame {
                reverse: render.pad_playback.reverse,
                gate_step_fraction: render.pad_playback.gate_step_fraction,
                playback_rate_multiplier: 0.35_f32.powf(progress as f32),
                terminal_gain: ((4.0 - local_beat) / 0.15).clamp(0.0, 1.0) as f32,
                continuous_cursor: true,
                silent: false,
            })
        }
        W30HookArticulationProfile::PitchDiveV1 if relative_beat >= 12.0 => {
            Some(W30HookArticulationFrame {
                reverse: render.pad_playback.reverse,
                gate_step_fraction: render.pad_playback.gate_step_fraction,
                playback_rate_multiplier: 0.35,
                terminal_gain: 0.0,
                continuous_cursor: true,
                silent: true,
            })
        }
        W30HookArticulationProfile::PitchDiveV1 => None,
        W30HookArticulationProfile::FilterSlamV1 => None,
    }
}

fn w30_post_render_articulation_sample(
    control_sample: f32,
    render: &RealtimeW30PreviewRenderState,
    state: &mut W30PreviewCallbackState,
    position_beats: f64,
) -> f32 {
    if !matches!(
        render.pad_playback.hook_articulation_profile,
        Some(W30HookArticulationProfile::PitchDiveV1)
    ) {
        state.pitch_dive.reset();
        return control_sample;
    }

    let Some(frame) = w30_hook_articulation_frame(render, position_beats) else {
        state.pitch_dive.reset();
        return control_sample;
    };
    if frame.silent {
        state.pitch_dive.reset();
        return 0.0;
    }

    let history_len = state.pitch_dive.history.len();
    if history_len < 2 {
        return 0.0;
    }
    if !state.pitch_dive.active {
        state.pitch_dive.reset();
        state.pitch_dive.active = true;
    }

    let write_frame = state.pitch_dive.write_frame;
    let lag_frames = write_frame as f64 - state.pitch_dive.source_cursor;
    if !lag_frames.is_finite()
        || lag_frames < 0.0
        || lag_frames.ceil() as usize >= history_len.saturating_sub(1)
    {
        return 0.0;
    }

    state.pitch_dive.history[write_frame as usize % history_len] = control_sample;
    let source_floor = state.pitch_dive.source_cursor.floor();
    let source_fraction = (state.pitch_dive.source_cursor - source_floor) as f32;
    let source_frame = source_floor as u64;
    let next_source_frame = source_frame.saturating_add(1).min(write_frame);
    let first = state.pitch_dive.history[source_frame as usize % history_len];
    let second = state.pitch_dive.history[next_source_frame as usize % history_len];
    let resampled = first + (second - first) * source_fraction;

    state.pitch_dive.source_cursor += f64::from(frame.playback_rate_multiplier);
    state.pitch_dive.write_frame = write_frame.saturating_add(1);
    resampled * frame.terminal_gain
}

fn apply_pad_edge_envelope(
    window: &RealtimeW30PadPlaybackSampleWindow,
    state: &W30PreviewCallbackState,
    sample_count: usize,
    cursor: f32,
    sample: f32,
) -> f32 {
    sample * pad_edge_envelope_gain(window, state, sample_count, cursor)
}

fn pad_edge_envelope_gain(
    window: &RealtimeW30PadPlaybackSampleWindow,
    state: &W30PreviewCallbackState,
    sample_count: usize,
    cursor: f32,
) -> f32 {
    const EDGE_FRAMES: f32 = 64.0;
    let attack = (state.pad_playback_age_frames as f32 / EDGE_FRAMES).clamp(0.0, 1.0);
    let release = if window.loop_enabled {
        1.0
    } else {
        ((sample_count as f32 - cursor) / EDGE_FRAMES).clamp(0.0, 1.0)
    };
    attack.min(release)
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
        state.beat_position = 0.0;
        state.source_sample_cursor = 0.0;
        state.last_character_input = 0.0;
        state.character_edge_memory = 0.0;
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
        state.beat_position = render.position_beats.max(0.0);
        state.envelope = 1.0;
        state.last_step = 0;
        state.source_sample_cursor = 0.0;
        state.last_character_input = 0.0;
        state.character_edge_memory = 0.0;
        state.was_active = true;
    }

    let transport_running = render.is_transport_running;
    let beats_per_sample =
        if transport_running && render.tempo_bpm.is_finite() && render.tempo_bpm > 0.0 {
            f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1))
        } else {
            0.0
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
                    state.source_sample_cursor = w30_resample_step_cursor(render, step);
                }
            }
        } else {
            state.envelope = state.envelope.max(0.42) * 0.99975;
        }

        let source_sample = w30_resample_source_sample(render, state, sample_rate);
        let sample = w30_resample_source_character(source_sample, render.grit_level, state)
            * state.envelope
            * w30_resample_render_gain(render, transport_running)
            * transport_stop_gain(
                state.transport_stop_latched,
                &mut state.transport_stop_fade_frames_remaining,
                transport_stop_fade_frames,
            );
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

    let output_frame_count = (window.source_frame_count.max(1) as f64
        * f64::from(output_sample_rate.max(1))
        / f64::from(window.source_sample_rate.max(1)))
    .max(1.0);
    let playback_rate = w30_resample_playback_rate(render);
    let cursor_increment =
        (sample_count as f64 / output_frame_count * f64::from(playback_rate)) as f32;
    let cursor = state.source_sample_cursor.rem_euclid(sample_count as f32);
    let base = cursor.floor() as usize % sample_count;
    let next = (base + 1) % sample_count;
    let sample =
        window.samples[base] + (window.samples[next] - window.samples[base]) * cursor.fract();
    state.source_sample_cursor = (cursor + cursor_increment).rem_euclid(sample_count as f32);
    sample
}

fn w30_resample_playback_rate(render: &RealtimeW30ResampleTapState) -> f32 {
    const GENERATION_RATE_DROP: f32 = 0.025;
    const GRIT_RATE_LIFT: f32 = 0.015;
    const MIN_RATE: f32 = 0.86;

    (1.0 - f32::from(render.generation_depth.min(4)) * GENERATION_RATE_DROP
        + render.grit_level.clamp(0.0, 1.0) * GRIT_RATE_LIFT)
        .clamp(MIN_RATE, 1.0)
}

fn w30_resample_step_cursor(render: &RealtimeW30ResampleTapState, step: i64) -> f32 {
    let sample_count = render
        .source_audio
        .sample_count
        .min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    if sample_count == 0 {
        return 0.0;
    }
    const STEP_SEQUENCE: [usize; 8] = [0, 3, 1, 5, 2, 7, 4, 6];
    let sequence_index = step.rem_euclid(STEP_SEQUENCE.len() as i64) as usize;
    sample_count as f32 * STEP_SEQUENCE[sequence_index] as f32 / STEP_SEQUENCE.len() as f32
}

/// Add source-reactive resample bite without a free-running oscillator or fallback voice.
fn w30_resample_source_character(
    sample: f32,
    grit_level: f32,
    state: &mut W30ResampleTapCallbackState,
) -> f32 {
    const EDGE_MEMORY: f32 = 0.68;
    const EDGE_RANGE: f32 = 1.1;
    const DRIVE_RANGE: f32 = 4.2;
    const WET_RANGE: f32 = 0.78;
    const BODY_SHARE: f32 = 0.76;
    const EDGE_SHARE: f32 = 0.24;

    let grit = grit_level.clamp(0.0, 1.0);
    let raw_edge = sample - state.last_character_input;
    state.last_character_input = sample;
    state.character_edge_memory =
        state.character_edge_memory * EDGE_MEMORY + raw_edge * (1.0 - EDGE_MEMORY);
    if grit <= f32::EPSILON {
        return sample;
    }

    let driven = sample + state.character_edge_memory * grit * EDGE_RANGE;
    let saturated = (driven * (1.0 + grit * DRIVE_RANGE)).tanh();
    let edge = (raw_edge * (5.0 + grit * 13.0)).tanh();
    let bitten = saturated * BODY_SHARE + edge * EDGE_SHARE;
    let wet = grit * WET_RANGE;
    (sample * (1.0 - wet) + bitten * wet).clamp(-0.98, 0.98)
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
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 0.1,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 0.16,
    };
    let lineage_boost = f32::from(render.lineage_capture_count.min(4)) * 0.04;
    let generation_boost = f32::from(render.generation_depth.min(4)) * 0.06;
    (0.45 + profile_boost + lineage_boost + generation_boost + render.grit_level * 0.18)
        .clamp(0.0, 0.95)
}

fn w30_resample_render_gain(render: &RealtimeW30ResampleTapState, transport_running: bool) -> f32 {
    let profile_gain = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => 0.42,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 0.5,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 0.58,
    };
    let transport_gain = if transport_running { 1.0 } else { 0.7 };
    (profile_gain
        * transport_gain
        * render.music_bus_level.clamp(0.0, 1.0)
        * (1.0 + render.grit_level.clamp(0.0, 1.0) * 0.18))
        .clamp(0.0, 0.72)
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
