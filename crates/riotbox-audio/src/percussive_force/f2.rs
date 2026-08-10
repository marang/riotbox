use std::f64::consts::TAU;

use super::common::{
    FrozenEventInput, FrozenEventRegion, PercussiveForceError, PercussiveForceRefusal,
    checked_output_sample, gain_multiplier, validate_frozen_event,
};

pub const F2_EXACT_COMPLEMENTARY_THREE_BAND_V1: &str = "f2_exact_complementary_three_band_v1";
const MINIMUM_TRUSTED_BANDS: usize = 2;
const MINIMUM_SPLIT_SEPARATION_BINS: usize = 2;
const LOOKBEHIND_NOISE_MULTIPLIER: f64 = 4.0;
const QUANTIZATION_LSB_MULTIPLIER: f64 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F2BandRole {
    Low,
    Mid,
    High,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F2BandPolicy {
    pub role: F2BandRole,
    pub attack_energy: f64,
    pub body_energy: f64,
    pub attack_mean_square: f64,
    pub body_mean_square: f64,
    pub lookbehind_mean_square: f64,
    pub trust_floor_mean_square: f64,
    pub trusted: bool,
    pub attack_share_of_trusted_bands: Option<f64>,
    pub attack_energy_multiplier: f64,
    pub attack_gain_squared: f64,
    pub body_gain_squared: f64,
    pub attack_gain: f64,
    pub body_gain: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F2ComplementaryPolicy {
    pub version_id: &'static str,
    pub dft_window: &'static str,
    pub dft_attack_frame_count: usize,
    pub usable_positive_spectral_bins: usize,
    pub f25_bin: usize,
    pub f75_bin: usize,
    pub minimum_split_separation_bins: usize,
    pub dft_bin_width_hz: f64,
    pub minimum_split_hz: f64,
    pub maximum_split_hz: f64,
    pub f25_hz: f64,
    pub f75_hz: f64,
    pub lowpass_f25_decay_coefficient: f64,
    pub lowpass_f25_feed_coefficient: f64,
    pub residual_lowpass_f75_decay_coefficient: f64,
    pub residual_lowpass_f75_feed_coefficient: f64,
    pub lookbehind_frames: usize,
    pub quantization_lsb: f64,
    pub lookbehind_noise_multiplier: f64,
    pub quantization_lsb_multiplier: f64,
    pub minimum_trusted_bands: usize,
    pub trusted_band_roles: Vec<F2BandRole>,
    pub trusted_attack_energy_sum: f64,
    pub bands: Vec<F2BandPolicy>,
    pub region: FrozenEventRegion,
    pub mask_definition: &'static str,
    pub attack_body_crossfade_frames: usize,
    pub attack_body_crossfade_start_frame: usize,
    pub attack_body_crossfade_end_frame: usize,
    pub body_fade_frames: usize,
    pub body_fade_start_frame: usize,
    pub mask_phase_denominator: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F2RenderSet {
    pub combined: Vec<f32>,
    pub attack_only: Vec<f32>,
    pub body_only: Vec<f32>,
    pub policy: F2ComplementaryPolicy,
}

pub fn render_f2_exact_complementary_three_band_v1(
    input: FrozenEventInput<'_>,
    lookbehind_frames: usize,
    quantization_lsb: f64,
) -> Result<F2RenderSet, PercussiveForceError> {
    let event = validate_frozen_event(input)?;
    if lookbehind_frames == 0 || event.region.onset_frame < lookbehind_frames {
        return Err(super::common::InvalidEventInput::InsufficientLookbehind {
            required_frames: lookbehind_frames.max(1),
            available_frames: event.region.onset_frame,
        }
        .into());
    }
    if quantization_lsb <= 0.0 || !quantization_lsb.is_finite() {
        return Err(super::common::InvalidEventInput::InvalidQuantizationLsb.into());
    }

    let spectral = resolve_attack_quantiles(
        event.samples,
        event.channel_count,
        event.sample_rate_hz,
        event.region.onset_frame,
        event.region.attack_end_frame,
    )?;
    let warmup_start = event.region.onset_frame - lookbehind_frames;
    let bank = split_exact_complementary_three_band(
        event.samples,
        event.channel_count,
        event.sample_rate_hz,
        spectral.f25_hz,
        spectral.f75_hz,
        warmup_start,
    );

    let mask_attack_sum: f64 = event.masks.attack.iter().sum();
    let mask_body_sum: f64 = event.masks.body.iter().sum();
    let quantization_floor = (QUANTIZATION_LSB_MULTIPLIER * quantization_lsb).powi(2);
    let roles = [F2BandRole::Low, F2BandRole::Mid, F2BandRole::High];
    let mut bands = Vec::with_capacity(3);
    for (role, samples) in roles.into_iter().zip(bank.bands()) {
        let attack_energy = weighted_band_energy(samples, event.channel_count, &event.masks.attack);
        let body_energy = weighted_band_energy(samples, event.channel_count, &event.masks.body);
        let lookbehind_mean_square = unweighted_mean_square(
            samples,
            event.channel_count,
            warmup_start,
            event.region.onset_frame,
        );
        let channel_count = event.channel_count as f64;
        let attack_mean_square = attack_energy / (channel_count * mask_attack_sum);
        let body_mean_square = body_energy / (channel_count * mask_body_sum);
        let trust_floor_mean_square =
            (LOOKBEHIND_NOISE_MULTIPLIER * lookbehind_mean_square).max(quantization_floor);
        let trusted = attack_mean_square > trust_floor_mean_square
            && body_mean_square > trust_floor_mean_square;
        bands.push(F2BandPolicy {
            role,
            attack_energy,
            body_energy,
            attack_mean_square,
            body_mean_square,
            lookbehind_mean_square,
            trust_floor_mean_square,
            trusted,
            attack_share_of_trusted_bands: None,
            attack_energy_multiplier: 1.0,
            attack_gain_squared: 1.0,
            body_gain_squared: 1.0,
            attack_gain: 1.0,
            body_gain: 1.0,
        });
    }
    if bands.iter().filter(|band| band.trusted).count() < MINIMUM_TRUSTED_BANDS {
        return Err(PercussiveForceRefusal::InsufficientTrustedBands {
            trusted_bands: bands.iter().filter(|band| band.trusted).count(),
        }
        .into());
    }
    let trusted_attack_energy_sum: f64 = bands
        .iter()
        .filter(|band| band.trusted)
        .map(|band| band.attack_energy)
        .sum();
    if trusted_attack_energy_sum == 0.0 || !trusted_attack_energy_sum.is_finite() {
        return Err(PercussiveForceRefusal::MissingTrustedAttackEnergy.into());
    }
    for band in &mut bands {
        if !band.trusted {
            continue;
        }
        let attack_share = band.attack_energy / trusted_attack_energy_sum;
        let multiplier = 2.0 - attack_share;
        let denominator = multiplier * band.attack_energy + band.body_energy;
        if denominator == 0.0 || !denominator.is_finite() {
            return Err(PercussiveForceRefusal::NonFiniteAnalysis {
                stage: "f2_band_conservation_denominator",
            }
            .into());
        }
        let total = band.attack_energy + band.body_energy;
        band.attack_share_of_trusted_bands = Some(attack_share);
        band.attack_energy_multiplier = multiplier;
        band.attack_gain_squared = multiplier * total / denominator;
        band.body_gain_squared = total / denominator;
        band.attack_gain = band.attack_gain_squared.sqrt();
        band.body_gain = band.body_gain_squared.sqrt();
    }

    let mut combined = event.samples.to_vec();
    let mut attack_only = event.samples.to_vec();
    let mut body_only = event.samples.to_vec();
    let bank_bands = bank.bands();
    // The bank is warmed from the frozen lookbehind, but only the affected
    // event may replace source samples. Qualification requires an exact source
    // copy before the physical onset and from body end onward.
    for frame in event.region.onset_frame..event.region.body_end_frame {
        let w_a = event.masks.attack[frame];
        let w_b = event.masks.body[frame];
        for channel in 0..event.channel_count {
            let sample_index = frame * event.channel_count + channel;
            let mut combined_sample = 0.0;
            let mut attack_sample = 0.0;
            let mut body_sample = 0.0;
            for (band_samples, band_policy) in bank_bands.iter().zip(&bands) {
                let sample = band_samples[sample_index];
                combined_sample += sample
                    * gain_multiplier(
                        w_a,
                        w_b,
                        band_policy.attack_gain_squared,
                        band_policy.body_gain_squared,
                    )?;
                attack_sample +=
                    sample * gain_multiplier(w_a, w_b, band_policy.attack_gain_squared, 1.0)?;
                body_sample +=
                    sample * gain_multiplier(w_a, w_b, 1.0, band_policy.body_gain_squared)?;
            }
            combined[sample_index] = checked_output_sample(combined_sample, sample_index)?;
            attack_only[sample_index] = checked_output_sample(attack_sample, sample_index)?;
            body_only[sample_index] = checked_output_sample(body_sample, sample_index)?;
        }
    }

    let trusted_band_roles = bands
        .iter()
        .filter(|band| band.trusted)
        .map(|band| band.role)
        .collect();
    Ok(F2RenderSet {
        combined,
        attack_only,
        body_only,
        policy: F2ComplementaryPolicy {
            version_id: F2_EXACT_COMPLEMENTARY_THREE_BAND_V1,
            dft_window: "periodic_hann_v1",
            dft_attack_frame_count: spectral.attack_frame_count,
            usable_positive_spectral_bins: spectral.usable_positive_bins,
            f25_bin: spectral.f25_bin,
            f75_bin: spectral.f75_bin,
            minimum_split_separation_bins: MINIMUM_SPLIT_SEPARATION_BINS,
            dft_bin_width_hz: spectral.bin_width_hz,
            minimum_split_hz: spectral.minimum_split_hz,
            maximum_split_hz: spectral.maximum_split_hz,
            f25_hz: spectral.f25_hz,
            f75_hz: spectral.f75_hz,
            lowpass_f25_decay_coefficient: one_pole_decay(spectral.f25_hz, event.sample_rate_hz),
            lowpass_f25_feed_coefficient: one_pole_feed(spectral.f25_hz, event.sample_rate_hz),
            residual_lowpass_f75_decay_coefficient: one_pole_decay(
                spectral.f75_hz,
                event.sample_rate_hz,
            ),
            residual_lowpass_f75_feed_coefficient: one_pole_feed(
                spectral.f75_hz,
                event.sample_rate_hz,
            ),
            lookbehind_frames,
            quantization_lsb,
            lookbehind_noise_multiplier: LOOKBEHIND_NOISE_MULTIPLIER,
            quantization_lsb_multiplier: QUANTIZATION_LSB_MULTIPLIER,
            minimum_trusted_bands: MINIMUM_TRUSTED_BANDS,
            trusted_band_roles,
            trusted_attack_energy_sum,
            bands,
            region: event.region,
            mask_definition: "centered_cos_squared_sin_squared_v1",
            attack_body_crossfade_frames: event.masks.attack_body_crossfade_frames,
            attack_body_crossfade_start_frame: event.masks.attack_body_crossfade_start_frame,
            attack_body_crossfade_end_frame: event.masks.attack_body_crossfade_end_frame,
            body_fade_frames: event.masks.body_fade_frames,
            body_fade_start_frame: event.masks.body_fade_start_frame,
            mask_phase_denominator: event.masks.phase_denominator,
        },
    })
}

#[derive(Debug)]
struct SpectralQuantiles {
    attack_frame_count: usize,
    usable_positive_bins: usize,
    f25_bin: usize,
    f75_bin: usize,
    bin_width_hz: f64,
    minimum_split_hz: f64,
    maximum_split_hz: f64,
    f25_hz: f64,
    f75_hz: f64,
}

fn resolve_attack_quantiles(
    samples: &[f32],
    channel_count: usize,
    sample_rate_hz: u32,
    onset_frame: usize,
    attack_end_frame: usize,
) -> Result<SpectralQuantiles, PercussiveForceError> {
    let frame_count = attack_end_frame - onset_frame;
    if frame_count < 4 {
        return Err(PercussiveForceRefusal::InsufficientSpectralBins { usable_bins: 0 }.into());
    }
    let last_bin_exclusive = frame_count.div_ceil(2);
    let mut bins = Vec::with_capacity(last_bin_exclusive.saturating_sub(1));
    for bin in 1..last_bin_exclusive {
        let mut channel_summed_energy = 0.0;
        for channel in 0..channel_count {
            let mut real = 0.0;
            let mut imaginary = 0.0;
            for attack_frame in 0..frame_count {
                let hann = 0.5 - 0.5 * (TAU * attack_frame as f64 / frame_count as f64).cos();
                let phase = TAU * bin as f64 * attack_frame as f64 / frame_count as f64;
                let sample_index = (onset_frame + attack_frame) * channel_count + channel;
                let sample = f64::from(samples[sample_index]) * hann;
                real += sample * phase.cos();
                imaginary -= sample * phase.sin();
            }
            channel_summed_energy += real * real + imaginary * imaginary;
        }
        // Included positive-frequency bins remain part of the frozen DFT even
        // when their power is exactly zero. Removing them would turn
        // `usable_positive_spectral_bins` into a signal-dependent count and
        // would no longer match the preregistered cumulative-bin definition.
        bins.push((bin, channel_summed_energy));
    }
    if bins.len() < 2 {
        return Err(PercussiveForceRefusal::InsufficientSpectralBins {
            usable_bins: bins.len(),
        }
        .into());
    }
    let total_energy: f64 = bins.iter().map(|(_, energy)| energy).sum();
    if total_energy == 0.0 || !total_energy.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f2_attack_dft_energy",
        }
        .into());
    }
    let quantile_bin = |quantile: f64| {
        let target = total_energy * quantile;
        let mut cumulative = 0.0;
        for (bin, energy) in &bins {
            cumulative += energy;
            if cumulative >= target {
                return *bin;
            }
        }
        bins.last().expect("non-empty bins").0
    };
    let f25_bin = quantile_bin(0.25);
    let f75_bin = quantile_bin(0.75);
    if f75_bin.saturating_sub(f25_bin) < MINIMUM_SPLIT_SEPARATION_BINS {
        return Err(PercussiveForceRefusal::InsufficientSpectralBinSeparation {
            f25_bin,
            f75_bin,
            minimum_separation_bins: MINIMUM_SPLIT_SEPARATION_BINS,
        }
        .into());
    }
    let bin_width_hz = f64::from(sample_rate_hz) / frame_count as f64;
    let f25_hz = f25_bin as f64 * bin_width_hz;
    let f75_hz = f75_bin as f64 * bin_width_hz;
    let nyquist_hz = f64::from(sample_rate_hz) / 2.0;
    let minimum_split_hz = bin_width_hz;
    let maximum_split_hz = nyquist_hz - bin_width_hz;
    if !(minimum_split_hz <= f25_hz && f25_hz < f75_hz && f75_hz <= maximum_split_hz) {
        return Err(PercussiveForceRefusal::InvalidSpectralQuantiles {
            f25_hz,
            f75_hz,
            minimum_split_hz,
            maximum_split_hz,
            nyquist_hz,
        }
        .into());
    }
    Ok(SpectralQuantiles {
        attack_frame_count: frame_count,
        usable_positive_bins: bins.len(),
        f25_bin,
        f75_bin,
        bin_width_hz,
        minimum_split_hz,
        maximum_split_hz,
        f25_hz,
        f75_hz,
    })
}

