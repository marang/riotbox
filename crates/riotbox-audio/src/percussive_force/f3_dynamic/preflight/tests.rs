use super::identity::{
    ACTIONABLE_POLICY_FIELD_ORDER, ACTIONABLE_POLICY_HASH_DOMAIN, IdentityInputError,
    SOURCE_RESPONSE_DIVERSITY_FIELD_ORDER, canonical_f64_bytes,
    source_response_identity_from_summary,
};
use super::{
    actionable_policy_identity, f3_source_response_identities_are_diversity_separated,
    near_identity_delta_rms, nearest_period_64_frames,
    source_response_cross_rate_pair_non_diversity_pass, source_response_diversity_identity,
    synthetic_artifact_hash,
};
use crate::percussive_force::FrozenEventRegion;

#[test]
fn synthetic_m64_boundaries_match_frozen_golden_values() {
    let expected = [
        (44_100, [4_224, 1_088, 192, 384, 2_112]),
        (48_000, [4_608, 1_152, 192, 384, 2_304]),
        (96_000, [9_216, 2_304, 384, 768, 4_608]),
    ];
    for (sample_rate_hz, [duration, onset, high, attack, body]) in expected {
        assert_eq!(nearest_period_64_frames(sample_rate_hz, 96), duration);
        assert_eq!(nearest_period_64_frames(sample_rate_hz, 24), onset);
        assert_eq!(nearest_period_64_frames(sample_rate_hz, 4), high);
        assert_eq!(nearest_period_64_frames(sample_rate_hz, 8), attack);
        assert_eq!(nearest_period_64_frames(sample_rate_hz, 48), body);
    }
}

#[test]
fn actionable_policy_hash_matches_independent_full_framing_golden() {
    let identity = actionable_policy_identity([1.0, 1.0]).unwrap();
    assert_eq!(
        identity.sha256,
        "21c1b5ee649db887b7e7233a0255c4f2eb4a53f408cc4f18cf758abf637f3a7c"
    );
    assert_eq!(identity.field_order, ACTIONABLE_POLICY_FIELD_ORDER);
    assert_eq!(identity.domain, ACTIONABLE_POLICY_HASH_DOMAIN);
    assert_eq!(identity.residual_scales, [1.0, 1.0]);
}

#[test]
fn actionable_policy_encoding_rejects_noncanonical_inputs() {
    assert_eq!(
        canonical_f64_bytes(-0.0),
        Err(IdentityInputError::NegativeZeroF64)
    );
    assert_eq!(
        canonical_f64_bytes(f64::NAN),
        Err(IdentityInputError::NonFiniteF64)
    );
}

#[test]
fn source_response_identity_matches_independent_vector_and_hash_golden() {
    let raw_summary = [0.0, 0.03125, 0.0625, 0.125, 0.25, 0.5, 0.75, 1.0];
    let identity = source_response_identity_from_summary(raw_summary, [1.0, 1.0]).unwrap();
    assert_eq!(identity.raw_summary, raw_summary);
    assert_eq!(identity.quantized_summary, [0, 1, 1, 3, 5, 10, 15, 20]);
    assert_eq!(identity.field_order, SOURCE_RESPONSE_DIVERSITY_FIELD_ORDER);
    assert_eq!(
        identity.sha256,
        "c6e8dbe34eaafc2274aeb36776aee8ca5f346fb64b6d63dc6b20e2de861050e8"
    );
}

#[test]
fn source_response_summary_uses_the_complete_declared_horizon() {
    let a0 = [0.9, 0.0, 0.5, 1.0, 0.9];
    let b0 = [0.9, 0.0, 0.25, 0.5, 0.9];
    let attack = [0.9, 0.25, 0.25, 0.25, 0.9];
    let body = [0.9, 0.0, 0.0, 0.75, 0.9];
    let identity =
        source_response_diversity_identity([&a0, &b0, &attack, &body], 1..4, [1.0, 1.0]).unwrap();
    assert_eq!(
        identity.raw_summary,
        [0.5, 1.0, 0.25, 0.5, 0.25, 0.25, 0.25, 0.75]
    );
    assert_eq!(
        source_response_diversity_identity([&a0[..3], &b0, &attack, &body], 1..4, [1.0, 1.0],),
        Err(IdentityInputError::IncompleteSourceResponseHorizon)
    );
}

#[test]
fn diversity_separation_requires_two_quantized_steps() {
    let baseline = source_response_identity_from_summary([0.0; 8], [1.0, 1.0]).unwrap();
    let one_step = source_response_identity_from_summary(
        [0.05, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        [1.0, 1.0],
    )
    .unwrap();
    let two_steps =
        source_response_identity_from_summary([0.1, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], [1.0, 1.0])
            .unwrap();
    assert!(!f3_source_response_identities_are_diversity_separated(
        &baseline, &baseline
    ));
    assert!(!f3_source_response_identities_are_diversity_separated(
        &baseline, &one_step
    ));
    assert!(f3_source_response_identities_are_diversity_separated(
        &baseline, &two_steps
    ));
    assert!(source_response_cross_rate_pair_non_diversity_pass(
        &baseline, &one_step
    ));
    assert!(!source_response_cross_rate_pair_non_diversity_pass(
        &baseline, &two_steps
    ));

    let mut invalid = one_step.clone();
    invalid.sha256 = "0".repeat(64);
    assert!(!source_response_cross_rate_pair_non_diversity_pass(
        &baseline, &invalid
    ));
}

#[test]
fn synthetic_artifact_hash_matches_independent_full_framing_golden() {
    let region = FrozenEventRegion {
        onset_frame: 0,
        attack_end_frame: 1,
        body_end_frame: 2,
    };
    assert_eq!(
        synthetic_artifact_hash("raw_input", &[0.0, 0.5, -0.5, 1.0], 48_000, 2, region),
        "f30bd5136fa8b009ce6f6f327b70715aaa67bec92590c0b308169e8fa70a5588"
    );
}

#[test]
fn near_identity_formula_matches_hand_calculated_all_channel_golden() {
    let region = FrozenEventRegion {
        onset_frame: 0,
        attack_end_frame: 1,
        body_end_frame: 2,
    };
    let source = [1.0, 0.0, 1.0, 0.0];
    let candidate = [1.1, 0.0, 1.1, 0.0];
    let delta = near_identity_delta_rms(&source, &candidate, 2, region).unwrap();
    assert!((delta - 0.1).abs() <= 3.0e-8, "delta={delta:.17e}");
}
