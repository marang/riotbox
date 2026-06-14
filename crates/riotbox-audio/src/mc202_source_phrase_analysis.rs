use crate::source_audio::SourceAudioCache;
use riotbox_core::source_graph::{PhraseAudioFeatures, PhraseSpan, SourceGraph, TimingModel};

const LOWPASS_CUTOFF_HZ: f32 = 180.0;
const ANALYSIS_WINDOW_FRAMES: usize = 512;
const ANALYSIS_HOP_FRAMES: usize = 256;

#[must_use]
pub fn analyze_mc202_phrase_audio_features(
    source: &SourceAudioCache,
    timing: &TimingModel,
) -> Vec<PhraseAudioFeatures> {
    phrase_grid(timing)
        .iter()
        .filter_map(|phrase| analyze_phrase(source, timing, phrase))
        .collect()
}

pub fn attach_mc202_phrase_audio_features(graph: &mut SourceGraph, source: &SourceAudioCache) {
    graph.phrase_audio_features = analyze_mc202_phrase_audio_features(source, &graph.timing);
}

fn analyze_phrase(
    source: &SourceAudioCache,
    timing: &TimingModel,
    phrase: &PhraseSpan,
) -> Option<PhraseAudioFeatures> {
    let (start_seconds, end_seconds) = phrase_seconds(timing, phrase)?;
    if end_seconds <= start_seconds {
        return None;
    }

    let window = source.window_by_seconds(start_seconds, end_seconds - start_seconds);
    if window.frame_count == 0 {
        return None;
    }

    let mono = mono_window_samples(source, window.start_frame, window.frame_count);
    if mono.is_empty() {
        return None;
    }

    let low = lowpass(&mono, source.sample_rate, LOWPASS_CUTOFF_HZ);
    let mid = mono
        .iter()
        .zip(low.iter())
        .map(|(sample, low)| sample - low)
        .collect::<Vec<_>>();
    let low_band_rms = rms(&low);
    let mid_rms = rms(&mid);
    let full_rms = rms(&mono);
    let low_mid_ratio = if low_band_rms + mid_rms <= f32::EPSILON {
        0.0
    } else {
        low_band_rms / (low_band_rms + mid_rms)
    };
    let low_band_movement = envelope_movement(&low);
    let onset_report = onset_report(&mono, source.sample_rate, start_seconds, timing, phrase);
    let spectral_roughness = roughness_proxy(&mono, full_rms);
    let spectral_brightness = if full_rms <= f32::EPSILON {
        0.0
    } else {
        (mid_rms / full_rms).clamp(0.0, 1.0)
    };
    let hook_restraint_hint = clamp01(
        spectral_brightness * 0.35
            + (1.0 - onset_report.transient_density) * 0.25
            + (1.0 - low_band_movement) * 0.20
            + (1.0 - low_mid_ratio) * 0.20,
    );
    let confidence = phrase_feature_confidence(
        phrase.confidence,
        timing.bpm_confidence,
        full_rms,
        window.frame_count,
        source.sample_rate,
    );

    Some(PhraseAudioFeatures {
        phrase_index: phrase.phrase_index,
        start_seconds,
        end_seconds,
        start_bar: phrase.start_bar,
        end_bar: phrase.end_bar,
        low_band_rms,
        low_mid_ratio,
        low_band_movement,
        transient_density: onset_report.transient_density,
        offbeat_onset_density: onset_report.offbeat_density,
        spectral_roughness,
        spectral_brightness,
        hook_restraint_hint,
        confidence,
        provenance_refs: vec![
            "mc202.phrase-audio-features.v0".into(),
            format!("source:{}", source.path.display()),
            format!("phrase:{}", phrase.phrase_index),
        ],
    })
}

fn phrase_grid(timing: &TimingModel) -> &[PhraseSpan] {
    timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.phrase_grid.as_slice())
        .filter(|phrases| !phrases.is_empty())
        .unwrap_or(timing.phrase_grid.as_slice())
}

