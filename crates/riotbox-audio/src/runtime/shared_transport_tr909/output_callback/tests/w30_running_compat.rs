//! The live safety placement must not change uninterrupted gesture samples.
use super::w30_fixture::{assert_pcm_eq, generated_preview, prepared_with_preview_format, process};
use crate::{
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
        SourceMonitorRenderState, render_runtime_mix_plan_sequence_realtime_simulation_offline,
    },
    w30::{W30HookArticulationProfile, W30HookArticulationRenderState},
};
use riotbox_core::action::SourceMonitorMode;

#[test]
fn uninterrupted_live_dry_pitch_and_filter_match_historical_running_bits() {
    for profile in [
        None,
        Some(W30HookArticulationProfile::PitchDiveV1),
        Some(W30HookArticulationProfile::FilterSlamV1),
    ] {
        for channels in [1, 2] {
            let mut preview = generated_preview(false);
            preview.pad_playback.as_mut().unwrap().hook_articulation =
                profile.map(|profile| W30HookArticulationRenderState {
                    profile,
                    started_at_beat: 0,
                });
            let plan = RuntimeMixRenderPlan {
                transport: AudioRuntimeTimingSnapshot {
                    is_transport_running: true,
                    tempo_bpm: 120.0,
                    position_beats: 0.0,
                },
                w30_preview_render: preview.clone(),
                source_monitor_render: SourceMonitorRenderState::control_only(
                    SourceMonitorMode::Riotbox,
                ),
                ..RuntimeMixRenderPlan::default()
            };
            // Thirteen beats cover both complete frozen articulation windows.
            let expected = render_runtime_mix_plan_sequence_realtime_simulation_offline(
                &[RuntimeMixRenderSequenceStep::new(&plan, 312_000)],
                48_000,
                channels,
                2_400,
            )
            .pop()
            .unwrap();
            assert!(expected.iter().any(|sample| sample.abs() > 1e-5));
            let (_state, _shell, mut callback) =
                prepared_with_preview_format(preview, 48_000, channels);
            let mut block = vec![0.0; 2_400 * usize::from(channels)];
            for expected_block in expected.chunks(block.len()) {
                process(&mut callback, &mut block);
                assert_pcm_eq(&block, expected_block);
            }
        }
    }
}
