use super::config::{CHANNEL_COUNT, SAMPLE_RATE};
use riotbox_audio::{
    runtime::{OfflineAudioMetrics, signal_metrics},
    source_audio::{SourceAudioError, write_interleaved_pcm16_wav},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn write_named_audio_with_metrics(
    path: &Path,
    samples: &[f32],
) -> Result<(), SourceAudioError> {
    write_interleaved_pcm16_wav(path, SAMPLE_RATE, CHANNEL_COUNT, samples)?;
    write_metrics_markdown(&metrics_path_for(path), signal_metrics(samples))
        .map_err(|error| SourceAudioError::Io(error.to_string()))
}

pub(super) fn metrics_path_for(path: &Path) -> PathBuf {
    let mut metrics_path = path.to_path_buf();
    metrics_path.set_file_name(match path.file_stem().and_then(|stem| stem.to_str()) {
        Some(stem) => format!("{stem}.metrics.md"),
        None => "metrics.md".to_string(),
    });
    metrics_path
}

pub(super) fn write_metrics_markdown(
    path: &Path,
    metrics: OfflineAudioMetrics,
) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Before / After Metrics\n\n\
             - Peak abs: `{:.6}`\n\
             - Clip count: `{}`\n\
             - Near clip count: `{}`\n\
             - Headroom to full scale: `{:.6}`\n\
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
             - Event density per bar: `{:.6}`\n",
            metrics.peak_abs,
            metrics.clip_count,
            metrics.near_clip_count,
            metrics.headroom_to_full_scale,
            metrics.rms,
            metrics.active_samples,
            metrics.sum,
            metrics.mean_abs,
            metrics.zero_crossings,
            metrics.crest_factor,
            metrics.active_sample_ratio,
            metrics.silence_ratio,
            metrics.dc_offset,
            metrics.onset_count,
            metrics.event_density_per_bar
        ),
    )
}
