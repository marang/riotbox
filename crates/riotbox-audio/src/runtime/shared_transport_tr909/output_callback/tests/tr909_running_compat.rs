//! Ordinary live drum Preserve/tempo output retains the historical numeric path.
use super::w30_fixture::{assert_pcm_eq, prepared_with_preview_format, process};
use crate::{
    runtime::{
        AudioRuntimeTimingSnapshot, AudioTransportPositionUpdate, RuntimeMixRenderPlan,
        RuntimeMixRenderSequenceStep, render_runtime_mix_plan_sequence_realtime_simulation_offline,
    },
    tr909::{
        Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
        Tr909RenderState,
    },
    w30::W30PreviewRenderState,
};

#[test]
fn uninterrupted_takeover_and_fill_preserve_and_tempo_match_historical_pcm_bits() {
    for (mode, adoption, variation) in [
        (Tr909RenderMode::Takeover, None, None),
        (Tr909RenderMode::Fill, None, None),
        (
            Tr909RenderMode::Fill,
            Some(Tr909PatternAdoption::MainlineDrive),
            Some(Tr909PhraseVariation::PhraseDrive),
        ),
        (
            Tr909RenderMode::Fill,
            Some(Tr909PatternAdoption::MainlineDrive),
            Some(Tr909PhraseVariation::PhraseDriveHardCut),
        ),
    ] {
        for rate in [1000, 48_000] {
            for channels in [1, 2] {
                let render = Tr909RenderState {
                    mode,
                    pattern_adoption: adoption,
                    phrase_variation: variation,
                    routing: if mode == Tr909RenderMode::Takeover {
                        Tr909RenderRouting::DrumBusTakeover
                    } else {
                        Tr909RenderRouting::DrumBusSupport
                    },
                    drum_bus_level: 0.7,
                    source_bar_grid_anchor_position_beats: Some(0.125),
                    ..Tr909RenderState::default()
                };
                let first = RuntimeMixRenderPlan {
                    transport: AudioRuntimeTimingSnapshot {
                        is_transport_running: true,
                        tempo_bpm: 120.0,
                        position_beats: 0.0,
                    },
                    tr909_render: render.clone(),
                    ..RuntimeMixRenderPlan::default()
                };
                let second = RuntimeMixRenderPlan {
                    transport: AudioRuntimeTimingSnapshot {
                        tempo_bpm: 90.0,
                        position_beats: 1.0,
                        ..first.transport
                    },
                    ..first.clone()
                };
                let expected = render_runtime_mix_plan_sequence_realtime_simulation_offline(
                    &[
                        RuntimeMixRenderSequenceStep::new(&first, rate as usize / 2),
                        RuntimeMixRenderSequenceStep::new(&second, rate as usize * 4),
                    ],
                    rate,
                    channels,
                    rate as usize / 20,
                );
                assert!(expected.iter().flatten().any(|sample| *sample != 0.0));
                let (_state, shell, mut callback) =
                    prepared_with_preview_format(W30PreviewRenderState::default(), rate, channels);
                shell.update_tr909_render_state(&render);
                let mut block = vec![0.0; rate as usize / 20 * usize::from(channels)];
                for (tempo, expected) in [120.0, 90.0].into_iter().zip(expected) {
                    shell
                        .update_transport_state(true, tempo, AudioTransportPositionUpdate::Preserve)
                        .unwrap();
                    for expected in expected.chunks(block.len()) {
                        process(&mut callback, &mut block);
                        assert_pcm_eq(&block, expected);
                    }
                }
                assert!((shell.timing_snapshot().unwrap().position_beats - 7.0).abs() < 1e-8);
                assert_eq!(shell.health_snapshot().callback_scratch_overflow_count, 0);
            }
        }
    }
}
