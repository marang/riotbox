//! Exact production callback with generated PCM; no device, source or file I/O.
use super::w30_fixture::{prepared, process, reposition, warm};
use crate::runtime::AudioTransportPositionUpdate;

fn render_after_transport_update(
    pitch_dive: bool,
    update: AudioTransportPositionUpdate,
) -> [f32; 50] {
    let (state, shell, mut callback) = prepared(pitch_dive);
    let material = state.w30_preview.try_snapshot().unwrap();
    // All cases have identical active material, trigger and rendering age.
    warm(&shell, &mut callback);
    shell.update_transport_state(true, 120.0, update).unwrap();
    let expected = if matches!(update, AudioTransportPositionUpdate::Reposition(_)) {
        assert!(
            shell.timing_snapshot().is_none(),
            "old timing cannot acknowledge a new request"
        );
        12.35
    } else {
        1.6
    };
    let mut output = [0.0; 50];
    process(&mut callback, &mut output);
    assert!((shell.timing_snapshot().unwrap().position_beats - expected).abs() < 1e-11);
    assert_eq!(
        state.w30_preview.try_snapshot().unwrap(),
        material,
        "transport intent must not alter published W-30 material or trigger"
    );
    output
}

#[test]
fn live_reposition_reaches_frozen_pitch_dive_terminal_silence_without_material_change() {
    let preserved = render_after_transport_update(true, AudioTransportPositionUpdate::Preserve);
    let dry_seek = render_after_transport_update(false, reposition(12.25));
    let pitch_seek = render_after_transport_update(true, reposition(12.25));
    let nonzero = |samples: &[f32]| samples.iter().filter(|sample| **sample != 0.0).count();
    eprintln!(
        "nonzero samples: same-age Preserve/PitchDive={}, Reposition/dry={}, Reposition/PitchDive={}",
        nonzero(&preserved),
        nonzero(&dry_seek),
        nonzero(&pitch_seek)
    );
    assert!(preserved.iter().any(|sample| *sample != 0.0));
    assert!(dry_seek.iter().any(|sample| *sample != 0.0));
    assert!(
        pitch_seek.iter().all(|sample| *sample == 0.0),
        "acknowledged live reposition to relative beat12.25 must emit frozen PitchDive terminal silence"
    );
}
