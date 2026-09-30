use crate::jam_app::transport_helpers::trusted_source_timing_bpm;
use riotbox_audio::source_audio::SourceAudioCache;
use riotbox_audio::w30::{
    W30_RESAMPLE_SOURCE_WINDOW_LEN, W30ResampleSourceWindow, W30ResampleTapAvailability,
    W30ResampleTapMode, W30ResampleTapRouting, W30ResampleTapSourceProfile, W30ResampleTapState,
};
use riotbox_core::ids::CaptureId;
use riotbox_core::session::SessionFile;
use riotbox_core::source_graph::SourceGraph;
use riotbox_core::transport::TransportClockState;
use std::collections::BTreeMap;

fn build_w30_capture_artifact_resample_source(
    capture: &riotbox_core::session::CaptureRef,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
) -> Option<Box<W30ResampleSourceWindow>> {
    let cache = capture_audio_cache?.get(&capture.capture_id)?;
    resample_source_from_interleaved(
        cache.interleaved_samples(),
        usize::from(cache.channel_count),
        cache.sample_rate,
    )
}

pub(in crate::jam_app) fn resample_source_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_sample_rate: u32,
) -> Option<Box<W30ResampleSourceWindow>> {
    if channel_count == 0
        || source_sample_rate == 0
        || samples.is_empty()
        || !samples.len().is_multiple_of(channel_count)
        || samples.iter().any(|sample| !sample.is_finite())
    {
        return None;
    }
    let frame_count = samples.len() / channel_count;

    let sample_count = frame_count.min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    let source_start_frame = select_resample_source_start(samples, channel_count, frame_count);
    let mut resample = [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN];
    for (index, slot) in resample.iter_mut().take(sample_count).enumerate() {
        let frame_index = source_start_frame + index;
        let base = frame_index * channel_count;
        *slot = samples[base..base + channel_count].iter().sum::<f32>() / channel_count as f32;
    }

    Some(Box::new(W30ResampleSourceWindow {
        source_start_frame: source_start_frame.try_into().unwrap_or(u64::MAX),
        source_sample_rate,
        source_frame_count: sample_count.try_into().unwrap_or(u64::MAX),
        sample_count,
        samples: resample,
    }))
}

fn select_resample_source_start(
    samples: &[f32],
    channel_count: usize,
    frame_count: usize,
) -> usize {
    const ANALYSIS_HOP_FRAMES: usize = 1_024;
    const TRANSIENT_WEIGHT: f32 = 2.5;

    let window_frames = frame_count.min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    if frame_count <= window_frames {
        return 0;
    }

    let last_start = frame_count - window_frames;
    let mut best_start = 0;
    let mut best_score = f32::NEG_INFINITY;
    let mut start = 0;
    loop {
        let mut energy = 0.0_f32;
        let mut motion = 0.0_f32;
        let mut previous = 0.0_f32;
        for frame_index in start..start + window_frames {
            let base = frame_index * channel_count;
            let mono =
                samples[base..base + channel_count].iter().sum::<f32>() / channel_count as f32;
            energy += mono.abs();
            motion += (mono - previous).abs();
            previous = mono;
        }
        let score = energy + motion * TRANSIENT_WEIGHT;
        if score > best_score {
            best_score = score;
            best_start = start;
        }
        if start == last_start {
            break;
        }
        start = (start + ANALYSIS_HOP_FRAMES).min(last_start);
    }
    best_start
}

pub(in crate::jam_app) fn build_w30_resample_tap_state(
    session: &SessionFile,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
) -> W30ResampleTapState {
    let w30 = &session.runtime_state.lane_state.w30;
    let Some(capture) = w30.last_capture.as_ref().and_then(|capture_id| {
        session
            .captures
            .iter()
            .find(|capture| capture.capture_id == *capture_id)
    }) else {
        return W30ResampleTapState::default();
    };
    if capture.capture_type != riotbox_core::session::CaptureType::Resample
        || capture.lineage_capture_refs.is_empty()
        || capture.resample_generation_depth == 0
    {
        return W30ResampleTapState::default();
    }

    let source_profile = if capture.is_pinned {
        Some(W30ResampleTapSourceProfile::PinnedCapture)
    } else if capture.assigned_target.is_some() {
        Some(W30ResampleTapSourceProfile::PromotedCapture)
    } else {
        Some(W30ResampleTapSourceProfile::RawCapture)
    };
    let source_audio = build_w30_capture_artifact_resample_source(capture, capture_audio_cache);
    let (availability, routing) = if source_audio.is_some() {
        (
            W30ResampleTapAvailability::SourceAudioReady,
            W30ResampleTapRouting::InternalCaptureTap,
        )
    } else {
        (
            W30ResampleTapAvailability::SourceAudioUnavailable,
            W30ResampleTapRouting::Silent,
        )
    };

    W30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing,
        availability,
        source_profile,
        source_capture_id: Some(capture.capture_id.to_string()),
        source_audio,
        lineage_capture_count: capture
            .lineage_capture_refs
            .len()
            .try_into()
            .unwrap_or(u8::MAX),
        generation_depth: capture.resample_generation_depth,
        music_bus_level: session
            .runtime_state
            .mixer_state
            .music_level
            .clamp(0.0, 1.0),
        grit_level: session.runtime_state.macro_state.w30_grit.clamp(0.0, 1.0),
        is_transport_running: transport.is_playing,
        tempo_bpm: trusted_source_timing_bpm(session, source_graph).unwrap_or(0.0),
        position_beats: transport.position_beats,
    }
}
