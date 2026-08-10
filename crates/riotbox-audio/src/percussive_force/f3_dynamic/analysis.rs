//! Canonical analysis and controller ownership for F3-v2.
//!
//! The phase-safe RMS implementation is crate-visible so later Stage-A event
//! qualification can reuse this exact formula instead of growing a subtly
//! different envelope implementation.

use sha2::{Digest, Sha256};

use super::{F3ControllerHashes, F3PcmEncoding};
use crate::percussive_force::common::{
    InvalidEventInput, PercussiveForceError, PercussiveForceRefusal, ValidatedFrozenEvent,
    effective_region_energy,
};

const ENVELOPE_WINDOWS_MS: [u32; 3] = [1, 8, 20];
const LOOKBEHIND_MS: u32 = 20;
const ATTACK_RISE_MS: u32 = 1;
const ATTACK_FALL_MS: u32 = 8;
const BODY_RISE_MS: u32 = 8;
const BODY_FALL_MS: u32 = 20;
const MAD_CONSISTENCY_SCALE: f64 = 1.4826;
const BASELINE_MAD_MULTIPLIER: f64 = 3.0;
const LSB_FLOOR_MULTIPLIER: f64 = 16.0;
const REVIEWABILITY_RATIO_MINIMUM: f64 = 0.05;
pub(super) const NUMERICAL_EPSILON_MULTIPLIER: f64 = 64.0;
const CONTROLLER_HASH_DOMAIN: &str =
    "riotbox.f3_causal_envelope_contrast_dynamic_residual_v2.controller.v1";

impl F3PcmEncoding {
    pub(super) fn valid_bits(self) -> u8 {
        match self {
            Self::SignedPcm16 => 16,
            Self::SignedPcm24 => 24,
        }
    }