fn phrase_seconds(timing: &TimingModel, phrase: &PhraseSpan) -> Option<(f32, f32)> {
    let bars = timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.bar_grid.as_slice())
        .filter(|bars| !bars.is_empty())
        .unwrap_or(timing.bar_grid.as_slice());
    let start = bars
        .iter()
        .find(|bar| bar.bar_index == phrase.start_bar || bar.bar_index + 1 == phrase.start_bar)
        .map(|bar| bar.start_seconds);
    let end = bars
        .iter()
        .find(|bar| bar.bar_index == phrase.end_bar || bar.bar_index + 1 == phrase.end_bar)
        .map(|bar| bar.end_seconds);
    match (start, end) {
        (Some(start), Some(end)) => return Some((start, end)),
        (Some(start), None) => {
            if let Some(seconds_per_bar) = seconds_per_bar(timing) {
                return Some((
                    start,
                    start
                        + seconds_per_bar
                            * phrase
                                .end_bar
                                .saturating_sub(phrase.start_bar)
                                .saturating_add(1) as f32,
                ));
            }
        }
        _ => {}
    }

    let seconds_per_bar = seconds_per_bar(timing)?;
    Some((
        phrase.start_bar.saturating_sub(1) as f32 * seconds_per_bar,
        phrase.end_bar as f32 * seconds_per_bar,
    ))
}

fn seconds_per_bar(timing: &TimingModel) -> Option<f32> {
    let bpm = timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.bpm)
        .or(timing.bpm_estimate)?;
    if bpm <= 0.0 {
        return None;
    }
    let beats_per_bar = timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.meter.beats_per_bar)
        .or(timing.meter_hint.map(|meter| meter.beats_per_bar))
        .unwrap_or(4)
        .max(1);
    Some(60.0 / bpm * f32::from(beats_per_bar))
}

fn mono_window_samples(
    source: &SourceAudioCache,
    start_frame: usize,
    frame_count: usize,
) -> Vec<f32> {
    let channels = usize::from(source.channel_count);
    if channels == 0 {
        return Vec::new();
    }
    let samples = source.interleaved_samples();
    let end_frame = start_frame
        .saturating_add(frame_count)
        .min(source.frame_count());
    let mut mono = Vec::with_capacity(end_frame.saturating_sub(start_frame));
    for frame in start_frame..end_frame {
        let sample_start = frame.saturating_mul(channels);
        let sum = (0..channels)
            .map(|channel| samples.get(sample_start + channel).copied().unwrap_or(0.0))
            .sum::<f32>();
        mono.push(sum / channels as f32);
    }
    mono
}

fn lowpass(samples: &[f32], sample_rate: u32, cutoff_hz: f32) -> Vec<f32> {
    if samples.is_empty() || sample_rate == 0 {
        return Vec::new();
    }
    let dt = 1.0 / sample_rate as f32;
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz.max(1.0));
    let alpha = dt / (rc + dt);
    let mut filtered = Vec::with_capacity(samples.len());
    let mut low = samples[0];
    for sample in samples {
        low += alpha * (sample - low);
        filtered.push(low);
    }
    filtered
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
}

