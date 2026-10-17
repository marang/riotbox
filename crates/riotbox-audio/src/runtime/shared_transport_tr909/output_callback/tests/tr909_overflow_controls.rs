//! Generated isolated drums through the exact processor, never a device.
//! All other contributors stay Idle; the fixture has Riotbox-only monitoring
//! without source PCM. Skipped frames are silent, not simulated voice time.
use super::w30_fixture::{assert_pcm_eq, prepared_with_preview, process, reposition};
use crate::{
    runtime::{AudioRuntimeShell, AudioTransportPositionUpdate},
    tr909::{Tr909FillRecipeId, Tr909RenderMode, Tr909RenderRouting, Tr909RenderState},
    w30::W30PreviewRenderState,
};

fn assert_position(shell: &AudioRuntimeShell, expected: f64) {
    let timing = shell.timing_snapshot().unwrap();
    assert!(timing.is_transport_running);
    assert_eq!(timing.tempo_bpm.to_bits(), 120.0_f32.to_bits());
    assert!(
        (timing.position_beats - expected).abs() < 1e-10,
        "observed {} instead of {expected}",
        timing.position_beats,
    );
}

#[test]
fn fill_overflow_into_next_bar_rest_clears_old_tails_without_replaying_downbeat() {
    // The existing prepared fixture owns mono/1000Hz/fixed50-frame scratch.
    let (state, shell, mut callback) = prepared_with_preview(W30PreviewRenderState::default());
    let fill = Tr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        drum_bus_level: 0.7,
        ..Tr909RenderState::default()
    };
    assert_eq!(
        fill.fill_recipe_id(),
        Some(Tr909FillRecipeId::GenericFillV1)
    );
    shell.update_tr909_render_state(&fill);
    let material = state.tr909_render.try_snapshot().unwrap();
    shell
        .update_transport_state(true, 120.0, reposition(3.85))
        .unwrap();
    assert!(shell.timing_snapshot().is_none());
    let mut output = [0.0; 50];
    process(&mut callback, &mut output);
    assert_position(&shell, 3.95);
    assert!(output.iter().any(|sample| *sample != 0.0));

    let mut overflow = [1.0; 155];
    process(&mut callback, &mut overflow);
    assert!(overflow.iter().all(|sample| *sample == 0.0));
    assert_position(&shell, 4.26);
    assert_eq!(shell.health_snapshot().callback_scratch_overflow_count, 1);
    // GenericFill step17 is a rest. Preserve the existing full-step jump
    // reset: no prior-bar hidden voice and no replay of skipped downbeat16.
    process(&mut callback, &mut output);
    assert_position(&shell, 4.36);
    assert!(output.iter().all(|sample| *sample == 0.0));
    let health = shell.health_snapshot();
    assert_eq!(health.callback_scratch_overflow_count, 1);
    assert_eq!(health.stream_error_count, 0);
    assert_eq!(state.tr909_render.try_snapshot().unwrap(), material);
}

