//! Existing source-window playback profile for the offline QA renderer.

use super::spectral_energy_metrics::spectral_energy_metrics;
use super::w30_source_chop::chop_articulation_metrics;
use riotbox_audio::w30::{W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewSampleWindow};

#[derive(Clone, Copy, Debug)]
pub(super) struct W30SourcePlaybackProfile {
    pub(super) phase_offset_samples: usize,
    pub(super) stride_divisor: usize,
    pub(super) gain: f32,
}

pub(super) fn w30_source_playback_profile(
    source_window_preview: &W30PreviewSampleWindow,
) -> W30SourcePlaybackProfile {
    let sample_count = source_window_preview
        .sample_count
        .clamp(1, W30_PREVIEW_SAMPLE_WINDOW_LEN);
    let samples = &source_window_preview.samples[..sample_count];
    let (_, _, tail_to_body_rms_ratio) = chop_articulation_metrics(samples);
    let spectral = spectral_energy_metrics(samples);
    let source_window_offset = source_window_preview.source_start_frame as usize % sample_count;
    let energy_phase = ((spectral.low_band_energy_ratio * 3.0
        + spectral.mid_band_energy_ratio * 5.0
        + spectral.high_band_energy_ratio * 7.0)
        * sample_count as f32
        * 0.125)
        .round() as usize;
    let phase_offset_samples = (source_window_offset + energy_phase) % sample_count;
    let stride_divisor = if spectral.high_band_energy_ratio > 0.08 {
        1
    } else if tail_to_body_rms_ratio > 0.75 {
        3
    } else {
        2
    };
    W30SourcePlaybackProfile {
        phase_offset_samples,
        stride_divisor,
        gain: 1.52,
    }
}
