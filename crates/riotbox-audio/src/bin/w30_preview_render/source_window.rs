use riotbox_audio::w30::{
    W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewRenderMode, W30PreviewRenderRouting,
    W30PreviewRenderState, W30PreviewSampleWindow, W30PreviewSourceProfile,
};

pub(super) fn source_window_smoke_state(
    source_window_preview: W30PreviewSampleWindow,
) -> W30PreviewRenderState {
    W30PreviewRenderState {
        mode: W30PreviewRenderMode::RawCaptureAudition,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
        active_bank_id: Some("bank-a".into()),
        focused_pad_id: Some("pad-01".into()),
        capture_id: Some("cap-01".into()),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: Some(source_window_preview),
        pad_playback: None,
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 32.0,
    }
}

pub(super) fn synthetic_source_window_preview() -> W30PreviewSampleWindow {
    let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    let denominator = W30_PREVIEW_SAMPLE_WINDOW_LEN.saturating_sub(1).max(1) as f32;
    for (index, sample) in samples.iter_mut().enumerate() {
        let progress = index as f32 / denominator;
        *sample = 0.18 + progress * 0.12;
    }

    W30PreviewSampleWindow {
        source_start_frame: 0,
        source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
        sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
        samples,
    }
}

pub(super) fn source_preview_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_start_frame: u64,
    source_end_frame: u64,
) -> Option<W30PreviewSampleWindow> {
    let channel_count = channel_count.max(1);
    let frame_count = samples.len() / channel_count;
    if frame_count == 0 {
        return None;
    }

    let sample_count = frame_count.min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    let stride = (frame_count / sample_count).max(1);
    let mut preview = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];

    for (index, slot) in preview.iter_mut().take(sample_count).enumerate() {
        let frame_index = (index * stride).min(frame_count - 1);
        let base = frame_index * channel_count;
        let sum: f32 = samples[base..base + channel_count].iter().sum();
        *slot = sum / channel_count as f32;
    }

    Some(W30PreviewSampleWindow {
        source_start_frame,
        source_end_frame,
        sample_count,
        samples: preview,
    })
}
