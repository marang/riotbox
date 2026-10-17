//! Final stop silence through the exact prepared production callback.
//! Generated in-memory PCM only; no device, source file or playback.
use super::w30_fixture::{
    assert_pcm_eq, generated_preview, prepared, prepared_with_preview_format, process, reposition,
    warm,
};
use crate::{
    runtime::{
        AudioRuntimeShell, AudioRuntimeTimingSnapshot, AudioTransportPositionUpdate,
        RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline,
    },
    w30::{
        W30_RESAMPLE_SOURCE_WINDOW_LEN, W30HookArticulationProfile, W30HookArticulationRenderState,
        W30PreviewRenderState, W30ResampleSourceWindow, W30ResampleTapAvailability,
        W30ResampleTapMode, W30ResampleTapRouting, W30ResampleTapSourceProfile,
        W30ResampleTapState,
    },
};

const SAMPLE_RATE: u32 = 48_000;
const BLOCK_FRAMES: usize = 2_400;
const FADE_FRAMES: usize = 240;

fn preview(profile: Option<W30HookArticulationProfile>) -> W30PreviewRenderState {
    let mut preview = generated_preview(false);
    let pad = preview.pad_playback.as_mut().unwrap();
    pad.source_end_frame = u64::from(SAMPLE_RATE);
    pad.source_sample_rate = SAMPLE_RATE;
    pad.playback_frame_count = u64::from(SAMPLE_RATE);
    pad.hook_articulation = profile.map(|profile| W30HookArticulationRenderState {
        profile,
        started_at_beat: 0,
    });
    preview
}

fn prime(
    shell: &AudioRuntimeShell,
    callback: &mut impl FnMut(&mut [f32]),
    channels: u16,
    target: f64,
) {
    // Half a second establishes causal history/filter state, not just an
    // articulation label. At120BPM it advances exactly one beat.
    shell
        .update_transport_state(true, 120.0, reposition(target - 1.0))
        .unwrap();
    let mut output = vec![0.0; BLOCK_FRAMES * usize::from(channels)];
    for _ in 0..10 {
        process(callback, &mut output);
    }
    assert!(output.iter().any(|sample| *sample != 0.0));
    assert!((shell.timing_snapshot().unwrap().position_beats - target).abs() < 1e-9);
}

