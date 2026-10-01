//! Existing offline W-30 trigger renderer; policy/evidence stay with their owners.
use super::{
    config::{CHANNEL_COUNT, SAMPLE_RATE},
    grid::{Grid, frames_for_beat_position},
    spectral_energy_metrics::spectral_energy_metrics,
    w30_slice_choice::{W30SourceSliceChoiceProof, w30_source_slice_choice_plan},
    w30_source_accent_dynamics::{W30SourceAccentDynamicsProof, w30_source_accent_dynamics_proof},
    w30_source_chop::chop_articulation_metrics,
    w30_source_events::W30SourceTriggerEvent,
    w30_source_playback_profile::w30_source_playback_profile,
    w30_source_trigger_policy::{
        W30SourceTriggerVariationProof, is_beat_anchor, w30_source_trigger_events_with_slice_plan,
        w30_source_trigger_variation_proof,
    },
};
use riotbox_audio::{
    runtime::render_w30_preview_offline,
    w30::{
        W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewRenderMode, W30PreviewRenderRouting,
        W30PreviewRenderState, W30PreviewSampleWindow, W30PreviewSourceProfile,
    },
};

#[allow(dead_code)]
pub(super) fn render_w30_source_chop(
    grid: &Grid,
    source_window_preview: W30PreviewSampleWindow,
) -> Vec<f32> {
    render_w30_source_chop_with_variation(grid, &source_window_preview).0
}

pub(super) fn render_w30_source_chop_with_variation(
    grid: &Grid,
    source_window_preview: &W30PreviewSampleWindow,
) -> (
    Vec<f32>,
    W30SourceTriggerVariationProof,
    W30SourceSliceChoiceProof,
    W30SourceAccentDynamicsProof,
) {
    let slice_plan = w30_source_slice_choice_plan(source_window_preview);
    let events =
        w30_source_trigger_events_with_slice_plan(grid, source_window_preview, &slice_plan);
    let proof = w30_source_trigger_variation_proof(grid, &events);
    let slice_proof = slice_plan.proof;
    let accent_proof = w30_source_accent_dynamics_proof(&events);
    let profile_gain = w30_source_trigger_profile_gain(source_window_preview);
    let mut output = vec![0.0; grid.total_frames * usize::from(CHANNEL_COUNT)];

    for event in events {
        render_w30_source_trigger_event(
            &mut output,
            grid,
            source_window_preview,
            event,
            profile_gain,
        );
    }

    for sample in &mut output {
        *sample = sample.clamp(-0.95, 0.95);
    }

    (output, proof, slice_proof, accent_proof)
}

fn render_w30_source_trigger_event(
    output: &mut [f32],
    grid: &Grid,
    source_window_preview: &W30PreviewSampleWindow,
    event: W30SourceTriggerEvent,
    profile_gain: f32,
) {
    let sample_count = source_window_preview
        .sample_count
        .min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    if sample_count == 0 {
        return;
    }

    let trigger_frame = frames_for_beat_position(grid.bpm, event.beat_position);
    let event_frame_count = sample_count.saturating_mul(2);
    let playback_profile = w30_source_playback_profile(source_window_preview);
    for frame_offset in 0..event_frame_count {
        let frame = trigger_frame.saturating_add(frame_offset);
        if frame >= grid.total_frames {
            break;
        }

        let source_index = (event.source_offset_samples
            + playback_profile.phase_offset_samples
            + frame_offset / playback_profile.stride_divisor)
            % sample_count;
        let previous_source_index = (source_index + sample_count - 1) % sample_count;
        let source_sample = source_window_preview.samples[source_index];
        let source_edge = source_sample - source_window_preview.samples[previous_source_index];
        let envelope = w30_source_trigger_event_envelope(frame_offset, event_frame_count);
        let sample = (source_sample + source_edge * 0.28)
            * event.velocity
            * envelope
            * w30_source_trigger_gain(event.beat_position)
            * profile_gain
            * playback_profile.gain;
        let output_index = frame.saturating_mul(usize::from(CHANNEL_COUNT));
        output[output_index] += sample;
        output[output_index + 1] += sample * 0.98;
    }
}

fn w30_source_trigger_event_envelope(frame_offset: usize, event_frame_count: usize) -> f32 {
    if event_frame_count <= 1 {
        return 1.0;
    }
    let position = frame_offset as f32 / (event_frame_count - 1) as f32;
    let attack = (position / 0.018).clamp(0.0, 1.0);
    let decay = ((1.0 - position) / 0.982).clamp(0.0, 1.0).powf(0.58);
    attack * decay
}

fn w30_source_trigger_gain(beat_position: f32) -> f32 {
    if is_beat_anchor(beat_position) {
        0.26
    } else {
        0.20
    }
}

fn w30_source_trigger_profile_gain(source_window_preview: &W30PreviewSampleWindow) -> f32 {
    let sample_count = source_window_preview
        .sample_count
        .min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    if sample_count == 0 {
        return 1.0;
    }

    let samples = &source_window_preview.samples[..sample_count];
    let (_, _, tail_to_body_rms_ratio) = chop_articulation_metrics(samples);
    let spectral = spectral_energy_metrics(samples);

    if tail_to_body_rms_ratio > 1.20 {
        0.55
    } else if spectral.high_band_energy_ratio > 0.08 {
        0.60
    } else if spectral.low_band_energy_ratio > 0.95 {
        0.90
    } else {
        0.96
    }
}

#[allow(dead_code)]
pub(super) fn render_w30_source_chop_legacy(
    grid: &Grid,
    source_window_preview: W30PreviewSampleWindow,
) -> Vec<f32> {
    render_w30_preview_offline(
        &W30PreviewRenderState {
            mode: W30PreviewRenderMode::RawCaptureAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
            active_bank_id: Some("bank-a".into()),
            focused_pad_id: Some("pad-01".into()),
            capture_id: Some("cap-feral-grid".into()),
            trigger_revision: 1,
            trigger_velocity: 0.82,
            source_window_preview: Some(source_window_preview),
            pad_playback: None,
            music_bus_level: 0.72,
            grit_level: 0.46,
            is_transport_running: true,
            tempo_bpm: grid.bpm,
            position_beats: 0.0,
        },
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.total_frames,
    )
}