    pub(super) fn normalized_lsb(self) -> f64 {
        2.0_f64.powi(1 - i32::from(self.valid_bits()))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PhaseSafeRmsEnvelopes {
    pub(crate) channel_means: Vec<f64>,
    pub(crate) r1: Vec<f64>,
    pub(crate) r8: Vec<f64>,
    pub(crate) r20: Vec<f64>,
}

#[derive(Clone, Debug)]
pub(super) struct DynamicAnalysis {
    pub(super) input_lsb: f64,
    pub(super) lsb_mean_square_floor: f64,
    pub(super) envelope_window_frames: [usize; 3],
    pub(super) lookbehind_frames: usize,
    pub(super) baseline_rms: f64,
    pub(super) controller_floor_rms: f64,
    pub(super) source_attack_mean_square: f64,
    pub(super) source_body_mean_square: f64,
    pub(super) attack_branch_mean_square: f64,
    pub(super) body_branch_mean_square: f64,
    pub(super) attack_contribution_ratio: f64,
    pub(super) body_contribution_ratio: f64,
    pub(super) attack_rise_coefficient: f64,
    pub(super) attack_fall_coefficient: f64,
    pub(super) body_rise_coefficient: f64,
    pub(super) body_fall_coefficient: f64,
    pub(super) raw_attack: Vec<f64>,
    pub(super) raw_body: Vec<f64>,
    pub(super) attack_state: Vec<f64>,
    pub(super) body_state: Vec<f64>,
    pub(super) attack_delta: Vec<f64>,
    pub(super) body_delta: Vec<f64>,
    pub(super) envelopes: PhaseSafeRmsEnvelopes,
}

#[derive(Clone, Debug)]
pub(super) struct DynamicControllerTrace {
    pub(super) input_lsb: f64,
    pub(super) lsb_mean_square_floor: f64,
    pub(super) envelope_window_frames: [usize; 3],
    pub(super) lookbehind_frames: usize,
    pub(super) baseline_rms: f64,
    pub(super) controller_floor_rms: f64,
    pub(super) source_attack_mean_square: f64,
    pub(super) source_body_mean_square: f64,
    pub(super) attack_rise_coefficient: f64,
    pub(super) attack_fall_coefficient: f64,
    pub(super) body_rise_coefficient: f64,
    pub(super) body_fall_coefficient: f64,
    pub(super) raw_attack: Vec<f64>,
    pub(super) raw_body: Vec<f64>,
    pub(super) attack_state: Vec<f64>,
    pub(super) body_state: Vec<f64>,
    pub(super) envelopes: PhaseSafeRmsEnvelopes,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct DirectionMetrics {
    pub(super) attack_fast_to_slow: f64,
    pub(super) body_fast_to_context: f64,
}

pub(super) fn analyze_dynamic_controller_trace(
    event: &ValidatedFrozenEvent<'_>,
    pcm_encoding: F3PcmEncoding,
) -> Result<DynamicControllerTrace, PercussiveForceError> {
    let envelope_window_frames =
        ENVELOPE_WINDOWS_MS.map(|ms| frames_for_ms(event.sample_rate_hz, ms));
    let lookbehind_frames = frames_for_ms(event.sample_rate_hz, LOOKBEHIND_MS);
    let required_lookbehind = lookbehind_frames + envelope_window_frames[0].saturating_sub(1);
    if event.region.onset_frame < required_lookbehind {
        return Err(InvalidEventInput::InsufficientLookbehind {
            required_frames: required_lookbehind,
            available_frames: event.region.onset_frame,
        }
        .into());
    }

    let envelopes = phase_safe_multichannel_rms_envelopes(
        event.samples,
        event.channel_count,
        envelope_window_frames,
    )?;
    let baseline_start = event.region.onset_frame - lookbehind_frames;
    let baseline_rms =
        robust_r1_anatomy_baseline(&envelopes.r1[baseline_start..event.region.onset_frame])?;
    let input_lsb = pcm_encoding.normalized_lsb();
    let lsb_rms_floor = LSB_FLOOR_MULTIPLIER * input_lsb;
    let lsb_mean_square_floor = lsb_rms_floor.powi(2);
    let controller_floor_rms = baseline_rms.max(lsb_rms_floor);

    let energy = effective_region_energy(event.samples, event.channel_count, &event.masks)?;
    let attack_weight_sum: f64 = event.masks.attack.iter().sum();
    let body_weight_sum: f64 = event.masks.body.iter().sum();
    let source_attack_mean_square =
        energy.attack / (event.channel_count as f64 * attack_weight_sum);
    let source_body_mean_square = energy.body / (event.channel_count as f64 * body_weight_sum);
    require_source_floor("attack", source_attack_mean_square, lsb_mean_square_floor)?;
    require_source_floor("body", source_body_mean_square, lsb_mean_square_floor)?;

    let attack_rise_coefficient = ballistic_coefficient(event.sample_rate_hz, ATTACK_RISE_MS);
    let attack_fall_coefficient = ballistic_coefficient(event.sample_rate_hz, ATTACK_FALL_MS);
    let body_rise_coefficient = ballistic_coefficient(event.sample_rate_hz, BODY_RISE_MS);
    let body_fall_coefficient = ballistic_coefficient(event.sample_rate_hz, BODY_FALL_MS);
    let mut raw_attack = vec![0.0; event.frame_count];
    let mut raw_body = vec![0.0; event.frame_count];
    let mut attack_state = vec![0.0; event.frame_count];
    let mut body_state = vec![0.0; event.frame_count];
    let mut previous_attack = 0.0;
    let mut previous_body = 0.0;
    for frame in event.region.onset_frame..event.region.body_end_frame {
        let r1 = envelopes.r1[frame];
        let r8 = envelopes.r8[frame];
        let r20 = envelopes.r20[frame];
        let attack = (directed_contrast(r1, r8, controller_floor_rms)
            * directed_contrast(r8, r20, controller_floor_rms))
        .sqrt();
        let body = directed_contrast(r8.max(r20), r1, controller_floor_rms);
        let next_attack = ballistic_step(
            attack,
            previous_attack,
            attack_rise_coefficient,
            attack_fall_coefficient,
        );
        let next_body = ballistic_step(
            body,
            previous_body,
            body_rise_coefficient,
            body_fall_coefficient,
        );
        for (stage, value) in [
            ("f3_v2_raw_attack", attack),
            ("f3_v2_raw_body", body),
            ("f3_v2_attack_state", next_attack),
            ("f3_v2_body_state", next_body),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(PercussiveForceRefusal::NonFiniteAnalysis { stage }.into());
            }
        }
        raw_attack[frame] = canonical_zero(attack);
        raw_body[frame] = canonical_zero(body);
        attack_state[frame] = canonical_zero(next_attack);
        body_state[frame] = canonical_zero(next_body);
        previous_attack = next_attack;
        previous_body = next_body;
    }

    Ok(DynamicControllerTrace {
        input_lsb,
        lsb_mean_square_floor,
        envelope_window_frames,
        lookbehind_frames,
        baseline_rms,
        controller_floor_rms,
        source_attack_mean_square,
        source_body_mean_square,
        attack_rise_coefficient,
        attack_fall_coefficient,
        body_rise_coefficient,
        body_fall_coefficient,
        raw_attack,
        raw_body,
        attack_state,
        body_state,
        envelopes,
    })
}

pub(super) fn resolve_dynamic_analysis(
    event: &ValidatedFrozenEvent<'_>,
    pcm_encoding: F3PcmEncoding,
) -> Result<DynamicAnalysis, PercussiveForceError> {
    let trace = analyze_dynamic_controller_trace(event, pcm_encoding)?;
    let zero_tolerance = NUMERICAL_EPSILON_MULTIPLIER * f64::from(f32::EPSILON);
    let attack_missing = finite_peak(&trace.attack_state)? <= zero_tolerance;
    let body_missing = finite_peak(&trace.body_state)? <= zero_tolerance;
    if attack_missing || body_missing {
        return Err(PercussiveForceRefusal::MissingDynamicContrast {
            attack_missing,
            body_missing,
        }
        .into());
    }

    let mut attack_delta = vec![0.0; event.samples.len()];
    let mut body_delta = vec![0.0; event.samples.len()];
    let mut attack_delta_energy = 0.0;
    let mut body_delta_energy = 0.0;
    let attack_weight_sum: f64 = event.masks.attack.iter().sum();
    let body_weight_sum: f64 = event.masks.body.iter().sum();
    for frame in event.region.onset_frame..event.region.body_end_frame {
        let attack_gain = event.masks.attack[frame] * trace.attack_state[frame];
        let body_gain = event.masks.body[frame] * trace.body_state[frame];
        for channel in 0..event.channel_count {
            let sample_index = frame * event.channel_count + channel;
            let dry = f64::from(event.samples[sample_index]);
            let attack = dry * attack_gain;
            let body = dry * body_gain;
            attack_delta[sample_index] = attack;
            body_delta[sample_index] = body;
            attack_delta_energy += attack * attack;
            body_delta_energy += body * body;
        }
    }
    let attack_branch_mean_square =
        attack_delta_energy / (event.channel_count as f64 * attack_weight_sum);
    let body_branch_mean_square =
        body_delta_energy / (event.channel_count as f64 * body_weight_sum);
    require_branch_floor(
        "attack",
        attack_branch_mean_square,
        trace.lsb_mean_square_floor,
    )?;
    require_branch_floor("body", body_branch_mean_square, trace.lsb_mean_square_floor)?;
    let attack_contribution_ratio =
        (attack_branch_mean_square / trace.source_attack_mean_square).sqrt();
    let body_contribution_ratio = (body_branch_mean_square / trace.source_body_mean_square).sqrt();
    require_reviewability("attack", attack_contribution_ratio)?;
    require_reviewability("body", body_contribution_ratio)?;

    Ok(DynamicAnalysis {
        input_lsb: trace.input_lsb,
        lsb_mean_square_floor: trace.lsb_mean_square_floor,
        envelope_window_frames: trace.envelope_window_frames,
        lookbehind_frames: trace.lookbehind_frames,
        baseline_rms: trace.baseline_rms,
        controller_floor_rms: trace.controller_floor_rms,
        source_attack_mean_square: trace.source_attack_mean_square,
        source_body_mean_square: trace.source_body_mean_square,
        attack_branch_mean_square,
        body_branch_mean_square,
        attack_contribution_ratio,
        body_contribution_ratio,
        attack_rise_coefficient: trace.attack_rise_coefficient,
        attack_fall_coefficient: trace.attack_fall_coefficient,
        body_rise_coefficient: trace.body_rise_coefficient,
        body_fall_coefficient: trace.body_fall_coefficient,
        raw_attack: trace.raw_attack,
        raw_body: trace.raw_body,
        attack_state: trace.attack_state,
        body_state: trace.body_state,
        attack_delta,
        body_delta,
        envelopes: trace.envelopes,
    })
}

pub(super) fn frames_for_ms(sample_rate_hz: u32, milliseconds: u32) -> usize {
    ((u64::from(sample_rate_hz) * u64::from(milliseconds) + 500) / 1_000).max(1) as usize
}

pub(crate) fn phase_safe_multichannel_rms_envelopes(
    samples: &[f32],
    channel_count: usize,
    window_frames: [usize; 3],
) -> Result<PhaseSafeRmsEnvelopes, PercussiveForceError> {
    if channel_count == 0 {
        return Err(InvalidEventInput::ZeroChannelCount.into());
    }
    if samples.is_empty() {
        return Err(InvalidEventInput::EmptyBuffer.into());
    }
    if !samples.len().is_multiple_of(channel_count) {
        return Err(InvalidEventInput::MisalignedInterleavedSamples {
            sample_count: samples.len(),
            channel_count,
        }
        .into());
    }
    let frame_count = samples.len() / channel_count;
    let mut channel_means = vec![0.0; channel_count];
    for frame_samples in samples.chunks_exact(channel_count) {
        for (channel, sample) in frame_samples.iter().enumerate() {
            channel_means[channel] += f64::from(*sample);
        }
    }
    for mean in &mut channel_means {
        *mean /= frame_count as f64;
    }
    phase_safe_multichannel_rms_envelopes_with_frozen_means(
        samples,
        channel_count,
        window_frames,
        &channel_means,
    )
}

pub(crate) fn phase_safe_multichannel_rms_envelopes_with_frozen_means(
    samples: &[f32],
    channel_count: usize,
    window_frames: [usize; 3],
    frozen_channel_means: &[f64],
) -> Result<PhaseSafeRmsEnvelopes, PercussiveForceError> {
    if channel_count == 0 {
        return Err(InvalidEventInput::ZeroChannelCount.into());
    }
    if samples.is_empty() {
        return Err(InvalidEventInput::EmptyBuffer.into());
    }
    if !samples.len().is_multiple_of(channel_count) {
        return Err(InvalidEventInput::MisalignedInterleavedSamples {
            sample_count: samples.len(),
            channel_count,
        }
        .into());
    }
    let frame_count = samples.len() / channel_count;
    if frozen_channel_means.len() != channel_count
        || frozen_channel_means.iter().any(|mean| !mean.is_finite())
    {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_v2_frozen_channel_means",
        }
        .into());
    }
    let channel_means = frozen_channel_means.to_vec();
    let mut power = Vec::with_capacity(frame_count);
    for frame_samples in samples.chunks_exact(channel_count) {
        let mut sum = 0.0;
        for (channel, sample) in frame_samples.iter().enumerate() {
            let centered = f64::from(*sample) - channel_means[channel];
            sum += centered * centered;
        }
        let value = sum / channel_count as f64;
        if !value.is_finite() || value < 0.0 {
            return Err(PercussiveForceRefusal::NonFiniteAnalysis {
                stage: "f3_v2_frame_power",
            }
            .into());
        }
        power.push(value);
    }
    let r1 = moving_rms(&power, window_frames[0])?;
    let r8 = moving_rms(&power, window_frames[1])?;
    let r20 = moving_rms(&power, window_frames[2])?;
    Ok(PhaseSafeRmsEnvelopes {
        channel_means,
        r1,
        r8,
        r20,
    })
}

