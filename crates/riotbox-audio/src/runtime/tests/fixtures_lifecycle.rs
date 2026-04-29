use super::*;
use crate::mc202::{Mc202PhraseShape, Mc202RenderMode, Mc202RenderRouting, Mc202RenderState};
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909RenderState, Tr909SourceSupportContext, Tr909SourceSupportProfile,
    Tr909TakeoverRenderProfile,
};
use crate::w30::{
    W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30PreviewRenderMode, W30PreviewRenderRouting,
    W30PreviewRenderState, W30PreviewSampleWindow, W30PreviewSourceProfile, W30ResampleTapMode,
    W30ResampleTapRouting, W30ResampleTapSourceProfile, W30ResampleTapState,
};
use serde::Deserialize;

fn fill_positive_preview_ramp(samples: &mut [f32; W30_PREVIEW_SAMPLE_WINDOW_LEN]) {
    let denominator = W30_PREVIEW_SAMPLE_WINDOW_LEN.saturating_sub(1).max(1) as f32;
    for (index, sample) in samples.iter_mut().enumerate() {
        let progress = index as f32 / denominator;
        *sample = 0.18 + progress * 0.12;
    }
}

#[derive(Debug, Deserialize)]
struct AudioFixtureCase {
    name: String,
    render_state: AudioFixtureRenderState,
    expected: AudioFixtureExpectation,
}

#[derive(Debug, Deserialize)]
struct AudioFixtureRenderState {
    mode: String,
    routing: String,
    source_support_profile: Option<String>,
    source_support_context: Option<String>,
    pattern_adoption: Option<String>,
    phrase_variation: Option<String>,
    takeover_profile: Option<String>,
    drum_bus_level: f32,
    slam_intensity: f32,
    is_transport_running: bool,
    tempo_bpm: f32,
    position_beats: f64,
}

