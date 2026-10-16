//! Ordinary Preserve overflow recovery through the exact generated callback.
use super::w30_fixture::{prepared, process, reposition, warm};
use crate::runtime::{
    AudioTransportPositionUpdate, shared_transport_tr909::callback_scratch_sample_count,
};

fn after_overflow(pitch_dive: bool) -> [f32; 50] {
    let (state, shell, mut callback) = prepared(pitch_dive);
    let material = state.w30_preview.try_snapshot().unwrap();
    warm(&shell, &mut callback);
    shell
        .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    assert!(
        shell.timing_snapshot().is_some(),
        "Preserve has no new request to acknowledge"
    );

    // Exactly the prepared fixture's config, admitted by the production scratch
    // sizing function. The oversized buffer is allocated before measurement.
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: 1000,
        buffer_size: cpal::BufferSize::Fixed(50),
    };
    let capacity = callback_scratch_sample_count(&config, 1);
    assert_eq!(capacity, 50);
    let mut oversized = vec![1.0; capacity + 6000];
    assert!(oversized.len() > capacity);
    process(&mut callback, &mut oversized);
    assert!(oversized.iter().all(|sample| *sample == 0.0));
    assert_eq!(shell.health_snapshot().callback_scratch_overflow_count, 1);
    let overflow_timing = shell.timing_snapshot().unwrap();
    assert!(overflow_timing.is_transport_running);
    // 750 warmed frames + 6050 skipped frames at 1000Hz/120BPM = beat13.6.
    assert!((overflow_timing.position_beats - 13.6).abs() < 1e-11);

    let mut output = [0.0; 50];
    process(&mut callback, &mut output);
    let recovered_timing = shell.timing_snapshot().unwrap();
    assert!(recovered_timing.is_transport_running);
    assert!((recovered_timing.position_beats - 13.7).abs() < 1e-11);
    assert_eq!(shell.health_snapshot().callback_scratch_overflow_count, 1);
    assert_eq!(
        state.w30_preview.try_snapshot().unwrap(),
        material,
        "overflow recovery must not alter published W-30 material or trigger"
    );
    output
}

#[test]
fn preserve_overflow_recovery_uses_advanced_transport_for_pitch_dive_terminal_silence() {
    let dry = after_overflow(false);
    let pitch = after_overflow(true);
    let nonzero = |samples: &[f32]| samples.iter().filter(|sample| **sample != 0.0).count();
    eprintln!(
        "post-overflow nonzero samples: dry={}, PitchDive={}",
        nonzero(&dry),
        nonzero(&pitch)
    );
    assert!(
        dry.iter().any(|sample| *sample != 0.0),
        "dry control must recover audibly"
    );
    assert!(
        pitch.iter().all(|sample| *sample == 0.0),
        "the normal callback after acknowledged Preserve overflow starts at13.6, beyond frozen PitchDive terminal beat12"
    );
}

#[test]
fn pending_superseded_and_repeated_requests_survive_repeated_overflow() {
    let (pitch_state, pitch_shell, mut pitch_callback) = prepared(true);
    let (dry_state, dry_shell, mut dry_callback) = prepared(false);
    let pitch_material = pitch_state.w30_preview.try_snapshot().unwrap();
    let dry_material = dry_state.w30_preview.try_snapshot().unwrap();
    warm(&pitch_shell, &mut pitch_callback);
    warm(&dry_shell, &mut dry_callback);
    // Twice the exact prepared scratch capacity, allocated outside measurement.
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: 1000,
        buffer_size: cpal::BufferSize::Fixed(50),
    };
    let mut overflow = vec![1.0; callback_scratch_sample_count(&config, 1) * 2];
    for shell in [&pitch_shell, &dry_shell] {
        shell
            .update_transport_state(true, 120.0, reposition(12.25))
            .unwrap();
        assert!(shell.timing_snapshot().is_none());
    }
    process(&mut pitch_callback, &mut overflow);
    assert!(overflow.iter().all(|sample| *sample == 0.0));
    overflow.fill(1.0);
    process(&mut dry_callback, &mut overflow);
    assert!(overflow.iter().all(|sample| *sample == 0.0));
    for shell in [&pitch_shell, &dry_shell] {
        assert!((shell.timing_snapshot().unwrap().position_beats - 12.45).abs() < 1e-11);
        // A new request at the just-observed coordinate still needs an ack,
        // then a newer request supersedes it before any W-30 render occurs.
        shell
            .update_transport_state(true, 120.0, reposition(12.45))
            .unwrap();
        assert!(shell.timing_snapshot().is_none());
        shell
            .update_transport_state(true, 120.0, reposition(11.95))
            .unwrap();
        assert!(shell.timing_snapshot().is_none());
    }
    let mut pitch = [0.0; 50];
    let mut dry = [0.0; 50];
    process(&mut pitch_callback, &mut pitch);
    process(&mut dry_callback, &mut dry);
    assert!(dry.iter().any(|sample| *sample > 0.0));
    assert!(pitch[..20].iter().any(|sample| *sample > 0.0));
    assert!(pitch[35..].iter().all(|sample| *sample == 0.0));
    for shell in [&pitch_shell, &dry_shell] {
        assert!((shell.timing_snapshot().unwrap().position_beats - 12.05).abs() < 1e-11);
        // Repeat that same requested coordinate, then overflow twice before
        // rendering. The latest callback clock, not the old request coordinate,
        // must be used on recovery.
        shell
            .update_transport_state(true, 120.0, reposition(11.95))
            .unwrap();
        assert!(shell.timing_snapshot().is_none());
    }
    for _ in 0..2 {
        overflow.fill(1.0);
        process(&mut pitch_callback, &mut overflow);
        assert!(overflow.iter().all(|sample| *sample == 0.0));
        overflow.fill(1.0);
        process(&mut dry_callback, &mut overflow);
        assert!(overflow.iter().all(|sample| *sample == 0.0));
    }
    for shell in [&pitch_shell, &dry_shell] {
        assert!((shell.timing_snapshot().unwrap().position_beats - 12.35).abs() < 1e-11);
        assert_eq!(shell.health_snapshot().callback_scratch_overflow_count, 3);
    }
    process(&mut pitch_callback, &mut pitch);
    process(&mut dry_callback, &mut dry);
    assert!(dry.iter().any(|sample| *sample > 0.0));
    assert!(
        pitch.iter().all(|sample| *sample == 0.0),
        "recovery must use advanced beat12.35, not repeated target11.95 or old W-30 phase"
    );
    for shell in [&pitch_shell, &dry_shell] {
        assert!((shell.timing_snapshot().unwrap().position_beats - 12.45).abs() < 1e-11);
    }
    assert_eq!(
        pitch_state.w30_preview.try_snapshot().unwrap(),
        pitch_material
    );
    assert_eq!(dry_state.w30_preview.try_snapshot().unwrap(), dry_material);
}
