//! Active generated pad continuity through the exact production callback.
use super::w30_fixture::{marked_preview, prepared_with_preview, process, reposition, warm};
use crate::{
    runtime::AudioTransportPositionUpdate,
    w30::{
        W30HookArticulationProfile, W30HookArticulationRenderState, W30PreviewRenderMode,
        W30PreviewRenderState, W30PreviewSourceProfile,
    },
};

#[test]
fn marked_pad_seeks_preserve_cursor_until_the_next_natural_step() {
    let (control_state, control_shell, mut control_callback) =
        prepared_with_preview(marked_preview());
    let (state, shell, mut callback) = prepared_with_preview(marked_preview());
    let material = state.w30_preview.try_snapshot().unwrap();
    warm(&control_shell, &mut control_callback);
    warm(&shell, &mut callback);
    let mut control = [0.0; 50];
    let mut candidate = [0.0; 50];
    // First render the natural beat1.5 trigger, then stay strictly inside its
    // half-beat step. Marked PCM and the 64-frame attack expose cursor/age resets.
    process(&mut control_callback, &mut control);
    process(&mut callback, &mut candidate);
    assert_eq!(candidate, control);
    for target in [1.65, 1.55, 1.55, 1.600_001] {
        control_shell
            .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
            .unwrap();
        shell
            .update_transport_state(true, 120.0, reposition(target))
            .unwrap();
        assert!(shell.timing_snapshot().is_none());
        process(&mut control_callback, &mut control);
        process(&mut callback, &mut candidate);
        assert_eq!(
            candidate, control,
            "seek to {target} must not reset material, attack or cursor"
        );
        assert!((shell.timing_snapshot().unwrap().position_beats - (target + 0.1)).abs() < 1e-11);
    }

    // Align the control's scheduling phase without changing rendering age.
    // Both then naturally approach beat2 together, retaining exact PCM parity.
    control_shell
        .update_transport_state(true, 120.0, reposition(1.700_001))
        .unwrap();
    assert!(control_shell.timing_snapshot().is_none());
    for _ in 0..3 {
        process(&mut control_callback, &mut control);
        process(&mut callback, &mut candidate);
        assert_eq!(candidate, control);
    }
    assert!((shell.timing_snapshot().unwrap().position_beats - 2.000_001).abs() < 1e-11);
    // Only the candidate now crosses a natural step. The control remains in
    // step3, so a source-cursor/attack retrigger must distinguish their output.
    control_shell
        .update_transport_state(true, 120.0, reposition(1.7))
        .unwrap();
    assert!(control_shell.timing_snapshot().is_none());
    process(&mut control_callback, &mut control);
    process(&mut callback, &mut candidate);
    assert_eq!(candidate[0], 0.0);
    assert!(control[0] > 0.0);
    assert_ne!(candidate, control);
    assert!(candidate.iter().any(|sample| *sample > 0.0));
    assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
    assert_eq!(control_state.w30_preview.try_snapshot().unwrap(), material);
}

fn filtered_preview() -> W30PreviewRenderState {
    let mut filtered_preview = marked_preview();
    filtered_preview
        .pad_playback
        .as_mut()
        .unwrap()
        .hook_articulation = Some(W30HookArticulationRenderState {
        profile: W30HookArticulationProfile::FilterSlamV1,
        started_at_beat: 9,
    });
    filtered_preview
}

