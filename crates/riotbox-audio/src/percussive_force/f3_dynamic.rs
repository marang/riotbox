//! Source-general Stage-A F3-v2: source-frozen-state-conditioned causal
//! envelope-contrast dynamics.
//!
//! This family deliberately has no residual-strength parameter. It derives a
//! sample-synchronous, positive gain trajectory from the frozen 1/8/20 ms
//! source envelopes and adds the unscaled source-aligned residual. Mechanical
//! checks may reject it; only the structured human gate can call it harder.
//! "Causal" applies to the right-aligned envelopes and controller after the
//! offline whole-source DC means, anatomy, and masks are frozen. It is not an
//! end-to-end streaming or audio-callback causality claim.

mod analysis;
mod preflight;

pub use preflight::{
    F3DynamicNearIdentityRecord, F3DynamicPreflightPolicyIdentity,
    F3DynamicSourceResponseDiversityIdentity, F3DynamicSyntheticOutcome,
    F3DynamicSyntheticPreflight, F3DynamicSyntheticPreflightAtRate, F3DynamicSyntheticRunIdentity,
    F3DynamicSyntheticRunRecord, f3_source_response_identities_are_diversity_separated,
    run_f3_dynamic_synthetic_preflight_v2,
};

use analysis::{
    NUMERICAL_EPSILON_MULTIPLIER, direction_metrics, finite_peak,
    phase_safe_multichannel_rms_envelopes_with_frozen_means, resolve_dynamic_analysis,
    resolved_controller_hashes,
};

use super::common::{
    FrozenEventInput, FrozenEventRegion, PercussiveForceError, PercussiveForceRefusal,
    checked_output_sample, validate_frozen_event,
};

pub const F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2: &str =
    "f3_causal_envelope_contrast_dynamic_residual_v2";