fn moving_rms(power: &[f64], window_frames: usize) -> Result<Vec<f64>, PercussiveForceError> {
    let mut prefix = Vec::with_capacity(power.len() + 1);
    prefix.push(0.0);
    for value in power {
        prefix.push(prefix.last().copied().unwrap_or(0.0) + value);
    }
    let mut output = vec![0.0; power.len()];
    for (frame, output_value) in output
        .iter_mut()
        .enumerate()
        .skip(window_frames.saturating_sub(1))
    {
        let end = frame + 1;
        let start = end - window_frames;
        let energy = (prefix[end] - prefix[start]).max(0.0);
        let rms = (energy / window_frames as f64).sqrt();
        if !rms.is_finite() {
            return Err(PercussiveForceRefusal::NonFiniteAnalysis {
                stage: "f3_v2_moving_rms",
            }
            .into());
        }
        *output_value = canonical_zero(rms);
    }
    Ok(output)
}

pub(crate) fn robust_r1_anatomy_baseline(values: &[f64]) -> Result<f64, PercussiveForceError> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_v2_anatomy_baseline_input",
        }
        .into());
    }
    let center = median(values);
    let deviations = values
        .iter()
        .map(|value| (value - center).abs())
        .collect::<Vec<_>>();
    let baseline = center + BASELINE_MAD_MULTIPLIER * MAD_CONSISTENCY_SCALE * median(&deviations);
    if !baseline.is_finite() || baseline < 0.0 {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_v2_anatomy_baseline",
        }
        .into());
    }
    Ok(canonical_zero(baseline))
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    }
}

