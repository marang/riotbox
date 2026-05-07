use crate::source_audio::SourceAudioCache;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceTimingProbeConfig {
    pub window_size_frames: usize,
    pub hop_size_frames: usize,
    pub onset_threshold_ratio: f32,
    pub min_onset_flux: f32,
}

impl Default for SourceTimingProbeConfig {
    fn default() -> Self {
        Self {
            window_size_frames: 1024,
            hop_size_frames: 512,
            onset_threshold_ratio: 0.35,
            min_onset_flux: 0.02,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceTimingProbe {
    pub sample_rate: u32,
    pub channel_count: u16,
    pub duration_seconds: f32,
    pub window_size_frames: usize,
    pub hop_size_frames: usize,
    pub windows: Vec<SourceTimingProbeWindow>,
    pub peak_energy: f32,
    pub peak_positive_flux: f32,
    pub onset_count: usize,
    pub onset_density_per_second: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceTimingProbeWindow {
    pub window_index: u32,
    pub start_frame: usize,
    pub start_seconds: f32,
    pub energy: f32,
    pub positive_flux: f32,
    pub onset: bool,
}

#[must_use]
pub fn analyze_source_timing_probe(
    source: &SourceAudioCache,
    config: SourceTimingProbeConfig,
) -> SourceTimingProbe {
    let window_size_frames = config.window_size_frames.max(1);
    let hop_size_frames = config.hop_size_frames.max(1);
    let mut windows = probe_windows(source, window_size_frames, hop_size_frames);
    let peak_energy = windows
        .iter()
        .map(|window| window.energy)
        .fold(0.0_f32, f32::max);
    let peak_positive_flux = windows
        .iter()
        .map(|window| window.positive_flux)
        .fold(0.0_f32, f32::max);
    let threshold = (peak_positive_flux * config.onset_threshold_ratio.max(0.0))
        .max(config.min_onset_flux.max(0.0));

    for window in &mut windows {
        window.onset = window.positive_flux >= threshold && window.positive_flux > 0.0;
    }

    let onset_count = windows.iter().filter(|window| window.onset).count();
    SourceTimingProbe {
        sample_rate: source.sample_rate,
        channel_count: source.channel_count,
        duration_seconds: source.duration_seconds(),
        window_size_frames,
        hop_size_frames,
        windows,
        peak_energy,
        peak_positive_flux,
        onset_count,
        onset_density_per_second: onset_density(onset_count, source.duration_seconds()),
    }
}

fn probe_windows(
    source: &SourceAudioCache,
    window_size_frames: usize,
    hop_size_frames: usize,
) -> Vec<SourceTimingProbeWindow> {
    let frame_count = source.frame_count();
    if frame_count == 0 {
        return Vec::new();
    }

    let mut windows = Vec::new();
    let mut previous_energy = 0.0_f32;
    let mut start_frame = 0_usize;
    while start_frame < frame_count {
        let energy = window_energy(source, start_frame, window_size_frames);
        let positive_flux = (energy - previous_energy).max(0.0);
        windows.push(SourceTimingProbeWindow {
            window_index: u32::try_from(windows.len()).unwrap_or(u32::MAX),
            start_frame,
            start_seconds: start_frame as f32 / source.sample_rate as f32,
            energy,
            positive_flux,
            onset: false,
        });
        previous_energy = energy;
        start_frame = start_frame.saturating_add(hop_size_frames);
    }

    windows
}

fn window_energy(source: &SourceAudioCache, start_frame: usize, window_size_frames: usize) -> f32 {
    let channels = usize::from(source.channel_count);
    let frame_count = source.frame_count();
    let end_frame = start_frame
        .saturating_add(window_size_frames)
        .min(frame_count);
    if channels == 0 || start_frame >= end_frame {
        return 0.0;
    }

    let samples = source.interleaved_samples();
    let mut sum_squares = 0.0_f32;
    let mut mono_frame_count = 0_usize;
    for frame in start_frame..end_frame {
        let sample_start = frame.saturating_mul(channels);
        let mut mono = 0.0_f32;
        for channel in 0..channels {
            mono += samples.get(sample_start + channel).copied().unwrap_or(0.0);
        }
        mono /= channels as f32;
        sum_squares += mono * mono;
        mono_frame_count += 1;
    }

    (sum_squares / mono_frame_count.max(1) as f32).sqrt()
}

fn onset_density(onset_count: usize, duration_seconds: f32) -> f32 {
    if duration_seconds <= 0.0 {
        return 0.0;
    }
    onset_count as f32 / duration_seconds
}

#[cfg(test)]
mod tests;