#[test]
fn repeated_overflow_then_superseding_seek_matches_same_warmed_seek_control() {
    for mode in [Tr909RenderMode::Takeover, Tr909RenderMode::Fill] {
        // Do not wrap these large prepared constructors in another by-value
        // helper; retain the ordinary test thread's established stack budget.
        let (candidate_state, candidate_shell, mut candidate_callback) =
            prepared_with_preview(W30PreviewRenderState::default());
        let (control_state, control_shell, mut control_callback) =
            prepared_with_preview(W30PreviewRenderState::default());
        let render = Tr909RenderState {
            mode,
            routing: if mode == Tr909RenderMode::Takeover {
                Tr909RenderRouting::DrumBusTakeover
            } else {
                Tr909RenderRouting::DrumBusSupport
            },
            drum_bus_level: 0.7,
            ..Tr909RenderState::default()
        };
        if mode == Tr909RenderMode::Fill {
            assert_eq!(
                render.fill_recipe_id(),
                Some(Tr909FillRecipeId::GenericFillV1)
            );
        }
        candidate_shell.update_tr909_render_state(&render);
        control_shell.update_tr909_render_state(&render);
        let candidate_material = candidate_state.tr909_render.try_snapshot().unwrap();
        let control_material = control_state.tr909_render.try_snapshot().unwrap();
        let mut candidate = [0.0; 50];
        let mut control = [0.0; 50];
        for _ in 0..4 {
            process(&mut candidate_callback, &mut candidate);
            process(&mut control_callback, &mut control);
            assert_pcm_eq(&candidate, &control);
        }
        assert!(candidate.iter().any(|sample| *sample != 0.0), "{mode:?}");
        assert_position(&candidate_shell, 0.4);
        assert_position(&control_shell, 0.4);

        candidate_shell
            .update_transport_state(true, 120.0, reposition(3.95))
            .unwrap();
        assert!(candidate_shell.timing_snapshot().is_none());
        let mut overflow = [1.0; 51];
        process(&mut candidate_callback, &mut overflow);
        assert!(overflow.iter().all(|sample| *sample == 0.0));
        // Overflow acknowledges shared timing even though drums did not render.
        assert_position(&candidate_shell, 4.052);
        candidate_shell
            .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
            .unwrap();
        overflow.fill(1.0);
        process(&mut candidate_callback, &mut overflow);
        assert!(overflow.iter().all(|sample| *sample == 0.0));
        assert_position(&candidate_shell, 4.154);
        assert_eq!(
            candidate_shell
                .health_snapshot()
                .callback_scratch_overflow_count,
            2
        );
        assert_position(&control_shell, 0.4);

        for shell in [&candidate_shell, &control_shell] {
            shell
                .update_transport_state(true, 120.0, reposition(12.25))
                .unwrap();
            assert!(shell.timing_snapshot().is_none());
            shell
                .update_transport_state(true, 120.0, reposition(4.0))
                .unwrap();
            assert!(shell.timing_snapshot().is_none());
            shell
                .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
                .unwrap();
            assert!(
                shell.timing_snapshot().is_none(),
                "Preserve cannot acknowledge the latest Seek"
            );
        }
        process(&mut candidate_callback, &mut candidate);
        process(&mut control_callback, &mut control);
        assert_position(&candidate_shell, 4.1);
        assert_position(&control_shell, 4.1);
        assert!(candidate.iter().any(|sample| *sample != 0.0), "{mode:?}");
        assert_pcm_eq(&candidate, &control);
        // Recovery and new request delivery neither erase overflow history nor
        // invent additional faults. Both callbacks retain published material.
        assert_eq!(
            candidate_shell
                .health_snapshot()
                .callback_scratch_overflow_count,
            2
        );
        assert_eq!(
            control_shell
                .health_snapshot()
                .callback_scratch_overflow_count,
            0
        );
        for shell in [&candidate_shell, &control_shell] {
            assert_eq!(shell.health_snapshot().stream_error_count, 0);
        }
        assert_eq!(
            candidate_state.tr909_render.try_snapshot().unwrap(),
            candidate_material
        );
        assert_eq!(
            control_state.tr909_render.try_snapshot().unwrap(),
            control_material
        );
    }
}

