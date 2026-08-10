use super::common::{
    FrozenEventInput, FrozenEventRegion, PercussiveForceError, PercussiveForceRefusal,
    checked_output_sample, effective_region_energy, gain_multiplier, validate_frozen_event,
};

pub const F1_AB_ENERGY_REDISTRIBUTION_V1: &str = "f1_ab_energy_redistribution_v1";
const ATTACK_ENERGY_MULTIPLIER: f64 = 2.0;

#[derive(Clone, Debug, PartialEq)]
pub struct F1EnergyRedistributionPolicy {
    pub version_id: &'static str,
    pub attack_energy_multiplier: f64,
    pub attack_energy: f64,
    pub body_energy: f64,
    pub attack_gain_squared: f64,
    pub body_gain_squared: f64,
    pub attack_gain: f64,
    pub body_gain: f64,
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
pub struct F1RenderSet {
    pub combined: Vec<f32>,
    pub attack_only: Vec<f32>,
    pub body_only: Vec<f32>,
    pub policy: F1EnergyRedistributionPolicy,
}

pub fn render_f1_ab_energy_redistribution_v1(
    input: FrozenEventInput<'_>,
) -> Result<F1RenderSet, PercussiveForceError> {
    let event = validate_frozen_event(input)?;
    let energy = effective_region_energy(event.samples, event.channel_count, &event.masks)?;
    if energy.attack == 0.0 {
        return Err(PercussiveForceRefusal::MissingAttackEnergy.into());
    }
    if energy.body == 0.0 {
        return Err(PercussiveForceRefusal::MissingBodyEnergy.into());
    }

    let denominator = ATTACK_ENERGY_MULTIPLIER * energy.attack + energy.body;
    if denominator == 0.0 || !denominator.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f1_conservation_denominator",
        }
        .into());
    }
    let total = energy.attack + energy.body;
    let attack_gain_squared = ATTACK_ENERGY_MULTIPLIER * total / denominator;
    let body_gain_squared = total / denominator;
    let policy = F1EnergyRedistributionPolicy {
        version_id: F1_AB_ENERGY_REDISTRIBUTION_V1,
        attack_energy_multiplier: ATTACK_ENERGY_MULTIPLIER,
        attack_energy: energy.attack,
        body_energy: energy.body,
        attack_gain_squared,
        body_gain_squared,
        attack_gain: attack_gain_squared.sqrt(),
        body_gain: body_gain_squared.sqrt(),
        region: event.region,
        mask_definition: "centered_cos_squared_sin_squared_v1",
        attack_body_crossfade_frames: event.masks.attack_body_crossfade_frames,
        attack_body_crossfade_start_frame: event.masks.attack_body_crossfade_start_frame,
        attack_body_crossfade_end_frame: event.masks.attack_body_crossfade_end_frame,
        body_fade_frames: event.masks.body_fade_frames,
        body_fade_start_frame: event.masks.body_fade_start_frame,
        mask_phase_denominator: event.masks.phase_denominator,
    };

    let mut combined = event.samples.to_vec();
    let mut attack_only = event.samples.to_vec();
    let mut body_only = event.samples.to_vec();
    for frame in event.region.onset_frame..event.region.body_end_frame {
        let w_a = event.masks.attack[frame];
        let w_b = event.masks.body[frame];
        let combined_gain = gain_multiplier(
            w_a,
            w_b,
            policy.attack_gain_squared,
            policy.body_gain_squared,
        )?;
        let attack_gain = gain_multiplier(w_a, w_b, policy.attack_gain_squared, 1.0)?;
        let body_gain = gain_multiplier(w_a, w_b, 1.0, policy.body_gain_squared)?;
        for channel in 0..event.channel_count {
            let sample_index = frame * event.channel_count + channel;
            let source = f64::from(event.samples[sample_index]);
            combined[sample_index] = checked_output_sample(source * combined_gain, sample_index)?;
            attack_only[sample_index] = checked_output_sample(source * attack_gain, sample_index)?;
            body_only[sample_index] = checked_output_sample(source * body_gain, sample_index)?;
        }
    }

    Ok(F1RenderSet {
        combined,
        attack_only,
        body_only,
        policy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::percussive_force::{FrozenEventRegion, PercussiveForceError};

    fn test_input(samples: &[f32]) -> FrozenEventInput<'_> {
        FrozenEventInput {
            interleaved_samples: samples,
            sample_rate_hz: 48_000,
            channel_count: 2,
            region: FrozenEventRegion {
                onset_frame: 8,
                attack_end_frame: 24,
                body_end_frame: 72,
            },
        }
    }

    fn event_energy(samples: &[f32], start: usize, end: usize, channels: usize) -> f64 {
        samples[start * channels..end * channels]
            .iter()
            .map(|sample| f64::from(*sample).powi(2))
            .sum::<f64>()
            / channels as f64
    }

    fn region_mean_square(samples: &[f32], start: usize, end: usize, channels: usize) -> f64 {
        event_energy(samples, start, end, channels) / (end - start) as f64
    }

    #[test]
    fn combined_conserves_event_energy_and_moves_it_toward_attack() {
        let mut samples = vec![0.03125; 80 * 2];
        for frame in 8..24 {
            samples[frame * 2] = 0.35;
            samples[frame * 2 + 1] = -0.27;
        }
        for frame in 24..72 {
            samples[frame * 2] = 0.16;
            samples[frame * 2 + 1] = -0.12;
        }
        let rendered = render_f1_ab_energy_redistribution_v1(test_input(&samples))
            .expect("bounded event should render");

        let before = event_energy(&samples, 8, 72, 2);
        let after = event_energy(&rendered.combined, 8, 72, 2);
        assert!((after - before).abs() / before <= 2.0e-7);
        let source_attack_ms = region_mean_square(&samples, 8, 24, 2);
        let candidate_attack_ms = region_mean_square(&rendered.combined, 8, 24, 2);
        let source_body_ms = region_mean_square(&samples, 24, 72, 2);
        let candidate_body_ms = region_mean_square(&rendered.combined, 24, 72, 2);
        let source_ratio = source_attack_ms / source_body_ms;
        let candidate_ratio = candidate_attack_ms / candidate_body_ms;
        let ratio_tolerance =
            64.0 * f64::EPSILON * 1.0_f64.max(source_ratio.abs()).max(candidate_ratio.abs());
        assert!(candidate_ratio > source_ratio + ratio_tolerance);
        assert!(candidate_body_ms >= 0.5 * source_body_ms);
        assert!(
            rendered
                .combined
                .iter()
                .map(|sample| sample.abs())
                .fold(0.0_f32, f32::max)
                < 1.0
        );
        assert!(rendered.policy.attack_gain > 1.0);
        assert!(rendered.policy.attack_gain > rendered.policy.body_gain);
        assert!(rendered.policy.body_gain < 1.0);
        assert!(rendered.policy.attack_gain <= 2.0_f64.sqrt());
        assert!(rendered.policy.body_gain >= 0.5_f64.sqrt());
    }

    #[test]
    fn pre_onset_and_tail_are_bit_identical_for_every_ablation() {
        let samples = (0..160)
            .map(|index| ((index as f32 * 0.37).sin() * 0.4) + 0.01)
            .collect::<Vec<_>>();
        let rendered = render_f1_ab_energy_redistribution_v1(test_input(&samples))
            .expect("bounded event should render");
        for output in [
            &rendered.combined,
            &rendered.attack_only,
            &rendered.body_only,
        ] {
            assert_eq!(&output[..16], &samples[..16]);
            assert_eq!(&output[144..], &samples[144..]);
        }
    }

    #[test]
    fn resolved_policy_and_output_do_not_accept_a_source_name() {
        let samples = (0..160)
            .map(|index| ((index as f32 * 0.11).sin() * 0.3) + 0.02)
            .collect::<Vec<_>>();
        let first = render_f1_ab_energy_redistribution_v1(test_input(&samples)).unwrap();
        let second = render_f1_ab_energy_redistribution_v1(test_input(&samples)).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn missing_attack_or_body_energy_is_a_typed_refusal_without_epsilon_repair() {
        let mut no_attack = vec![0.0; 80 * 2];
        no_attack[50..144].fill(0.2);
        assert_eq!(
            render_f1_ab_energy_redistribution_v1(test_input(&no_attack)),
            Err(PercussiveForceError::Refused(
                PercussiveForceRefusal::MissingAttackEnergy
            ))
        );

        let mut no_body = vec![0.0; 80 * 2];
        no_body[16..44].fill(0.2);
        assert_eq!(
            render_f1_ab_energy_redistribution_v1(test_input(&no_body)),
            Err(PercussiveForceError::Refused(
                PercussiveForceRefusal::MissingBodyEnergy
            ))
        );
    }
}