struct ComplementaryBank {
    low: Vec<f64>,
    mid: Vec<f64>,
    high: Vec<f64>,
}

impl ComplementaryBank {
    fn bands(&self) -> [&[f64]; 3] {
        [&self.low, &self.mid, &self.high]
    }
}

fn split_exact_complementary_three_band(
    samples: &[f32],
    channel_count: usize,
    sample_rate_hz: u32,
    f25_hz: f64,
    f75_hz: f64,
    warmup_start_frame: usize,
) -> ComplementaryBank {
    let mut low = vec![0.0; samples.len()];
    let mut mid = vec![0.0; samples.len()];
    let mut high = vec![0.0; samples.len()];
    let feed_low = one_pole_feed(f25_hz, sample_rate_hz);
    let feed_mid = one_pole_feed(f75_hz, sample_rate_hz);
    let mut low_state = vec![0.0; channel_count];
    let mut mid_state = vec![0.0; channel_count];
    for frame in warmup_start_frame..samples.len() / channel_count {
        for channel in 0..channel_count {
            let index = frame * channel_count + channel;
            let source = f64::from(samples[index]);
            low_state[channel] += feed_low * (source - low_state[channel]);
            let residual = source - low_state[channel];
            mid_state[channel] += feed_mid * (residual - mid_state[channel]);
            low[index] = low_state[channel];
            mid[index] = mid_state[channel];
            high[index] = source - low[index] - mid[index];
        }
    }
    ComplementaryBank { low, mid, high }
}