#[test]
fn typed_filter_slam_reposition_preserves_delay_state_then_returns_sample_exactly_to_dry() {
    let (control_state, control_shell, mut control_callback) =
        prepared_with_preview(filtered_preview());
    let (state, shell, mut callback) = prepared_with_preview(filtered_preview());
    let (dry_state, dry_shell, mut dry_callback) = prepared_with_preview(marked_preview());
    let material = state.w30_preview.try_snapshot().unwrap();
    let dry_material = dry_state.w30_preview.try_snapshot().unwrap();
    warm(&control_shell, &mut control_callback);
    warm(&shell, &mut callback);
    warm(&dry_shell, &mut dry_callback);
    let mut control = [0.0; 50];
    let mut candidate = [0.0; 50];
    let mut dry = [0.0; 50];

    // Relative beat6.25 is the frozen 280Hz/Q1.2 hold, valid at this 1000Hz
    // fixture rate. Before the explicit request all callbacks were pre-gesture.
    for owner in [&control_shell, &shell, &dry_shell] {
        owner
            .update_transport_state(true, 120.0, reposition(15.25))
            .unwrap();
        assert!(owner.timing_snapshot().is_none());
    }
    process(&mut control_callback, &mut control);
    process(&mut callback, &mut candidate);
    process(&mut dry_callback, &mut dry);
    assert_eq!(candidate, control);
    assert_ne!(
        candidate, dry,
        "the typed request must reach the active FilterSlam phase"
    );
    for owner in [&control_shell, &shell, &dry_shell] {
        assert!((owner.timing_snapshot().unwrap().position_beats - 15.35).abs() < 1e-11);
    }

    // A numerically identical current cursor gets a fresh request identity.
    // Equal hold coefficients make any delay-state/cursor reset observable as
    // PCM disagreement with the same-age ordinary-Preserve filtered control.
    control_shell
        .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    shell
        .update_transport_state(true, 120.0, reposition(15.35))
        .unwrap();
    dry_shell
        .update_transport_state(true, 120.0, reposition(15.35))
        .unwrap();
    assert!(shell.timing_snapshot().is_none());
    assert!(dry_shell.timing_snapshot().is_none());
    process(&mut control_callback, &mut control);
    process(&mut callback, &mut candidate);
    process(&mut dry_callback, &mut dry);
    assert_eq!(
        candidate, control,
        "seek must preserve the prepared filter's delay state"
    );
    assert_ne!(candidate, dry);
    for owner in [&control_shell, &shell, &dry_shell] {
        assert!((owner.timing_snapshot().unwrap().position_beats - 15.45).abs() < 1e-11);
    }

    // Relative beat8.25 is beyond the frozen 20ms return-to-dry interval.
    // Compare equally aged playback, never a cold-start renderer.
    for owner in [&control_shell, &shell, &dry_shell] {
        owner
            .update_transport_state(true, 120.0, reposition(17.25))
            .unwrap();
        assert!(owner.timing_snapshot().is_none());
    }
    process(&mut control_callback, &mut control);
    process(&mut callback, &mut candidate);
    process(&mut dry_callback, &mut dry);
    assert_eq!(candidate, dry);
    assert_eq!(control, dry);
    assert!(dry.iter().any(|sample| *sample > 0.0));
    for owner in [&control_shell, &shell, &dry_shell] {
        assert!((owner.timing_snapshot().unwrap().position_beats - 17.35).abs() < 1e-11);
    }
    assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
    assert_eq!(control_state.w30_preview.try_snapshot().unwrap(), material);
    assert_eq!(dry_state.w30_preview.try_snapshot().unwrap(), dry_material);
}

#[test]
fn running_preserve_and_tempo_refresh_keep_marked_playback_continuous_within_a_step() {
    let (control_state, control_shell, mut control_callback) =
        prepared_with_preview(marked_preview());
    let (state, shell, mut callback) = prepared_with_preview(marked_preview());
    let material = state.w30_preview.try_snapshot().unwrap();
    warm(&control_shell, &mut control_callback);
    warm(&shell, &mut callback);
    let mut control = [0.0; 50];
    let mut candidate = [0.0; 50];
    process(&mut control_callback, &mut control);
    process(&mut callback, &mut candidate);
    assert_eq!(candidate, control);

    // Publishing ordinary running state must behave like no new publication.
    shell
        .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    assert!(shell.timing_snapshot().is_some());
    process(&mut control_callback, &mut control);
    process(&mut callback, &mut candidate);
    assert_eq!(candidate, control);
    assert!((shell.timing_snapshot().unwrap().position_beats - 1.7).abs() < 1e-11);

    // Native pad sample playback is not tempo-resampled. Both clocks remain
    // within the same trigger step, so a tempo refresh must change timing but
    // preserve the equally aged source cursor and attack/envelope exactly.
    for (expected, control_expected) in [(1.85, 1.8), (2.0, 1.9)] {
        shell
            .update_transport_state(true, 180.0, AudioTransportPositionUpdate::Preserve)
            .unwrap();
        assert!(shell.timing_snapshot().is_some());
        process(&mut control_callback, &mut control);
        process(&mut callback, &mut candidate);
        assert_eq!(candidate, control);
        let timing = shell.timing_snapshot().unwrap();
        assert!((timing.position_beats - expected).abs() < 1e-11);
        assert_eq!(timing.tempo_bpm, 180.0);
        assert!(
            (control_shell.timing_snapshot().unwrap().position_beats - control_expected).abs()
                < 1e-11
        );
    }
    assert!(candidate.iter().any(|sample| *sample > 0.0));
    assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
    assert_eq!(control_state.w30_preview.try_snapshot().unwrap(), material);
}

