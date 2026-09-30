use crate::runtime::render_tr909_w30_preview::{
    render_w30_preview_buffer, render_w30_resample_tap_buffer,
};
use crate::runtime::shared_w30_resample_callback::{
    RealtimeW30ResampleTapState, W30PreviewCallbackState, W30ResampleTapCallbackState,
};
use crate::runtime::tests::mix_plan_fixtures::{
    runtime_mix_parity_source_window, runtime_mix_resample_source,
};
use crate::runtime::tests::synthetic_sources::{
    positive_realtime_resample_source, positive_realtime_source_window,
};
use crate::runtime::w30_preview_snapshot::{
    RealtimeW30PadPlaybackSampleWindow, RealtimeW30PreviewRenderState,
};
use crate::runtime::{
    AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
    render_runtime_mix_plan_sequence_realtime_simulation_offline,
};
use crate::w30::{
    W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState, W30PreviewSourceProfile,
    W30ResampleTapAvailability, W30ResampleTapMode, W30ResampleTapRouting,
    W30ResampleTapSourceProfile, W30ResampleTapState,
};

#[test]
fn transport_stop_fades_an_active_w30_preview_and_latches_silence() {
    let mut state = W30PreviewCallbackState::default();
    let running_render = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::RawCaptureAudition,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: positive_realtime_source_window(),
        pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 8.0,
    };
    let mut running = [0.0_f32; 1_024];
    render_w30_preview_buffer(&mut running, 44_100, 2, &running_render, &mut state);
    assert!(running.iter().any(|sample| sample.abs() > 0.0001));

    let mut stopped = [0.0_f32; 1_024];
    render_w30_preview_buffer(
        &mut stopped,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            is_transport_running: false,
            position_beats: 8.25,
            ..running_render
        },
        &mut state,
    );

    let fade_sample_count = usize::try_from(44_100 / 200).unwrap() * 2;
    assert!(
        stopped[..fade_sample_count]
            .iter()
            .any(|sample| sample.abs() > 0.0001)
    );
    assert!(
        (stopped[0] - running[running.len() - 2]).abs() < 0.10,
        "W-30 transport-stop fade introduced a hard edge"
    );
    assert!(
        stopped[fade_sample_count..]
            .iter()
            .all(|sample| sample.abs() <= f32::EPSILON)
    );

    let mut latched = [0.0_f32; 1_024];
    render_w30_preview_buffer(
        &mut latched,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            is_transport_running: false,
            position_beats: 8.25,
            ..running_render
        },
        &mut state,
    );
    assert!(latched.iter().all(|sample| sample.abs() <= f32::EPSILON));

    let mut manually_retriggered = [0.0_f32; 1_024];
    render_w30_preview_buffer(
        &mut manually_retriggered,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            trigger_revision: 1,
            is_transport_running: false,
            position_beats: 8.25,
            ..running_render
        },
        &mut state,
    );
    assert!(
        manually_retriggered
            .iter()
            .any(|sample| sample.abs() > 0.0001)
    );

    let mut resumed = [0.0_f32; 1_024];
    render_w30_preview_buffer(&mut resumed, 44_100, 2, &running_render, &mut state);
    assert!(resumed.iter().any(|sample| sample.abs() > 0.0001));
}

#[test]
fn transport_stop_fades_the_internal_resample_tap_and_stays_silent() {
    let mut state = W30ResampleTapCallbackState::default();
    let running_render = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::RawCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 1,
        generation_depth: 0,
        music_bus_level: 0.58,
        grit_level: 0.4,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 0.0,
    };
    let mut running = [0.0_f32; 1_024];
    render_w30_resample_tap_buffer(&mut running, 44_100, 2, &running_render, &mut state);
    assert!(running.iter().any(|sample| sample.abs() > 0.0001));
    let expected_beats = 512.0 * 128.0 / 60.0 / 44_100.0;
    assert!(
        (state.beat_position - expected_beats).abs() < 1.0e-9,
        "resample tap drifted from transport tempo: expected {expected_beats}, got {}",
        state.beat_position
    );

    let mut stopped = [0.0_f32; 1_024];
    render_w30_resample_tap_buffer(
        &mut stopped,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            is_transport_running: false,
            ..running_render
        },
        &mut state,
    );
    let fade_sample_count = usize::try_from(44_100 / 200).unwrap() * 2;
    assert!(
        stopped[..fade_sample_count]
            .iter()
            .any(|sample| sample.abs() > 0.0001)
    );
    assert!(
        (stopped[0] - running[running.len() - 2]).abs() < 0.10,
        "resample-tap transport-stop fade introduced a hard edge"
    );
    assert!(
        stopped[fade_sample_count..]
            .iter()
            .all(|sample| sample.abs() <= f32::EPSILON)
    );

    let mut latched = [0.0_f32; 1_024];
    render_w30_resample_tap_buffer(
        &mut latched,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            is_transport_running: false,
            ..running_render
        },
        &mut state,
    );
    assert!(latched.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn exact_runtime_mix_transport_stop_fades_and_silences_all_w30_paths() {
    let running = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 130.0,
            position_beats: 8.0,
        },
        w30_preview_render: W30PreviewRenderState {
            mode: W30PreviewRenderMode::RawCaptureAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
            source_window_preview: Some(runtime_mix_parity_source_window()),
            music_bus_level: 0.58,
            grit_level: 0.4,
            is_transport_running: true,
            tempo_bpm: 130.0,
            position_beats: 8.0,
            ..W30PreviewRenderState::default()
        },
        w30_resample_tap: W30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            availability: W30ResampleTapAvailability::SourceAudioReady,
            source_profile: Some(W30ResampleTapSourceProfile::RawCapture),
            source_capture_id: Some("stop-proof-capture".into()),
            source_audio: Some(Box::new(runtime_mix_resample_source())),
            lineage_capture_count: 1,
            generation_depth: 0,
            music_bus_level: 0.34,
            grit_level: 0.4,
            is_transport_running: true,
            tempo_bpm: 130.0,
            position_beats: 8.0,
        },
        ..RuntimeMixRenderPlan::default()
    };
    let mut stopped = running.clone();
    stopped.transport.is_transport_running = false;
    stopped.transport.position_beats = 8.25;
    stopped.w30_preview_render.is_transport_running = false;
    stopped.w30_preview_render.position_beats = 8.25;
    stopped.w30_resample_tap.is_transport_running = false;

    let segments = render_runtime_mix_plan_sequence_realtime_simulation_offline(
        &[
            RuntimeMixRenderSequenceStep::new(&running, 512),
            RuntimeMixRenderSequenceStep::new(&stopped, 1_024),
        ],
        44_100,
        2,
        128,
    );

    assert!(segments[0].iter().any(|sample| sample.abs() > 0.0001));
    let fade_sample_count = usize::try_from(44_100 / 200).unwrap() * 2;
    assert!(
        segments[1][..fade_sample_count]
            .iter()
            .any(|sample| sample.abs() > 0.0001)
    );
    assert!(
        segments[1][fade_sample_count..]
            .iter()
            .all(|sample| sample.abs() <= f32::EPSILON)
    );
}
