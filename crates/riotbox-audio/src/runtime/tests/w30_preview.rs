use crate::runtime::render_tr909_w30_preview::render_w30_preview_buffer;
use crate::runtime::shared_w30_resample_callback::W30PreviewCallbackState;
use crate::runtime::tests::signal_test_helpers::adjacent_sample_delta_rms;
use crate::runtime::tests::synthetic_sources::{
    fill_positive_preview_ramp, positive_realtime_source_window,
};
use crate::runtime::w30_preview_snapshot::{
    RealtimeW30PadPlaybackSampleWindow, RealtimeW30PreviewRenderState,
    RealtimeW30PreviewSampleWindow,
};
use crate::w30::{
    W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewRenderMode, W30PreviewRenderRouting,
    W30PreviewSourceProfile,
};

#[test]
fn w30_live_recall_uses_source_window_samples_when_available() {
    let mut positive_state = W30PreviewCallbackState::default();
    let mut negative_state = W30PreviewCallbackState::default();
    let mut positive = [0.0_f32; 512];
    let mut negative = [0.0_f32; 512];
    let mut positive_samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    let mut negative_samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    fill_positive_preview_ramp(&mut positive_samples);
    for index in 0..W30_PREVIEW_SAMPLE_WINDOW_LEN {
        negative_samples[index] = -positive_samples[index];
    }

    let base_render = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: RealtimeW30PreviewSampleWindow {
            source_start_frame: 0,
            source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
            sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
            samples: positive_samples,
        },
        pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 0.0,
    };
    let negative_render = RealtimeW30PreviewRenderState {
        source_window_preview: RealtimeW30PreviewSampleWindow {
            samples: negative_samples,
            ..base_render.source_window_preview
        },
        ..base_render
    };

    render_w30_preview_buffer(&mut positive, 44_100, 2, &base_render, &mut positive_state);
    render_w30_preview_buffer(
        &mut negative,
        44_100,
        2,
        &negative_render,
        &mut negative_state,
    );

    assert!(positive.iter().any(|sample| *sample > 0.001));
    assert!(negative.iter().any(|sample| *sample < -0.001));
    assert_ne!(positive, negative);
}

#[test]
fn w30_preview_respects_zero_music_bus_level() {
    let mut state = W30PreviewCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_preview_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: positive_realtime_source_window(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.0,
            grit_level: 0.6,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn promoted_w30_audition_is_more_present_than_pinned_recall() {
    let mut pinned_state = W30PreviewCallbackState::default();
    let mut audition_state = W30PreviewCallbackState::default();
    let mut pinned = [0.0_f32; 512];
    let mut audition = [0.0_f32; 512];

    render_w30_preview_buffer(
        &mut pinned,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PinnedRecall),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: positive_realtime_source_window(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.64,
            grit_level: 0.4,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
        },
        &mut pinned_state,
    );

    render_w30_preview_buffer(
        &mut audition,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::PromotedAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PromotedAudition),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: positive_realtime_source_window(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.64,
            grit_level: 0.68,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
        },
        &mut audition_state,
    );

    let pinned_peak = pinned
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let audition_peak = audition
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let pinned_energy = pinned.iter().map(|sample| sample.abs()).sum::<f32>();
    let audition_energy = audition.iter().map(|sample| sample.abs()).sum::<f32>();

    assert!(audition_peak > pinned_peak);
    assert!(audition_energy > pinned_energy);
}

#[test]
fn slice_pool_browse_preview_differs_from_promoted_recall() {
    let mut recall_state = W30PreviewCallbackState::default();
    let mut browse_state = W30PreviewCallbackState::default();
    let mut recall = [0.0_f32; 512];
    let mut browse = [0.0_f32; 512];

    render_w30_preview_buffer(
        &mut recall,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: positive_realtime_source_window(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.64,
            grit_level: 0.0,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 32.0,
        },
        &mut recall_state,
    );

    render_w30_preview_buffer(
        &mut browse,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::SlicePoolBrowse),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: positive_realtime_source_window(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.64,
            grit_level: 0.0,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 32.0,
        },
        &mut browse_state,
    );

    let recall_peak = recall
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let browse_peak = browse
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));

    assert!((browse_peak - recall_peak).abs() > 0.0005);
    assert_ne!(browse, recall);
}

#[test]
fn w30_trigger_revision_retriggers_preview_accent() {
    let mut state = W30PreviewCallbackState::default();
    let mut retriggered = [0.0_f32; 512];
    let render = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PinnedRecall),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: positive_realtime_source_window(),
        pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
        music_bus_level: 0.64,
        grit_level: 0.45,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 0.0,
    };

    let mut primed = [0.0_f32; 512];
    render_w30_preview_buffer(&mut primed, 44_100, 2, &render, &mut state);
    state.envelope = 0.0;
    state.was_active = true;
    state.last_trigger_revision = 0;

    let mut retrigger_render = render;
    retrigger_render.trigger_revision = 7;
    retrigger_render.trigger_velocity = 0.92;
    render_w30_preview_buffer(&mut retriggered, 44_100, 2, &retrigger_render, &mut state);

    assert!(retriggered.iter().any(|sample| sample.abs() > 0.0001));
    assert_eq!(state.last_trigger_revision, 7);
}

#[test]
fn w30_grit_adds_source_backed_bite_without_changing_the_clean_path() {
    let mut source = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: RealtimeW30PreviewSampleWindow::default(),
        pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
        music_bus_level: 0.8,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 130.0,
        position_beats: 0.0,
    };
    source.pad_playback.sample_count = 1_024;
    source.pad_playback.source_sample_rate = 44_100;
    source.pad_playback.playback_frame_count = 1_024;
    source.pad_playback.loop_enabled = true;
    for (index, sample) in source
        .pad_playback
        .samples
        .iter_mut()
        .take(source.pad_playback.sample_count)
        .enumerate()
    {
        let phase = index as f32 / 32.0;
        *sample = phase.sin() * 0.38 + (phase * 3.7).sin() * 0.08;
    }

    let mut clean = vec![0.0; 4_096];
    let mut clean_state = W30PreviewCallbackState::default();
    render_w30_preview_buffer(&mut clean, 44_100, 2, &source, &mut clean_state);

    source.grit_level = 0.64;
    let mut bitten = vec![0.0; 4_096];
    let mut bitten_state = W30PreviewCallbackState::default();
    render_w30_preview_buffer(&mut bitten, 44_100, 2, &source, &mut bitten_state);

    let delta_rms = clean
        .iter()
        .zip(&bitten)
        .map(|(clean, bitten)| (bitten - clean).powi(2))
        .sum::<f32>()
        / clean.len() as f32;
    let delta_rms = delta_rms.sqrt();
    let clean_edge_rms = adjacent_sample_delta_rms(&clean);
    let bitten_edge_rms = adjacent_sample_delta_rms(&bitten);
    assert!(delta_rms > 0.02, "bite delta was too small: {delta_rms}");
    assert!(
        bitten_edge_rms > clean_edge_rms * 1.25,
        "bite did not add enough source-motion edge: clean={clean_edge_rms}, bitten={bitten_edge_rms}"
    );
    assert!(bitten.iter().all(|sample| sample.is_finite()));
    assert!(bitten.iter().all(|sample| sample.abs() < 1.0));
}
