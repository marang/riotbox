//! Source-general Stage-A F4: source-native low/low-mid body sustain.
//!
//! The physical attack remains bit-identical. The renderer selects one
//! event-coupled body band from the source itself and gives only its decaying
//! body a bounded, smoothly entered/exited lift. This tests lower-mid body and
//! longer decay as coupled velocity cues without pitch, delay, duplicate hits,
//! resampling, limiting, or a generated oscillator.

use std::f64::consts::TAU;

use super::common::{
    FrozenEventInput, FrozenEventRegion, InvalidEventInput, PercussiveForceError,
    PercussiveForceRefusal, checked_output_sample, validate_frozen_event,
};

pub const F4_SOURCE_NATIVE_BODY_SUSTAIN_V1: &str = "f4_source_native_body_sustain_v1";

const BAND_EDGES_HZ: [f64; 4] = [55.0, 180.0, 560.0, 1_120.0];
const LOOKBEHIND_NOISE_MULTIPLIER: f64 = 4.0;
const QUANTIZATION_LSB_MULTIPLIER: f64 = 16.0;
const BODY_ENVELOPE_MS: f64 = 8.0;
const BODY_ENTRY_MS: f64 = 2.0;
const BODY_EXIT_MS: f64 = 10.0;
const MAXIMUM_ADDITIONAL_BAND_GAIN: f64 = 0.5;
const MINIMUM_ADDITIONAL_GAIN_FRACTION: f64 = 0.35;
const OUTPUT_PEAK_STRICT_MAXIMUM: f64 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F4BodyBand {
    Low,
    LowMid,
    MidBody,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F4BodySustainPolicy {
    pub version_id: &'static str,
    pub region: FrozenEventRegion,
    pub band_edges_hz: [f64; 4],
    pub selected_band: F4BodyBand,
    pub selected_band_index: usize,
    pub band_attack_mean_square: [f64; 3],
    pub band_body_mean_square: [f64; 3],
    pub band_lookbehind_mean_square: [f64; 3],
    pub band_trust_floor_mean_square: [f64; 3],
    pub trusted_bands: [bool; 3],
    pub selected_band_score: f64,
    pub lookbehind_frames: usize,
    pub body_envelope_frames: usize,
    pub body_entry_frames: usize,
    pub body_exit_frames: usize,
    pub maximum_additional_band_gain: f64,
    pub minimum_additional_gain_fraction: f64,
    pub selected_body_envelope_peak: f64,
    pub maximum_resolved_additional_gain: f64,
    pub source_body_energy: f64,
    pub candidate_body_energy: f64,
    pub body_energy_ratio: f64,
    pub output_peak: f64,
    pub attack_bit_identical: bool,
    pub playback_rate_numerator: u32,
    pub playback_rate_denominator: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F4RenderSet {
    pub combined: Vec<f32>,
    pub body_only: Vec<f32>,
    pub policy: F4BodySustainPolicy,
}

pub fn render_f4_source_native_body_sustain_v1(
    input: FrozenEventInput<'_>,
    lookbehind_frames: usize,
    quantization_lsb: f64,
) -> Result<F4RenderSet, PercussiveForceError> {
    let event = validate_frozen_event(input)?;
    if lookbehind_frames == 0 || event.region.onset_frame < lookbehind_frames {
        return Err(InvalidEventInput::InsufficientLookbehind {
            required_frames: lookbehind_frames.max(1),
            available_frames: event.region.onset_frame,
        }
        .into());
    }
    if quantization_lsb <= 0.0 || !quantization_lsb.is_finite() {
        return Err(InvalidEventInput::InvalidQuantizationLsb.into());
    }
    if BAND_EDGES_HZ[3] >= f64::from(event.sample_rate_hz) / 2.0 {
        return Err(PercussiveForceRefusal::InsufficientSpectralBins { usable_bins: 0 }.into());
    }

    let warmup_start = event.region.onset_frame - lookbehind_frames;
    let body_start_sample = event.region.attack_end_frame * event.channel_count;
    let body_end_sample = event.region.body_end_frame * event.channel_count;
    let source_body_energy = event.samples[body_start_sample..body_end_sample]
        .iter()
        .map(|sample| f64::from(*sample).powi(2))
        .sum::<f64>();
    if source_body_energy <= 0.0 || !source_body_energy.is_finite() {
        return Err(PercussiveForceRefusal::MissingBodyEnergy.into());
    }
    let bands = split_body_bands(
        event.samples,
        event.channel_count,
        event.sample_rate_hz,
        warmup_start,
    );
    let attack_weight: f64 = event.masks.attack.iter().sum();
    let body_weight: f64 = event.masks.body.iter().sum();
    let quantization_floor = (QUANTIZATION_LSB_MULTIPLIER * quantization_lsb).powi(2);

    let mut attack_mean_square = [0.0; 3];
    let mut body_mean_square = [0.0; 3];
    let mut lookbehind_mean_square = [0.0; 3];
    let mut trust_floor_mean_square = [0.0; 3];
    let mut trusted = [false; 3];
    let mut scores = [0.0; 3];
    for band_index in 0..3 {
        attack_mean_square[band_index] = weighted_mean_square(
            &bands[band_index],
            event.channel_count,
            &event.masks.attack,
            attack_weight,
        );
        body_mean_square[band_index] = weighted_mean_square(
            &bands[band_index],
            event.channel_count,
            &event.masks.body,
            body_weight,
        );
        lookbehind_mean_square[band_index] = mean_square(
            &bands[band_index],
            event.channel_count,
            warmup_start,
            event.region.onset_frame,
        );
        trust_floor_mean_square[band_index] = (LOOKBEHIND_NOISE_MULTIPLIER
            * lookbehind_mean_square[band_index])
            .max(quantization_floor);
        trusted[band_index] = attack_mean_square[band_index] > trust_floor_mean_square[band_index]
            && body_mean_square[band_index] > trust_floor_mean_square[band_index];
        if trusted[band_index] {
            scores[band_index] = body_mean_square[band_index] / trust_floor_mean_square[band_index];
        }
    }

    let selected_band_index = scores
        .iter()
        .enumerate()
        .filter(|(_, score)| score.is_finite() && **score > 0.0)
        .max_by(|left, right| {
            left.1
                .partial_cmp(right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.0.cmp(&left.0))
        })
        .map(|(index, _)| index)
        .ok_or(PercussiveForceRefusal::InsufficientTrustedBands { trusted_bands: 0 })?;

    let envelope_frames = milliseconds_to_frames(BODY_ENVELOPE_MS, event.sample_rate_hz);
    let body_envelope = causal_rms_envelope(
        &bands[selected_band_index],
        event.channel_count,
        envelope_frames,
    );
    let selected_body_envelope_peak = body_envelope
        [event.region.attack_end_frame..event.region.body_end_frame]
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    if selected_body_envelope_peak <= 0.0 || !selected_body_envelope_peak.is_finite() {
        return Err(PercussiveForceRefusal::MissingBodyEnergy.into());
    }

    let body_len = event.region.body_end_frame - event.region.attack_end_frame;
    let body_entry_frames =
        milliseconds_to_frames(BODY_ENTRY_MS, event.sample_rate_hz).min((body_len / 4).max(1));
    let body_exit_frames =
        milliseconds_to_frames(BODY_EXIT_MS, event.sample_rate_hz).min((body_len / 4).max(1));
    let mut combined = event.samples.to_vec();
    let mut body_only = event.samples.to_vec();
    let mut maximum_resolved_additional_gain = 0.0_f64;
    let mut output_peak = 0.0_f64;
    for (frame, envelope) in body_envelope
        .iter()
        .enumerate()
        .take(event.region.body_end_frame)
        .skip(event.region.attack_end_frame)
    {
        let entry = smoothstep_fraction(frame - event.region.attack_end_frame, body_entry_frames);
        let remaining = event.region.body_end_frame - frame;
        let exit = smoothstep_fraction(remaining.saturating_sub(1), body_exit_frames);
        let normalized_envelope = (*envelope / selected_body_envelope_peak).clamp(0.0, 1.0);
        let decay_deficit = (1.0 - normalized_envelope).sqrt();
        let additional_gain = MAXIMUM_ADDITIONAL_BAND_GAIN
            * entry
            * exit
            * (MINIMUM_ADDITIONAL_GAIN_FRACTION
                + (1.0 - MINIMUM_ADDITIONAL_GAIN_FRACTION) * decay_deficit);
        maximum_resolved_additional_gain = maximum_resolved_additional_gain.max(additional_gain);
        for channel in 0..event.channel_count {
            let sample_index = frame * event.channel_count + channel;
            let dry = f64::from(event.samples[sample_index]);
            let body_delta = additional_gain * bands[selected_band_index][sample_index];
            let output = checked_output_sample(dry + body_delta, sample_index)?;
            output_peak = output_peak.max(f64::from(output).abs());
            combined[sample_index] = output;
            body_only[sample_index] = output;
        }
    }
    if output_peak >= OUTPUT_PEAK_STRICT_MAXIMUM {
        return Err(PercussiveForceRefusal::OutputPeakWithoutHeadroom {
            peak: output_peak,
            strict_maximum: OUTPUT_PEAK_STRICT_MAXIMUM,
        }
        .into());
    }

    let candidate_body_energy = combined[body_start_sample..body_end_sample]
        .iter()
        .map(|sample| f64::from(*sample).powi(2))
        .sum::<f64>();

    Ok(F4RenderSet {
        combined,
        body_only,
        policy: F4BodySustainPolicy {
            version_id: F4_SOURCE_NATIVE_BODY_SUSTAIN_V1,
            region: event.region,
            band_edges_hz: BAND_EDGES_HZ,
            selected_band: match selected_band_index {
                0 => F4BodyBand::Low,
                1 => F4BodyBand::LowMid,
                _ => F4BodyBand::MidBody,
            },
            selected_band_index,
            band_attack_mean_square: attack_mean_square,
            band_body_mean_square: body_mean_square,
            band_lookbehind_mean_square: lookbehind_mean_square,
            band_trust_floor_mean_square: trust_floor_mean_square,
            trusted_bands: trusted,
            selected_band_score: scores[selected_band_index],
            lookbehind_frames,
            body_envelope_frames: envelope_frames,
            body_entry_frames,
            body_exit_frames,
            maximum_additional_band_gain: MAXIMUM_ADDITIONAL_BAND_GAIN,
            minimum_additional_gain_fraction: MINIMUM_ADDITIONAL_GAIN_FRACTION,
            selected_body_envelope_peak,
            maximum_resolved_additional_gain,
            source_body_energy,
            candidate_body_energy,
            body_energy_ratio: candidate_body_energy / source_body_energy,
            output_peak,
            attack_bit_identical: true,
            playback_rate_numerator: 1,
            playback_rate_denominator: 1,
        },
    })
}

fn split_body_bands(
    samples: &[f32],
    channels: usize,
    sample_rate_hz: u32,
    warmup_start: usize,
) -> [Vec<f64>; 3] {
    let coefficients = BAND_EDGES_HZ.map(|edge| (-TAU * edge / f64::from(sample_rate_hz)).exp());
    let mut bands = [
        vec![0.0; samples.len()],
        vec![0.0; samples.len()],
        vec![0.0; samples.len()],
    ];
    let mut highpass_lowpass_states = vec![[0.0_f64; 3]; channels];
    let mut band_lowpass_states = vec![[0.0_f64; 3]; channels];
    for frame in warmup_start..samples.len() / channels {
        for channel in 0..channels {
            let index = frame * channels + channel;
            let sample = f64::from(samples[index]);
            for band in 0..3 {
                highpass_lowpass_states[channel][band] = coefficients[band]
                    * highpass_lowpass_states[channel][band]
                    + (1.0 - coefficients[band]) * sample;
                let highpassed = sample - highpass_lowpass_states[channel][band];
                band_lowpass_states[channel][band] = coefficients[band + 1]
                    * band_lowpass_states[channel][band]
                    + (1.0 - coefficients[band + 1]) * highpassed;
                bands[band][index] = band_lowpass_states[channel][band];
            }
        }
    }
    bands
}

fn weighted_mean_square(samples: &[f64], channels: usize, weights: &[f64], weight_sum: f64) -> f64 {
    let energy = weights
        .iter()
        .enumerate()
        .map(|(frame, weight)| {
            samples[frame * channels..(frame + 1) * channels]
                .iter()
                .map(|sample| sample * sample * weight)
                .sum::<f64>()
        })
        .sum::<f64>();
    energy / (channels as f64 * weight_sum)
}

fn mean_square(samples: &[f64], channels: usize, start: usize, end: usize) -> f64 {
    samples[start * channels..end * channels]
        .iter()
        .map(|sample| sample * sample)
        .sum::<f64>()
        / ((end - start) * channels) as f64
}

fn causal_rms_envelope(samples: &[f64], channels: usize, window_frames: usize) -> Vec<f64> {
    let frame_count = samples.len() / channels;
    let mut per_frame = vec![0.0; frame_count];
    for (frame, energy) in per_frame.iter_mut().enumerate() {
        *energy = samples[frame * channels..(frame + 1) * channels]
            .iter()
            .map(|sample| sample * sample)
            .sum::<f64>()
            / channels as f64;
    }
    let mut output = vec![0.0; frame_count];
    let mut sum = 0.0;
    for frame in 0..frame_count {
        sum += per_frame[frame];
        if frame >= window_frames {
            sum -= per_frame[frame - window_frames];
        }
        let count = (frame + 1).min(window_frames);
        output[frame] = (sum / count as f64).sqrt();
    }
    output
}

fn milliseconds_to_frames(milliseconds: f64, sample_rate_hz: u32) -> usize {
    ((milliseconds * f64::from(sample_rate_hz) / 1_000.0) + 0.5)
        .floor()
        .max(1.0) as usize
}

fn smoothstep_fraction(position: usize, length: usize) -> f64 {
    let value = (position as f64 / length.max(1) as f64).clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(body_hz: f64, sample_rate_hz: u32, amplitude: f64) -> Vec<f32> {
        let frames = sample_rate_hz as usize / 5;
        let onset = sample_rate_hz as usize / 20;
        let attack_end = onset + sample_rate_hz as usize / 200;
        let body_end = onset + sample_rate_hz as usize / 10;
        let mut samples = vec![0.0_f32; frames * 2];
        for frame in onset..body_end {
            let time = (frame - onset) as f64 / f64::from(sample_rate_hz);
            let envelope = if frame < attack_end {
                (frame - onset + 1) as f64 / (attack_end - onset) as f64
            } else {
                (-(time - 0.005) * 28.0).exp()
            };
            let value = amplitude * envelope * (TAU * body_hz * time).sin();
            samples[frame * 2] = value as f32;
            samples[frame * 2 + 1] = (-0.83 * value) as f32;
        }
        samples
    }

    fn input(samples: &[f32], sample_rate_hz: u32) -> FrozenEventInput<'_> {
        let onset = sample_rate_hz as usize / 20;
        FrozenEventInput {
            interleaved_samples: samples,
            sample_rate_hz,
            channel_count: 2,
            region: FrozenEventRegion {
                onset_frame: onset,
                attack_end_frame: onset + sample_rate_hz as usize / 200,
                body_end_frame: onset + sample_rate_hz as usize / 10,
            },
        }
    }

    #[test]
    fn preserves_attack_and_outside_region_while_lifting_decay() {
        let rate = 48_000;
        let samples = fixture(330.0, rate, 0.35);
        let rendered = render_f4_source_native_body_sustain_v1(
            input(&samples, rate),
            rate as usize * 20 / 1_000,
            2.0_f64.powi(-15),
        )
        .unwrap();
        let region = rendered.policy.region;
        assert_eq!(
            &rendered.combined[..region.attack_end_frame * 2],
            &samples[..region.attack_end_frame * 2]
        );
        assert_eq!(
            &rendered.combined[region.body_end_frame * 2..],
            &samples[region.body_end_frame * 2..]
        );
        assert_ne!(
            &rendered.combined[region.attack_end_frame * 2..region.body_end_frame * 2],
            &samples[region.attack_end_frame * 2..region.body_end_frame * 2]
        );
        assert!(rendered.policy.body_energy_ratio > 1.0);
        assert!(rendered.policy.body_energy_ratio < 2.0);
        assert!(rendered.policy.maximum_resolved_additional_gain <= 0.5);
        assert_eq!(rendered.policy.playback_rate_numerator, 1);
        assert_eq!(rendered.policy.playback_rate_denominator, 1);
    }

    #[test]
    fn source_frequency_changes_the_selected_body_band() {
        let rate = 48_000;
        let low = fixture(110.0, rate, 0.3);
        let middle = fixture(330.0, rate, 0.3);
        let upper = fixture(820.0, rate, 0.3);
        let render = |samples: &[f32]| {
            render_f4_source_native_body_sustain_v1(
                input(samples, rate),
                rate as usize * 20 / 1_000,
                2.0_f64.powi(-15),
            )
            .unwrap()
            .policy
            .selected_band
        };
        assert_eq!(render(&low), F4BodyBand::Low);
        assert_eq!(render(&middle), F4BodyBand::LowMid);
        assert_eq!(render(&upper), F4BodyBand::MidBody);
    }

    #[test]
    fn deterministic_across_repeats_and_supported_rates() {
        for rate in [44_100, 48_000, 96_000] {
            let samples = fixture(330.0, rate, 0.3);
            let render = || {
                render_f4_source_native_body_sustain_v1(
                    input(&samples, rate),
                    rate as usize * 20 / 1_000,
                    2.0_f64.powi(-15),
                )
                .unwrap()
            };
            assert_eq!(render(), render());
        }
    }

    #[test]
    fn refuses_missing_event_coupled_body() {
        let rate = 48_000;
        let mut samples = fixture(330.0, rate, 0.3);
        let region = input(&samples, rate).region;
        samples[region.attack_end_frame * 2..region.body_end_frame * 2].fill(0.0);
        assert!(matches!(
            render_f4_source_native_body_sustain_v1(
                input(&samples, rate),
                rate as usize * 20 / 1_000,
                2.0_f64.powi(-15),
            ),
            Err(PercussiveForceError::Refused(
                PercussiveForceRefusal::MissingBodyEnergy
            ))
        ));
    }

    #[test]
    fn refuses_output_without_headroom_instead_of_limiting() {
        let rate = 48_000;
        let samples = fixture(330.0, rate, 0.92);
        assert!(matches!(
            render_f4_source_native_body_sustain_v1(
                input(&samples, rate),
                rate as usize * 20 / 1_000,
                2.0_f64.powi(-15),
            ),
            Err(PercussiveForceError::Refused(
                PercussiveForceRefusal::OutputPeakWithoutHeadroom { .. }
            ))
        ));
    }
}
