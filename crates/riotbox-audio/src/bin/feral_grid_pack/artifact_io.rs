//! Existing offline WAV/metrics publication over the writer-owned path layout.
use super::{
    config::{CHANNEL_COUNT, PACK_ID, SAMPLE_RATE},
    grid::Grid,
    output_paths::metrics_path_for,
    render_measurements::{RenderMetrics, render_metrics},
};
use riotbox_audio::source_audio::{SourceAudioError, write_interleaved_pcm16_wav};
use std::{fs, path::Path};

pub(super) fn write_audio_with_metrics(
    path: &Path,
    samples: &[f32],
    grid: &Grid,
) -> Result<(), SourceAudioError> {
    write_interleaved_pcm16_wav(path, SAMPLE_RATE, CHANNEL_COUNT, samples)?;
    write_metrics_markdown(&metrics_path_for(path), render_metrics(samples, grid))
        .map_err(|error| SourceAudioError::Io(error.to_string()))
}

fn write_metrics_markdown(path: &Path, metrics: RenderMetrics) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Grid Demo Metrics\n\n\
             - Pack: `{PACK_ID}`\n\
             - Peak abs: `{:.6}`\n\
             - RMS: `{:.6}`\n\
             - Active samples: `{}`\n\
             - Sum: `{:.6}`\n\
             - Mean abs: `{:.6}`\n\
             - Zero crossings: `{}`\n\
             - Crest factor: `{:.6}`\n\
             - Active sample ratio: `{:.6}`\n\
             - Silence ratio: `{:.6}`\n\
             - DC offset: `{:.6}`\n\
             - Onset count: `{}`\n\
             - Event density per bar: `{:.6}`\n\
             - Low-band peak abs: `{:.6}`\n\
             - Low-band RMS: `{:.6}`\n",
            metrics.signal.peak_abs,
            metrics.signal.rms,
            metrics.signal.active_samples,
            metrics.signal.sum,
            metrics.signal.mean_abs,
            metrics.signal.zero_crossings,
            metrics.signal.crest_factor,
            metrics.signal.active_sample_ratio,
            metrics.signal.silence_ratio,
            metrics.signal.dc_offset,
            metrics.signal.onset_count,
            metrics.signal.event_density_per_bar,
            metrics.low_band.peak_abs,
            metrics.low_band.rms
        ),
    )
}
