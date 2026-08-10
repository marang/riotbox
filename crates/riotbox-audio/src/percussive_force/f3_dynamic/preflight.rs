//! Synthetic-only F3-v2 falsification harness.
//!
//! Every vector is generated in memory. This module has no file or source
//! access seam and cannot turn a passing preflight into a musical claim.

use std::f64::consts::TAU;

mod identity;

pub use identity::{
    F3DynamicPreflightPolicyIdentity, F3DynamicSourceResponseDiversityIdentity,
    f3_source_response_identities_are_diversity_separated,
};

use identity::{
    actionable_policy_identity, actionable_policy_identity_is_canonical,
    controller_hashes_are_canonical, is_lowercase_sha256, source_response_diversity_identity,
    source_response_identity_bucket_equal, source_response_identity_is_canonical,
    synthetic_artifact_hash, synthetic_outcome_hash,
};

use super::analysis::{
    DynamicControllerTrace, NUMERICAL_EPSILON_MULTIPLIER, analyze_dynamic_controller_trace,
    frames_for_ms, trace_controller_hashes,
};
use super::{
    F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2, F3ControllerHashes, F3DynamicRenderSet,
    F3PcmEncoding, render_f3_causal_envelope_contrast_dynamic_residual_v2,
};
use crate::percussive_force::common::validate_frozen_event;
use crate::percussive_force::{
    FrozenEventInput, FrozenEventRegion, PercussiveForceError, PercussiveForceRefusal,
};

const NEAR_IDENTITY_DELTA_RMS_MINIMUM: f64 = 0.05;

