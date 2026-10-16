//! Reposition preserves active PitchDive causal history in the exact callback.
use super::w30_fixture::{
    assert_pcm_eq, marked_preview, prepared_with_preview, process, reposition, warm,
};
use crate::{
    runtime::AudioTransportPositionUpdate,
    w30::{W30HookArticulationProfile, W30HookArticulationRenderState, W30PreviewRenderState},
};

fn pitch_preview() -> W30PreviewRenderState {
    let mut preview = marked_preview();
    preview.pad_playback.as_mut().unwrap().hook_articulation =
        Some(W30HookArticulationRenderState {
            profile: W30HookArticulationProfile::PitchDiveV1,
            started_at_beat: 0,
        });
    preview
}

#[test]
fn active_pitch_dive_fresh_request_preserves_same_age_history() {
    let (state, shell, mut callback) = prepared_with_preview(pitch_preview());
    let (control_state, control_shell, mut control_callback) =
        prepared_with_preview(pitch_preview());
    let (dry_state, dry_shell, mut dry_callback) = prepared_with_preview(marked_preview());
    let material = state.w30_preview.try_snapshot().unwrap();
    let dry_material = dry_state.w30_preview.try_snapshot().unwrap();
    warm(&shell, &mut callback);
    warm(&control_shell, &mut control_callback);
    warm(&dry_shell, &mut dry_callback);
    for shell in [&shell, &control_shell, &dry_shell] {
        shell
            .update_transport_state(true, 120.0, reposition(8.25))
            .unwrap();
        assert!(shell.timing_snapshot().is_none());
    }
    let mut candidate = [0.0; 50];
    let mut control = [0.0; 50];
    let mut dry = [0.0; 50];
    process(&mut callback, &mut candidate);
    process(&mut control_callback, &mut control);
    process(&mut dry_callback, &mut dry);
    assert_pcm_eq(&candidate, &control);
    assert_ne!(
        candidate, dry,
        "the active dive must already affect marked PCM"
    );
    assert!(candidate.iter().any(|sample| *sample > 0.0));
    let observed = shell.timing_snapshot().unwrap().position_beats;
    assert!((observed - 8.35).abs() < 1e-11);
    // New identity at the current coordinate: no natural half-step crossing,
    // so any cold cursor, attack or PitchDive history reset is observable.
    shell
        .update_transport_state(true, 120.0, reposition(observed))
        .unwrap();
    assert!(shell.timing_snapshot().is_none());
    control_shell
        .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    dry_shell
        .update_transport_state(true, 120.0, reposition(observed))
        .unwrap();
    process(&mut callback, &mut candidate);
    process(&mut control_callback, &mut control);
    process(&mut dry_callback, &mut dry);
    assert_pcm_eq(&candidate, &control);
    assert_ne!(candidate, dry);
    assert!(candidate.iter().any(|sample| *sample > 0.0));
    for shell in [&shell, &control_shell, &dry_shell] {
        assert!((shell.timing_snapshot().unwrap().position_beats - 8.45).abs() < 1e-11);
    }
    assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
    assert_eq!(control_state.w30_preview.try_snapshot().unwrap(), material);
    assert_eq!(dry_state.w30_preview.try_snapshot().unwrap(), dry_material);
}