#[derive(Debug, Deserialize)]
struct AudioFixtureExpectation {
    min_active_samples: usize,
    max_active_samples: usize,
    min_peak_abs: f32,
    max_peak_abs: f32,
    min_sum: Option<f32>,
    max_sum: Option<f32>,
    min_rms: Option<f32>,
    max_rms: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct W30AudioFixtureCase {
    name: String,
    render_state: W30AudioFixtureRenderState,
    expected: AudioFixtureExpectation,
}

#[derive(Debug, Deserialize)]
struct W30ResampleAudioFixtureCase {
    name: String,
    render_state: W30ResampleAudioFixtureRenderState,
    expected: AudioFixtureExpectation,
}

#[derive(Debug, Deserialize)]
struct W30AudioFixtureRenderState {
    mode: String,
    routing: String,
    source_profile: Option<String>,
    trigger_revision: u64,
    trigger_velocity: f32,
    source_window_preview: Option<W30AudioFixtureSourceWindow>,
    music_bus_level: f32,
    grit_level: f32,
    is_transport_running: bool,
    tempo_bpm: f32,
    position_beats: f64,
}

#[derive(Debug, Deserialize)]
struct W30AudioFixtureSourceWindow {
    source_start_frame: u64,
    source_end_frame: u64,
    sample_count: usize,
    sample_pattern: String,
}

#[derive(Debug, Deserialize)]
struct W30ResampleAudioFixtureRenderState {
    mode: String,
    routing: String,
    source_profile: Option<String>,
    lineage_capture_count: u8,
    generation_depth: u8,
    music_bus_level: f32,
    grit_level: f32,
    is_transport_running: bool,
}

impl AudioFixtureRenderState {
    fn to_realtime(&self) -> RealtimeTr909RenderState {
        RealtimeTr909RenderState {
            mode: match self.mode.as_str() {
                "source_support" => Tr909RenderMode::SourceSupport,
                "fill" => Tr909RenderMode::Fill,
                "break_reinforce" => Tr909RenderMode::BreakReinforce,
                "takeover" => Tr909RenderMode::Takeover,
                "idle" => Tr909RenderMode::Idle,
                other => panic!("unknown TR-909 fixture mode: {other}"),
            },
            routing: match self.routing.as_str() {
                "drum_bus_support" => Tr909RenderRouting::DrumBusSupport,
                "drum_bus_takeover" => Tr909RenderRouting::DrumBusTakeover,
                "source_only" => Tr909RenderRouting::SourceOnly,
                other => panic!("unknown TR-909 fixture routing: {other}"),
            },
            source_support_profile: self.source_support_profile.as_deref().map(|profile| {
                match profile {
                    "break_lift" => Tr909SourceSupportProfile::BreakLift,
                    "drop_drive" => Tr909SourceSupportProfile::DropDrive,
                    "steady_pulse" => Tr909SourceSupportProfile::SteadyPulse,
                    other => panic!("unknown TR-909 fixture source support profile: {other}"),
                }
            }),
            source_support_context: self.source_support_context.as_deref().map(|context| {
                match context {
                    "scene_target" => Tr909SourceSupportContext::SceneTarget,
                    "transport_bar" => Tr909SourceSupportContext::TransportBar,
                    other => panic!("unknown TR-909 fixture source support context: {other}"),
                }
            }),
            pattern_adoption: self
                .pattern_adoption
                .as_deref()
                .map(|pattern| match pattern {
                    "mainline_drive" => Tr909PatternAdoption::MainlineDrive,
                    "takeover_grid" => Tr909PatternAdoption::TakeoverGrid,
                    "support_pulse" => Tr909PatternAdoption::SupportPulse,
                    other => panic!("unknown TR-909 fixture pattern adoption: {other}"),
                }),
            phrase_variation: self
                .phrase_variation
                .as_deref()
                .map(|variation| match variation {
                    "phrase_lift" => Tr909PhraseVariation::PhraseLift,
                    "phrase_drive" => Tr909PhraseVariation::PhraseDrive,
                    "phrase_release" => Tr909PhraseVariation::PhraseRelease,
                    "phrase_anchor" => Tr909PhraseVariation::PhraseAnchor,
                    other => panic!("unknown TR-909 fixture phrase variation: {other}"),
                }),
            takeover_profile: self
                .takeover_profile
                .as_deref()
                .map(|profile| match profile {
                    "scene_lock" => Tr909TakeoverRenderProfile::SceneLock,
                    "controlled_phrase" => Tr909TakeoverRenderProfile::ControlledPhrase,
                    other => panic!("unknown TR-909 fixture takeover profile: {other}"),
                }),
            drum_bus_level: self.drum_bus_level,
            slam_intensity: self.slam_intensity,
            is_transport_running: self.is_transport_running,
            tempo_bpm: self.tempo_bpm,
            position_beats: self.position_beats,
        }
    }
}

impl W30AudioFixtureRenderState {
    fn to_realtime(&self) -> RealtimeW30PreviewRenderState {
        RealtimeW30PreviewRenderState {
            mode: match self.mode.as_str() {
                "live_recall" => W30PreviewRenderMode::LiveRecall,
                "raw_capture_audition" => W30PreviewRenderMode::RawCaptureAudition,
                "promoted_audition" => W30PreviewRenderMode::PromotedAudition,
                _ => W30PreviewRenderMode::Idle,
            },
            routing: match self.routing.as_str() {
                "music_bus_preview" => W30PreviewRenderRouting::MusicBusPreview,
                _ => W30PreviewRenderRouting::Silent,
            },
            source_profile: self.source_profile.as_deref().map(|profile| match profile {
                "pinned_recall" => W30PreviewSourceProfile::PinnedRecall,
                "slice_pool_browse" => W30PreviewSourceProfile::SlicePoolBrowse,
                "raw_capture_audition" => W30PreviewSourceProfile::RawCaptureAudition,
                "promoted_audition" => W30PreviewSourceProfile::PromotedAudition,
                _ => W30PreviewSourceProfile::PromotedRecall,
            }),
            trigger_revision: self.trigger_revision,
            trigger_velocity: self.trigger_velocity,
            source_window_preview: self
                .source_window_preview
                .as_ref()
                .map_or_else(RealtimeW30PreviewSampleWindow::default, |source| {
                    source.to_realtime()
                }),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: self.music_bus_level,
            grit_level: self.grit_level,
            is_transport_running: self.is_transport_running,
            tempo_bpm: self.tempo_bpm,
            position_beats: self.position_beats,
        }
    }
}

impl W30AudioFixtureSourceWindow {
    fn to_realtime(&self) -> RealtimeW30PreviewSampleWindow {
        let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
        match self.sample_pattern.as_str() {
            "positive_ramp" => fill_positive_preview_ramp(&mut samples),
            other => panic!("unknown W-30 source-window sample pattern: {other}"),
        }

        RealtimeW30PreviewSampleWindow {
            source_start_frame: self.source_start_frame,
            source_end_frame: self.source_end_frame,
            sample_count: self.sample_count.min(W30_PREVIEW_SAMPLE_WINDOW_LEN),
            samples,
        }
    }
}

impl W30ResampleAudioFixtureRenderState {
    fn to_realtime(&self) -> RealtimeW30ResampleTapState {
        RealtimeW30ResampleTapState {
            mode: match self.mode.as_str() {
                "capture_lineage_ready" => W30ResampleTapMode::CaptureLineageReady,
                _ => W30ResampleTapMode::Idle,
            },
            routing: match self.routing.as_str() {
                "internal_capture_tap" => W30ResampleTapRouting::InternalCaptureTap,
                _ => W30ResampleTapRouting::Silent,
            },
            source_profile: self.source_profile.as_deref().map(|profile| match profile {
                "promoted_capture" => W30ResampleTapSourceProfile::PromotedCapture,
                "pinned_capture" => W30ResampleTapSourceProfile::PinnedCapture,
                _ => W30ResampleTapSourceProfile::RawCapture,
            }),
            lineage_capture_count: self.lineage_capture_count,
            generation_depth: self.generation_depth,
            music_bus_level: self.music_bus_level,
            grit_level: self.grit_level,
            is_transport_running: self.is_transport_running,
        }
    }
}

fn sample_output() -> AudioOutputInfo {
    AudioOutputInfo {
        host_name: "Alsa".into(),
        device_name: "default".into(),
        sample_format: "F32".into(),
        sample_rate: 44_100,
        channel_count: 2,
        buffer_size: "Default".into(),
        supported_output_config_count: Some(12),
    }
}

fn sample_timing(position_beats: f64) -> CallbackTimingSnapshot {
    CallbackTimingSnapshot {
        is_transport_running: true,
        tempo_bpm: 128.0,
        render_position_beats: position_beats,
        completed_position_beats: position_beats,
    }
}

#[test]
fn telemetry_tracks_callback_count_and_max_gap() {
    let telemetry = RuntimeTelemetry::new();
    telemetry.record_callback_at(100, &sample_timing(16.0));
    telemetry.record_callback_at(350, &sample_timing(16.5));
    telemetry.record_callback_at(775, &sample_timing(17.0));

    let snapshot = telemetry.snapshot();

    assert_eq!(snapshot.callback_count, 3);
    assert_eq!(snapshot.max_callback_gap_micros, Some(425));
    assert!(snapshot.timing.is_transport_running);
    assert_eq!(snapshot.timing.position_beats, 17.0);
}

#[test]
fn health_snapshot_reflects_faulted_runtime_state() {
    let telemetry = Arc::new(RuntimeTelemetry::new());
    let tr909_render_state = Arc::new(SharedTr909RenderState::new(&Tr909RenderState::default()));
    let mc202_render_state = Arc::new(SharedMc202RenderState::new(&Mc202RenderState::default()));
    let w30_preview_state = Arc::new(SharedW30PreviewRenderState::new(
        &W30PreviewRenderState::default(),
    ));
    let w30_resample_tap_state = Arc::new(SharedW30ResampleTapState::new(
        &W30ResampleTapState::default(),
    ));
    let transport = Arc::new(SharedTransportTimingState::new(false, 128.0, 0.0));
    telemetry.record_callback_at(100, &sample_timing(12.0));
    telemetry.record_callback_at(240, &sample_timing(12.25));
    telemetry.record_stream_error("stream stalled".into());

    let shell = AudioRuntimeShell::from_test_parts(AudioRuntimeShellTestParts {
        lifecycle: AudioRuntimeLifecycle::Running,
        output: Some(sample_output()),
        telemetry,
        transport,
        tr909_render: tr909_render_state,
        mc202_render: mc202_render_state,
        w30_preview: w30_preview_state,
        w30_resample_tap: w30_resample_tap_state,
    });

    let snapshot = shell.health_snapshot();

    assert_eq!(snapshot.lifecycle, AudioRuntimeLifecycle::Faulted);
    assert_eq!(snapshot.callback_count, 2);
    assert_eq!(snapshot.max_callback_gap_micros, Some(140));
    assert_eq!(snapshot.stream_error_count, 1);
    assert_eq!(
        snapshot.last_stream_error.as_deref(),
        Some("stream stalled")
    );
}

#[test]
fn stop_transitions_runtime_to_stopped() {
    let telemetry = Arc::new(RuntimeTelemetry::new());
    let tr909_render_state = Arc::new(SharedTr909RenderState::new(&Tr909RenderState::default()));
    let mc202_render_state = Arc::new(SharedMc202RenderState::new(&Mc202RenderState::default()));
    let w30_preview_state = Arc::new(SharedW30PreviewRenderState::new(
        &W30PreviewRenderState::default(),
    ));
    let w30_resample_tap_state = Arc::new(SharedW30ResampleTapState::new(
        &W30ResampleTapState::default(),
    ));
    let transport = Arc::new(SharedTransportTimingState::new(false, 128.0, 0.0));
    let mut shell = AudioRuntimeShell::from_test_parts(AudioRuntimeShellTestParts {
        lifecycle: AudioRuntimeLifecycle::Running,
        output: Some(sample_output()),
        telemetry,
        transport,
        tr909_render: tr909_render_state,
        mc202_render: mc202_render_state,
        w30_preview: w30_preview_state,
        w30_resample_tap: w30_resample_tap_state,
    });

    shell.stop();

    assert_eq!(shell.lifecycle(), AudioRuntimeLifecycle::Stopped);
}

#[test]
fn timing_snapshot_reflects_callback_owned_transport_progress() {
    let telemetry = Arc::new(RuntimeTelemetry::new());
    let transport = Arc::new(SharedTransportTimingState::new(true, 126.0, 32.0));
    let shell = AudioRuntimeShell::from_test_parts(AudioRuntimeShellTestParts {
        lifecycle: AudioRuntimeLifecycle::Running,
        output: Some(sample_output()),
        telemetry: Arc::clone(&telemetry),
        transport: Arc::clone(&transport),
        tr909_render: Arc::new(SharedTr909RenderState::new(&Tr909RenderState::default())),
        mc202_render: Arc::new(SharedMc202RenderState::new(&Mc202RenderState::default())),
        w30_preview: Arc::new(SharedW30PreviewRenderState::new(
            &W30PreviewRenderState::default(),
        )),
        w30_resample_tap: Arc::new(SharedW30ResampleTapState::new(
            &W30ResampleTapState::default(),
        )),
    });

    telemetry.record_callback_at(
        100,
        &CallbackTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 126.0,
            render_position_beats: 32.0,
            completed_position_beats: 32.5,
        },
    );

    let snapshot = shell.timing_snapshot();
    assert!(snapshot.is_transport_running);
    assert_eq!(snapshot.tempo_bpm, 126.0);
    assert_eq!(snapshot.position_beats, 32.5);
}

