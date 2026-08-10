use std::f64::consts::{PI, TAU};

use super::common::{
    FrozenEventInput, FrozenEventRegion, InvalidEventInput, PercussiveForceError,
    PercussiveForceRefusal, checked_output_sample, effective_region_energy, validate_frozen_event,
    weighted_branch_rms, weighted_rms,
};

pub const F3_OS4_ONSET_RESIDUAL_V1: &str = "f3_os4_onset_residual_v1";
pub const F3_ORACLE_OS8_V1: &str = "f3_oracle_os8_v1";

const OVERSAMPLING_FACTOR: usize = 4;
const OVERSAMPLING_TAPS: usize = 257;
const OVERSAMPLING_CUTOFF: f64 = 0.45 / OVERSAMPLING_FACTOR as f64;
const ORACLE_FACTOR: usize = 8;
const ORACLE_TAPS: usize = 513;
const ORACLE_CUTOFF: f64 = 0.45 / ORACLE_FACTOR as f64;
const REQUIRED_PADDING_FRAMES: usize = 64;
const RESIDUAL_SCALE_CAP: f64 = 8.0;

#[derive(Clone, Debug, PartialEq)]
pub struct ResamplerPolicy {
    pub version_id: &'static str,
    pub factor: usize,
    pub tap_count: usize,
    pub cutoff_cycles_per_high_rate_sample: f64,
    pub window: &'static str,
    pub down_filter_normalization: &'static str,
    pub up_filter_scale: f64,
    pub one_way_group_delay_high_rate_frames: usize,
    pub round_trip_group_delay_base_frames: usize,
    pub h_down: Vec<f64>,
    pub h_up: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F3OnsetResidualPolicy {
    pub version_id: &'static str,
    pub resampler: ResamplerPolicy,
    pub required_lookbehind_frames: usize,
    pub required_tail_padding_frames: usize,
    pub actual_lookbehind_frames: usize,
    pub actual_tail_padding_frames: usize,
    pub attack_energy: f64,
    pub body_energy: f64,
    pub body_energy_share_q: f64,
    pub nonlinear_drive_d: f64,
    pub phi_definition: &'static str,
    pub interpolated_peak_p4: f64,
    pub attack_dry_rms: f64,
    pub body_dry_rms: f64,
    pub unscaled_attack_branch_rms: f64,
    pub unscaled_body_branch_rms: f64,
    pub attack_target_ratio: f64,
    pub body_target_ratio: f64,
    pub attack_residual_scale: f64,
    pub body_residual_scale: f64,
    pub residual_scale_cap: f64,
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
pub struct F3RenderSet {
    pub combined: Vec<f32>,
    pub attack_only: Vec<f32>,
    pub body_only: Vec<f32>,
    pub policy: F3OnsetResidualPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SyntheticToneComponent {
    pub amplitude: f64,
    pub cycles_per_base_sample: f64,
    pub phase_radians: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResamplerPreflight {
    pub subject: ResamplerPolicy,
    pub oracle: ResamplerPolicy,
    pub response_grid_points: usize,
    pub dc_measurement: &'static str,
    pub dc_unity_tolerance: f64,
    pub measured_dc_unity_error: f64,
    pub dc_unity_pass: bool,
    pub passband_end_base_cycles_per_sample: f64,
    pub passband_measurement: &'static str,
    pub maximum_passband_deviation_db: f64,
    pub measured_passband_deviation_db: f64,
    pub passband_response_pass: bool,
    pub stopband_start_base_cycles_per_sample: f64,
    pub stopband_measurement: &'static str,
    pub minimum_stopband_attenuation_db: f64,
    pub measured_stopband_attenuation_db: f64,
    pub stopband_attenuation_pass: bool,
    pub expected_round_trip_delay_base_frames: usize,
    pub measured_round_trip_delay_base_frames: usize,
    pub aligned_impulse_peak_offset_frames: isize,
    pub impulse_alignment_pass: bool,
    pub maximum_residual_alias_db: f64,
    pub alias_probe_id: &'static str,
    pub alias_probe_frame_count: usize,
    pub alias_probe_comparison_margin_frames: usize,
    pub alias_probe_drive: f64,
    pub alias_probe_components: Vec<SyntheticToneComponent>,
    pub measured_residual_alias_db: f64,
    pub residual_alias_pass: bool,
    pub passed: bool,
}

pub fn render_f3_os4_onset_residual_v1(
    input: FrozenEventInput<'_>,
) -> Result<F3RenderSet, PercussiveForceError> {
    let event = validate_frozen_event(input)?;
    if event.region.onset_frame < REQUIRED_PADDING_FRAMES {
        return Err(InvalidEventInput::InsufficientLookbehind {
            required_frames: REQUIRED_PADDING_FRAMES,
            available_frames: event.region.onset_frame,
        }
        .into());
    }
    let tail_padding = event.frame_count - event.region.body_end_frame;
    if tail_padding < REQUIRED_PADDING_FRAMES {
        return Err(InvalidEventInput::InsufficientTailPadding {
            required_frames: REQUIRED_PADDING_FRAMES,
            available_frames: tail_padding,
        }
        .into());
    }

    let energy = effective_region_energy(event.samples, event.channel_count, &event.masks)?;
    if energy.attack == 0.0 {
        return Err(PercussiveForceRefusal::MissingAttackEnergy.into());
    }
    if energy.body == 0.0 {
        return Err(PercussiveForceRefusal::MissingBodyEnergy.into());
    }
    let total_energy = energy.attack + energy.body;
    if total_energy == 0.0 || !total_energy.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_total_region_energy",
        }
        .into());
    }
    let q = energy.body / total_energy;
    let drive = 1.0 + 2.0 * q;
    let resampler = resampler_policy(
        F3_OS4_ONSET_RESIDUAL_V1,
        OVERSAMPLING_FACTOR,
        OVERSAMPLING_TAPS,
        OVERSAMPLING_CUTOFF,
    );
    let source_f64 = event
        .samples
        .iter()
        .map(|sample| f64::from(*sample))
        .collect::<Vec<_>>();
    let (residual, interpolated_peak) =
        nonlinear_residual(&source_f64, event.channel_count, &resampler, drive)?;

    let attack_dry_rms = weighted_rms(&source_f64, event.channel_count, &event.masks.attack)?;
    let body_dry_rms = weighted_rms(&source_f64, event.channel_count, &event.masks.body)?;
    let attack_branch_rms =
        weighted_branch_rms(&residual, event.channel_count, &event.masks.attack)?;
    let body_branch_rms = weighted_branch_rms(&residual, event.channel_count, &event.masks.body)?;
    if attack_branch_rms == 0.0 {
        return Err(PercussiveForceRefusal::ZeroAttackResidual.into());
    }
    if body_branch_rms == 0.0 {
        return Err(PercussiveForceRefusal::ZeroBodyResidual.into());
    }
    let attack_target_ratio = 0.25 + q / 4.0;
    let body_target_ratio = attack_target_ratio / 2.0;
    let attack_scale = attack_target_ratio * attack_dry_rms / attack_branch_rms;
    let body_scale = body_target_ratio * body_dry_rms / body_branch_rms;
    if !attack_scale.is_finite() || !body_scale.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_residual_scale",
        }
        .into());
    }
    if attack_scale > RESIDUAL_SCALE_CAP || body_scale > RESIDUAL_SCALE_CAP {
        return Err(PercussiveForceRefusal::ResidualScaleExceedsCap {
            attack_scale,
            body_scale,
            cap: RESIDUAL_SCALE_CAP,
        }
        .into());
    }

    let mut combined = event.samples.to_vec();
    let mut attack_only = event.samples.to_vec();
    let mut body_only = event.samples.to_vec();
    for frame in event.region.onset_frame..event.region.body_end_frame {
        let w_a = event.masks.attack[frame];
        let w_b = event.masks.body[frame];
        for channel in 0..event.channel_count {
            let sample_index = frame * event.channel_count + channel;
            let dry = source_f64[sample_index];
            let attack_delta = w_a * attack_scale * residual[sample_index];
            let body_delta = w_b * body_scale * residual[sample_index];
            combined[sample_index] =
                checked_output_sample(dry + attack_delta + body_delta, sample_index)?;
            attack_only[sample_index] = checked_output_sample(dry + attack_delta, sample_index)?;
            body_only[sample_index] = checked_output_sample(dry + body_delta, sample_index)?;
        }
    }

    Ok(F3RenderSet {
        combined,
        attack_only,
        body_only,
        policy: F3OnsetResidualPolicy {
            version_id: F3_OS4_ONSET_RESIDUAL_V1,
            resampler,
            required_lookbehind_frames: REQUIRED_PADDING_FRAMES,
            required_tail_padding_frames: REQUIRED_PADDING_FRAMES,
            actual_lookbehind_frames: event.region.onset_frame,
            actual_tail_padding_frames: tail_padding,
            attack_energy: energy.attack,
            body_energy: energy.body,
            body_energy_share_q: q,
            nonlinear_drive_d: drive,
            phi_definition: "P4*tanh(d*z/P4)/tanh(d)",
            interpolated_peak_p4: interpolated_peak,
            attack_dry_rms,
            body_dry_rms,
            unscaled_attack_branch_rms: attack_branch_rms,
            unscaled_body_branch_rms: body_branch_rms,
            attack_target_ratio,
            body_target_ratio,
            attack_residual_scale: attack_scale,
            body_residual_scale: body_scale,
            residual_scale_cap: RESIDUAL_SCALE_CAP,
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

pub fn run_f3_resampler_preflight() -> ResamplerPreflight {
    const RESPONSE_GRID_POINTS: usize = 16_384;
    const DC_TOLERANCE: f64 = 1.0e-9;
    const PASSBAND_END: f64 = 0.35;
    const PASSBAND_DEVIATION_DB: f64 = 0.1;
    const STOPBAND_START: f64 = 0.55;
    const STOPBAND_ATTENUATION_DB: f64 = 60.0;
    const MAXIMUM_RESIDUAL_ALIAS_DB: f64 = -60.0;
    const ALIAS_PROBE_FRAME_COUNT: usize = 4_096;
    const ALIAS_PROBE_COMPARISON_MARGIN_FRAMES: usize = 512;
    const ALIAS_PROBE_DRIVE: f64 = 3.0;

    let subject = resampler_policy(
        F3_OS4_ONSET_RESIDUAL_V1,
        OVERSAMPLING_FACTOR,
        OVERSAMPLING_TAPS,
        OVERSAMPLING_CUTOFF,
    );
    let oracle = resampler_policy(F3_ORACLE_OS8_V1, ORACLE_FACTOR, ORACLE_TAPS, ORACLE_CUTOFF);

    let measured_dc_unity_error = (subject.h_down.iter().sum::<f64>() - 1.0).abs();
    let dc_unity_pass = measured_dc_unity_error <= DC_TOLERANCE;

    let mut measured_passband_deviation_db = 0.0_f64;
    for point in 0..=RESPONSE_GRID_POINTS {
        let base_frequency = PASSBAND_END * point as f64 / RESPONSE_GRID_POINTS as f64;
        let high_frequency = base_frequency / subject.factor as f64;
        let magnitude = fir_magnitude(&subject.h_down, high_frequency);
        let response_db = 20.0 * magnitude.log10();
        measured_passband_deviation_db = measured_passband_deviation_db.max(response_db.abs());
    }
    let passband_response_pass = measured_passband_deviation_db <= PASSBAND_DEVIATION_DB;

    let mut stopband_peak = 0.0_f64;
    for point in 0..=RESPONSE_GRID_POINTS {
        let high_frequency = STOPBAND_START / subject.factor as f64
            + (0.5 - STOPBAND_START / subject.factor as f64) * point as f64
                / RESPONSE_GRID_POINTS as f64;
        stopband_peak = stopband_peak.max(fir_magnitude(&subject.h_down, high_frequency));
    }
    let measured_stopband_attenuation_db = if stopband_peak == 0.0 {
        f64::INFINITY
    } else {
        -20.0 * stopband_peak.log10()
    };
    let stopband_attenuation_pass = measured_stopband_attenuation_db >= STOPBAND_ATTENUATION_DB;

    let mut impulse = vec![0.0; 512];
    let impulse_frame = 128;
    impulse[impulse_frame] = 1.0;
    let delayed = linear_round_trip_delayed(&impulse, 1, &subject);
    let delayed_peak = max_abs_index(&delayed);
    let measured_delay = delayed_peak.saturating_sub(impulse_frame);
    let aligned = linear_round_trip_aligned(&impulse, 1, &subject);
    let aligned_peak = max_abs_index(&aligned);
    let aligned_offset = aligned_peak as isize - impulse_frame as isize;
    let impulse_alignment_pass =
        measured_delay == subject.round_trip_group_delay_base_frames && aligned_offset == 0;

    let alias_probe_components = synthetic_alias_probe_components();
    let alias_source =
        synthetic_alias_probe_signal(ALIAS_PROBE_FRAME_COUNT, &alias_probe_components);
    let subject_residual =
        nonlinear_residual_unchecked(&alias_source, 1, &subject, ALIAS_PROBE_DRIVE);
    let oracle_residual =
        nonlinear_residual_unchecked(&alias_source, 1, &oracle, ALIAS_PROBE_DRIVE);
    let comparison_start = ALIAS_PROBE_COMPARISON_MARGIN_FRAMES;
    let comparison_end = alias_source.len() - ALIAS_PROBE_COMPARISON_MARGIN_FRAMES;
    let mut error_energy = 0.0;
    let mut oracle_energy = 0.0;
    for index in comparison_start..comparison_end {
        let error = subject_residual[index] - oracle_residual[index];
        error_energy += error * error;
        oracle_energy += oracle_residual[index] * oracle_residual[index];
    }
    let measured_residual_alias_db = if error_energy == 0.0 {
        f64::NEG_INFINITY
    } else if oracle_energy == 0.0 {
        f64::INFINITY
    } else {
        10.0 * (error_energy / oracle_energy).log10()
    };
    let residual_alias_pass = measured_residual_alias_db <= MAXIMUM_RESIDUAL_ALIAS_DB;
    let passed = dc_unity_pass
        && passband_response_pass
        && stopband_attenuation_pass
        && impulse_alignment_pass
        && residual_alias_pass;

    ResamplerPreflight {
        subject,
        oracle,
        response_grid_points: RESPONSE_GRID_POINTS,
        dc_measurement: "abs(sum(h_down)-1)",
        dc_unity_tolerance: DC_TOLERANCE,
        measured_dc_unity_error,
        dc_unity_pass,
        passband_end_base_cycles_per_sample: PASSBAND_END,
        passband_measurement: "max_abs_20log10_h_down_magnitude_on_uniform_grid",
        maximum_passband_deviation_db: PASSBAND_DEVIATION_DB,
        measured_passband_deviation_db,
        passband_response_pass,
        stopband_start_base_cycles_per_sample: STOPBAND_START,
        stopband_measurement: "negative_20log10_max_h_down_magnitude_on_uniform_grid",
        minimum_stopband_attenuation_db: STOPBAND_ATTENUATION_DB,
        measured_stopband_attenuation_db,
        stopband_attenuation_pass,
        expected_round_trip_delay_base_frames: REQUIRED_PADDING_FRAMES,
        measured_round_trip_delay_base_frames: measured_delay,
        aligned_impulse_peak_offset_frames: aligned_offset,
        impulse_alignment_pass,
        maximum_residual_alias_db: MAXIMUM_RESIDUAL_ALIAS_DB,
        alias_probe_id: "three_tone_0071_0193_0317_v1",
        alias_probe_frame_count: ALIAS_PROBE_FRAME_COUNT,
        alias_probe_comparison_margin_frames: ALIAS_PROBE_COMPARISON_MARGIN_FRAMES,
        alias_probe_drive: ALIAS_PROBE_DRIVE,
        alias_probe_components,
        measured_residual_alias_db,
        residual_alias_pass,
        passed,
    }
}

fn resampler_policy(
    version_id: &'static str,
    factor: usize,
    tap_count: usize,
    cutoff: f64,
) -> ResamplerPolicy {
    let h_down = blackman_windowed_sinc(tap_count, cutoff);
    let h_up = h_down
        .iter()
        .map(|coefficient| coefficient * factor as f64)
        .collect();
    let one_way_group_delay_high_rate_frames = (tap_count - 1) / 2;
    ResamplerPolicy {
        version_id,
        factor,
        tap_count,
        cutoff_cycles_per_high_rate_sample: cutoff,
        window: "symmetric_blackman_v1",
        down_filter_normalization: "coefficient_sum_equals_one",
        up_filter_scale: factor as f64,
        one_way_group_delay_high_rate_frames,
        round_trip_group_delay_base_frames: 2 * one_way_group_delay_high_rate_frames / factor,
        h_down,
        h_up,
    }
}

fn blackman_windowed_sinc(tap_count: usize, cutoff: f64) -> Vec<f64> {
    let midpoint = (tap_count - 1) as f64 / 2.0;
    let mut coefficients = (0..tap_count)
        .map(|tap| {
            let centered = tap as f64 - midpoint;
            let sinc = if centered == 0.0 {
                2.0 * cutoff
            } else {
                (TAU * cutoff * centered).sin() / (PI * centered)
            };
            let window_phase = TAU * tap as f64 / (tap_count - 1) as f64;
            let blackman = 0.42 - 0.5 * window_phase.cos() + 0.08 * (2.0 * window_phase).cos();
            sinc * blackman
        })
        .collect::<Vec<_>>();
    let sum: f64 = coefficients.iter().sum();
    for coefficient in &mut coefficients {
        *coefficient /= sum;
    }
    coefficients
}

fn nonlinear_residual(
    samples: &[f64],
    channel_count: usize,
    policy: &ResamplerPolicy,
    drive: f64,
) -> Result<(Vec<f64>, f64), PercussiveForceError> {
    let interpolated = interpolate(samples, channel_count, policy);
    let peak = interpolated
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f64::max);
    if peak == 0.0 {
        return Err(PercussiveForceRefusal::ZeroInterpolatedPeak.into());
    }
    if !peak.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_interpolated_peak",
        }
        .into());
    }
    let normalization = drive.tanh();
    let high_residual = interpolated
        .iter()
        .map(|sample| peak * (drive * sample / peak).tanh() / normalization - sample)
        .collect::<Vec<_>>();
    let residual = downsample_aligned(
        &high_residual,
        samples.len() / channel_count,
        channel_count,
        policy,
    );
    if residual.iter().any(|sample| !sample.is_finite()) {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_aligned_residual",
        }
        .into());
    }
    Ok((residual, peak))
}

