use crate::runtime::render_tr909_w30_preview::render_w30_resample_tap_buffer;
use crate::runtime::shared_w30_resample_callback::{
    RealtimeW30ResampleSourceWindow, RealtimeW30ResampleTapState, W30ResampleTapCallbackState,
};
use crate::runtime::tests::synthetic_sources::positive_realtime_resample_source;
use crate::w30::{W30ResampleTapMode, W30ResampleTapRouting, W30ResampleTapSourceProfile};

#[test]
fn w30_resample_tap_stays_silent_when_idle() {
    let mut state = W30ResampleTapCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_resample_tap_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            mode: W30ResampleTapMode::Idle,
            routing: W30ResampleTapRouting::Silent,
            source_profile: None,
            source_audio: RealtimeW30ResampleSourceWindow::default(),
            lineage_capture_count: 0,
            generation_depth: 0,
            music_bus_level: 0.64,
            grit_level: 0.4,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn w30_resample_tap_produces_audible_samples_when_lineage_is_ready() {
    let mut state = W30ResampleTapCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_resample_tap_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
            source_audio: positive_realtime_resample_source(),
            lineage_capture_count: 2,
            generation_depth: 1,
            music_bus_level: 0.58,
            grit_level: 0.62,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().any(|sample| sample.abs() > 0.0001));
}

#[test]
fn w30_resample_tap_is_deterministic_and_follows_source_material() {
    let source = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 2,
        generation_depth: 1,
        music_bus_level: 0.72,
        grit_level: 0.62,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 0.0,
    };
    let mut inverted = source;
    for sample in inverted
        .source_audio
        .samples
        .iter_mut()
        .take(inverted.source_audio.sample_count)
    {
        *sample = -*sample;
    }

    let render = |state: &RealtimeW30ResampleTapState| {
        let mut callback = W30ResampleTapCallbackState::default();
        let mut buffer = [0.0_f32; 1_024];
        render_w30_resample_tap_buffer(&mut buffer, 44_100, 2, state, &mut callback);
        buffer
    };
    let first = render(&source);
    let repeated = render(&source);
    let contrasting = render(&inverted);
    let mut raw_source = [0.0_f32; 1_024];
    for (frame, stereo) in raw_source.as_chunks_mut::<2>().0.iter_mut().enumerate() {
        stereo.fill(source.source_audio.samples[frame]);
    }
    let dry_delta_rms = (first
        .iter()
        .zip(raw_source.iter())
        .map(|(rendered, raw)| (rendered - raw).powi(2))
        .sum::<f32>()
        / first.len() as f32)
        .sqrt();

    assert_eq!(
        first, repeated,
        "same source and state must render deterministically"
    );
    assert_ne!(
        first, contrasting,
        "contrasting source PCM must change output"
    );
    assert!(
        dry_delta_rms > 0.01,
        "tap collapsed to raw source PCM: delta RMS {dry_delta_rms}"
    );
    let polarity_product = first
        .iter()
        .zip(contrasting.iter())
        .map(|(left, right)| left * right)
        .sum::<f32>();
    assert!(
        polarity_product < -0.001,
        "tap output did not follow inverted source polarity: {polarity_product}"
    );
}

#[test]
fn w30_resample_tap_stays_silent_without_source_audio() {
    let mut state = W30ResampleTapCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_resample_tap_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
            source_audio: RealtimeW30ResampleSourceWindow::default(),
            lineage_capture_count: 2,
            generation_depth: 1,
            music_bus_level: 0.58,
            grit_level: 0.62,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn w30_resample_tap_does_not_invent_grid_progress_without_a_valid_tempo() {
    let mut state = W30ResampleTapCallbackState::default();
    let mut buffer = [0.0_f32; 1_024];
    let render = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 2,
        generation_depth: 1,
        music_bus_level: 0.58,
        grit_level: 0.62,
        is_transport_running: true,
        tempo_bpm: 0.0,
        position_beats: 0.0,
    };

    render_w30_resample_tap_buffer(&mut buffer, 44_100, 2, &render, &mut state);

    assert_eq!(state.beat_position, 0.0);
    assert!(buffer.iter().any(|sample| sample.abs() > 0.0001));
}

#[test]
fn w30_resample_tap_respects_zero_music_bus_level() {
    let mut state = W30ResampleTapCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_resample_tap_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            source_profile: Some(W30ResampleTapSourceProfile::PinnedCapture),
            source_audio: positive_realtime_resample_source(),
            lineage_capture_count: 3,
            generation_depth: 2,
            music_bus_level: 0.0,
            grit_level: 0.7,
            is_transport_running: false,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}
