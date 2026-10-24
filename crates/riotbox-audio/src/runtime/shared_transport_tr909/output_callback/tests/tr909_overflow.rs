//! Isolated generated drums through the exact production callback, not a device.
use super::w30_fixture::{assert_pcm_eq, prepared_with_preview, process, reposition};
use crate::{
    runtime::AudioTransportPositionUpdate,
    tr909::{Tr909RenderMode, Tr909RenderRouting, Tr909RenderState},
    w30::W30PreviewRenderState,
};

fn routing(mode: Tr909RenderMode) -> Tr909RenderRouting {
    if mode == Tr909RenderMode::Takeover {
        Tr909RenderRouting::DrumBusTakeover
    } else {
        Tr909RenderRouting::DrumBusSupport
    }
}

#[test]
fn short_overflow_does_not_replay_the_old_takeover_timeline() {
    short_gap_recovery(Tr909RenderMode::Takeover, 0.0);
}

#[test]
fn short_fill_overflow_uses_the_existing_nonzero_source_bar_anchor() {
    short_gap_recovery(Tr909RenderMode::Fill, 0.125);
}

fn short_gap_recovery(mode: Tr909RenderMode, anchor: f64) {
    // Fixed 50-frame scratch at 1000 Hz/mono/120 BPM. All other contributors
    // are explicitly Idle; source monitoring is Riotbox-only without source PCM.
    let (candidate_state, candidate_shell, mut candidate_callback) =
        prepared_with_preview(W30PreviewRenderState::default());
    let (control_state, control_shell, mut control_callback) =
        prepared_with_preview(W30PreviewRenderState::default());
    let render = Tr909RenderState {
        mode,
        routing: routing(mode),
        drum_bus_level: 0.7,
        source_bar_grid_anchor_position_beats: Some(anchor),
        ..Tr909RenderState::default()
    };
    candidate_shell.update_tr909_render_state(&render);
    control_shell.update_tr909_render_state(&render);
    for shell in [&candidate_shell, &control_shell] {
        shell
            .update_transport_state(true, 120.0, reposition(anchor))
            .unwrap();
    }
    let mut candidate = [0.0; 50];
    let mut control = [0.0; 50];
    for _ in 0..4 {
        process(&mut candidate_callback, &mut candidate);
        process(&mut control_callback, &mut control);
        assert_pcm_eq(&candidate, &control);
    }
    assert!(
        (candidate_shell.timing_snapshot().unwrap().position_beats - (anchor + 0.4)).abs() < 1e-10
    );
    let mut overflow = [1.0; 51];
    process(&mut candidate_callback, &mut overflow);
    assert!(overflow.iter().all(|sample| *sample == 0.0));
    assert_eq!(
        candidate_shell
            .health_snapshot()
            .callback_scratch_overflow_count,
        1
    );
    assert!(
        (candidate_shell.timing_snapshot().unwrap().position_beats - (anchor + 0.502)).abs()
            < 1e-10
    );

    // Recovery starts beyond the natural half-beat trigger. A no-overflow
    // control still starts at 0.4; output must not repeat that old interval.
    process(&mut candidate_callback, &mut candidate);
    process(&mut control_callback, &mut control);
    assert!(
        (candidate_shell.timing_snapshot().unwrap().position_beats - (anchor + 0.602)).abs()
            < 1e-10
    );
    assert!(
        (control_shell.timing_snapshot().unwrap().position_beats - (anchor + 0.5)).abs() < 1e-10
    );
    assert_eq!(
        candidate_state.tr909_render.try_snapshot(),
        control_state.tr909_render.try_snapshot()
    );
    assert!(candidate.iter().any(|sample| *sample != 0.0));
    assert_ne!(
        candidate, control,
        "recovery must not render the stale 0.4→0.5 timeline while the clock reaches 0.602"
    );
}