fn nonlinear_residual_unchecked(
    samples: &[f64],
    channel_count: usize,
    policy: &ResamplerPolicy,
    drive: f64,
) -> Vec<f64> {
    nonlinear_residual(samples, channel_count, policy, drive)
        .expect("non-zero finite synthetic preflight")
        .0
}

fn interpolate(samples: &[f64], channel_count: usize, policy: &ResamplerPolicy) -> Vec<f64> {
    let input_frames = samples.len() / channel_count;
    let high_frames = input_frames * policy.factor + policy.tap_count - 1;
    let mut output = vec![0.0; high_frames * channel_count];
    for frame in 0..input_frames {
        for channel in 0..channel_count {
            let sample = samples[frame * channel_count + channel];
            for (tap, coefficient) in policy.h_up.iter().enumerate() {
                let high_frame = frame * policy.factor + tap;
                output[high_frame * channel_count + channel] += sample * coefficient;
            }
        }
    }
    output
}

fn convolve_high_rate(samples: &[f64], channel_count: usize, coefficients: &[f64]) -> Vec<f64> {
    let input_frames = samples.len() / channel_count;
    let mut output = vec![0.0; (input_frames + coefficients.len() - 1) * channel_count];
    for frame in 0..input_frames {
        for channel in 0..channel_count {
            let sample = samples[frame * channel_count + channel];
            for (tap, coefficient) in coefficients.iter().enumerate() {
                output[(frame + tap) * channel_count + channel] += sample * coefficient;
            }
        }
    }
    output
}