fn envelope_movement(samples: &[f32]) -> f32 {
    let windows = window_rms(samples, ANALYSIS_WINDOW_FRAMES, ANALYSIS_HOP_FRAMES);
    let peak = windows.iter().copied().fold(0.0_f32, f32::max);
    if windows.len() < 2 || peak <= f32::EPSILON {
        return 0.0;
    }
    let mean_delta = windows
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .sum::<f32>()
        / (windows.len() - 1) as f32;
    clamp01(mean_delta / peak * 3.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OnsetReport {
    transient_density: f32,
    offbeat_density: f32,
}

fn onset_report(
    samples: &[f32],
    sample_rate: u32,
    start_seconds: f32,
    timing: &TimingModel,
    phrase: &PhraseSpan,
) -> OnsetReport {
    let windows = window_rms(samples, ANALYSIS_WINDOW_FRAMES, ANALYSIS_HOP_FRAMES);
    if windows.len() < 2 || sample_rate == 0 {
        return OnsetReport {
            transient_density: 0.0,
            offbeat_density: 0.0,
        };
    }
    let flux = windows
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).max(0.0))
        .collect::<Vec<_>>();
    let peak_flux = flux.iter().copied().fold(0.0_f32, f32::max);
    let threshold = (peak_flux * 0.35).max(0.005);
    let mut onset_count = 0_u32;
    let mut offbeat_count = 0_u32;
    for (index, value) in flux.iter().enumerate() {
        if *value < threshold || *value <= 0.0 {
            continue;
        }
        onset_count += 1;
        let time_seconds =
            start_seconds + (index * ANALYSIS_HOP_FRAMES) as f32 / sample_rate as f32;
        if is_offbeat_onset(time_seconds, timing) {
            offbeat_count += 1;
        }
    }
    let beat_count = phrase_beat_count(timing, phrase).max(1) as f32;
    let transient_density = clamp01(onset_count as f32 / beat_count);
    let offbeat_density = if onset_count == 0 {
        0.0
    } else {
        clamp01(offbeat_count as f32 / onset_count as f32)
    };
    OnsetReport {
        transient_density,
        offbeat_density,
    }
}

fn window_rms(samples: &[f32], window_size: usize, hop_size: usize) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let window_size = window_size.max(1);
    let hop_size = hop_size.max(1);
    let mut start = 0_usize;
    while start < samples.len() {
        let end = start.saturating_add(window_size).min(samples.len());
        result.push(rms(&samples[start..end]));
        start = start.saturating_add(hop_size);
    }
    result
}

fn is_offbeat_onset(time_seconds: f32, timing: &TimingModel) -> bool {
    let Some(bpm) = timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.bpm)
        .or(timing.bpm_estimate)
    else {
        return false;
    };
    if bpm <= 0.0 {
        return false;
    }
    let seconds_per_beat = 60.0 / bpm;
    let beat_position = time_seconds / seconds_per_beat;
    let fraction = beat_position.fract();
    (0.18..=0.82).contains(&fraction) && !(0.43..=0.57).contains(&fraction)
}

fn phrase_beat_count(timing: &TimingModel, phrase: &PhraseSpan) -> u32 {
    let beats_per_bar = timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.meter.beats_per_bar)
        .or(timing.meter_hint.map(|meter| meter.beats_per_bar))
        .unwrap_or(4)
        .max(1);
    phrase
        .end_bar
        .saturating_sub(phrase.start_bar)
        .saturating_add(1)
        .saturating_mul(u32::from(beats_per_bar))
}

fn roughness_proxy(samples: &[f32], full_rms: f32) -> f32 {
    if samples.len() < 2 || full_rms <= f32::EPSILON {
        return 0.0;
    }
    let mean_abs_delta = samples
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .sum::<f32>()
        / (samples.len() - 1) as f32;
    clamp01(mean_abs_delta / full_rms * 0.45)
}

fn phrase_feature_confidence(
    phrase_confidence: f32,
    bpm_confidence: f32,
    full_rms: f32,
    frame_count: usize,
    sample_rate: u32,
) -> f32 {
    let duration_seconds = if sample_rate == 0 {
        0.0
    } else {
        frame_count as f32 / sample_rate as f32
    };
    let duration_coverage = clamp01(duration_seconds / 2.0);
    let signal_presence = clamp01(full_rms * 6.0);
    clamp01(
        phrase_confidence.clamp(0.0, 1.0) * 0.35
            + bpm_confidence.clamp(0.0, 1.0) * 0.25
            + signal_presence * 0.25
            + duration_coverage * 0.15,
    )
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
#[path = "mc202_source_phrase_analysis_tests.rs"]
mod mc202_source_phrase_analysis_tests;
