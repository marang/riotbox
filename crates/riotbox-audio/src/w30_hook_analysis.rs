use riotbox_core::source_graph::{
    BarSpan, W30HookCandidateEvidence, W30HookCandidateStatus, W30HookFeatures,
};

use crate::source_audio::SourceAudioCache;

const RMS_FLOOR: f32 = 0.003;
const DOWNBEAT_CONFIDENCE_FLOOR: f32 = 0.35;
const EPSILON: f32 = 1.0e-6;
const LOWPASS_HZ: f32 = 1_200.0;

#[must_use]
pub fn analyze_w30_hook_candidates(
    source: &SourceAudioCache,
    bars: &[BarSpan],
    beats_per_bar: u8,
) -> Vec<W30HookCandidateEvidence> {
    let mono = mono_samples(source);
    let sample_rate = source.sample_rate as f32;
    let mut evidence = bars
        .iter()
        .map(|bar| analyze_bar(&mono, sample_rate, *bar, beats_per_bar))
        .collect::<Vec<_>>();

    let eligible_indices = evidence
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            (candidate.status == W30HookCandidateStatus::Eligible).then_some(index)
        })
        .collect::<Vec<_>>();
    if eligible_indices.len() < 2 {
        return evidence;
    }

    let median_rms = median(
        eligible_indices
            .iter()
            .map(|index| evidence[*index].bar_rms)
            .collect(),
    );
    let median_brightness = median(
        eligible_indices
            .iter()
            .map(|index| evidence[*index].raw_features.spectral_contrast)
            .collect(),
    );

    for index in eligible_indices.iter().copied() {
        let brightness = evidence[index].raw_features.spectral_contrast;
        evidence[index].raw_features.spectral_contrast = (brightness - median_brightness).abs();
        let previous = index
            .checked_sub(1)
            .and_then(|index| evidence.get(index))
            .filter(|candidate| candidate.status != W30HookCandidateStatus::IncompleteBar)
            .map(|candidate| candidate.bar_rms);
        let next = evidence
            .get(index + 1)
            .filter(|candidate| candidate.status != W30HookCandidateStatus::IncompleteBar)
            .map(|candidate| candidate.bar_rms);
        let mut distance_sum = 0.0;
        let mut neighbor_count = 0_u8;
        for neighbor in [previous, next].into_iter().flatten() {
            distance_sum += (evidence[index].bar_rms - neighbor).abs();
            neighbor_count = neighbor_count.saturating_add(1);
        }
        evidence[index].raw_features.phrase_contrast = if neighbor_count == 0 {
            0.0
        } else {
            (distance_sum / f32::from(neighbor_count) / median_rms.max(EPSILON)).clamp(0.0, 2.0)
        };
    }

    normalize_and_score(&mut evidence, &eligible_indices);
    evidence
}

fn analyze_bar(
    mono: &[f32],
    sample_rate: f32,
    bar: BarSpan,
    beats_per_bar: u8,
) -> W30HookCandidateEvidence {
    let start_frame = seconds_to_frame(bar.start_seconds, sample_rate);
    let end_frame = seconds_to_frame(bar.end_seconds, sample_rate);
    let complete = bar.start_seconds.is_finite()
        && bar.end_seconds.is_finite()
        && end_frame > start_frame
        && end_frame <= mono.len();
    let samples = if complete {
        &mono[start_frame..end_frame]
    } else {
        &[]
    };
    let bar_rms = rms(samples);
    let status = if !complete {
        W30HookCandidateStatus::IncompleteBar
    } else if bar_rms < RMS_FLOOR {
        W30HookCandidateStatus::BarRmsBelowFloor
    } else if bar.downbeat_confidence < DOWNBEAT_CONFIDENCE_FLOOR {
        W30HookCandidateStatus::DownbeatConfidenceBelowFloor
    } else {
        W30HookCandidateStatus::Eligible
    };
    let raw_features = if complete {
        bar_features(samples, sample_rate, beats_per_bar, bar_rms)
    } else {
        W30HookFeatures::default()
    };

    W30HookCandidateEvidence {
        bar_index: bar.bar_index,
        start_seconds: bar.start_seconds,
        end_seconds: bar.end_seconds,
        downbeat_confidence: bar.downbeat_confidence,
        bar_rms,
        status,
        raw_features,
        normalized_features: None,
        attack_body_contrast_score: None,
        repetition_salience_score: None,
    }
}