fn directed_contrast(primary: f64, reference: f64, floor: f64) -> f64 {
    if primary > reference {
        (primary - reference) / primary.max(reference).max(floor)
    } else {
        0.0
    }
}

fn ballistic_coefficient(sample_rate_hz: u32, milliseconds: u32) -> f64 {
    (-1.0 / frames_for_ms(sample_rate_hz, milliseconds) as f64).exp()
}

fn ballistic_step(input: f64, previous: f64, rise: f64, fall: f64) -> f64 {
    let coefficient = if input > previous { rise } else { fall };
    coefficient * previous + (1.0 - coefficient) * input
}

fn require_source_floor(
    region: &'static str,
    mean_square: f64,
    strict_floor: f64,
) -> Result<(), PercussiveForceError> {
    if !mean_square.is_finite() || mean_square <= strict_floor {
        return Err(PercussiveForceRefusal::SourceRegionBelowLsbFloor {
            region,
            mean_square,
            strict_floor,
        }
        .into());
    }
    Ok(())
}

fn require_branch_floor(
    branch: &'static str,
    mean_square: f64,
    strict_floor: f64,
) -> Result<(), PercussiveForceError> {
    if !mean_square.is_finite() || mean_square <= strict_floor {
        return Err(PercussiveForceRefusal::DynamicBranchBelowLsbFloor {
            branch,
            mean_square,
            strict_floor,
        }
        .into());
    }
    Ok(())
}