#[test]
fn stopped_audition_resumes_without_cold_reset_and_matches_explicit_cursor_delivery() {
    let mut preview = marked_preview();
    preview.mode = W30PreviewRenderMode::RawCaptureAudition;
    preview.source_profile = Some(W30PreviewSourceProfile::RawCaptureAudition);
    let (control_state, control_shell, mut control_callback) =
        prepared_with_preview(preview.clone());
    let (state, shell, mut callback) = prepared_with_preview(preview);
    let material = state.w30_preview.try_snapshot().unwrap();
    let mut control = [0.0; 50];
    let mut candidate = [0.0; 50];

    // Begin active audition while the authoritative transport is stopped.
    // Its idle audition clock advances, but the committed cursor stays1.6.
    for owner in [&control_shell, &shell] {
        owner
            .update_transport_state(false, 120.0, reposition(1.6))
            .unwrap();
        assert!(owner.timing_snapshot().is_none());
    }
    process(&mut control_callback, &mut control);
    process(&mut callback, &mut candidate);
    assert_eq!(candidate, control);
    assert!(candidate.iter().any(|sample| *sample > 0.0));
    for owner in [&control_shell, &shell] {
        let timing = owner.timing_snapshot().unwrap();
        assert!(!timing.is_transport_running);
        assert!((timing.position_beats - 1.6).abs() < 1e-11);
    }

    // Preserve/Play and explicit delivery must resume from the same cursor
    // with the existing PCM cursor/50-frame attack history still intact.
    shell
        .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    assert!(shell.timing_snapshot().is_some());
    control_shell
        .update_transport_state(true, 120.0, reposition(1.6))
        .unwrap();
    assert!(control_shell.timing_snapshot().is_none());
    process(&mut control_callback, &mut control);
    process(&mut callback, &mut candidate);
    assert_eq!(candidate, control);
    assert!(
        candidate[0] > 0.0,
        "a cold pad restart would reset its attack to zero"
    );
    for owner in [&control_shell, &shell] {
        let timing = owner.timing_snapshot().unwrap();
        assert!(timing.is_transport_running);
        assert!((timing.position_beats - 1.7).abs() < 1e-11);
    }
    assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
    assert_eq!(control_state.w30_preview.try_snapshot().unwrap(), material);
}

#[test]
fn long_backward_pitch_dive_seek_returns_to_the_same_age_dry_prefix() {
    let mut pitch_preview = marked_preview();
    pitch_preview
        .pad_playback
        .as_mut()
        .unwrap()
        .hook_articulation = Some(W30HookArticulationRenderState {
        profile: W30HookArticulationProfile::PitchDiveV1,
        started_at_beat: 0,
    });
    let (state, shell, mut callback) = prepared_with_preview(pitch_preview);
    let (dry_state, dry_shell, mut dry_callback) = prepared_with_preview(marked_preview());
    let material = state.w30_preview.try_snapshot().unwrap();
    let dry_material = dry_state.w30_preview.try_snapshot().unwrap();
    warm(&shell, &mut callback);
    warm(&dry_shell, &mut dry_callback);
    let mut candidate = [0.0; 50];
    let mut dry = [0.0; 50];

    for owner in [&shell, &dry_shell] {
        owner
            .update_transport_state(true, 120.0, reposition(12.25))
            .unwrap();
        assert!(owner.timing_snapshot().is_none());
    }
    process(&mut callback, &mut candidate);
    process(&mut dry_callback, &mut dry);
    assert!(candidate.iter().all(|sample| *sample == 0.0));
    assert!(dry.iter().any(|sample| *sample > 0.0));
    for owner in [&shell, &dry_shell] {
        assert!((owner.timing_snapshot().unwrap().position_beats - 12.35).abs() < 1e-11);
    }

    // The unchanged PitchDive profile bypasses processing before relative
    // beat8. A backward seek must reach that branch without restarting PCM.
    for owner in [&shell, &dry_shell] {
        owner
            .update_transport_state(true, 120.0, reposition(2.25))
            .unwrap();
        assert!(owner.timing_snapshot().is_none());
    }
    process(&mut callback, &mut candidate);
    process(&mut dry_callback, &mut dry);
    assert_eq!(candidate, dry);
    assert!(candidate.iter().all(|sample| *sample > 0.0));
    for owner in [&shell, &dry_shell] {
        assert!((owner.timing_snapshot().unwrap().position_beats - 2.35).abs() < 1e-11);
    }
    assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
    assert_eq!(dry_state.w30_preview.try_snapshot().unwrap(), dry_material);
}
