use super::args::Args;
use super::config::{CASE_ID, CHANNEL_COUNT, PACK_ID, SAMPLE_RATE};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub(super) fn write_pcm16_wav(
    path: &Path,
    sample_rate: u32,
    channel_count: u16,
    samples: &[f32],
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let bytes_per_sample = 2_u16;
    let bits_per_sample = 16_u16;
    let byte_rate = sample_rate * u32::from(channel_count) * u32::from(bytes_per_sample);
    let block_align = channel_count * bytes_per_sample;
    let data_bytes = u32::try_from(samples.len().saturating_mul(usize::from(bytes_per_sample)))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "WAV output too large"))?;
    let riff_size = 36_u32
        .checked_add(data_bytes)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "WAV output too large"))?;

    let mut file = fs::File::create(path)?;
    file.write_all(b"RIFF")?;
    file.write_all(&riff_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;
    file.write_all(b"fmt ")?;
    file.write_all(&16_u32.to_le_bytes())?;
    file.write_all(&1_u16.to_le_bytes())?;
    file.write_all(&channel_count.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_bytes.to_le_bytes())?;

    for sample in samples {
        let pcm = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
        file.write_all(&pcm.to_le_bytes())?;
    }

    Ok(())
}

pub(super) fn metrics_path_for(output_path: &Path) -> PathBuf {
    let mut path = output_path.to_path_buf();
    path.set_file_name(
        match output_path.file_stem().and_then(|stem| stem.to_str()) {
            Some(stem) => format!("{stem}.metrics.md"),
            None => "candidate.metrics.md".to_string(),
        },
    );
    path
}

pub(super) fn write_metrics_markdown(
    path: &Path,
    args: &Args,
    sample_count: usize,
    metrics: riotbox_audio::runtime::OfflineAudioMetrics,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(
        path,
        format!(
            "# W-30 Preview Smoke Metrics\n\n\
             - Pack: `{PACK_ID}`\n\
             - Case: `{CASE_ID}`\n\
             - Role: `{}`\n\
             - Source input: `{}`\n\
             - Output: `{}`\n\
             - Sample rate: `{SAMPLE_RATE}`\n\
             - Channels: `{CHANNEL_COUNT}`\n\
             - Duration seconds: `{:.3}`\n\
             - Samples: `{sample_count}`\n\
             - Active samples: `{}`\n\
             - Peak abs: `{:.6}`\n\
             - Clip count: `{}`\n\
             - Near clip count: `{}`\n\
             - Headroom to full scale: `{:.6}`\n\
             - RMS: `{:.6}`\n\
             - Sum: `{:.6}`\n\
             - Mean abs: `{:.6}`\n\
             - Zero crossings: `{}`\n\
             - Crest factor: `{:.6}`\n\
             - Active sample ratio: `{:.6}`\n\
             - Silence ratio: `{:.6}`\n\
             - DC offset: `{:.6}`\n\
             - Onset count: `{}`\n\
             - Event density per bar: `{:.6}`\n",
            args.role.label(),
            args.source_input_label(),
            args.output_path.display(),
            args.duration_seconds,
            metrics.active_samples,
            metrics.peak_abs,
            metrics.clip_count,
            metrics.near_clip_count,
            metrics.headroom_to_full_scale,
            metrics.rms,
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