fn bar_features(
    samples: &[f32],
    sample_rate: f32,
    beats_per_bar: u8,
    bar_rms: f32,
) -> W30HookFeatures {
    let rms_window = (sample_rate * 0.020).round().max(1.0) as usize;
    let rms_hop = (sample_rate * 0.010).round().max(1.0) as usize;
    let envelope = rms_envelope(samples, rms_window, rms_hop);
    let positive_flux = envelope
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).max(0.0))
        .collect::<Vec<_>>();
    let max_flux = positive_flux.iter().copied().fold(0.0_f32, f32::max);
    let peak_threshold = (0.35 * max_flux).max(0.02 * bar_rms);
    let local_flux_peak_count = positive_flux
        .iter()
        .enumerate()
        .filter(|(index, value)| {
            let previous = index
                .checked_sub(1)
                .and_then(|index| positive_flux.get(index))
                .copied()
                .unwrap_or(0.0);
            let next = positive_flux.get(index + 1).copied().unwrap_or(0.0);
            **value >= peak_threshold && **value >= previous && **value > next
        })
        .count();
    let strongest_onset_frame = positive_flux
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.total_cmp(right.1))
        .map_or(0, |(index, _)| (index + 1).saturating_mul(rms_hop));
    let attack_end = strongest_onset_frame
        .saturating_add((sample_rate * 0.040).round() as usize)
        .min(samples.len());
    let body_start = attack_end;
    let body_end = strongest_onset_frame
        .saturating_add((sample_rate * 0.200).round() as usize)
        .min(samples.len());
    let attack_rms = rms(&samples[strongest_onset_frame.min(samples.len())..attack_end]);
    let body_rms = rms(&samples[body_start.min(samples.len())..body_end]);
    let brightness = highpass_residual_rms(samples, sample_rate) / bar_rms.max(EPSILON);

    W30HookFeatures {
        onset_flux: (max_flux / bar_rms.max(EPSILON)).clamp(0.0, 4.0),
        body_retention: (body_rms / attack_rms.max(bar_rms).max(EPSILON)).clamp(0.0, 1.5),
        onset_density: (local_flux_peak_count as f32 / f32::from(beats_per_bar.max(1)))
            .clamp(0.0, 2.0),
        spectral_contrast: brightness.clamp(0.0, 1.5),
        repetition_salience: half_cosine_similarity(&envelope),
        phrase_contrast: 0.0,
    }
}

fn normalize_and_score(evidence: &mut [W30HookCandidateEvidence], indices: &[usize]) {
    let mut mins = [f32::INFINITY; 6];
    let mut maxs = [f32::NEG_INFINITY; 6];
    for index in indices.iter().copied() {
        for (feature_index, value) in feature_values(evidence[index].raw_features)
            .into_iter()
            .enumerate()
        {
            mins[feature_index] = mins[feature_index].min(value);
            maxs[feature_index] = maxs[feature_index].max(value);
        }
    }
    for index in indices.iter().copied() {
        let raw = feature_values(evidence[index].raw_features);
        let mut normalized = [0.0; 6];
        for feature_index in 0..6 {
            let range = maxs[feature_index] - mins[feature_index];
            normalized[feature_index] = if range <= EPSILON {
                0.0
            } else {
                ((raw[feature_index] - mins[feature_index]) / range).clamp(0.0, 1.0)
            };
        }
        let features = features_from_values(normalized);
        evidence[index].normalized_features = Some(features);
        evidence[index].attack_body_contrast_score = Some(
            0.40 * features.onset_flux
                + 0.25 * features.body_retention
                + 0.20 * features.onset_density
                + 0.15 * features.spectral_contrast,
        );
        evidence[index].repetition_salience_score = Some(
            0.25 * features.onset_flux
                + 0.15 * features.body_retention
                + 0.10 * features.onset_density
                + 0.15 * features.spectral_contrast
                + 0.25 * features.repetition_salience
                + 0.10 * features.phrase_contrast,
        );
    }
}

fn mono_samples(source: &SourceAudioCache) -> Vec<f32> {
    let channels = usize::from(source.channel_count);
    source
        .interleaved_samples()
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

fn rms_envelope(samples: &[f32], window: usize, hop: usize) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }
    (0..samples.len())
        .step_by(hop)
        .map(|start| rms(&samples[start..start.saturating_add(window).min(samples.len())]))
        .collect()
}

