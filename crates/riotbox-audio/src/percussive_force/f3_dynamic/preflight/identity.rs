//! Deterministic synthetic provenance, policy, and response identities.
//!
//! Region-bound controller hashes remain provenance. The ordinary actionable
//! policy includes only the fixed F3 scales, while source-response diversity
//! uses a separately versioned, anatomy-independent summary.

use std::ops::Range;

use sha2::{Digest, Sha256};

use super::super::{F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2, F3ControllerHashes};
use super::F3DynamicSyntheticOutcome;
use crate::percussive_force::FrozenEventRegion;

const SYNTHETIC_ARTIFACT_HASH_DOMAIN: &str =
    "riotbox.f3_causal_envelope_contrast_dynamic_residual_v2.synthetic_artifact.v1";
const SYNTHETIC_OUTCOME_HASH_DOMAIN: &str =
    "riotbox.f3_causal_envelope_contrast_dynamic_residual_v2.synthetic_outcome.v1";
pub(super) const ACTIONABLE_POLICY_HASH_DOMAIN: &str =
    "riotbox.percussive_force_actionable_policy.v1";
pub(super) const ACTIONABLE_POLICY_FIELD_ORDER: [&str; 2] = ["sA_f64", "sB_f64"];
pub(super) const SOURCE_RESPONSE_DIVERSITY_HASH_DOMAIN: &str =
    "riotbox.f3_source_response_diversity.v1";
pub(super) const SOURCE_RESPONSE_DIVERSITY_FIELD_ORDER: [&str; 8] = [
    "mean_a0", "max_a0", "mean_b0", "max_b0", "mean_A", "max_A", "mean_B", "max_B",
];
const SOURCE_RESPONSE_QUANTIZATION_STEPS: f64 = 20.0;
const SOURCE_RESPONSE_CONTROLLER_LABELS: [&str; 4] = ["a0", "b0", "A", "B"];

/// Anatomy-independent actionable policy identity for the synthetic preflight.
///
/// This deliberately excludes region-bound controller provenance and does not
/// turn [`super::super::F3DynamicPolicy`] into qualification evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct F3DynamicPreflightPolicyIdentity {
    pub domain: &'static str,
    pub family_id: &'static str,
    pub field_order: [&'static str; 2],
    pub residual_scales: [f64; 2],
    pub sha256: String,
}