fn one_pole_decay(cutoff_hz: f64, sample_rate_hz: u32) -> f64 {
    (-TAU * cutoff_hz / f64::from(sample_rate_hz)).exp()
}

fn one_pole_feed(cutoff_hz: f64, sample_rate_hz: u32) -> f64 {
    1.0 - one_pole_decay(cutoff_hz, sample_rate_hz)
}

fn weighted_band_energy(samples: &[f64], channel_count: usize, weights: &[f64]) -> f64 {
    let mut energy = 0.0;
    for (frame, frame_samples) in samples.chunks_exact(channel_count).enumerate() {
        for sample in frame_samples {
            energy += sample * sample * weights[frame];
        }
    }
    energy
}

fn unweighted_mean_square(
    samples: &[f64],
    channel_count: usize,
    start_frame: usize,
    end_frame: usize,
) -> f64 {
    let mut energy = 0.0;
    for frame in start_frame..end_frame {
        for channel in 0..channel_count {
            let sample = samples[frame * channel_count + channel];
            energy += sample * sample;
        }
    }
    energy / ((end_frame - start_frame) * channel_count) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::percussive_force::FrozenEventRegion;

    fn deterministic_noise(state: &mut u64) -> f32 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (((*state >> 32) as u32 as f64 / u32::MAX as f64) * 2.0 - 1.0) as f32
    }

    #[test]
    fn residual_bank_reconstructs_synthetic_signals_at_all_registered_rates() {
        for sample_rate_hz in [44_100, 48_000, 96_000] {
            let frame_count = 1_024;
            let mut impulse = vec![0.0; frame_count];
            impulse[256] = 0.8;
            let dc = vec![0.21; frame_count];
            let mut state = 7_u64;
            let noise = (0..frame_count)
                .map(|_| deterministic_noise(&mut state) * 0.35)
                .collect::<Vec<_>>();
            let bounded = (0..frame_count)
                .map(|frame| {
                    (0.31 * (TAU * 430.0 * frame as f64 / sample_rate_hz as f64).sin()
                        + 0.17 * (TAU * 3_700.0 * frame as f64 / sample_rate_hz as f64).sin())
                        as f32
                })
                .collect::<Vec<_>>();

            for signal in [&impulse, &dc, &noise, &bounded] {
                let bank = split_exact_complementary_three_band(
                    signal,
                    1,
                    sample_rate_hz,
                    900.0,
                    5_500.0,
                    0,
                );
                let mut squared_error = 0.0;
                let mut source_energy = 0.0;
                let mut max_error = 0.0_f64;
                let mut peak = 0.0_f64;
                for (index, source) in signal.iter().enumerate() {
                    let reconstructed = bank.low[index] + bank.mid[index] + bank.high[index];
                    let error = reconstructed - f64::from(*source);
                    squared_error += error * error;
                    source_energy += f64::from(*source).powi(2);
                    max_error = max_error.max(error.abs());
                    peak = peak.max(f64::from(*source).abs());
                }
                let normalized_rms = (squared_error / source_energy).sqrt();
                assert!(normalized_rms <= 1.0e-6, "rate={sample_rate_hz}");
                assert!(
                    max_error <= 64.0 * f64::from(f32::EPSILON) * 1.0_f64.max(peak),
                    "rate={sample_rate_hz} max_error={max_error}"
                );
            }
        }
    }

    #[test]
    fn policy_records_trusted_band_shares_and_conserving_mappings() {
        let sample_rate_hz = 48_000;
        let frame_count = 1_280;
        let mut samples = vec![0.0_f32; frame_count];
        for (frame, sample) in samples.iter_mut().enumerate().take(1_024).skip(256) {
            let relative = frame - 256;
            let envelope = if relative < 256 { 0.7 } else { 0.35 };
            *sample = (envelope
                * (0.52 * (TAU * 320.0 * frame as f64 / sample_rate_hz as f64).sin()
                    + 0.38 * (TAU * 2_300.0 * frame as f64 / sample_rate_hz as f64).sin()
                    + 0.31 * (TAU * 8_400.0 * frame as f64 / sample_rate_hz as f64).sin()))
                as f32;
        }
        let rendered = render_f2_exact_complementary_three_band_v1(
            FrozenEventInput {
                interleaved_samples: &samples,
                sample_rate_hz,
                channel_count: 1,
                region: FrozenEventRegion {
                    onset_frame: 256,
                    attack_end_frame: 512,
                    body_end_frame: 1_024,
                },
            },
            128,
            1.0 / 32_768.0,
        )
        .expect("three-band synthetic event should qualify");

        assert!(rendered.policy.trusted_band_roles.len() >= 2);
        assert_eq!(rendered.policy.dft_window, "periodic_hann_v1");
        assert!(
            rendered.policy.f75_bin - rendered.policy.f25_bin
                >= rendered.policy.minimum_split_separation_bins
        );
        let share_sum: f64 = rendered
            .policy
            .bands
            .iter()
            .filter_map(|band| band.attack_share_of_trusted_bands)
            .sum();
        assert!((share_sum - 1.0).abs() <= 1.0e-12);
        for band in rendered.policy.bands.iter().filter(|band| band.trusted) {
            let before = band.attack_energy + band.body_energy;
            let after = band.attack_gain_squared * band.attack_energy
                + band.body_gain_squared * band.body_energy;
            assert!((after - before).abs() / before <= 1.0e-12);
            assert_eq!(
                band.attack_energy_multiplier,
                2.0 - band.attack_share_of_trusted_bands.unwrap()
            );
        }
    }

    #[test]
    fn lookbehind_warms_filter_state_without_replacing_samples_outside_event() {
        let sample_rate_hz = 48_000;
        let frame_count = 1_280;
        let mut samples = (0..frame_count)
            .map(|frame| {
                let value = ((frame as f64 * 0.137).sin() * 0.000_001) as f32;
                if value == 0.0 {
                    f32::MIN_POSITIVE
                } else {
                    value
                }
            })
            .collect::<Vec<_>>();
        for (frame, sample) in samples.iter_mut().enumerate().take(1_024).skip(256) {
            let relative = frame - 256;
            let envelope = if relative < 256 { 0.7 } else { 0.35 };
            *sample = (envelope
                * (0.52 * (TAU * 320.0 * frame as f64 / sample_rate_hz as f64).sin()
                    + 0.38 * (TAU * 2_300.0 * frame as f64 / sample_rate_hz as f64).sin()
                    + 0.31 * (TAU * 8_400.0 * frame as f64 / sample_rate_hz as f64).sin()))
                as f32;
        }

        let rendered = render_f2_exact_complementary_three_band_v1(
            FrozenEventInput {
                interleaved_samples: &samples,
                sample_rate_hz,
                channel_count: 1,
                region: FrozenEventRegion {
                    onset_frame: 256,
                    attack_end_frame: 512,
                    body_end_frame: 1_024,
                },
            },
            128,
            1.0 / 32_768.0,
        )
        .expect("three-band synthetic event should qualify");

        for output in [
            &rendered.combined,
            &rendered.attack_only,
            &rendered.body_only,
        ] {
            assert_eq!(&output[..256], &samples[..256]);
            assert_eq!(&output[1_024..], &samples[1_024..]);
        }
    }

    #[test]
    fn adjacent_or_collapsed_quartile_splits_are_typed_refusals() {
        let frame_count = 64;
        let samples = (0..frame_count)
            .map(|frame| (TAU * 8.0 * frame as f64 / frame_count as f64).sin() as f32)
            .collect::<Vec<_>>();
        let error = resolve_attack_quantiles(&samples, 1, 48_000, 0, frame_count)
            .expect_err("one narrow bin must not qualify a multiband split");
        assert!(matches!(
            error,
            PercussiveForceError::Refused(
                PercussiveForceRefusal::InsufficientSpectralBinSeparation { .. }
            )
        ));
    }

    #[test]
    fn periodic_hann_quantiles_match_the_frozen_two_tone_golden_case() {
        // Frozen source-independent passport:
        // x[n] = 0.25*cos(2*pi*8*n/128) + 0.25*cos(2*pi*24*n/128),
        // mono at 48 kHz. The periodic Hann produces separated 1:4:1 power
        // lobes, so the first 25% and 75% cumulative-power crossings are the
        // two center bins exactly.
        const FRAME_COUNT: usize = 128;
        const SAMPLE_RATE_HZ: u32 = 48_000;
        let samples = (0..FRAME_COUNT)
            .map(|frame| {
                let phase = TAU * frame as f64 / FRAME_COUNT as f64;
                (0.25 * (8.0 * phase).cos() + 0.25 * (24.0 * phase).cos()) as f32
            })
            .collect::<Vec<_>>();

        let spectral =
            resolve_attack_quantiles(&samples, 1, SAMPLE_RATE_HZ, 0, FRAME_COUNT).unwrap();

        assert_eq!(spectral.f25_bin, 8);
        assert_eq!(spectral.f75_bin, 24);
    }

    #[test]
    fn weighted_band_energy_is_an_all_channel_sum() {
        let mono = [0.25_f64, -0.5, 0.75, -0.125];
        let stereo = mono
            .iter()
            .flat_map(|sample| [*sample, *sample])
            .collect::<Vec<_>>();
        let weights = [1.0, 0.75, 0.5, 0.25];

        let mono_energy = weighted_band_energy(&mono, 1, &weights);
        let stereo_energy = weighted_band_energy(&stereo, 2, &weights);

        assert!((stereo_energy - 2.0 * mono_energy).abs() <= f64::EPSILON);
    }
}