fn require_reviewability(
    branch: &'static str,
    contribution_ratio: f64,
) -> Result<(), PercussiveForceError> {
    if !contribution_ratio.is_finite() || contribution_ratio < REVIEWABILITY_RATIO_MINIMUM {
        return Err(
            PercussiveForceRefusal::DynamicBranchBelowReviewabilityFloor {
                branch,
                contribution_ratio,
                minimum_ratio: REVIEWABILITY_RATIO_MINIMUM,
            }
            .into(),
        );
    }
    Ok(())
}

pub(super) fn direction_metrics(
    envelopes: &PhaseSafeRmsEnvelopes,
    attack_mask: &[f64],
    body_mask: &[f64],
) -> Result<DirectionMetrics, PercussiveForceError> {
    let mut attack_fast = 0.0;
    let mut attack_slow = 0.0;
    let mut body_fast = 0.0;
    let mut body_context = 0.0;
    for frame in 0..attack_mask.len() {
        attack_fast += attack_mask[frame] * envelopes.r1[frame].powi(2);
        attack_slow += attack_mask[frame] * envelopes.r20[frame].powi(2);
        body_fast += body_mask[frame] * envelopes.r1[frame].powi(2);
        body_context += body_mask[frame] * envelopes.r8[frame].max(envelopes.r20[frame]).powi(2);
    }
    if attack_slow <= 0.0 || body_context <= 0.0 {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_v2_direction_denominator",
        }
        .into());
    }
    let attack_fast_to_slow = attack_fast / attack_slow;
    let body_fast_to_context = body_fast / body_context;
    if !attack_fast_to_slow.is_finite() || !body_fast_to_context.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_v2_direction_metric",
        }
        .into());
    }
    Ok(DirectionMetrics {
        attack_fast_to_slow,
        body_fast_to_context,
    })
}