fn highpass_residual_rms(samples: &[f32], sample_rate: f32) -> f32 {
    let time_constant = 1.0 / (2.0 * std::f32::consts::PI * LOWPASS_HZ);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (time_constant + dt);
    let mut lowpass = 0.0;
    let mut energy = 0.0;
    for sample in samples.iter().copied() {
        lowpass += alpha * (sample - lowpass);
        let residual = sample - lowpass;
        energy += residual * residual;
    }
    (energy / samples.len().max(1) as f32).sqrt()
}

fn half_cosine_similarity(envelope: &[f32]) -> f32 {
    let half = envelope.len() / 2;
    if half == 0 {
        return 0.0;
    }
    let left = &envelope[..half];
    let right = &envelope[envelope.len() - half..];
    let dot = left.iter().zip(right).map(|(a, b)| a * b).sum::<f32>();
    let left_norm = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    (dot / (left_norm * right_norm).max(EPSILON)).clamp(0.0, 1.0)
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
}

fn seconds_to_frame(seconds: f32, sample_rate: f32) -> usize {
    (seconds.max(0.0) * sample_rate).round() as usize
}

fn feature_values(features: W30HookFeatures) -> [f32; 6] {
    [
        features.onset_flux,
        features.body_retention,
        features.onset_density,
        features.spectral_contrast,
        features.repetition_salience,
        features.phrase_contrast,
    ]
}

fn features_from_values(values: [f32; 6]) -> W30HookFeatures {
    W30HookFeatures {
        onset_flux: values[0],
        body_retention: values[1],
        onset_density: values[2],
        spectral_contrast: values[3],
        repetition_salience: values[4],
        phrase_contrast: values[5],
    }
}

fn median(mut values: Vec<f32>) -> f32 {
    values.sort_by(f32::total_cmp);
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[middle - 1] + values[middle]) * 0.5
    } else {
        values[middle]
    }
}

#[cfg(test)]
mod tests {
    use riotbox_core::source_graph::BarSpan;

    use super::*;

    #[test]
    fn synthetic_bars_are_deterministic_and_filename_independent() {
        let samples = synthetic_four_bars();
        let first =
            SourceAudioCache::from_interleaved_samples("first.wav", 1_000, 1, samples.clone())
                .expect("first source");
        let renamed = SourceAudioCache::from_interleaved_samples("renamed.wav", 1_000, 1, samples)
            .expect("renamed source");
        let bars = four_bars();

        assert_eq!(
            analyze_w30_hook_candidates(&first, &bars, 4),
            analyze_w30_hook_candidates(&renamed, &bars, 4)
        );
    }

    #[test]
    fn flat_evidence_normalizes_to_zero_without_arbitrary_promotion() {
        let source =
            SourceAudioCache::from_interleaved_samples("flat.wav", 1_000, 1, vec![0.1; 4_000])
                .expect("flat source");
        let evidence = analyze_w30_hook_candidates(&source, &four_bars(), 4);

        assert_eq!(evidence.len(), 4);
        assert!(evidence.iter().all(|candidate| {
            candidate.normalized_features == Some(W30HookFeatures::default())
                && candidate.attack_body_contrast_score == Some(0.0)
                && candidate.repetition_salience_score == Some(0.0)
        }));
    }

    #[test]
    fn fewer_than_two_eligible_bars_cannot_produce_scores() {
        let source =
            SourceAudioCache::from_interleaved_samples("short.wav", 1_000, 1, vec![0.1; 1_000])
                .expect("short source");
        let evidence = analyze_w30_hook_candidates(&source, &four_bars(), 4);

        assert_eq!(evidence[0].status, W30HookCandidateStatus::Eligible);
        assert!(
            evidence
                .iter()
                .all(|candidate| candidate.normalized_features.is_none())
        );
    }

    fn four_bars() -> Vec<BarSpan> {
        (0..4)
            .map(|index| BarSpan {
                bar_index: index + 1,
                start_seconds: index as f32,
                end_seconds: index as f32 + 1.0,
                downbeat_confidence: 1.0,
                phrase_index: Some(1),
            })
            .collect()
    }

    fn synthetic_four_bars() -> Vec<f32> {
        let mut samples = vec![0.02; 4_000];
        for index in [1_010, 1_250, 1_500, 1_750] {
            samples[index] = 0.9;
        }
        for sample in &mut samples[2_000..2_200] {
            *sample = 0.35;
        }
        for index in [3_010, 3_510] {
            samples[index] = 0.7;
        }
        samples
    }
}
