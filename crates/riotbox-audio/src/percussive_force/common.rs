use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrozenEventRegion {
    pub onset_frame: usize,
    pub attack_end_frame: usize,
    pub body_end_frame: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct FrozenEventInput<'a> {
    pub interleaved_samples: &'a [f32],
    pub sample_rate_hz: u32,
    pub channel_count: usize,
    pub region: FrozenEventRegion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvalidEventInput {
    ZeroSampleRate,
    ZeroChannelCount,
    EmptyBuffer,
    MisalignedInterleavedSamples {
        sample_count: usize,
        channel_count: usize,
    },
    NonFiniteSample {
        sample_index: usize,
    },
    InvalidRegion {
        frame_count: usize,
        onset_frame: usize,
        attack_end_frame: usize,
        body_end_frame: usize,
    },
    InvalidMaskRanges {
        frame_count: usize,
        crossfade_start_frame: usize,
        crossfade_end_frame: usize,
        body_fade_start_frame: usize,
        body_end_frame: usize,
    },
    InsufficientLookbehind {
        required_frames: usize,
        available_frames: usize,
    },
    InsufficientTailPadding {
        required_frames: usize,
        available_frames: usize,
    },
    InvalidQuantizationLsb,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PercussiveForceRefusal {
    MissingAttackEnergy,
    MissingBodyEnergy,
    InsufficientSpectralBins {
        usable_bins: usize,
    },
    InvalidSpectralQuantiles {
        f25_hz: f64,
        f75_hz: f64,
        minimum_split_hz: f64,
        maximum_split_hz: f64,
        nyquist_hz: f64,
    },
    InsufficientSpectralBinSeparation {
        f25_bin: usize,
        f75_bin: usize,
        minimum_separation_bins: usize,
    },
    InsufficientTrustedBands {
        trusted_bands: usize,
    },
    MissingTrustedAttackEnergy,
    ZeroInterpolatedPeak,
    ZeroAttackResidual,
    ZeroBodyResidual,
    ResidualScaleExceedsCap {
        attack_scale: f64,
        body_scale: f64,
        cap: f64,
    },
    SourceRegionBelowLsbFloor {
        region: &'static str,
        mean_square: f64,
        strict_floor: f64,
    },
    MissingDynamicContrast {
        attack_missing: bool,
        body_missing: bool,
    },
    DynamicBranchBelowLsbFloor {
        branch: &'static str,
        mean_square: f64,
        strict_floor: f64,
    },
    DynamicBranchBelowReviewabilityFloor {
        branch: &'static str,
        contribution_ratio: f64,
        minimum_ratio: f64,
    },
    DynamicDirectionNotIncreased {
        metric: &'static str,
        source_value: f64,
        candidate_value: f64,
    },
    DynamicAblationIdentityMismatch {
        normalized_error: f64,
        maximum_error: f64,
    },
    NegativeZeroControllerValue {
        label: &'static str,
        frame: usize,
    },
    OutputPeakWithoutHeadroom {
        peak: f64,
        strict_maximum: f64,
    },
    NonFiniteAnalysis {
        stage: &'static str,
    },
    NonFiniteOutput {
        sample_index: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum PercussiveForceError {
    InvalidInput(InvalidEventInput),
    Refused(PercussiveForceRefusal),
}

impl fmt::Display for PercussiveForceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(reason) => write!(formatter, "invalid frozen event: {reason:?}"),
            Self::Refused(reason) => {
                write!(formatter, "percussive-force candidate refused: {reason:?}")
            }
        }
    }
}

impl Error for PercussiveForceError {}

impl From<InvalidEventInput> for PercussiveForceError {
    fn from(value: InvalidEventInput) -> Self {
        Self::InvalidInput(value)
    }
}

impl From<PercussiveForceRefusal> for PercussiveForceError {
    fn from(value: PercussiveForceRefusal) -> Self {
        Self::Refused(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectiveRegionEnergy {
    /// Sum of `sample^2 * w_a` across every channel.
    pub attack: f64,
    /// Sum of `sample^2 * w_b` across every channel.
    pub body: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EqualPowerMasks {
    pub attack: Vec<f64>,
    pub body: Vec<f64>,
    pub attack_body_crossfade_frames: usize,
    pub attack_body_crossfade_start_frame: usize,
    pub attack_body_crossfade_end_frame: usize,
    pub body_fade_frames: usize,
    pub body_fade_start_frame: usize,
    pub phase_denominator: usize,
}

impl EqualPowerMasks {
    pub fn for_region(
        frame_count: usize,
        region: FrozenEventRegion,
    ) -> Result<Self, PercussiveForceError> {
        if region.onset_frame >= region.attack_end_frame
            || region.attack_end_frame >= region.body_end_frame
            || region.body_end_frame > frame_count
        {
            return Err(InvalidEventInput::InvalidRegion {
                frame_count,
                onset_frame: region.onset_frame,
                attack_end_frame: region.attack_end_frame,
                body_end_frame: region.body_end_frame,
            }
            .into());
        }
        let attack_len = region.attack_end_frame - region.onset_frame;
        let body_len = region.body_end_frame - region.attack_end_frame;
        let crossfade_frames = ((attack_len.min(body_len) as f64 / 8.0).round() as usize).max(1);

        let crossfade_start = region.attack_end_frame - crossfade_frames / 2;
        let crossfade_end = crossfade_start + crossfade_frames;
        let body_fade_start = region.body_end_frame - crossfade_frames;
        if crossfade_start < region.onset_frame
            || crossfade_end > body_fade_start
            || body_fade_start < region.attack_end_frame
        {
            return Err(InvalidEventInput::InvalidMaskRanges {
                frame_count,
                crossfade_start_frame: crossfade_start,
                crossfade_end_frame: crossfade_end,
                body_fade_start_frame: body_fade_start,
                body_end_frame: region.body_end_frame,
            }
            .into());
        }

        let mut attack = vec![0.0; frame_count];
        let mut body = vec![0.0; frame_count];
        attack[region.onset_frame..crossfade_start].fill(1.0);
        for offset in 0..crossfade_frames {
            let phase = (offset as f64 + 1.0) / (crossfade_frames + 1) as f64;
            let angle = phase * std::f64::consts::FRAC_PI_2;
            attack[crossfade_start + offset] = angle.cos().powi(2);
            body[crossfade_start + offset] = angle.sin().powi(2);
        }

        body[crossfade_end..body_fade_start].fill(1.0);
        for offset in 0..crossfade_frames {
            let phase = (offset as f64 + 1.0) / (crossfade_frames + 1) as f64;
            body[body_fade_start + offset] = (phase * std::f64::consts::FRAC_PI_2).cos().powi(2);
        }

        Ok(Self {
            attack,
            body,
            attack_body_crossfade_frames: crossfade_frames,
            attack_body_crossfade_start_frame: crossfade_start,
            attack_body_crossfade_end_frame: crossfade_end,
            body_fade_frames: crossfade_frames,
            body_fade_start_frame: body_fade_start,
            phase_denominator: crossfade_frames + 1,
        })
    }
}

pub(crate) struct ValidatedFrozenEvent<'a> {
    pub samples: &'a [f32],
    pub sample_rate_hz: u32,
    pub channel_count: usize,
    pub frame_count: usize,
    pub region: FrozenEventRegion,
    pub masks: EqualPowerMasks,
}

pub(crate) fn validate_frozen_event(
    input: FrozenEventInput<'_>,
) -> Result<ValidatedFrozenEvent<'_>, PercussiveForceError> {
    if input.sample_rate_hz == 0 {
        return Err(InvalidEventInput::ZeroSampleRate.into());
    }
    if input.channel_count == 0 {
        return Err(InvalidEventInput::ZeroChannelCount.into());
    }
    if input.interleaved_samples.is_empty() {
        return Err(InvalidEventInput::EmptyBuffer.into());
    }
    if !input
        .interleaved_samples
        .len()
        .is_multiple_of(input.channel_count)
    {
        return Err(InvalidEventInput::MisalignedInterleavedSamples {
            sample_count: input.interleaved_samples.len(),
            channel_count: input.channel_count,
        }
        .into());
    }
    if let Some(sample_index) = input
        .interleaved_samples
        .iter()
        .position(|sample| !sample.is_finite())
    {
        return Err(InvalidEventInput::NonFiniteSample { sample_index }.into());
    }

    let frame_count = input.interleaved_samples.len() / input.channel_count;
    if input.region.onset_frame >= input.region.attack_end_frame
        || input.region.attack_end_frame >= input.region.body_end_frame
        || input.region.body_end_frame > frame_count
    {
        return Err(InvalidEventInput::InvalidRegion {
            frame_count,
            onset_frame: input.region.onset_frame,
            attack_end_frame: input.region.attack_end_frame,
            body_end_frame: input.region.body_end_frame,
        }
        .into());
    }

    Ok(ValidatedFrozenEvent {
        samples: input.interleaved_samples,
        sample_rate_hz: input.sample_rate_hz,
        channel_count: input.channel_count,
        frame_count,
        region: input.region,
        masks: EqualPowerMasks::for_region(frame_count, input.region)?,
    })
}

pub(crate) fn effective_region_energy(
    samples: &[f32],
    channel_count: usize,
    masks: &EqualPowerMasks,
) -> Result<EffectiveRegionEnergy, PercussiveForceError> {
    let mut attack = 0.0;
    let mut body = 0.0;
    for (frame, frame_samples) in samples.chunks_exact(channel_count).enumerate() {
        let w_a = masks.attack[frame];
        let w_b = masks.body[frame];
        for &sample in frame_samples {
            let square = f64::from(sample).powi(2);
            attack += square * w_a;
            body += square * w_b;
        }
    }
    if !attack.is_finite() || !body.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "effective_region_energy",
        }
        .into());
    }
    Ok(EffectiveRegionEnergy { attack, body })
}

