use crate::runtime::shared_transport_tr909::RealtimeTr909RenderState;
use crate::runtime::shared_w30_resample_callback::{
    RealtimeW30ResampleSourceWindow, RealtimeW30ResampleTapState,
};
use crate::runtime::tests::synthetic_sources::{
    fill_positive_preview_ramp, positive_realtime_resample_source,
};
use crate::runtime::w30_preview_snapshot::{
    RealtimeW30PadPlaybackSampleWindow, RealtimeW30PreviewRenderState,
    RealtimeW30PreviewSampleWindow,
};
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909SourceSupportContext, Tr909SourceSupportProfile, Tr909TakeoverRenderProfile,
};
use crate::w30::{
    W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewRenderMode, W30PreviewRenderRouting,
    W30PreviewSourceProfile, W30ResampleTapMode, W30ResampleTapRouting,
    W30ResampleTapSourceProfile,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct AudioFixtureCase {
    pub(super) name: String,
    pub(super) render_state: AudioFixtureRenderState,
    pub(super) expected: AudioFixtureExpectation,
}

#[derive(Debug, Deserialize)]
pub(super) struct AudioFixtureRenderState {
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
pub(super) struct AudioFixtureExpectation {
    pub(super) min_active_samples: usize,
    pub(super) max_active_samples: usize,
    pub(super) min_peak_abs: f32,
    pub(super) max_peak_abs: f32,
    pub(super) min_sum: Option<f32>,
    pub(super) max_sum: Option<f32>,
    pub(super) min_rms: Option<f32>,
    pub(super) max_rms: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub(super) struct W30AudioFixtureCase {
    pub(super) name: String,
    pub(super) render_state: W30AudioFixtureRenderState,
    pub(super) expected: AudioFixtureExpectation,
}

#[derive(Debug, Deserialize)]
pub(super) struct W30ResampleAudioFixtureCase {
    pub(super) name: String,
    pub(super) render_state: W30ResampleAudioFixtureRenderState,
    pub(super) expected: AudioFixtureExpectation,
}

#[derive(Debug, Deserialize)]
pub(super) struct W30AudioFixtureRenderState {
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
pub(super) struct W30ResampleAudioFixtureRenderState {
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
    pub(super) fn to_realtime(&self) -> RealtimeTr909RenderState {
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
            slam_enabled: false,
            slam_intensity: self.slam_intensity,
            is_transport_running: self.is_transport_running,
            tempo_bpm: self.tempo_bpm,
            position_beats: self.position_beats,
            source_bar_grid_anchor_position_beats: None,
        }
    }
}

impl W30AudioFixtureRenderState {
    pub(super) fn to_realtime(&self) -> RealtimeW30PreviewRenderState {
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
    pub(super) fn to_realtime(&self) -> RealtimeW30ResampleTapState {
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
            source_audio: if self.routing == "internal_capture_tap" {
                positive_realtime_resample_source()
            } else {
                RealtimeW30ResampleSourceWindow::default()
            },
            lineage_capture_count: self.lineage_capture_count,
            generation_depth: self.generation_depth,
            music_bus_level: self.music_bus_level,
            grit_level: self.grit_level,
            is_transport_running: self.is_transport_running,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        }
    }
}