pub(super) fn finite_peak(values: &[f64]) -> Result<f64, PercussiveForceError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_v2_controller_peak",
        }
        .into());
    }
    Ok(values.iter().copied().fold(0.0_f64, f64::max))
}

fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

pub(super) fn resolved_controller_hashes(
    analysis: &DynamicAnalysis,
    sample_rate_hz: u32,
    channel_count: usize,
    region: crate::percussive_force::FrozenEventRegion,
) -> Result<F3ControllerHashes, PercussiveForceError> {
    controller_hashes(
        &analysis.raw_attack,
        &analysis.raw_body,
        &analysis.attack_state,
        &analysis.body_state,
        sample_rate_hz,
        channel_count,
        region,
    )
}

pub(super) fn trace_controller_hashes(
    trace: &DynamicControllerTrace,
    sample_rate_hz: u32,
    channel_count: usize,
    region: crate::percussive_force::FrozenEventRegion,
) -> Result<F3ControllerHashes, PercussiveForceError> {
    controller_hashes(
        &trace.raw_attack,
        &trace.raw_body,
        &trace.attack_state,
        &trace.body_state,
        sample_rate_hz,
        channel_count,
        region,
    )
}

fn controller_hashes(
    raw_attack: &[f64],
    raw_body: &[f64],
    attack_state: &[f64],
    body_state: &[f64],
    sample_rate_hz: u32,
    channel_count: usize,
    region: crate::percussive_force::FrozenEventRegion,
) -> Result<F3ControllerHashes, PercussiveForceError> {
    Ok(F3ControllerHashes {
        raw_attack_sha256: controller_hash(
            "a0",
            raw_attack,
            sample_rate_hz,
            channel_count,
            region,
        )?,
        raw_body_sha256: controller_hash("b0", raw_body, sample_rate_hz, channel_count, region)?,
        attack_state_sha256: controller_hash(
            "attack_state",
            attack_state,
            sample_rate_hz,
            channel_count,
            region,
        )?,
        body_state_sha256: controller_hash(
            "body_state",
            body_state,
            sample_rate_hz,
            channel_count,
            region,
        )?,
    })
}

fn controller_hash(
    label: &'static str,
    values: &[f64],
    sample_rate_hz: u32,
    channel_count: usize,
    region: crate::percussive_force::FrozenEventRegion,
) -> Result<String, PercussiveForceError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_v2_controller_hash",
        }
        .into());
    }
    if let Some(frame) = values
        .iter()
        .position(|value| value.to_bits() == (-0.0_f64).to_bits())
    {
        return Err(PercussiveForceRefusal::NegativeZeroControllerValue { label, frame }.into());
    }
    let mut digest = Sha256::new();
    hash_length_prefixed(&mut digest, CONTROLLER_HASH_DOMAIN.as_bytes());
    hash_length_prefixed(&mut digest, label.as_bytes());
    digest.update(sample_rate_hz.to_be_bytes());
    digest.update((channel_count as u32).to_be_bytes());
    digest.update((values.len() as u64).to_be_bytes());
    digest.update((region.onset_frame as u64).to_be_bytes());
    digest.update((region.attack_end_frame as u64).to_be_bytes());
    digest.update((region.body_end_frame as u64).to_be_bytes());
    digest.update((values.len() as u64).to_be_bytes());
    for value in values {
        digest.update(value.to_bits().to_be_bytes());
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn hash_length_prefixed(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u32).to_be_bytes());
    digest.update(bytes);
}

#[cfg(test)]
mod tests;