pub(crate) fn weighted_rms(
    samples: &[f64],
    channel_count: usize,
    weights: &[f64],
) -> Result<f64, PercussiveForceError> {
    let weight_sum: f64 = weights.iter().sum();
    if weight_sum == 0.0 || !weight_sum.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "weighted_rms_weight_sum",
        }
        .into());
    }
    let mut energy = 0.0;
    for (frame, frame_samples) in samples.chunks_exact(channel_count).enumerate() {
        for sample in frame_samples {
            energy += sample * sample * weights[frame];
        }
    }
    let rms = (energy / (channel_count as f64 * weight_sum)).sqrt();
    if !rms.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "weighted_rms",
        }
        .into());
    }
    Ok(rms)
}

#[allow(dead_code)] // retained only by the immutable rejected F3-v1 renderer
pub(crate) fn weighted_branch_rms(
    samples: &[f64],
    channel_count: usize,
    mask: &[f64],
) -> Result<f64, PercussiveForceError> {
    let weight_sum: f64 = mask.iter().sum();
    if weight_sum == 0.0 || !weight_sum.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "weighted_branch_rms_weight_sum",
        }
        .into());
    }
    let mut energy = 0.0;
    for (frame, frame_samples) in samples.chunks_exact(channel_count).enumerate() {
        let branch_gain = mask[frame];
        for sample in frame_samples {
            let branch_sample = branch_gain * sample;
            energy += branch_sample * branch_sample;
        }
    }
    let rms = (energy / (channel_count as f64 * weight_sum)).sqrt();
    if !rms.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "weighted_branch_rms",
        }
        .into());
    }
    Ok(rms)
}

