use crate::runtime::render_tr909_w30_preview::render_tr909_buffer;
use crate::runtime::shared_transport_tr909::{RealtimeTr909RenderState, SharedTr909RenderState};
use crate::runtime::shared_w30_resample_callback::Tr909CallbackState;
use crate::runtime::tr909_tail_telemetry::envelope_decay;
use crate::runtime::w30_tr909_signal_helpers::{
    break_performance_slam, fill_performance_slam, render_gain, render_subdivision,
    should_trigger_step, tr909_step_waveform, trigger_envelope, trigger_frequency,
};
use crate::runtime::{tr909_fill_recipe, tr909_fill_voice};
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909RenderState,
};

pub(super) fn cursor_20_phrase_drive_fill_render() -> RealtimeTr909RenderState {
    RealtimeTr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: None,
        source_support_context: None,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDriveHardCut),
        takeover_profile: None,
        drum_bus_level: 0.799_204_35,
        slam_enabled: false,
        slam_intensity: 0.659_204_36,
        is_transport_running: true,
        tempo_bpm: 131.878,
        position_beats: 20.0,
        source_bar_grid_anchor_position_beats: None,
    }
}

pub(super) fn test_fill_recipe_trigger(
    render: &RealtimeTr909RenderState,
    step: i64,
) -> tr909_fill_recipe::Tr909FillVoiceTrigger {
    tr909_fill_recipe::prepared_fill_step(
        render,
        render_subdivision(render),
        step,
        fill_performance_slam(render),
    )
    .trigger()
    .unwrap_or_default()
}

pub(super) fn render_isolated_fill_voice_component(
    render: &RealtimeTr909RenderState,
    step: i64,
    sample_rate: u32,
    frame_count: usize,
    component: usize,
) -> Vec<f32> {
    let mut voices = tr909_fill_voice::Tr909FillVoiceState::default();
    voices.start();
    voices.trigger(
        test_fill_recipe_trigger(render, step),
        trigger_envelope(render),
        step,
    );
    (0..frame_count)
        .map(|_| {
            let sample = voices.render_sample(render, sample_rate);
            match component {
                0 => sample.kick,
                1 => sample.snare,
                _ => sample.hat,
            }
        })
        .collect()
}

pub(super) fn render_tr909_in_callback_blocks(
    render: &RealtimeTr909RenderState,
    sample_rate: u32,
    channel_count: usize,
    frame_count: usize,
    callback_frame_count: usize,
) -> Vec<f32> {
    let mut state = Tr909CallbackState::default();
    let mut output = Vec::with_capacity(frame_count * channel_count);
    let beats_per_frame = f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate);
    for frame_offset in (0..frame_count).step_by(callback_frame_count.max(1)) {
        let block_frames = (frame_count - frame_offset).min(callback_frame_count.max(1));
        let block_render = RealtimeTr909RenderState {
            position_beats: render.position_beats + frame_offset as f64 * beats_per_frame,
            ..*render
        };
        let mut block = vec![0.0_f32; block_frames * channel_count];
        render_tr909_buffer(
            &mut block,
            sample_rate,
            channel_count,
            &block_render,
            &mut state,
        );
        output.extend(block);
    }
    output
}

pub(super) fn render_legacy_tr909_composite_control(
    render: &RealtimeTr909RenderState,
    sample_rate: u32,
    channel_count: usize,
    frame_count: usize,
) -> Vec<f32> {
    let mut state = Tr909CallbackState::default();
    let subdivision = render_subdivision(render);
    let current_step = (render.position_beats * f64::from(subdivision)).floor() as i64;
    state.beat_position = render.position_beats;
    state.last_step = current_step.saturating_sub(1);
    state.was_running = true;
    let beats_per_sample = f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1));
    let mut output = vec![0.0; frame_count.saturating_mul(channel_count)];

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
            let waveform = tr909_step_waveform(render, state.last_step, state.oscillator_phase);
            state.oscillator_phase =
                (state.oscillator_phase + state.oscillator_hz / sample_rate.max(1) as f32).fract();
            let rendered = waveform * state.envelope * render_gain(render);
            state.envelope *= envelope_decay(render);
            rendered
        } else {
            0.0
        };
        let base = frame_index * channel_count;
        output[base..base + channel_count].fill(sample);
        state.beat_position += beats_per_sample;
    }
    output
}

pub(super) fn render_candidate6_composite_control(
    render_state: &Tr909RenderState,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
) -> Vec<f32> {
    let render = SharedTr909RenderState::new(render_state).snapshot();
    let channel_count = usize::from(channel_count.max(1));
    let mut state = Tr909CallbackState::default();
    let subdivision = render_subdivision(&render);
    let current_step = (render.position_beats * f64::from(subdivision)).floor() as i64;
    state.beat_position = render.position_beats;
    state.last_step = current_step.saturating_sub(1);
    state.was_running = true;
    let beats_per_sample = f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1));
    let mut output = vec![0.0; frame_count.saturating_mul(channel_count)];

    for frame_index in 0..frame_count {
        let step = (state.beat_position * f64::from(subdivision)).floor() as i64;
        if step != state.last_step {
            state.last_step = step;
            if candidate6_should_trigger_step(&render, step) {
                state.envelope = trigger_envelope(&render);
                state.oscillator_hz = trigger_frequency(&render, step);
                state.oscillator_phase = fill_performance_slam(&render) * 0.25;
            }
        }

        let sample = if state.envelope > 0.0005 {
            let waveform = tr909_step_waveform(&render, state.last_step, state.oscillator_phase);
            state.oscillator_phase =
                (state.oscillator_phase + state.oscillator_hz / sample_rate.max(1) as f32).fract();
            let rendered = waveform * state.envelope * render_gain(&render);
            state.envelope *= envelope_decay(&render);
            rendered
        } else {
            0.0
        };
        let base = frame_index * channel_count;
        output[base..base + channel_count].fill(sample);
        state.beat_position += beats_per_sample;
    }
    output
}

fn candidate6_should_trigger_step(render: &RealtimeTr909RenderState, step: i64) -> bool {
    let subdivision = i64::from(render_subdivision(render)).max(1);
    let step_in_bar = step.rem_euclid(subdivision * 4);
    let beat_in_bar = step_in_bar / subdivision;
    let step_in_beat = step_in_bar % subdivision;
    match beat_in_bar {
        0 => step_in_beat == 0,
        1 => step_in_beat % (subdivision / 2).max(1) == 0,
        2 => step_in_beat % (subdivision / 4).max(1) == 0,
        _ => true,
    }
}