fn downsample_aligned(
    high_rate_samples: &[f64],
    output_base_frames: usize,
    channel_count: usize,
    policy: &ResamplerPolicy,
) -> Vec<f64> {
    let filtered = convolve_high_rate(high_rate_samples, channel_count, &policy.h_down);
    let total_delay_high_frames = 2 * policy.one_way_group_delay_high_rate_frames;
    let mut output = vec![0.0; output_base_frames * channel_count];
    for frame in 0..output_base_frames {
        let high_frame = frame * policy.factor + total_delay_high_frames;
        for channel in 0..channel_count {
            output[frame * channel_count + channel] =
                filtered[high_frame * channel_count + channel];
        }
    }
    output
}

fn linear_round_trip_delayed(
    samples: &[f64],
    channel_count: usize,
    policy: &ResamplerPolicy,
) -> Vec<f64> {
    let interpolated = interpolate(samples, channel_count, policy);
    let filtered = convolve_high_rate(&interpolated, channel_count, &policy.h_down);
    let high_frames = filtered.len() / channel_count;
    let output_frames = high_frames.div_ceil(policy.factor);
    let mut output = vec![0.0; output_frames * channel_count];
    for frame in 0..output_frames {
        let high_frame = frame * policy.factor;
        if high_frame >= high_frames {
            break;
        }
        for channel in 0..channel_count {
            output[frame * channel_count + channel] =
                filtered[high_frame * channel_count + channel];
        }
    }
    output
}

