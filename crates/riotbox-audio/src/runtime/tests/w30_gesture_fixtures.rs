use crate::runtime::render_tr909_w30_preview::render_w30_preview_buffer;
use crate::runtime::shared_w30_resample_callback::W30PreviewCallbackState;
use crate::runtime::w30_preview_snapshot::SharedW30PreviewRenderState;
use crate::w30::{
    W30_PAD_CHOP_SLICE_COUNT, W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30PadPlaybackSampleWindow,
    W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState, W30PreviewSourceProfile,
};

pub(super) fn hook_turnaround_test_render(
    articulation: Option<crate::w30::W30HookArticulationRenderState>,
) -> W30PreviewRenderState {
    let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        let phase = index as f32 / W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as f32;
        let transient = if index % 1_024 < 48 { 0.48 } else { 0.0 };
        *sample = (phase * std::f32::consts::TAU * 5.0).sin() * 0.38
            + (phase * std::f32::consts::TAU * 11.0).sin() * 0.17
            + transient;
    }

    W30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        active_bank_id: Some("bank-a".into()),
        focused_pad_id: Some("pad-01".into()),
        capture_id: Some("cap-hook".into()),
        trigger_revision: 1,
        trigger_velocity: 0.82,
        source_window_preview: None,
        pad_playback: Some(W30PadPlaybackSampleWindow {
            source_start_frame: 0,
            source_end_frame: 96_000,
            source_sample_rate: 48_000,
            playback_frame_count: 96_000,
            sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
            loop_enabled: true,
            playback_rate: 1.0,
            reverse: false,
            gate_step_fraction: 0.0,
            loop_crossfade_sample_count: 128,
            chop_slice_count: W30_PAD_CHOP_SLICE_COUNT,
            chop_slice_starts: [0, 1_024, 2_048, 3_072, 4_096, 5_120, 6_144, 7_168],
            hook_articulation: articulation,
            samples,
        }),
        music_bus_level: 0.72,
        grit_level: 0.28,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 4.0,
    }
}

pub(super) fn render_hook_turnaround_in_chunks(
    render: &W30PreviewRenderState,
    chunk_frames: usize,
    total_frames: usize,
) -> Vec<f32> {
    let shared = SharedW30PreviewRenderState::new(render);
    let snapshot = shared.snapshot();
    let mut state = W30PreviewCallbackState::default();
    let mut output = vec![0.0; total_frames];

    for chunk in output.chunks_mut(chunk_frames) {
        render_w30_preview_buffer(chunk, 48_000, 1, &snapshot, &mut state);
    }

    output
}
