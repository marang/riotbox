//! Stop/Preserve-Play scheduling through the exact generated callback.
use super::w30_fixture::{prepared, process, reposition, warm};
use crate::runtime::AudioTransportPositionUpdate;

fn resumed(pitch_dive: bool) -> [f32; 50] {
    let (state, shell, mut callback) = prepared(pitch_dive);
    let material = state.w30_preview.try_snapshot().unwrap();
    warm(&shell, &mut callback);
    shell
        .update_transport_state(false, 120.0, reposition(11.99))
        .unwrap();
    assert!(shell.timing_snapshot().is_none());
    // Ten stopped frames complete the existing 5ms fade, but no later stopped
    // callback runs to reset activation. Idle audition time crosses beat12;
    // the authoritative stopped transport remains exactly11.99.
    process(&mut callback, &mut [0.0; 10]);
    let stopped = shell.timing_snapshot().unwrap();
    assert!(!stopped.is_transport_running);
    assert!((stopped.position_beats - 11.99).abs() < 1e-11);
    shell
        .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    let mut output = [0.0; 50];
    process(&mut callback, &mut output);
    let running = shell.timing_snapshot().unwrap();
    assert!(running.is_transport_running);
    assert!((running.position_beats - 12.09).abs() < 1e-11);
    assert_eq!(
        state.w30_preview.try_snapshot().unwrap(),
        material,
        "Stop/Play must not alter published W-30 material or trigger"
    );
    output
}

#[test]
fn preserve_play_resumes_pitch_dive_at_the_stopped_transport_not_idle_audition_time() {
    let dry = resumed(false);
    let pitch = resumed(true);
    let positive = |samples: &[f32]| samples.iter().filter(|sample| **sample > 0.0).count();
    eprintln!(
        "positive samples: dry resume={}, PitchDive prefix={}, PitchDive total={}",
        positive(&dry),
        positive(&pitch[..3]),
        positive(&pitch)
    );
    assert!(
        dry.iter().any(|sample| *sample > 0.0),
        "dry control must resume audibly"
    );
    assert!(
        pitch[30..].iter().all(|sample| *sample == 0.0),
        "resumed PitchDive must still reach its frozen terminal silence"
    );
    assert!(
        pitch[..3].iter().any(|sample| *sample > 0.0),
        "Preserve/Play starts at11.99: its preterminal prefix must not inherit idle-clock terminal silence"
    );
}