const OUTPUT_PEAK_STRICT_MAXIMUM: f64 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F3PcmEncoding {
    SignedPcm16,
    SignedPcm24,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct F3ControllerHashes {
    pub raw_attack_sha256: String,
    pub raw_body_sha256: String,
    pub attack_state_sha256: String,
    pub body_state_sha256: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F3DynamicPolicy {
    pub version_id: &'static str,
    pub pcm_encoding: F3PcmEncoding,
    pub pcm_valid_bits: u8,
    pub normalized_input_lsb: f64,
    pub lsb_mean_square_floor: f64,
    pub per_channel_dc_means: Vec<f64>,
    pub envelope_definition: &'static str,
    pub envelope_window_frames: [usize; 3],
    pub lookbehind_frames: usize,
    pub anatomy_baseline_rms: f64,
    pub controller_floor_rms: f64,
    pub attack_rise_coefficient: f64,
    pub attack_fall_coefficient: f64,
    pub body_rise_coefficient: f64,
    pub body_fall_coefficient: f64,
    pub attack_source_mean_square: f64,
    pub body_source_mean_square: f64,
    pub attack_branch_mean_square: f64,
    pub body_branch_mean_square: f64,
    pub attack_contribution_ratio: f64,
    pub body_contribution_ratio: f64,
    pub residual_scales: [f64; 2],
    pub raw_attack_peak: f64,
    pub raw_body_peak: f64,
    pub attack_state_peak: f64,
    pub body_state_peak: f64,
    pub source_attack_fast_to_slow_ratio: f64,
    pub attack_only_fast_to_slow_ratio: f64,
    pub combined_attack_fast_to_slow_ratio: f64,
    pub source_body_fast_to_context_ratio: f64,
    pub body_only_fast_to_context_ratio: f64,
    pub combined_body_fast_to_context_ratio: f64,
    pub ablation_identity_normalized_error: f64,
    pub controller_hashes: F3ControllerHashes,
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
pub struct F3DynamicRenderSet {
    pub combined: Vec<f32>,
    pub attack_only: Vec<f32>,
    pub body_only: Vec<f32>,
    pub policy: F3DynamicPolicy,
}

pub fn render_f3_causal_envelope_contrast_dynamic_residual_v2(
    input: FrozenEventInput<'_>,
    pcm_encoding: F3PcmEncoding,
) -> Result<F3DynamicRenderSet, PercussiveForceError> {
    let event = validate_frozen_event(input)?;
    let analysis = resolve_dynamic_analysis(&event, pcm_encoding)?;

    let source_f64 = event
        .samples
        .iter()
        .map(|sample| f64::from(*sample))
        .collect::<Vec<_>>();
    let mut combined = event.samples.to_vec();
    let mut attack_only = event.samples.to_vec();
    let mut body_only = event.samples.to_vec();
    let mut peak = 0.0_f64;
    for sample_index in 0..source_f64.len() {
        let dry = source_f64[sample_index];
        let attack = analysis.attack_delta[sample_index];
        let body = analysis.body_delta[sample_index];
        let combined_sample = dry + attack + body;
        peak = peak.max(combined_sample.abs());
        combined[sample_index] = checked_output_sample(combined_sample, sample_index)?;
        attack_only[sample_index] = checked_output_sample(dry + attack, sample_index)?;
        body_only[sample_index] = checked_output_sample(dry + body, sample_index)?;
    }
    if peak >= OUTPUT_PEAK_STRICT_MAXIMUM {
        return Err(PercussiveForceRefusal::OutputPeakWithoutHeadroom {
            peak,
            strict_maximum: OUTPUT_PEAK_STRICT_MAXIMUM,
        }
        .into());
    }

    let ablation_identity_error =
        ablation_identity_error(&source_f64, &combined, &attack_only, &body_only)?;
    let ablation_maximum = NUMERICAL_EPSILON_MULTIPLIER * f64::from(f32::EPSILON);
    if ablation_identity_error > ablation_maximum {
        return Err(PercussiveForceRefusal::DynamicAblationIdentityMismatch {
            normalized_error: ablation_identity_error,
            maximum_error: ablation_maximum,
        }
        .into());
    }

    let attack_only_envelopes = phase_safe_multichannel_rms_envelopes_with_frozen_means(
        &attack_only,
        event.channel_count,
        analysis.envelope_window_frames,
        &analysis.envelopes.channel_means,
    )?;
    let body_only_envelopes = phase_safe_multichannel_rms_envelopes_with_frozen_means(
        &body_only,
        event.channel_count,
        analysis.envelope_window_frames,
        &analysis.envelopes.channel_means,
    )?;
    let combined_envelopes = phase_safe_multichannel_rms_envelopes_with_frozen_means(
        &combined,
        event.channel_count,
        analysis.envelope_window_frames,
        &analysis.envelopes.channel_means,
    )?;
    let source_direction =
        direction_metrics(&analysis.envelopes, &event.masks.attack, &event.masks.body)?;
    let attack_direction = direction_metrics(
        &attack_only_envelopes,
        &event.masks.attack,
        &event.masks.body,
    )?;
    let body_direction =
        direction_metrics(&body_only_envelopes, &event.masks.attack, &event.masks.body)?;
    let combined_direction =
        direction_metrics(&combined_envelopes, &event.masks.attack, &event.masks.body)?;
    require_strict_direction_increase(
        "attack_only_fast_to_slow",
        source_direction.attack_fast_to_slow,
        attack_direction.attack_fast_to_slow,
    )?;
    require_strict_direction_increase(
        "body_only_fast_to_context",
        source_direction.body_fast_to_context,
        body_direction.body_fast_to_context,
    )?;

    let controller_hashes = resolved_controller_hashes(
        &analysis,
        event.sample_rate_hz,
        event.channel_count,
        event.region,
    )?;

    Ok(F3DynamicRenderSet {
        combined,
        attack_only,
        body_only,
        policy: F3DynamicPolicy {
            version_id: F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2,
            pcm_encoding,
            pcm_valid_bits: pcm_encoding.valid_bits(),
            normalized_input_lsb: analysis.input_lsb,
            lsb_mean_square_floor: analysis.lsb_mean_square_floor,
            per_channel_dc_means: analysis.envelopes.channel_means,
            envelope_definition: "standard_phase_safe_right_aligned_multichannel_rms_v1",
            envelope_window_frames: analysis.envelope_window_frames,
            lookbehind_frames: analysis.lookbehind_frames,
            anatomy_baseline_rms: analysis.baseline_rms,
            controller_floor_rms: analysis.controller_floor_rms,
            attack_rise_coefficient: analysis.attack_rise_coefficient,
            attack_fall_coefficient: analysis.attack_fall_coefficient,
            body_rise_coefficient: analysis.body_rise_coefficient,
            body_fall_coefficient: analysis.body_fall_coefficient,
            attack_source_mean_square: analysis.source_attack_mean_square,
            body_source_mean_square: analysis.source_body_mean_square,
            attack_branch_mean_square: analysis.attack_branch_mean_square,
            body_branch_mean_square: analysis.body_branch_mean_square,
            attack_contribution_ratio: analysis.attack_contribution_ratio,
            body_contribution_ratio: analysis.body_contribution_ratio,
            residual_scales: [1.0, 1.0],
            raw_attack_peak: finite_peak(&analysis.raw_attack)?,
            raw_body_peak: finite_peak(&analysis.raw_body)?,
            attack_state_peak: finite_peak(&analysis.attack_state)?,
            body_state_peak: finite_peak(&analysis.body_state)?,
            source_attack_fast_to_slow_ratio: source_direction.attack_fast_to_slow,
            attack_only_fast_to_slow_ratio: attack_direction.attack_fast_to_slow,
            combined_attack_fast_to_slow_ratio: combined_direction.attack_fast_to_slow,
            source_body_fast_to_context_ratio: source_direction.body_fast_to_context,
            body_only_fast_to_context_ratio: body_direction.body_fast_to_context,
            combined_body_fast_to_context_ratio: combined_direction.body_fast_to_context,
            ablation_identity_normalized_error: ablation_identity_error,
            controller_hashes,
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

fn require_strict_direction_increase(
    metric: &'static str,
    source: f64,
    candidate: f64,
) -> Result<(), PercussiveForceError> {
    let tolerance = NUMERICAL_EPSILON_MULTIPLIER
        * f64::EPSILON
        * 1.0_f64.max(source.abs()).max(candidate.abs());
    if candidate <= source + tolerance {
        return Err(PercussiveForceRefusal::DynamicDirectionNotIncreased {
            metric,
            source_value: source,
            candidate_value: candidate,
        }
        .into());
    }
    Ok(())
}

fn ablation_identity_error(
    source: &[f64],
    combined: &[f32],
    attack_only: &[f32],
    body_only: &[f32],
) -> Result<f64, PercussiveForceError> {
    let mut errors = [0.0_f64; 3];
    let mut references = [0.0_f64; 3];
    for index in 0..source.len() {
        let combined_delta = f64::from(combined[index]) - source[index];
        let attack_delta = f64::from(attack_only[index]) - source[index];
        let body_delta = f64::from(body_only[index]) - source[index];
        let residuals = [
            combined_delta - attack_delta - body_delta,
            (f64::from(combined[index]) - f64::from(attack_only[index])) - body_delta,
            (f64::from(combined[index]) - f64::from(body_only[index])) - attack_delta,
        ];
        let expected = [combined_delta, body_delta, attack_delta];
        for slot in 0..3 {
            errors[slot] += residuals[slot] * residuals[slot];
            references[slot] += expected[slot] * expected[slot];
        }
    }
    let mut maximum = 0.0_f64;
    for slot in 0..3 {
        if references[slot] <= 0.0 {
            return Err(PercussiveForceRefusal::NonFiniteAnalysis {
                stage: "f3_v2_ablation_reference",
            }
            .into());
        }
        maximum = maximum.max((errors[slot] / references[slot]).sqrt());
    }
    if !maximum.is_finite() {
        return Err(PercussiveForceRefusal::NonFiniteAnalysis {
            stage: "f3_v2_ablation_identity",
        }
        .into());
    }
    Ok(maximum)
}

#[cfg(test)]
mod tests {
    use super::preflight::{ProbeShape, synthetic_probe};
    use super::*;

    #[test]
    fn source_independent_preflight_passes_every_registered_rate() {
        let preflight = run_f3_dynamic_synthetic_preflight_v2();
        assert!(!preflight.source_audio_accessed);
        assert_eq!(preflight.sample_rates_hz, [44_100, 48_000, 96_000]);
        assert!(
            preflight.source_response_cross_rate_non_diversity_pass,
            "{preflight:#?}"
        );
        let expected_response_identities = [
            (
                [2, 9, 3, 8, 4, 8, 2, 4],
                "1b6005e428a738ca882526e0a5febf6e36a855fec344cd373bbe934a8efc6954",
            ),
            (
                [2, 9, 3, 7, 4, 8, 2, 3],
                "93d41ec4edb295aaef0df334e2d00efb04a1474ea983dfb1bbe44336ab5b47c5",
            ),
            (
                [2, 9, 3, 7, 4, 8, 2, 3],
                "93d41ec4edb295aaef0df334e2d00efb04a1474ea983dfb1bbe44336ab5b47c5",
            ),
        ];
        for (case, (expected_quantized, expected_sha256)) in
            preflight.cases.iter().zip(expected_response_identities)
        {
            assert!(case.constant_dynamic_refusal_pass, "{case:#?}");
            assert!(case.constant_controller_zero_pass, "{case:#?}");
            assert!(case.step_render_pass, "{case:#?}");
            assert_eq!(
                case.first_raw_attack_frame,
                Some(case.expected_attack_state_frame),
                "{case:#?}"
            );
            assert_eq!(
                case.first_attack_state_frame,
                Some(case.expected_attack_state_frame),
                "{case:#?}"
            );
            assert!(
                case.first_raw_body_frame
                    .is_some_and(|frame| frame >= case.expected_body_state_frame_range[0]
                        && frame < case.expected_body_state_frame_range[1]),
                "{case:#?}"
            );
            assert!(
                case.first_body_state_frame
                    .is_some_and(|frame| frame >= case.expected_body_state_frame_range[0]
                        && frame < case.expected_body_state_frame_range[1]),
                "{case:#?}"
            );
            assert!(case.source_frozen_activation_causality_pass, "{case:#?}");
            assert!(case.actionable_policy_invariance_pass, "{case:#?}");
            assert!(case.source_response_invariance_pass, "{case:#?}");
            assert!(case.global_near_identity_pass, "{case:#?}");
            assert_eq!(case.near_identity.len(), 4, "{case:#?}");
            assert!(
                case.near_identity.iter().all(|record| record.passed),
                "{case:#?}"
            );
            assert!(case.deterministic_repeat_pass, "{case:#?}");
            assert!(case.complete_hash_coverage_pass, "{case:#?}");
            assert_eq!(case.runs.len(), 5, "{case:#?}");
            assert!(
                case.runs.iter().all(|run| run.exact_repeat_hash_match),
                "{case:#?}"
            );
            let constant = &case.runs[0];
            assert_eq!(
                constant.first.outcome,
                F3DynamicSyntheticOutcome::RefusedMissingAttackAndBodyDynamicContrast
            );
            assert!(constant.first.controller_hashes.is_some());
            assert!(constant.first.actionable_policy.is_some());
            assert!(constant.first.source_response_diversity.is_none());
            assert!(constant.first.combined_output_sha256.is_none());
            assert!(constant.first.renderer_actionable_policy_match.is_none());
            assert_eq!(
                constant.first.refusal_or_failure,
                Some(PercussiveForceError::Refused(
                    PercussiveForceRefusal::MissingDynamicContrast {
                        attack_missing: true,
                        body_missing: true,
                    }
                ))
            );
            for rendered in &case.runs[1..] {
                assert_eq!(rendered.first.outcome, F3DynamicSyntheticOutcome::Rendered);
                assert!(rendered.first.controller_hashes.is_some());
                assert!(rendered.first.combined_output_sha256.is_some());
                assert!(rendered.first.attack_only_output_sha256.is_some());
                assert!(rendered.first.body_only_output_sha256.is_some());
                assert!(rendered.first.actionable_policy.is_some());
                assert!(rendered.first.source_response_diversity.is_some());
                assert_eq!(rendered.first.renderer_controller_hash_match, Some(true));
                assert_eq!(rendered.first.renderer_actionable_policy_match, Some(true));
                assert!(rendered.first.refusal_or_failure.is_none());
                let response = rendered.first.source_response_diversity.as_ref().unwrap();
                assert_eq!(response.quantized_summary, expected_quantized);
                assert_eq!(response.sha256, expected_sha256);
            }
            assert!(case.passed, "{case:#?}");
        }
        assert!(preflight.passed, "{preflight:#?}");
    }

    #[test]
    fn render_has_no_strength_knob_and_preserves_sample_support_and_polarity() {
        let probe = synthetic_probe(48_000, 1.0 / 64.0, ProbeShape::StepBody, 1.0, 1.0);
        let rendered = render_f3_causal_envelope_contrast_dynamic_residual_v2(
            probe.input(),
            F3PcmEncoding::SignedPcm16,
        )
        .expect("frozen synthetic dynamic probe");
        assert_eq!(rendered.policy.residual_scales, [1.0, 1.0]);
        assert_eq!(
            &rendered.combined[..probe.region.onset_frame * 2],
            &probe.samples[..probe.region.onset_frame * 2]
        );
        assert_eq!(
            &rendered.combined[probe.region.body_end_frame * 2..],
            &probe.samples[probe.region.body_end_frame * 2..]
        );
        for (source, candidate) in probe.samples.iter().zip(&rendered.combined) {
            if *source == 0.0 {
                assert_eq!(*candidate, 0.0);
            } else {
                assert_eq!(source.is_sign_positive(), candidate.is_sign_positive());
                let gain = f64::from(*candidate) / f64::from(*source);
                assert!((1.0 - 1.0e-6..=2.0 + 1.0e-6).contains(&gain));
            }
        }
    }

    #[test]
    fn pcm_encoding_binds_exact_lsb_without_a_float_parameter() {
        assert_eq!(
            F3PcmEncoding::SignedPcm16.normalized_lsb(),
            2.0_f64.powi(-15)
        );
        assert_eq!(
            F3PcmEncoding::SignedPcm24.normalized_lsb(),
            2.0_f64.powi(-23)
        );
    }

    #[test]
    fn candidate_direction_uses_source_frozen_dc_means() {
        let mut probe = synthetic_probe(48_000, 1.0 / 64.0, ProbeShape::StepBody, 1.0, 1.0);
        for sample in &mut probe.samples {
            *sample += 0.08;
        }
        let event = validate_frozen_event(probe.input()).unwrap();
        let source_analysis = resolve_dynamic_analysis(&event, F3PcmEncoding::SignedPcm16)
            .expect("biased source remains a valid dynamic probe");
        let rendered = render_f3_causal_envelope_contrast_dynamic_residual_v2(
            probe.input(),
            F3PcmEncoding::SignedPcm16,
        )
        .expect("frozen-mean candidate direction should pass");

        let frozen = phase_safe_multichannel_rms_envelopes_with_frozen_means(
            &rendered.combined,
            2,
            source_analysis.envelope_window_frames,
            &source_analysis.envelopes.channel_means,
        )
        .unwrap();
        let own = analysis::phase_safe_multichannel_rms_envelopes(
            &rendered.combined,
            2,
            source_analysis.envelope_window_frames,
        )
        .unwrap();
        let frozen_direction =
            direction_metrics(&frozen, &event.masks.attack, &event.masks.body).unwrap();
        let own_direction =
            direction_metrics(&own, &event.masks.attack, &event.masks.body).unwrap();

        assert_eq!(
            rendered.policy.combined_attack_fast_to_slow_ratio,
            frozen_direction.attack_fast_to_slow
        );
        assert_eq!(
            rendered.policy.combined_body_fast_to_context_ratio,
            frozen_direction.body_fast_to_context
        );
        assert!(
            (own_direction.attack_fast_to_slow - frozen_direction.attack_fast_to_slow).abs()
                > 1.0e-4
        );
        assert!(
            (own_direction.body_fast_to_context - frozen_direction.body_fast_to_context).abs()
                > 1.0e-4
        );
    }
}