pub(crate) fn gain_multiplier(
    w_a: f64,
    w_b: f64,
    attack_gain_squared: f64,
    body_gain_squared: f64,
) -> Result<f64, PercussiveForceError> {
    let square = 1.0 + w_a * (attack_gain_squared - 1.0) + w_b * (body_gain_squared - 1.0);
    if square < 0.0 || !square.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "single_path_gain_multiplier",
        }
        .into());
    }
    Ok(square.sqrt())
}

pub(crate) fn checked_output_sample(
    value: f64,
    sample_index: usize,
) -> Result<f32, PercussiveForceError> {
    let output = value as f32;
    if output.is_finite() {
        Ok(output)
    } else {
        Err(PercussiveForceRefusal::NonFiniteOutput { sample_index }.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_power_masks_have_frozen_shape_and_zero_exterior() {
        let region = FrozenEventRegion {
            onset_frame: 4,
            attack_end_frame: 20,
            body_end_frame: 52,
        };
        let masks = EqualPowerMasks::for_region(60, region).unwrap();

        assert_eq!(masks.attack_body_crossfade_frames, 2);
        assert_eq!(masks.attack_body_crossfade_start_frame, 19);
        assert_eq!(masks.attack_body_crossfade_end_frame, 21);
        assert_eq!(masks.body_fade_start_frame, 50);
        assert_eq!(masks.phase_denominator, 3);
        assert!(masks.attack[..4].iter().all(|weight| *weight == 0.0));
        assert!(masks.body[..4].iter().all(|weight| *weight == 0.0));
        assert!(masks.attack[4..19].iter().all(|weight| *weight == 1.0));
        assert!(masks.body[21..50].iter().all(|weight| *weight == 1.0));
        assert!(masks.attack[52..].iter().all(|weight| *weight == 0.0));
        assert!(masks.body[52..].iter().all(|weight| *weight == 0.0));
        for frame in 19..21 {
            assert!((masks.attack[frame] + masks.body[frame] - 1.0).abs() <= 1.0e-12);
        }
        assert!(masks.body[50] > masks.body[51]);
        assert!(masks.body[51] > 0.0);
    }

    #[test]
    fn validation_rejects_non_finite_audio_before_analysis() {
        let samples = [0.0, f32::NAN, 0.0, 0.0];
        let error = validate_frozen_event(FrozenEventInput {
            interleaved_samples: &samples,
            sample_rate_hz: 48_000,
            channel_count: 1,
            region: FrozenEventRegion {
                onset_frame: 0,
                attack_end_frame: 1,
                body_end_frame: 4,
            },
        })
        .err()
        .expect("non-finite source must fail");
        assert_eq!(
            error,
            PercussiveForceError::InvalidInput(InvalidEventInput::NonFiniteSample {
                sample_index: 1
            })
        );
    }

    #[test]
    fn mask_builder_refuses_overlapping_frozen_ranges_instead_of_shortening_them() {
        let error = EqualPowerMasks::for_region(
            2,
            FrozenEventRegion {
                onset_frame: 0,
                attack_end_frame: 1,
                body_end_frame: 2,
            },
        )
        .expect_err("one-frame regions cannot contain both frozen transitions");
        assert!(matches!(
            error,
            PercussiveForceError::InvalidInput(InvalidEventInput::InvalidMaskRanges { .. })
        ));
    }

    #[test]
    fn effective_region_energy_is_an_all_channel_sum() {
        let region = FrozenEventRegion {
            onset_frame: 4,
            attack_end_frame: 20,
            body_end_frame: 52,
        };
        let masks = EqualPowerMasks::for_region(60, region).unwrap();
        let mono = (0..60)
            .map(|frame| 0.01 + frame as f32 * 0.001)
            .collect::<Vec<_>>();
        let stereo = mono
            .iter()
            .flat_map(|sample| [*sample, *sample])
            .collect::<Vec<_>>();

        let mono_energy = effective_region_energy(&mono, 1, &masks).unwrap();
        let stereo_energy = effective_region_energy(&stereo, 2, &masks).unwrap();

        assert!((stereo_energy.attack - 2.0 * mono_energy.attack).abs() <= 1.0e-15);
        assert!((stereo_energy.body - 2.0 * mono_energy.body).abs() <= 1.0e-15);
    }
}