fn pause(shell: &AudioRuntimeShell) {
    shell
        .update_transport_state(false, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
}

fn assert_stop(output: &[f32], channels: u16) {
    let channels = usize::from(channels);
    assert!(
        output[..FADE_FRAMES * channels]
            .iter()
            .any(|sample| *sample != 0.0)
    );
    assert!(
        output[(FADE_FRAMES - 1) * channels..FADE_FRAMES * channels]
            .iter()
            .all(|sample| *sample != 0.0),
        "the fade counts frames once, not once per output channel"
    );
    assert!(
        output[FADE_FRAMES * channels..]
            .iter()
            .all(|sample| *sample == 0.0),
        "stateful output must be silent after the existing five-ms bound"
    );
}

#[test]
fn active_pitch_dive_is_silent_after_five_ms_inside_the_first_stopped_block() {
    let (state, shell, mut callback) = prepared(true);
    let material = state.w30_preview.try_snapshot().unwrap();
    warm(&shell, &mut callback);
    shell
        .update_transport_state(true, 120.0, reposition(8.25))
        .unwrap();
    let mut output = [0.0; 50];
    for _ in 0..10 {
        process(&mut callback, &mut output);
    }
    assert!(output.iter().any(|sample| *sample > 0.0));
    assert!((shell.timing_snapshot().unwrap().position_beats - 9.25).abs() < 1e-10);
    shell
        .update_transport_state(false, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    process(&mut callback, &mut output);
    assert!(!shell.timing_snapshot().unwrap().is_transport_running);
    assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
    assert!(output[..5].iter().any(|sample| *sample != 0.0));
    assert!(
        output[5..].iter().all(|sample| *sample == 0.0),
        "PitchDive history must not bypass the existing five-ms lane stop bound"
    );
}

#[test]
fn dry_pitch_and_filter_close_hold_return_stop_in_every_mono_and_stereo_channel() {
    for channels in [1, 2] {
        for (profile, target) in [
            (None, 9.25),
            (Some(W30HookArticulationProfile::PitchDiveV1), 9.25),
            (Some(W30HookArticulationProfile::FilterSlamV1), 2.25),
            (Some(W30HookArticulationProfile::FilterSlamV1), 6.25),
            // Five ms into the frozen20ms return leaves a genuinely active
            // filter during the entire subsequent five-ms transport fade.
            (Some(W30HookArticulationProfile::FilterSlamV1), 7.01),
        ] {
            let (state, shell, mut callback) =
                prepared_with_preview_format(preview(profile), SAMPLE_RATE, channels);
            let material = state.w30_preview.try_snapshot().unwrap();
            prime(&shell, &mut callback, channels, target);
            pause(&shell);
            let mut output = vec![0.0; BLOCK_FRAMES * usize::from(channels)];
            process(&mut callback, &mut output);
            assert_stop(&output, channels);
            let stopped = shell.timing_snapshot().unwrap();
            assert!(!stopped.is_transport_running);
            assert!((stopped.position_beats - target).abs() < 1e-9);
            assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
        }
    }
}

#[test]
fn historical_filter_counterfactual_has_a_real_tail_that_live_stop_must_remove() {
    for channels in [1, 2] {
        for target in [2.25, 6.25, 7.01] {
            let running = RuntimeMixRenderPlan {
                transport: AudioRuntimeTimingSnapshot {
                    is_transport_running: true,
                    tempo_bpm: 120.0,
                    position_beats: target - 1.0,
                },
                w30_preview_render: preview(Some(W30HookArticulationProfile::FilterSlamV1)),
                ..RuntimeMixRenderPlan::default()
            };
            let mut stopped = running.clone();
            stopped.transport.is_transport_running = false;
            stopped.transport.position_beats = target;
            let output = render_runtime_mix_plan_sequence_realtime_simulation_offline(
                &[
                    RuntimeMixRenderSequenceStep::new(&running, SAMPLE_RATE as usize / 2),
                    RuntimeMixRenderSequenceStep::new(&stopped, BLOCK_FRAMES),
                ],
                SAMPLE_RATE,
                channels,
                BLOCK_FRAMES,
            );
            // Deliberately historical diagnostic output, never live-safety
            // evidence: proves these fixtures excite the original filter tail.
            assert!(
                output[1][FADE_FRAMES * usize::from(channels)..]
                    .iter()
                    .any(|sample| sample.abs() > 1e-8)
            );
        }
    }
}

fn stopped_partition(
    profile: W30HookArticulationProfile,
    channels: u16,
    chunks: &[usize],
) -> Vec<f32> {
    let target = if profile == W30HookArticulationProfile::PitchDiveV1 {
        9.25
    } else {
        6.25
    };
    let (state, shell, mut callback) =
        prepared_with_preview_format(preview(Some(profile)), SAMPLE_RATE, channels);
    let material = state.w30_preview.try_snapshot().unwrap();
    prime(&shell, &mut callback, channels, target);
    pause(&shell);
    let mut stopped = Vec::new();
    for frames in chunks {
        let mut output = vec![0.0; frames * usize::from(channels)];
        process(&mut callback, &mut output);
        stopped.extend(output);
    }
    for _ in 0..2 {
        let mut latched = vec![1.0; 128 * usize::from(channels)];
        process(&mut callback, &mut latched);
        assert!(latched.iter().all(|sample| *sample == 0.0));
    }
    assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
    let timing = shell.timing_snapshot().unwrap();
    assert!(!timing.is_transport_running);
    assert!((timing.position_beats - target).abs() < 1e-9);
    stopped
}

#[test]
fn split_stop_matches_the_same_stop_window_and_keeps_latched_silence() {
    for profile in [
        W30HookArticulationProfile::PitchDiveV1,
        W30HookArticulationProfile::FilterSlamV1,
    ] {
        for channels in [1, 2] {
            let whole = stopped_partition(profile, channels, &[BLOCK_FRAMES]);
            let split = stopped_partition(profile, channels, &[239, 1, 2160]);
            assert_stop(&split, channels);
            assert_pcm_eq(&split, &whole);
        }
    }
}

#[test]
fn resume_preserves_the_three_existing_stop_activation_topologies() {
    for profile in [
        W30HookArticulationProfile::PitchDiveV1,
        W30HookArticulationProfile::FilterSlamV1,
    ] {
        for channels in [1, 2] {
            for (stop_frames, extra_latched_block) in [(120, false), (2400, false), (2400, true)] {
                let target = if profile == W30HookArticulationProfile::PitchDiveV1 {
                    9.25
                } else {
                    6.25
                };
                let (state, shell, mut callback) =
                    prepared_with_preview_format(preview(Some(profile)), SAMPLE_RATE, channels);
                let material = state.w30_preview.try_snapshot().unwrap();
                prime(&shell, &mut callback, channels, target);
                pause(&shell);
                let mut stopped = vec![0.0; stop_frames * usize::from(channels)];
                process(&mut callback, &mut stopped);
                if extra_latched_block {
                    process(&mut callback, &mut stopped);
                    assert!(stopped.iter().all(|sample| *sample == 0.0));
                }
                shell
                    .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
                    .unwrap();
                let mut resumed = vec![0.0; BLOCK_FRAMES * usize::from(channels)];
                process(&mut callback, &mut resumed);
                assert!(resumed.iter().any(|sample| *sample != 0.0));
                let first_frame = &resumed[..usize::from(channels)];
                if extra_latched_block {
                    assert!(
                        first_frame.iter().all(|sample| *sample == 0.0),
                        "ordinary activation retains its attack reset"
                    );
                } else {
                    assert!(
                        first_frame.iter().all(|sample| *sample != 0.0),
                        "no later stopped callback consumed activation/history"
                    );
                }
                let timing = shell.timing_snapshot().unwrap();
                assert!(timing.is_transport_running);
                assert!((timing.position_beats - target - 0.1).abs() < 1e-9);
                assert_eq!(state.w30_preview.try_snapshot().unwrap(), material);
            }
        }
    }
}

#[test]
fn explicit_stopped_trigger_uses_published_material_without_the_superseded_articulation() {
    for profile in [
        W30HookArticulationProfile::PitchDiveV1,
        W30HookArticulationProfile::FilterSlamV1,
    ] {
        for channels in [1, 2] {
            let target = if profile == W30HookArticulationProfile::PitchDiveV1 {
                9.25
            } else {
                6.25
            };
            let mut material = preview(Some(profile));
            let (state, shell, mut callback) =
                prepared_with_preview_format(material.clone(), SAMPLE_RATE, channels);
            prime(&shell, &mut callback, channels, target);
            pause(&shell);
            let mut output = vec![0.0; BLOCK_FRAMES * usize::from(channels)];
            process(&mut callback, &mut output);
            process(&mut callback, &mut output);
            assert!(output.iter().all(|sample| *sample == 0.0));
            // Ordinary committed playback supersedes timed articulation under
            // RBX306. Publish that real render material, not just a revision
            // increment that leaves an impossible product articulation behind.
            material.trigger_revision += 1;
            material.pad_playback.as_mut().unwrap().hook_articulation = None;
            shell.update_w30_preview_render_state(&material);
            let published = state.w30_preview.try_snapshot().unwrap();
            process(&mut callback, &mut output);
            assert!(
                output[..usize::from(channels)]
                    .iter()
                    .all(|sample| *sample == 0.0)
            );
            assert!(
                output[usize::from(channels)..]
                    .iter()
                    .any(|sample| *sample != 0.0)
            );
            let timing = shell.timing_snapshot().unwrap();
            assert!(!timing.is_transport_running);
            assert!((timing.position_beats - target).abs() < 1e-9);
            assert_eq!(state.w30_preview.try_snapshot().unwrap(), published);
        }
    }
}

#[test]
fn articulated_stop_does_not_apply_a_second_fade_to_the_named_resample_contributor() {
    let tap = W30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        availability: W30ResampleTapAvailability::SourceAudioReady,
        source_profile: Some(W30ResampleTapSourceProfile::RawCapture),
        source_capture_id: Some("generated-lineage-only".into()),
        source_audio: Some(Box::new(W30ResampleSourceWindow {
            source_start_frame: 0,
            source_sample_rate: SAMPLE_RATE,
            source_frame_count: W30_RESAMPLE_SOURCE_WINDOW_LEN as u64,
            sample_count: W30_RESAMPLE_SOURCE_WINDOW_LEN,
            samples: [0.1; W30_RESAMPLE_SOURCE_WINDOW_LEN],
        })),
        lineage_capture_count: 1,
        generation_depth: 1,
        music_bus_level: 0.6,
        ..W30ResampleTapState::default()
    };
    let mut outputs = Vec::new();
    // The only contributors are the named preview and resample tap. TR909,
    // MC202 and source PCM remain absent; Riotbox-only monitoring is explicit.
    for (with_preview, with_tap) in [(true, false), (false, true), (true, true)] {
        let material = if with_preview {
            preview(Some(W30HookArticulationProfile::PitchDiveV1))
        } else {
            W30PreviewRenderState::default()
        };
        let (_, shell, mut callback) = prepared_with_preview_format(material, SAMPLE_RATE, 2);
        if with_tap {
            shell.update_w30_resample_tap_state(&tap);
        }
        prime(&shell, &mut callback, 2, 9.25);
        pause(&shell);
        let mut output = vec![0.0; BLOCK_FRAMES * 2];
        process(&mut callback, &mut output);
        assert_stop(&output, 2);
        assert!(
            output.iter().all(|sample| sample.abs() < 0.3),
            "limiter stays inactive"
        );
        outputs.push(output);
    }
    let expected: Vec<_> = outputs[0]
        .iter()
        .zip(&outputs[1])
        .map(|(preview, tap)| preview + tap)
        .collect();
    assert_pcm_eq(&outputs[2], &expected);
}