#[test]
fn inactive_admission_after_overflow_preserves_resume_and_mode_activation_pcm() {
    for (initial_mode, resumed_mode) in [
        (Tr909RenderMode::Takeover, Tr909RenderMode::Takeover),
        (Tr909RenderMode::Fill, Tr909RenderMode::Fill),
        (Tr909RenderMode::Fill, Tr909RenderMode::Takeover),
    ] {
        for stopped in [false, true] {
            let (candidate_state, candidate_shell, mut candidate_callback) =
                prepared_with_preview(W30PreviewRenderState::default());
            let (control_state, control_shell, mut control_callback) =
                prepared_with_preview(W30PreviewRenderState::default());
            let render = Tr909RenderState {
                mode: initial_mode,
                routing: if initial_mode == Tr909RenderMode::Takeover {
                    Tr909RenderRouting::DrumBusTakeover
                } else {
                    Tr909RenderRouting::DrumBusSupport
                },
                drum_bus_level: 0.7,
                ..Tr909RenderState::default()
            };
            candidate_shell.update_tr909_render_state(&render);
            control_shell.update_tr909_render_state(&render);
            let mut candidate = [0.0; 50];
            let mut control = [0.0; 50];
            for _ in 0..4 {
                process(&mut candidate_callback, &mut candidate);
                process(&mut control_callback, &mut control);
                assert_pcm_eq(&candidate, &control);
            }
            assert!(candidate.iter().any(|sample| *sample != 0.0));
            // Advance the control clock silently in Idle, preserving its
            // oscillator age. The existing inactive reset does NOT erase all
            // sound history, and recovery must not simulate skipped samples.
            control_shell.update_tr909_render_state(&Tr909RenderState {
                mode: Tr909RenderMode::Idle,
                ..render.clone()
            });
            let mut overflow = [1.0; 51];
            process(&mut candidate_callback, &mut overflow);
            assert!(overflow.iter().all(|sample| *sample == 0.0));
            process(&mut control_callback, &mut control);
            process(&mut control_callback, &mut [0.0; 1]);
            assert_position(&candidate_shell, 0.502);
            assert_position(&control_shell, 0.502);
            assert_eq!(
                candidate_shell
                    .timing_snapshot()
                    .unwrap()
                    .position_beats
                    .to_bits(),
                control_shell
                    .timing_snapshot()
                    .unwrap()
                    .position_beats
                    .to_bits(),
            );

            let inactive = Tr909RenderState {
                mode: if stopped {
                    initial_mode
                } else {
                    Tr909RenderMode::Idle
                },
                ..render.clone()
            };
            for shell in [&candidate_shell, &control_shell] {
                shell.update_tr909_render_state(&inactive);
                shell
                    .update_transport_state(!stopped, 120.0, AudioTransportPositionUpdate::Preserve)
                    .unwrap();
            }
            process(&mut candidate_callback, &mut candidate);
            process(&mut control_callback, &mut control);
            assert!(candidate.iter().all(|sample| *sample == 0.0));
            assert_pcm_eq(&candidate, &control);
            let inactive_position = if stopped { 0.502 } else { 0.602 };
            for shell in [&candidate_shell, &control_shell] {
                let timing = shell.timing_snapshot().unwrap();
                assert_eq!(timing.is_transport_running, !stopped);
                assert!((timing.position_beats - inactive_position).abs() < 1e-10);
            }

            let resumed = Tr909RenderState {
                mode: resumed_mode,
                routing: if resumed_mode == Tr909RenderMode::Takeover {
                    Tr909RenderRouting::DrumBusTakeover
                } else {
                    Tr909RenderRouting::DrumBusSupport
                },
                ..render
            };
            for shell in [&candidate_shell, &control_shell] {
                shell.update_tr909_render_state(&resumed);
                shell
                    .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
                    .unwrap();
            }
            let candidate_material = candidate_state.tr909_render.try_snapshot().unwrap();
            let control_material = control_state.tr909_render.try_snapshot().unwrap();
            for block_index in 1..=2 {
                process(&mut candidate_callback, &mut candidate);
                process(&mut control_callback, &mut control);
                assert_pcm_eq(&candidate, &control);
                assert_position(
                    &candidate_shell,
                    inactive_position + f64::from(block_index) * 0.1,
                );
                assert_position(
                    &control_shell,
                    inactive_position + f64::from(block_index) * 0.1,
                );
                assert!(
                    candidate.iter().any(|sample| *sample != 0.0),
                    "{initial_mode:?}→{resumed_mode:?}, stopped={stopped}"
                );
            }
            assert_eq!(
                candidate_shell
                    .health_snapshot()
                    .callback_scratch_overflow_count,
                1
            );
            assert_eq!(
                control_shell
                    .health_snapshot()
                    .callback_scratch_overflow_count,
                0
            );
            for shell in [&candidate_shell, &control_shell] {
                assert_eq!(shell.health_snapshot().stream_error_count, 0);
            }
            assert_eq!(
                candidate_state.tr909_render.try_snapshot().unwrap(),
                candidate_material
            );
            assert_eq!(
                control_state.tr909_render.try_snapshot().unwrap(),
                control_material
            );
        }
    }
}
