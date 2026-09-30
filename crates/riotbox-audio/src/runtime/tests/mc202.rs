use crate::mc202::{
    Mc202ContourHint, Mc202HookResponse, Mc202NoteBudget, Mc202PhraseShape, Mc202RenderMode,
    Mc202RenderRouting, Mc202RenderState,
};
use crate::runtime::shared_mc202::RealtimeMc202RenderState;
use crate::runtime::shared_transport_tr909::RealtimeTr909RenderState;
use crate::runtime::shared_w30_resample_callback::{
    RealtimeW30ResampleSourceWindow, RealtimeW30ResampleTapState, Tr909CallbackState,
    W30MixRenderState, W30PreviewCallbackState, W30ResampleTapCallbackState, render_mix_buffer,
};
use crate::runtime::source_monitor::SourceMonitorRenderState;
use crate::runtime::tests::mix_plan_fixtures::runtime_mix_parity_source_plan;
use crate::runtime::tests::synthetic_sources::mc202_source_plan;
use crate::runtime::w30_preview_snapshot::{
    RealtimeW30PadPlaybackSampleWindow, RealtimeW30PreviewRenderState,
    RealtimeW30PreviewSampleWindow,
};
use crate::runtime::{
    AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, render_mc202_offline,
    render_runtime_mix_realtime_simulation_offline, signal_metrics,
};
use crate::tr909::{Tr909RenderMode, Tr909RenderRouting};
use crate::w30::{
    W30PreviewRenderMode, W30PreviewRenderRouting, W30ResampleTapMode, W30ResampleTapRouting,
};

#[test]
fn realtime_transport_progress_reaches_late_mc202_source_steps() {
    let mut source_plan = runtime_mix_parity_source_plan();
    source_plan.active_mask = 1_u16 << 7;
    source_plan.accent_mask = 1_u16 << 7;
    source_plan.semitones[7] = -12;
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 130.0,
            position_beats: 0.0,
        },
        mc202_render: Mc202RenderState {
            mode: Mc202RenderMode::Pressure,
            routing: Mc202RenderRouting::MusicBusBass,
            source_phrase_plan: Some(source_plan),
            touch: 0.84,
            music_bus_level: 0.82,
            ..Mc202RenderState::default()
        },
        source_monitor_render: SourceMonitorRenderState::control_only(
            riotbox_core::action::SourceMonitorMode::Riotbox,
        ),
        ..RuntimeMixRenderPlan::default()
    };
    let frame_count = 48_000;

    let rendered =
        render_runtime_mix_realtime_simulation_offline(&plan, 48_000, 2, frame_count, 128);
    let metrics = signal_metrics(&rendered);

    assert!(metrics.active_samples > 1_000);
    assert!(metrics.rms > 0.005);
}

#[test]
fn render_mix_buffer_includes_live_mc202_bass_seam() {
    let mut tr909_state = Tr909CallbackState::default();
    let mut w30_preview_state = W30PreviewCallbackState::default();
    let mut w30_resample_state = W30ResampleTapCallbackState::default();
    let mut buffer = vec![0.0_f32; 44_100 * 2];

    render_mix_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::Idle,
            routing: Tr909RenderRouting::SourceOnly,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: None,
            phrase_variation: None,
            takeover_profile: None,
            drum_bus_level: 0.0,
            slam_enabled: false,
            slam_intensity: 0.0,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &RealtimeMc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            note_budget: Mc202NoteBudget::Balanced,
            contour_hint: Mc202ContourHint::Neutral,
            hook_response: Mc202HookResponse::Direct,
            source_phrase_plan: Some(mc202_source_plan()),
            touch: 0.78,
            music_bus_level: 0.64,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
        },
        &mut tr909_state,
        &mut W30MixRenderState {
            preview_render: &RealtimeW30PreviewRenderState {
                mode: W30PreviewRenderMode::Idle,
                routing: W30PreviewRenderRouting::Silent,
                source_profile: None,
                trigger_revision: 0,
                trigger_velocity: 0.0,
                source_window_preview: RealtimeW30PreviewSampleWindow::default(),
                pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
                music_bus_level: 0.0,
                grit_level: 0.0,
                is_transport_running: true,
                tempo_bpm: 128.0,
                position_beats: 32.0,
            },
            preview_state: &mut w30_preview_state,
            resample_render: &RealtimeW30ResampleTapState {
                mode: W30ResampleTapMode::Idle,
                routing: W30ResampleTapRouting::Silent,
                source_profile: None,
                source_audio: RealtimeW30ResampleSourceWindow::default(),
                lineage_capture_count: 0,
                generation_depth: 0,
                music_bus_level: 0.0,
                grit_level: 0.0,
                is_transport_running: true,
                tempo_bpm: 128.0,
                position_beats: 32.0,
            },
            resample_state: &mut w30_resample_state,
        },
    );

    let metrics = signal_metrics(&buffer);
    assert!(metrics.active_samples > 10_000);
    assert!(metrics.rms > 0.001);
}

#[test]
fn offline_mc202_render_stays_silent_until_source_phrase_exists() {
    let follower = render_mc202_offline(
        &Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            touch: 0.62,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        },
        44_100,
        2,
        44_100,
    );
    let answer = render_mc202_offline(
        &Mc202RenderState {
            mode: Mc202RenderMode::Answer,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::RootPulse,
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        },
        44_100,
        2,
        44_100,
    );
    let follower_metrics = signal_metrics(&follower);
    let answer_metrics = signal_metrics(&answer);

    assert_eq!(follower_metrics.active_samples, 0);
    assert_eq!(follower_metrics.rms, 0.0);
    assert_eq!(answer_metrics.active_samples, 0);
    assert_eq!(answer_metrics.rms, 0.0);
}

#[test]
fn offline_mc202_render_produces_distinct_source_backed_instigator_metrics() {
    let follower = render_mc202_offline(
        &Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            source_phrase_plan: Some(mc202_source_plan()),
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        },
        44_100,
        2,
        44_100,
    );
    let instigator = render_mc202_offline(
        &Mc202RenderState {
            mode: Mc202RenderMode::Instigator,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::InstigatorSpike,
            source_phrase_plan: Some(mc202_source_plan()),
            touch: 0.90,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        },
        44_100,
        2,
        44_100,
    );
    let follower_metrics = signal_metrics(&follower);
    let instigator_metrics = signal_metrics(&instigator);
    let delta_rms = (follower
        .iter()
        .zip(instigator.iter())
        .map(|(follower, instigator)| (follower - instigator).powi(2))
        .sum::<f32>()
        / follower.len() as f32)
        .sqrt();

    assert!(follower_metrics.active_samples > 10_000);
    assert!(instigator_metrics.active_samples > 8_000);
    assert!(
        delta_rms > 0.010,
        "instigator offline delta RMS {delta_rms}"
    );
}