fn linear_round_trip_aligned(
    samples: &[f64],
    channel_count: usize,
    policy: &ResamplerPolicy,
) -> Vec<f64> {
    let delayed = linear_round_trip_delayed(samples, channel_count, policy);
    let delay = policy.round_trip_group_delay_base_frames;
    let output_frames = samples.len() / channel_count;
    let mut output = vec![0.0; samples.len()];
    for frame in 0..output_frames {
        for channel in 0..channel_count {
            output[frame * channel_count + channel] =
                delayed[(frame + delay) * channel_count + channel];
        }
    }
    output
}

fn fir_magnitude(coefficients: &[f64], cycles_per_sample: f64) -> f64 {
    let mut real = 0.0;
    let mut imaginary = 0.0;
    for (tap, coefficient) in coefficients.iter().enumerate() {
        let phase = TAU * cycles_per_sample * tap as f64;
        real += coefficient * phase.cos();
        imaginary -= coefficient * phase.sin();
    }
    (real * real + imaginary * imaginary).sqrt()
}

fn max_abs_index(samples: &[f64]) -> usize {
    samples
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.abs().total_cmp(&right.abs()))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn synthetic_alias_probe_components() -> Vec<SyntheticToneComponent> {
    vec![
        SyntheticToneComponent {
            amplitude: 0.44,
            cycles_per_base_sample: 0.071,
            phase_radians: 0.0,
        },
        SyntheticToneComponent {
            amplitude: 0.29,
            cycles_per_base_sample: 0.193,
            phase_radians: 0.31,
        },
        SyntheticToneComponent {
            amplitude: 0.17,
            cycles_per_base_sample: 0.317,
            phase_radians: 0.79,
        },
    ]
}