const CONSTANT_PROBE_ID: &str = "constant_quadrature_v1";
const STEP_PROBE_ID: &str = "step_body_quadrature_v1";
const SCALE_PROBE_ID: &str = "amplitude_scale_equivariance_v1";
const POLARITY_PROBE_ID: &str = "polarity_equivariance_v1";
const CARRIER_PROBE_ID: &str = "carrier_invariance_v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F3DynamicSyntheticOutcome {
    Rendered,
    RefusedMissingAttackAndBodyDynamicContrast,
    UnexpectedFailure,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F3DynamicSyntheticRunIdentity {
    pub raw_input_sha256: String,
    pub controller_hashes: Option<F3ControllerHashes>,
    pub combined_output_sha256: Option<String>,
    pub attack_only_output_sha256: Option<String>,
    pub body_only_output_sha256: Option<String>,
    pub actionable_policy: Option<F3DynamicPreflightPolicyIdentity>,
    pub source_response_diversity: Option<F3DynamicSourceResponseDiversityIdentity>,
    pub renderer_controller_hash_match: Option<bool>,
    pub renderer_actionable_policy_match: Option<bool>,
    pub outcome: F3DynamicSyntheticOutcome,
    pub refusal_or_failure: Option<PercussiveForceError>,
    pub outcome_sha256: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F3DynamicSyntheticRunRecord {
    pub probe_id: &'static str,
    pub first: F3DynamicSyntheticRunIdentity,
    pub repeated: F3DynamicSyntheticRunIdentity,
    pub exact_repeat_hash_match: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F3DynamicNearIdentityRecord {
    pub probe_id: &'static str,
    pub normalized_delta_rms: Option<f64>,
    pub minimum: f64,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F3DynamicSyntheticPreflightAtRate {
    pub sample_rate_hz: u32,
    pub constant_dynamic_refusal_pass: bool,
    pub constant_controller_zero_pass: bool,
    pub step_render_pass: bool,
    pub first_raw_attack_frame: Option<usize>,
    pub first_attack_state_frame: Option<usize>,
    pub expected_attack_state_frame: usize,
    pub first_raw_body_frame: Option<usize>,
    pub first_body_state_frame: Option<usize>,
    pub expected_body_state_frame_range: [usize; 2],
    pub strict_causality_pass: bool,
    pub amplitude_scale_controller_max_error: f64,
    pub amplitude_scale_output_max_error: f64,
    pub polarity_controller_max_error: f64,
    pub polarity_output_max_error: f64,
    pub carrier_controller_max_error: f64,
    pub actionable_policy_invariance_pass: bool,
    pub source_response_invariance_pass: bool,
    pub near_identity: Vec<F3DynamicNearIdentityRecord>,
    pub global_near_identity_pass: bool,
    pub deterministic_repeat_pass: bool,
    pub complete_hash_coverage_pass: bool,
    pub runs: Vec<F3DynamicSyntheticRunRecord>,
    pub step_input_sha256: String,
    pub step_combined_sha256: String,
    pub step_controller_hashes: Option<F3ControllerHashes>,
    pub step_actionable_policy: Option<F3DynamicPreflightPolicyIdentity>,
    pub step_source_response_diversity: Option<F3DynamicSourceResponseDiversityIdentity>,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct F3DynamicSyntheticPreflight {
    pub family_version: &'static str,
    pub sample_rates_hz: [u32; 3],
    pub cases: Vec<F3DynamicSyntheticPreflightAtRate>,
    pub source_response_cross_rate_non_diversity_pass: bool,
    pub passed: bool,
    pub source_audio_accessed: bool,
}

pub fn run_f3_dynamic_synthetic_preflight_v2() -> F3DynamicSyntheticPreflight {
    let sample_rates_hz = [44_100, 48_000, 96_000];
    let cases = sample_rates_hz
        .into_iter()
        .map(run_synthetic_preflight_at_rate)
        .collect::<Vec<_>>();
    let source_response_cross_rate_non_diversity_pass =
        source_response_cross_rate_non_diversity_pass(&cases);
    let passed =
        cases.iter().all(|case| case.passed) && source_response_cross_rate_non_diversity_pass;
    F3DynamicSyntheticPreflight {
        family_version: F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2,
        sample_rates_hz,
        cases,
        source_response_cross_rate_non_diversity_pass,
        passed,
        source_audio_accessed: false,
    }
}

fn source_response_cross_rate_non_diversity_pass(
    cases: &[F3DynamicSyntheticPreflightAtRate],
) -> bool {
    if cases.len() != 3 {
        return false;
    }
    [
        STEP_PROBE_ID,
        SCALE_PROBE_ID,
        POLARITY_PROBE_ID,
        CARRIER_PROBE_ID,
    ]
    .into_iter()
    .all(|probe_id| {
        let identities = cases
            .iter()
            .filter_map(|case| {
                case.runs
                    .iter()
                    .find(|run| run.probe_id == probe_id)
                    .and_then(|run| run.first.source_response_diversity.as_ref())
            })
            .collect::<Vec<_>>();
        identities.len() == cases.len()
            && (0..identities.len()).all(|first| {
                (first + 1..identities.len()).all(|second| {
                    source_response_cross_rate_pair_non_diversity_pass(
                        identities[first],
                        identities[second],
                    )
                })
            })
    })
}

fn source_response_cross_rate_pair_non_diversity_pass(
    first: &F3DynamicSourceResponseDiversityIdentity,
    second: &F3DynamicSourceResponseDiversityIdentity,
) -> bool {
    source_response_identity_is_canonical(first)
        && source_response_identity_is_canonical(second)
        && first.domain == second.domain
        && first.family_id == second.family_id
        && first.field_order == second.field_order
        && first.residual_scales == second.residual_scales
        && first
            .quantized_summary
            .iter()
            .zip(second.quantized_summary)
            .all(|(first, second)| first.abs_diff(second) < 2)
}

fn run_synthetic_preflight_at_rate(sample_rate_hz: u32) -> F3DynamicSyntheticPreflightAtRate {
    let constant = synthetic_probe(sample_rate_hz, 1.0 / 64.0, ProbeShape::Constant, 1.0, 1.0);
    let step = synthetic_probe(sample_rate_hz, 1.0 / 64.0, ProbeShape::StepBody, 1.0, 1.0);
    let half = synthetic_probe(sample_rate_hz, 1.0 / 64.0, ProbeShape::StepBody, 0.5, 1.0);
    let inverted = synthetic_probe(sample_rate_hz, 1.0 / 64.0, ProbeShape::StepBody, 1.0, -1.0);
    let high = synthetic_probe(sample_rate_hz, 1.0 / 8.0, ProbeShape::StepBody, 1.0, 1.0);

    let constant_pair = execute_probe_pair(CONSTANT_PROBE_ID, &constant);
    let step_pair = execute_probe_pair(STEP_PROBE_ID, &step);
    let half_pair = execute_probe_pair(SCALE_PROBE_ID, &half);
    let inverted_pair = execute_probe_pair(POLARITY_PROBE_ID, &inverted);
    let high_pair = execute_probe_pair(CARRIER_PROBE_ID, &high);

    let constant_dynamic_refusal_pass = constant_pair.first.identity.outcome
        == F3DynamicSyntheticOutcome::RefusedMissingAttackAndBodyDynamicContrast;
    let controller_tolerance = NUMERICAL_EPSILON_MULTIPLIER * f64::from(f32::EPSILON);
    let constant_controller_zero_pass = constant_pair.first.trace.as_ref().is_some_and(|trace| {
        controller_values(trace).into_iter().all(|values| {
            values.iter().all(|value| {
                value.is_finite()
                    && value.to_bits() != (-0.0_f64).to_bits()
                    && *value >= 0.0
                    && value.abs() <= controller_tolerance
            })
        })
    });
    let step_render_pass = step_pair.first.render.is_some();

    let mut first_raw_attack_frame = None;
    let mut first_attack_state_frame = None;
    let mut first_raw_body_frame = None;
    let mut first_body_state_frame = None;
    let mut strict_causality_pass = false;
    let mut amplitude_scale_controller_max_error = f64::INFINITY;
    let mut polarity_controller_max_error = f64::INFINITY;
    let mut carrier_controller_max_error = f64::INFINITY;
    if let (Some(base), Some(scaled), Some(polarity), Some(carrier)) = (
        &step_pair.first.trace,
        &half_pair.first.trace,
        &inverted_pair.first.trace,
        &high_pair.first.trace,
    ) {
        first_raw_attack_frame = first_positive(&base.raw_attack);
        first_attack_state_frame = first_positive(&base.attack_state);
        first_raw_body_frame = first_positive(&base.raw_body);
        first_body_state_frame = first_positive(&base.body_state);
        let expected_body_end = step.high_end + frames_for_ms(sample_rate_hz, 1);
        strict_causality_pass = first_raw_attack_frame == Some(step.region.onset_frame)
            && first_attack_state_frame == Some(step.region.onset_frame)
            && first_raw_body_frame
                .is_some_and(|frame| frame >= step.high_end && frame < expected_body_end)
            && first_body_state_frame
                .is_some_and(|frame| frame >= step.high_end && frame < expected_body_end)
            && canonical_positive_zero_before(&base.raw_attack, step.region.onset_frame)
            && canonical_positive_zero_before(&base.attack_state, step.region.onset_frame)
            && first_raw_body_frame
                .is_some_and(|frame| canonical_positive_zero_before(&base.raw_body, frame))
            && first_body_state_frame
                .is_some_and(|frame| canonical_positive_zero_before(&base.body_state, frame));
        amplitude_scale_controller_max_error = controller_max_error(base, scaled);
        polarity_controller_max_error = controller_max_error(base, polarity);
        carrier_controller_max_error = controller_max_error(base, carrier);
    }

    let amplitude_scale_output_max_error = match (&step_pair.first.render, &half_pair.first.render)
    {
        (Some(full), Some(scaled)) => full
            .combined
            .iter()
            .zip(&scaled.combined)
            .map(|(full, half)| (f64::from(*full) - 2.0 * f64::from(*half)).abs())
            .fold(0.0_f64, f64::max),
        _ => f64::INFINITY,
    };
    let polarity_output_max_error = match (&step_pair.first.render, &inverted_pair.first.render) {
        (Some(positive), Some(negative)) => positive
            .combined
            .iter()
            .zip(&negative.combined)
            .map(|(positive, negative)| (f64::from(*positive) + f64::from(*negative)).abs())
            .fold(0.0_f64, f64::max),
        _ => f64::INFINITY,
    };
    let actionable_policy_invariance_pass = step_pair
        .record
        .first
        .actionable_policy
        .as_ref()
        .is_some_and(|reference| {
            reference.residual_scales == [1.0, 1.0]
                && [
                    &constant_pair.record,
                    &half_pair.record,
                    &inverted_pair.record,
                    &high_pair.record,
                ]
                .into_iter()
                .all(|record| record.first.actionable_policy.as_ref() == Some(reference))
        });
    let source_response_invariance_pass = step_pair
        .record
        .first
        .source_response_diversity
        .as_ref()
        .is_some_and(|reference| {
            [&half_pair.record, &inverted_pair.record, &high_pair.record]
                .into_iter()
                .all(|record| {
                    record
                        .first
                        .source_response_diversity
                        .as_ref()
                        .is_some_and(|candidate| {
                            source_response_identity_bucket_equal(reference, candidate)
                        })
                })
        });
    let output_tolerance = controller_tolerance
        * step_pair
            .first
            .render
            .as_ref()
            .map_or(1.0, |render| 1.0_f64.max(peak_abs(&render.combined)));

    let near_identity = [
        (&step_pair, &step),
        (&half_pair, &half),
        (&inverted_pair, &inverted),
        (&high_pair, &high),
    ]
    .into_iter()
    .map(|(execution, probe)| {
        let normalized_delta_rms = execution.first.render.as_ref().and_then(|render| {
            near_identity_delta_rms(&probe.samples, &render.combined, 2, probe.region)
        });
        F3DynamicNearIdentityRecord {
            probe_id: execution.record.probe_id,
            normalized_delta_rms,
            minimum: NEAR_IDENTITY_DELTA_RMS_MINIMUM,
            passed: normalized_delta_rms
                .is_some_and(|value| value >= NEAR_IDENTITY_DELTA_RMS_MINIMUM),
        }
    })
    .collect::<Vec<_>>();
    let global_near_identity_pass = near_identity.iter().all(|record| record.passed);

    let deterministic_repeat_pass = [
        &constant_pair.record,
        &step_pair.record,
        &half_pair.record,
        &inverted_pair.record,
        &high_pair.record,
    ]
    .into_iter()
    .all(|record| record.exact_repeat_hash_match);
    let complete_hash_coverage_pass = expected_refusal_hash_coverage(&constant_pair.record)
        && [
            &step_pair.record,
            &half_pair.record,
            &inverted_pair.record,
            &high_pair.record,
        ]
        .into_iter()
        .all(expected_render_hash_coverage);
    let expected_body_end = step.high_end + frames_for_ms(sample_rate_hz, 1);
    let all_rendered = [
        &step_pair.first,
        &half_pair.first,
        &inverted_pair.first,
        &high_pair.first,
    ]
    .into_iter()
    .all(|execution| execution.render.is_some());
    let passed = constant_dynamic_refusal_pass
        && constant_controller_zero_pass
        && step_render_pass
        && all_rendered
        && strict_causality_pass
        && amplitude_scale_controller_max_error <= controller_tolerance
        && amplitude_scale_output_max_error <= output_tolerance
        && polarity_controller_max_error <= controller_tolerance
        && polarity_output_max_error <= output_tolerance
        && carrier_controller_max_error <= controller_tolerance
        && actionable_policy_invariance_pass
        && source_response_invariance_pass
        && global_near_identity_pass
        && deterministic_repeat_pass
        && complete_hash_coverage_pass;

    let step_input_sha256 = step_pair.record.first.raw_input_sha256.clone();
    let step_combined_sha256 = step_pair
        .record
        .first
        .combined_output_sha256
        .clone()
        .unwrap_or_default();
    let step_controller_hashes = step_pair.record.first.controller_hashes.clone();
    let step_actionable_policy = step_pair.record.first.actionable_policy.clone();
    let step_source_response_diversity = step_pair.record.first.source_response_diversity.clone();
    let runs = vec![
        constant_pair.record,
        step_pair.record,
        half_pair.record,
        inverted_pair.record,
        high_pair.record,
    ];

    F3DynamicSyntheticPreflightAtRate {
        sample_rate_hz,
        constant_dynamic_refusal_pass,
        constant_controller_zero_pass,
        step_render_pass,
        first_raw_attack_frame,
        first_attack_state_frame,
        expected_attack_state_frame: step.region.onset_frame,
        first_raw_body_frame,
        first_body_state_frame,
        expected_body_state_frame_range: [step.high_end, expected_body_end],
        strict_causality_pass,
        amplitude_scale_controller_max_error,
        amplitude_scale_output_max_error,
        polarity_controller_max_error,
        polarity_output_max_error,
        carrier_controller_max_error,
        actionable_policy_invariance_pass,
        source_response_invariance_pass,
        near_identity,
        global_near_identity_pass,
        deterministic_repeat_pass,
        complete_hash_coverage_pass,
        runs,
        step_input_sha256,
        step_combined_sha256,
        step_controller_hashes,
        step_actionable_policy,
        step_source_response_diversity,
        passed,
    }
}

struct ProbeExecution {
    identity: F3DynamicSyntheticRunIdentity,
    trace: Option<DynamicControllerTrace>,
    render: Option<F3DynamicRenderSet>,
}

struct ProbeExecutionPair {
    record: F3DynamicSyntheticRunRecord,
    first: ProbeExecution,
}

fn execute_probe_pair(probe_id: &'static str, probe: &SyntheticProbe) -> ProbeExecutionPair {
    let first = execute_probe(probe);
    let repeated = execute_probe(probe);
    let exact_repeat_hash_match = first.identity == repeated.identity;
    let record = F3DynamicSyntheticRunRecord {
        probe_id,
        first: first.identity.clone(),
        repeated: repeated.identity,
        exact_repeat_hash_match,
    };
    ProbeExecutionPair { record, first }
}

fn execute_probe(probe: &SyntheticProbe) -> ProbeExecution {
    let trace = validate_frozen_event(probe.input())
        .and_then(|event| analyze_dynamic_controller_trace(&event, F3PcmEncoding::SignedPcm16));
    let controller_hashes = trace.as_ref().ok().and_then(|trace| {
        trace_controller_hashes(trace, probe.sample_rate_hz, 2, probe.region).ok()
    });
    let render = render_f3_causal_envelope_contrast_dynamic_residual_v2(
        probe.input(),
        F3PcmEncoding::SignedPcm16,
    );
    let outcome = match &render {
        Ok(_) => F3DynamicSyntheticOutcome::Rendered,
        Err(PercussiveForceError::Refused(PercussiveForceRefusal::MissingDynamicContrast {
            attack_missing: true,
            body_missing: true,
        })) => F3DynamicSyntheticOutcome::RefusedMissingAttackAndBodyDynamicContrast,
        Err(_) => F3DynamicSyntheticOutcome::UnexpectedFailure,
    };
    let refusal_or_failure = render.as_ref().err().cloned();
    let rendered = render.ok();
    let actionable_policy = actionable_policy_identity([1.0, 1.0]).ok();
    let source_response_diversity = rendered.as_ref().and_then(|rendered| {
        trace.as_ref().ok().and_then(|trace| {
            let horizon_end = probe.region.onset_frame + frames_for_ms(probe.sample_rate_hz, 20);
            source_response_diversity_identity(
                controller_values(trace),
                probe.region.onset_frame..horizon_end,
                rendered.policy.residual_scales,
            )
            .ok()
        })
    });
    let renderer_controller_hash_match = rendered.as_ref().map(|rendered| {
        controller_hashes
            .as_ref()
            .is_some_and(|hashes| *hashes == rendered.policy.controller_hashes)
    });
    let renderer_actionable_policy_match = rendered.as_ref().map(|rendered| {
        actionable_policy
            .as_ref()
            .is_some_and(|policy| policy.residual_scales == rendered.policy.residual_scales)
    });
    let identity = F3DynamicSyntheticRunIdentity {
        raw_input_sha256: synthetic_artifact_hash(
            "raw_input",
            &probe.samples,
            probe.sample_rate_hz,
            2,
            probe.region,
        ),
        controller_hashes,
        combined_output_sha256: rendered.as_ref().map(|rendered| {
            synthetic_artifact_hash(
                "combined_output",
                &rendered.combined,
                probe.sample_rate_hz,
                2,
                probe.region,
            )
        }),
        attack_only_output_sha256: rendered.as_ref().map(|rendered| {
            synthetic_artifact_hash(
                "attack_only_output",
                &rendered.attack_only,
                probe.sample_rate_hz,
                2,
                probe.region,
            )
        }),
        body_only_output_sha256: rendered.as_ref().map(|rendered| {
            synthetic_artifact_hash(
                "body_only_output",
                &rendered.body_only,
                probe.sample_rate_hz,
                2,
                probe.region,
            )
        }),
        actionable_policy,
        source_response_diversity,
        renderer_controller_hash_match,
        renderer_actionable_policy_match,
        outcome,
        refusal_or_failure,
        outcome_sha256: synthetic_outcome_hash(outcome),
    };
    ProbeExecution {
        identity,
        trace: trace.ok(),
        render: rendered,
    }
}

fn expected_refusal_hash_coverage(record: &F3DynamicSyntheticRunRecord) -> bool {
    let expected_refusal =
        PercussiveForceError::Refused(PercussiveForceRefusal::MissingDynamicContrast {
            attack_missing: true,
            body_missing: true,
        });
    [&record.first, &record.repeated]
        .into_iter()
        .all(|identity| {
            identity.outcome
                == F3DynamicSyntheticOutcome::RefusedMissingAttackAndBodyDynamicContrast
                && complete_provenance_hash_coverage(identity)
                && identity.combined_output_sha256.is_none()
                && identity.attack_only_output_sha256.is_none()
                && identity.body_only_output_sha256.is_none()
                && identity
                    .actionable_policy
                    .as_ref()
                    .is_some_and(actionable_policy_identity_is_canonical)
                && identity.source_response_diversity.is_none()
                && identity.renderer_controller_hash_match.is_none()
                && identity.renderer_actionable_policy_match.is_none()
                && identity.refusal_or_failure.as_ref() == Some(&expected_refusal)
        })
}

fn expected_render_hash_coverage(record: &F3DynamicSyntheticRunRecord) -> bool {
    [&record.first, &record.repeated]
        .into_iter()
        .all(|identity| {
            identity.outcome == F3DynamicSyntheticOutcome::Rendered
                && complete_provenance_hash_coverage(identity)
                && identity
                    .combined_output_sha256
                    .as_deref()
                    .is_some_and(is_lowercase_sha256)
                && identity
                    .attack_only_output_sha256
                    .as_deref()
                    .is_some_and(is_lowercase_sha256)
                && identity
                    .body_only_output_sha256
                    .as_deref()
                    .is_some_and(is_lowercase_sha256)
                && identity
                    .actionable_policy
                    .as_ref()
                    .is_some_and(actionable_policy_identity_is_canonical)
                && identity
                    .source_response_diversity
                    .as_ref()
                    .is_some_and(|source_response| {
                        source_response_identity_bucket_equal(source_response, source_response)
                    })
                && identity.renderer_controller_hash_match == Some(true)
                && identity.renderer_actionable_policy_match == Some(true)
                && identity.refusal_or_failure.is_none()
        })
}

fn complete_provenance_hash_coverage(identity: &F3DynamicSyntheticRunIdentity) -> bool {
    is_lowercase_sha256(&identity.raw_input_sha256)
        && is_lowercase_sha256(&identity.outcome_sha256)
        && identity
            .controller_hashes
            .as_ref()
            .is_some_and(controller_hashes_are_canonical)
}

fn controller_values(trace: &DynamicControllerTrace) -> [&[f64]; 4] {
    [
        &trace.raw_attack,
        &trace.raw_body,
        &trace.attack_state,
        &trace.body_state,
    ]
}

fn controller_max_error(first: &DynamicControllerTrace, second: &DynamicControllerTrace) -> f64 {
    if controller_values(first)
        .into_iter()
        .zip(controller_values(second))
        .any(|(first, second)| first.len() != second.len())
    {
        return f64::INFINITY;
    }
    controller_values(first)
        .into_iter()
        .zip(controller_values(second))
        .flat_map(|(first, second)| first.iter().zip(second))
        .map(|(first, second)| (first - second).abs())
        .fold(0.0_f64, f64::max)
}

fn first_positive(values: &[f64]) -> Option<usize> {
    values.iter().position(|value| *value > 0.0)
}

fn canonical_positive_zero_before(values: &[f64], boundary: usize) -> bool {
    values.get(..boundary).is_some_and(|prefix| {
        prefix
            .iter()
            .all(|value| value.to_bits() == 0.0_f64.to_bits())
    })
}

fn peak_abs(samples: &[f32]) -> f64 {
    samples
        .iter()
        .map(|sample| f64::from(*sample).abs())
        .fold(0.0_f64, f64::max)
}

fn near_identity_delta_rms(
    source: &[f32],
    candidate: &[f32],
    channel_count: usize,
    region: FrozenEventRegion,
) -> Option<f64> {
    if channel_count == 0 || source.len() != candidate.len() {
        return None;
    }
    let start = region.onset_frame.checked_mul(channel_count)?;
    let end = region.body_end_frame.checked_mul(channel_count)?;
    let source = source.get(start..end)?;
    let candidate = candidate.get(start..end)?;
    let mut source_energy = 0.0;
    let mut delta_energy = 0.0;
    for (source, candidate) in source.iter().zip(candidate) {
        let source = f64::from(*source);
        let delta = f64::from(*candidate) - source;
        source_energy += source * source;
        delta_energy += delta * delta;
    }
    if source_energy <= 0.0 || !source_energy.is_finite() || !delta_energy.is_finite() {
        return None;
    }
    let value = (delta_energy / source_energy).sqrt();
    value.is_finite().then_some(value)
}

#[derive(Clone, Copy)]
pub(super) enum ProbeShape {
    Constant,
    StepBody,
}

pub(super) struct SyntheticProbe {
    pub(super) samples: Vec<f32>,
    pub(super) sample_rate_hz: u32,
    pub(super) region: FrozenEventRegion,
    pub(super) high_end: usize,
}

impl SyntheticProbe {
    pub(super) fn input(&self) -> FrozenEventInput<'_> {
        FrozenEventInput {
            interleaved_samples: &self.samples,
            sample_rate_hz: self.sample_rate_hz,
            channel_count: 2,
            region: self.region,
        }
    }
}

pub(super) fn synthetic_probe(
    sample_rate_hz: u32,
    cycles_per_sample: f64,
    shape: ProbeShape,
    amplitude_scale: f64,
    polarity: f64,
) -> SyntheticProbe {
    let duration = nearest_period_64_frames(sample_rate_hz, 96);
    let onset = nearest_period_64_frames(sample_rate_hz, 24);
    let high_end = onset + nearest_period_64_frames(sample_rate_hz, 4);
    let attack_end = onset + nearest_period_64_frames(sample_rate_hz, 8);
    let body_end = onset + nearest_period_64_frames(sample_rate_hz, 48);
    let mut samples = Vec::with_capacity(duration * 2);
    for frame in 0..duration {
        let amplitude = match shape {
            ProbeShape::Constant => 0.25,
            ProbeShape::StepBody if frame < onset || frame >= body_end => 1.0 / 32.0,
            ProbeShape::StepBody if frame < high_end => 3.0 / 8.0,
            ProbeShape::StepBody => 3.0 / 16.0,
        } * amplitude_scale
            * polarity;
        let phase = TAU * cycles_per_sample * frame as f64;
        samples.push((amplitude * phase.cos()) as f32);
        samples.push((amplitude * phase.sin()) as f32);
    }
    SyntheticProbe {
        samples,
        sample_rate_hz,
        region: FrozenEventRegion {
            onset_frame: onset,
            attack_end_frame: attack_end,
            body_end_frame: body_end,
        },
        high_end,
    }
}

fn nearest_period_64_frames(sample_rate_hz: u32, milliseconds: u32) -> usize {
    let numerator = u64::from(sample_rate_hz) * u64::from(milliseconds);
    let blocks = ((numerator + 32_000) / 64_000).max(1);
    (blocks * 64) as usize
}

#[cfg(test)]
mod tests;
