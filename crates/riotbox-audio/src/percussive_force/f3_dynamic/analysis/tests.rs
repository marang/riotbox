use super::*;
use crate::percussive_force::FrozenEventRegion;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0e-12,
        "actual={actual:.17e}, expected={expected:.17e}"
    );
}

#[test]
fn production_window_conversion_matches_frozen_golden_values() {
    let expected = [
        (44_100, [44, 353, 882]),
        (48_000, [48, 384, 960]),
        (96_000, [96, 768, 1_920]),
    ];
    for (sample_rate_hz, windows) in expected {
        assert_eq!(
            [
                frames_for_ms(sample_rate_hz, 1),
                frames_for_ms(sample_rate_hz, 8),
                frames_for_ms(sample_rate_hz, 20),
            ],
            windows
        );
    }
}

#[test]
fn phase_safe_rms_matches_hand_calculated_multichannel_golden() {
    // Per-frame powers with frozen zero means are [1, 5, 5]. This
    // independently guards standard RMS rather than a fourth-moment proxy.
    let samples = [1.0, -1.0, 3.0, 1.0, -1.0, 3.0];
    let envelopes = phase_safe_multichannel_rms_envelopes_with_frozen_means(
        &samples,
        2,
        [1, 2, 3],
        &[0.0, 0.0],
    )
    .unwrap();
    assert_close(envelopes.r1[0], 1.0);
    assert_close(envelopes.r1[1], 5.0_f64.sqrt());
    assert_close(envelopes.r1[2], 5.0_f64.sqrt());
    assert_eq!(envelopes.r8[0].to_bits(), 0.0_f64.to_bits());
    assert_close(envelopes.r8[1], 3.0_f64.sqrt());
    assert_close(envelopes.r8[2], 5.0_f64.sqrt());
    assert_eq!(envelopes.r20[0].to_bits(), 0.0_f64.to_bits());
    assert_eq!(envelopes.r20[1].to_bits(), 0.0_f64.to_bits());
    assert_close(envelopes.r20[2], (11.0_f64 / 3.0).sqrt());
}

#[test]
fn causal_controller_uses_strict_positive_first_crossing_golden() {
    let floor = 0.1;
    let raw_attack =
        (directed_contrast(0.8, 0.4, floor) * directed_contrast(0.4, 0.2, floor)).sqrt();
    assert_close(raw_attack, 0.5);

    let onset = 2;
    let raw = [0.0, 0.0, raw_attack, 0.0];
    let rise = ballistic_coefficient(1_000, 1);
    let fall = ballistic_coefficient(1_000, 8);
    let mut state = [0.0; 4];
    for frame in onset..raw.len() {
        state[frame] = ballistic_step(raw[frame], state[frame - 1], rise, fall);
    }
    assert!(
        raw[..onset]
            .iter()
            .all(|value| value.to_bits() == 0.0_f64.to_bits())
    );
    assert!(
        state[..onset]
            .iter()
            .all(|value| value.to_bits() == 0.0_f64.to_bits())
    );
    assert_eq!(raw.iter().position(|value| *value > 0.0), Some(onset));
    assert_eq!(state.iter().position(|value| *value > 0.0), Some(onset));
    assert_close(state[onset], (1.0 - (-1.0_f64).exp()) * 0.5);
}

#[test]
fn controller_hash_matches_independent_full_framing_golden() {
    let region = FrozenEventRegion {
        onset_frame: 0,
        attack_end_frame: 1,
        body_end_frame: 2,
    };
    let golden = controller_hash("a0", &[0.0, 0.5], 48_000, 1, region).unwrap();
    assert_eq!(
        golden,
        "e29fa3118b88a4897db968a3c16b87f3235842715a32116b928ba0c0e249fcf2"
    );
    assert_ne!(
        controller_hash("b0", &[0.0, 0.5], 48_000, 1, region).unwrap(),
        golden
    );
    assert_ne!(
        controller_hash("a0", &[0.0, 0.5], 44_100, 1, region).unwrap(),
        golden
    );
    assert_ne!(
        controller_hash("a0", &[0.0, 0.5], 48_000, 2, region).unwrap(),
        golden
    );
    assert_ne!(
        controller_hash(
            "a0",
            &[0.0, 0.5, 0.0],
            48_000,
            1,
            FrozenEventRegion {
                body_end_frame: 3,
                ..region
            },
        )
        .unwrap(),
        golden
    );
    assert_ne!(
        controller_hash(
            "a0",
            &[0.0, 0.5],
            48_000,
            1,
            FrozenEventRegion {
                onset_frame: 1,
                attack_end_frame: 2,
                body_end_frame: 3,
            },
        )
        .unwrap(),
        golden
    );
}

#[test]
fn controller_hash_rejects_negative_zero_instead_of_canonicalizing_it() {
    let error = controller_hash(
        "a0",
        &[0.0, -0.0],
        48_000,
        1,
        FrozenEventRegion {
            onset_frame: 0,
            attack_end_frame: 1,
            body_end_frame: 2,
        },
    )
    .expect_err("negative zero is not canonical controller provenance");
    assert_eq!(
        error,
        PercussiveForceError::Refused(PercussiveForceRefusal::NegativeZeroControllerValue {
            label: "a0",
            frame: 1,
        })
    );
}