fn synthetic_alias_probe_signal(
    frame_count: usize,
    components: &[SyntheticToneComponent],
) -> Vec<f64> {
    (0..frame_count)
        .map(|frame| {
            components
                .iter()
                .map(|component| {
                    component.amplitude
                        * (TAU * component.cycles_per_base_sample * frame as f64
                            + component.phase_radians)
                            .sin()
                })
                .sum()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::percussive_force::FrozenEventRegion;

    #[test]
    fn frozen_resampler_preflight_reports_every_requirement_without_repair() {
        let preflight = run_f3_resampler_preflight();
        assert_eq!(preflight.subject.factor, 4);
        assert_eq!(preflight.subject.tap_count, 257);
        assert_eq!(preflight.oracle.factor, 8);
        assert_eq!(preflight.oracle.tap_count, 513);
        assert_eq!(preflight.alias_probe_id, "three_tone_0071_0193_0317_v1");
        assert_eq!(preflight.alias_probe_frame_count, 4_096);
        assert_eq!(preflight.alias_probe_comparison_margin_frames, 512);
        assert_eq!(preflight.alias_probe_drive, 3.0);
        assert_eq!(
            preflight.alias_probe_components,
            vec![
                SyntheticToneComponent {
                    amplitude: 0.44,
                    cycles_per_base_sample: 0.071,
                    phase_radians: 0.0,
                },
                SyntheticToneComponent {
                    amplitude: 0.29,
                    cycles_per_base_sample: 0.193,
                    phase_radians: 0.31,
                },
                SyntheticToneComponent {
                    amplitude: 0.17,
                    cycles_per_base_sample: 0.317,
                    phase_radians: 0.79,
                },
            ]
        );
        assert_eq!(
            preflight.passed,
            preflight.dc_unity_pass
                && preflight.passband_response_pass
                && preflight.stopband_attenuation_pass
                && preflight.impulse_alignment_pass
                && preflight.residual_alias_pass
        );
        assert!(preflight.measured_dc_unity_error.is_finite());
        assert!(preflight.measured_passband_deviation_db.is_finite());
        assert!(preflight.measured_stopband_attenuation_db.is_finite());
        assert!(preflight.measured_residual_alias_db.is_finite());
        assert!(preflight.dc_unity_pass);
        assert!(preflight.passband_response_pass);
        assert!(preflight.stopband_attenuation_pass);
        assert!(preflight.impulse_alignment_pass);
        // The frozen 4x topology misses its fixed 8x-oracle alias screen. Keep
        // the failure executable instead of changing its preregistered filter.
        assert!(!preflight.residual_alias_pass);
        assert!(!preflight.passed);
    }

    #[test]
    fn residual_targets_land_and_pre_onset_tail_remain_bit_identical() {
        let sample_rate_hz = 48_000;
        let frame_count = 768;
        let mut samples = vec![0.0_f32; frame_count * 2];
        for frame in 0..frame_count {
            let envelope = if frame < 128 || frame >= 640 {
                0.02
            } else if frame < 288 {
                0.78
            } else {
                0.49
            };
            for channel in 0..2 {
                let phase = channel as f64 * 0.17;
                samples[frame * 2 + channel] = (envelope
                    * (0.56 * (TAU * 510.0 * frame as f64 / sample_rate_hz as f64 + phase).sin()
                        + 0.31
                            * (TAU * 3_700.0 * frame as f64 / sample_rate_hz as f64 + phase).sin()))
                    as f32;
            }
        }
        let region = FrozenEventRegion {
            onset_frame: 128,
            attack_end_frame: 288,
            body_end_frame: 640,
        };
        let rendered = render_f3_os4_onset_residual_v1(FrozenEventInput {
            interleaved_samples: &samples,
            sample_rate_hz,
            channel_count: 2,
            region,
        })
        .expect("bounded nonlinear residual should render");

        for output in [
            &rendered.combined,
            &rendered.attack_only,
            &rendered.body_only,
        ] {
            assert_eq!(
                &output[..region.onset_frame * 2],
                &samples[..region.onset_frame * 2]
            );
            assert_eq!(
                &output[region.body_end_frame * 2..],
                &samples[region.body_end_frame * 2..]
            );
        }

        let masks = super::super::common::EqualPowerMasks::for_region(frame_count, region).unwrap();
        let source_f64 = samples
            .iter()
            .map(|sample| f64::from(*sample))
            .collect::<Vec<_>>();
        let attack_delta = rendered
            .attack_only
            .iter()
            .zip(&source_f64)
            .map(|(candidate, source)| f64::from(*candidate) - source)
            .collect::<Vec<_>>();
        let body_delta = rendered
            .body_only
            .iter()
            .zip(&source_f64)
            .map(|(candidate, source)| f64::from(*candidate) - source)
            .collect::<Vec<_>>();
        let branch_rms = |delta: &[f64], mask: &[f64]| {
            let energy: f64 = delta.iter().map(|sample| sample * sample).sum();
            (energy / (2.0 * mask.iter().sum::<f64>())).sqrt()
        };
        let attack_delta_rms = branch_rms(&attack_delta, &masks.attack);
        let body_delta_rms = branch_rms(&body_delta, &masks.body);
        let attack_dry_rms = weighted_rms(&source_f64, 2, &masks.attack).unwrap();
        let body_dry_rms = weighted_rms(&source_f64, 2, &masks.body).unwrap();
        assert!(
            (attack_delta_rms / attack_dry_rms - rendered.policy.attack_target_ratio).abs()
                <= 2.0e-6
        );
        assert!(
            (body_delta_rms / body_dry_rms - rendered.policy.body_target_ratio).abs() <= 2.0e-6
        );
    }
}