#[test]
fn same_step_recovery_retains_voice_age_but_the_next_attack_follows_the_clock() {
    for mode in [Tr909RenderMode::Takeover, Tr909RenderMode::Fill] {
        let (_state, shell, mut callback) = prepared_with_preview(W30PreviewRenderState::default());
        let (_control_state, control_shell, mut control_callback) =
            prepared_with_preview(W30PreviewRenderState::default());
        let render = Tr909RenderState {
            mode,
            routing: routing(mode),
            drum_bus_level: 0.7,
            ..Tr909RenderState::default()
        };
        shell.update_tr909_render_state(&render);
        control_shell.update_tr909_render_state(&render);
        let mut block = [0.0; 50];
        let mut control = [0.0; 50];
        process(&mut callback, &mut block);
        process(&mut control_callback, &mut control);
        assert_pcm_eq(&block, &control);
        let mut overflow = [1.0; 51];
        process(&mut callback, &mut overflow);
        assert!(overflow.iter().all(|sample| *sample == 0.0));
        // Both windows remain inside step0, including the finer Fill grid.
        process(&mut callback, &mut block[..20]);
        process(&mut control_callback, &mut control[..20]);
        assert!(block[..20].iter().any(|sample| *sample != 0.0));
        assert_pcm_eq(&block[..20], &control[..20]);
        assert!((shell.timing_snapshot().unwrap().position_beats - 0.242).abs() < 1e-10);
        // Generic Fill step1 is a rest. Neither it nor the overflow fabricates
        // an attack; the next sounding boundary is beat0.5 in both modes.
        for _ in 0..2 {
            process(&mut callback, &mut block);
            process(&mut control_callback, &mut control);
            assert_pcm_eq(&block, &control);
        }
        process(&mut callback, &mut block);
        process(&mut control_callback, &mut control);
        assert!((shell.timing_snapshot().unwrap().position_beats - 0.542).abs() < 1e-10);
        assert!((control_shell.timing_snapshot().unwrap().position_beats - 0.44).abs() < 1e-10);
        assert_ne!(
            block, control,
            "the next natural attack cannot remain delayed by the skipped interval"
        );
    }
}

#[test]
fn repeated_preserve_overflows_accumulate_below_both_legacy_deadbands() {
    for mode in [Tr909RenderMode::Takeover, Tr909RenderMode::Fill] {
        let (_state, shell, mut callback) = prepared_with_preview(W30PreviewRenderState::default());
        let (_control_state, control_shell, mut control_callback) =
            prepared_with_preview(W30PreviewRenderState::default());
        let render = Tr909RenderState {
            mode,
            routing: routing(mode),
            drum_bus_level: 0.7,
            ..Tr909RenderState::default()
        };
        shell.update_tr909_render_state(&render);
        control_shell.update_tr909_render_state(&render);
        for shell in [&shell, &control_shell] {
            shell
                .update_transport_state(true, 60.0, AudioTransportPositionUpdate::Preserve)
                .unwrap();
        }
        let mut block = [0.0; 50];
        let mut control = [0.0; 50];
        for _ in 0..8 {
            process(&mut callback, &mut block);
            process(&mut control_callback, &mut control);
            assert_pcm_eq(&block, &control);
        }
        for _ in 0..2 {
            let mut overflow = [1.0; 51];
            process(&mut callback, &mut overflow);
            assert!(overflow.iter().all(|sample| *sample == 0.0));
        }
        assert_eq!(shell.health_snapshot().callback_scratch_overflow_count, 2);
        assert!((shell.timing_snapshot().unwrap().position_beats - 0.502).abs() < 1e-10);
        process(&mut callback, &mut block);
        process(&mut control_callback, &mut control);
        assert!((shell.timing_snapshot().unwrap().position_beats - 0.552).abs() < 1e-10);
        assert!((control_shell.timing_snapshot().unwrap().position_beats - 0.45).abs() < 1e-10);
        assert_ne!(
            block, control,
            "a total0.102beat gap must bypass both deadbands, not just a large-jump reset"
        );
    }
}
