//! Historical public numeric offline stop bytes, frozen from pre-fix dd89ec9.
use crate::{
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
        SourceMonitorRenderState, render_runtime_mix_plan_sequence_realtime_simulation_offline,
    },
    w30::{
        W30_PAD_CHOP_SLICE_COUNT, W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30HookArticulationProfile,
        W30HookArticulationRenderState, W30PadPlaybackSampleWindow, W30PreviewRenderMode,
        W30PreviewRenderRouting, W30PreviewRenderState, W30PreviewSourceProfile,
    },
};
use riotbox_core::action::SourceMonitorMode;
use sha2::{Digest, Sha256};

fn historical_sequence(profile: Option<W30HookArticulationProfile>, channels: u16) -> String {
    let sample_rate = 48_000;
    let start = if profile == Some(W30HookArticulationProfile::FilterSlamV1) {
        5.25
    } else {
        8.25
    };
    let running = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 120.0,
            position_beats: start,
        },
        w30_preview_render: W30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
            trigger_revision: 1,
            trigger_velocity: 0.8,
            pad_playback: Some(W30PadPlaybackSampleWindow {
                source_start_frame: 0,
                source_end_frame: 48_000,
                source_sample_rate: sample_rate,
                playback_frame_count: 48_000,
                sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
                loop_enabled: true,
                playback_rate: 1.0,
                reverse: false,
                gate_step_fraction: 0.0,
                loop_crossfade_sample_count: 0,
                chop_slice_count: 0,
                chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
                hook_articulation: profile.map(|profile| W30HookArticulationRenderState {
                    profile,
                    started_at_beat: 0,
                }),
                samples: [0.2; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
            }),
            music_bus_level: 0.6,
            ..W30PreviewRenderState::default()
        },
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
        ..RuntimeMixRenderPlan::default()
    };
    let mut stopped = running.clone();
    stopped.transport.is_transport_running = false;
    let mut digest = Sha256::new();
    // Before, at and beyond the240-frame fade; each separately warms the
    // original state. Include resume with/without another latched stop block.
    for stop_frames in [120, 240, 2_400] {
        for extra_stopped_block in [false, true] {
            let mut steps = vec![
                RuntimeMixRenderSequenceStep::new(&running, 24_000),
                RuntimeMixRenderSequenceStep::new(&stopped, stop_frames),
            ];
            if extra_stopped_block {
                steps.push(RuntimeMixRenderSequenceStep::new(&stopped, 128));
            }
            steps.push(RuntimeMixRenderSequenceStep::new(&running, 512));
            for output in render_runtime_mix_plan_sequence_realtime_simulation_offline(
                &steps,
                sample_rate,
                channels,
                128,
            ) {
                assert_eq!(output.len() % usize::from(channels), 0);
                for sample in output {
                    assert!(sample.is_finite());
                    digest.update(sample.to_le_bytes());
                }
            }
        }
    }
    hex::encode(digest.finalize())
}

#[test]
fn public_numeric_stop_and_resume_keep_pre_fix_sample_bytes() {
    for (profile, channels, expected) in [
        (
            None,
            1,
            "6e94c4f1048a67cf65ac6eebfe8504b66fe78a1b4034ac882196bf420db01f8c",
        ),
        (
            None,
            2,
            "e90279771f2c91888f7f08fce1e1928e7574e17cbc7dfa499fd26da6a77bc9a3",
        ),
        (
            Some(W30HookArticulationProfile::PitchDiveV1),
            1,
            "6cc2d8acba567b07db3427b7970888ba50f95c73fb8e4374f335323d87d5e042",
        ),
        (
            Some(W30HookArticulationProfile::PitchDiveV1),
            2,
            "003297bc379d6edd8ebebddd6d5480d2243cc5802d967214f9d1971408b4cf28",
        ),
        (
            Some(W30HookArticulationProfile::FilterSlamV1),
            1,
            "b79dc7554e36721aa32b15f9a4c246ea3fae27e81ced1e485ab22f119a054c86",
        ),
        (
            Some(W30HookArticulationProfile::FilterSlamV1),
            2,
            "dd4389ace9d07d68b70f274ece8745791d802e53cf2551e1999d24ebd389fc92",
        ),
    ] {
        let actual = historical_sequence(profile, channels);
        assert_eq!(actual, expected, "historical {profile:?}/{channels}");
    }
}