/// Fixed-horizon source response used only as F3 diversity evidence.
///
/// No sample rate, channel count, frame/index, anatomy, mask, or source
/// metadata enters this record or its digest.
#[derive(Clone, Debug, PartialEq)]
pub struct F3DynamicSourceResponseDiversityIdentity {
    pub domain: &'static str,
    pub family_id: &'static str,
    pub field_order: [&'static str; 8],
    pub residual_scales: [f64; 2],
    pub raw_summary: [f64; 8],
    pub quantized_summary: [u8; 8],
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IdentityInputError {
    IncompleteSourceResponseHorizon,
    NonFiniteF64,
    NegativeZeroF64,
    SourceResponseOutsideUnitInterval {
        field: &'static str,
        relative_frame: usize,
    },
    SourceResponseMeanExceedsMaximum {
        controller: &'static str,
    },
    InvalidSourceResponseQuantization,
}

pub(super) fn synthetic_artifact_hash(
    role: &str,
    samples: &[f32],
    sample_rate_hz: u32,
    channel_count: usize,
    region: FrozenEventRegion,
) -> String {
    let mut digest = Sha256::new();
    hash_length_prefixed(&mut digest, SYNTHETIC_ARTIFACT_HASH_DOMAIN.as_bytes());
    hash_length_prefixed(&mut digest, role.as_bytes());
    digest.update(sample_rate_hz.to_be_bytes());
    digest.update((channel_count as u32).to_be_bytes());
    digest.update(((samples.len() / channel_count) as u64).to_be_bytes());
    digest.update((region.onset_frame as u64).to_be_bytes());
    digest.update((region.attack_end_frame as u64).to_be_bytes());
    digest.update((region.body_end_frame as u64).to_be_bytes());
    digest.update((samples.len() as u64).to_be_bytes());
    for sample in samples {
        digest.update(sample.to_bits().to_be_bytes());
    }
    format!("{:x}", digest.finalize())
}

pub(super) fn actionable_policy_identity(
    residual_scales: [f64; 2],
) -> Result<F3DynamicPreflightPolicyIdentity, IdentityInputError> {
    let mut digest = Sha256::new();
    hash_length_prefixed(&mut digest, ACTIONABLE_POLICY_HASH_DOMAIN.as_bytes());
    hash_length_prefixed(
        &mut digest,
        F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2.as_bytes(),
    );
    for value in residual_scales {
        digest.update(canonical_f64_bytes(value)?);
    }
    Ok(F3DynamicPreflightPolicyIdentity {
        domain: ACTIONABLE_POLICY_HASH_DOMAIN,
        family_id: F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2,
        field_order: ACTIONABLE_POLICY_FIELD_ORDER,
        residual_scales,
        sha256: format!("{:x}", digest.finalize()),
    })
}

pub(super) fn actionable_policy_identity_is_canonical(
    identity: &F3DynamicPreflightPolicyIdentity,
) -> bool {
    if identity.domain != ACTIONABLE_POLICY_HASH_DOMAIN
        || identity.family_id != F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2
        || identity.field_order != ACTIONABLE_POLICY_FIELD_ORDER
        || !is_lowercase_sha256(&identity.sha256)
    {
        return false;
    }
    actionable_policy_identity(identity.residual_scales)
        .is_ok_and(|recomputed| recomputed.sha256 == identity.sha256)
}

pub(super) fn source_response_diversity_identity(
    controllers: [&[f64]; 4],
    horizon: Range<usize>,
    residual_scales: [f64; 2],
) -> Result<F3DynamicSourceResponseDiversityIdentity, IdentityInputError> {
    if horizon.is_empty() || controllers.iter().any(|values| values.len() < horizon.end) {
        return Err(IdentityInputError::IncompleteSourceResponseHorizon);
    }
    let mut raw_summary = [0.0; 8];
    for (controller_index, values) in controllers.into_iter().enumerate() {
        let field = SOURCE_RESPONSE_CONTROLLER_LABELS[controller_index];
        let values = &values[horizon.clone()];
        let mut sum = 0.0;
        let mut maximum = 0.0_f64;
        for (relative_frame, value) in values.iter().copied().enumerate() {
            validate_source_response_value(field, relative_frame, value)?;
            sum += value;
            maximum = maximum.max(value);
        }
        let mean = sum / values.len() as f64;
        validate_source_response_value(field, 0, mean)?;
        raw_summary[controller_index * 2] = canonical_zero(mean);
        raw_summary[controller_index * 2 + 1] = canonical_zero(maximum);
    }
    source_response_identity_from_summary(raw_summary, residual_scales)
}

pub(super) fn source_response_identity_from_summary(
    raw_summary: [f64; 8],
    residual_scales: [f64; 2],
) -> Result<F3DynamicSourceResponseDiversityIdentity, IdentityInputError> {
    let mut quantized_summary = [0_u8; 8];
    for (index, value) in raw_summary.iter().copied().enumerate() {
        validate_source_response_value(SOURCE_RESPONSE_DIVERSITY_FIELD_ORDER[index], 0, value)?;
        let quantized = (SOURCE_RESPONSE_QUANTIZATION_STEPS * value + 0.5).floor();
        if !quantized.is_finite()
            || !(0.0..=SOURCE_RESPONSE_QUANTIZATION_STEPS).contains(&quantized)
        {
            return Err(IdentityInputError::InvalidSourceResponseQuantization);
        }
        quantized_summary[index] = quantized as u8;
    }
    for (controller_index, pair) in raw_summary.chunks_exact(2).enumerate() {
        if pair[0] > pair[1] {
            return Err(IdentityInputError::SourceResponseMeanExceedsMaximum {
                controller: SOURCE_RESPONSE_CONTROLLER_LABELS[controller_index],
            });
        }
    }
    let sha256 = source_response_digest(residual_scales, quantized_summary)?;
    Ok(F3DynamicSourceResponseDiversityIdentity {
        domain: SOURCE_RESPONSE_DIVERSITY_HASH_DOMAIN,
        family_id: F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2,
        field_order: SOURCE_RESPONSE_DIVERSITY_FIELD_ORDER,
        residual_scales,
        raw_summary,
        quantized_summary,
        sha256,
    })
}

fn source_response_digest(
    residual_scales: [f64; 2],
    quantized_summary: [u8; 8],
) -> Result<String, IdentityInputError> {
    let mut digest = Sha256::new();
    hash_length_prefixed(
        &mut digest,
        SOURCE_RESPONSE_DIVERSITY_HASH_DOMAIN.as_bytes(),
    );
    hash_length_prefixed(
        &mut digest,
        F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2.as_bytes(),
    );
    for value in residual_scales {
        digest.update(canonical_f64_bytes(value)?);
    }
    digest.update(quantized_summary);
    Ok(format!("{:x}", digest.finalize()))
}

pub fn f3_source_response_identities_are_diversity_separated(
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
            .any(|(first, second)| first.abs_diff(second) >= 2)
}

pub(super) fn source_response_identity_bucket_equal(
    first: &F3DynamicSourceResponseDiversityIdentity,
    second: &F3DynamicSourceResponseDiversityIdentity,
) -> bool {
    source_response_identity_is_canonical(first)
        && source_response_identity_is_canonical(second)
        && first.domain == second.domain
        && first.family_id == second.family_id
        && first.field_order == second.field_order
        && first.residual_scales == second.residual_scales
        && first.quantized_summary == second.quantized_summary
        && first.sha256 == second.sha256
}

pub(super) fn source_response_identity_is_canonical(
    identity: &F3DynamicSourceResponseDiversityIdentity,
) -> bool {
    if identity.domain != SOURCE_RESPONSE_DIVERSITY_HASH_DOMAIN
        || identity.family_id != F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2
        || identity.field_order != SOURCE_RESPONSE_DIVERSITY_FIELD_ORDER
        || !is_lowercase_sha256(&identity.sha256)
    {
        return false;
    }
    let Ok(recomputed) =
        source_response_identity_from_summary(identity.raw_summary, identity.residual_scales)
    else {
        return false;
    };
    recomputed.quantized_summary == identity.quantized_summary
        && recomputed.sha256 == identity.sha256
}

fn validate_source_response_value(
    field: &'static str,
    relative_frame: usize,
    value: f64,
) -> Result<(), IdentityInputError> {
    if !value.is_finite() {
        return Err(IdentityInputError::NonFiniteF64);
    }
    if value.to_bits() == (-0.0_f64).to_bits() {
        return Err(IdentityInputError::NegativeZeroF64);
    }
    if !(0.0..=1.0).contains(&value) {
        return Err(IdentityInputError::SourceResponseOutsideUnitInterval {
            field,
            relative_frame,
        });
    }
    Ok(())
}

pub(super) fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn controller_hashes_are_canonical(hashes: &F3ControllerHashes) -> bool {
    [
        hashes.raw_attack_sha256.as_str(),
        hashes.raw_body_sha256.as_str(),
        hashes.attack_state_sha256.as_str(),
        hashes.body_state_sha256.as_str(),
    ]
    .into_iter()
    .all(is_lowercase_sha256)
}

pub(super) fn canonical_f64_bytes(value: f64) -> Result<[u8; 8], IdentityInputError> {
    if !value.is_finite() {
        return Err(IdentityInputError::NonFiniteF64);
    }
    if value.to_bits() == (-0.0_f64).to_bits() {
        return Err(IdentityInputError::NegativeZeroF64);
    }
    Ok(value.to_bits().to_be_bytes())
}

fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

pub(super) fn synthetic_outcome_hash(outcome: F3DynamicSyntheticOutcome) -> String {
    let label = match outcome {
        F3DynamicSyntheticOutcome::Rendered => "rendered",
        F3DynamicSyntheticOutcome::RefusedMissingAttackAndBodyDynamicContrast => {
            "refused_missing_attack_and_body_dynamic_contrast"
        }
        F3DynamicSyntheticOutcome::UnexpectedFailure => "unexpected_failure",
    };
    let mut digest = Sha256::new();
    hash_length_prefixed(&mut digest, SYNTHETIC_OUTCOME_HASH_DOMAIN.as_bytes());
    hash_length_prefixed(&mut digest, label.as_bytes());
    format!("{:x}", digest.finalize())
}

fn hash_length_prefixed(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u32).to_be_bytes());
    digest.update(bytes);
}
