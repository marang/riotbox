use super::*;

impl SharedW30PreviewRenderState {
    pub(super) fn new(render_state: &W30PreviewRenderState) -> Self {
        let shared = Self {
            revision: AtomicU64::new(0),
            mode: AtomicU32::new(0),
            routing: AtomicU32::new(0),
            source_profile: AtomicU32::new(0),
            trigger_revision: AtomicU64::new(0),
            trigger_velocity_bits: AtomicU32::new(0),
            source_start_frame: AtomicU64::new(0),
            source_end_frame: AtomicU64::new(0),
            source_sample_count: AtomicU32::new(0),
            source_samples: std::array::from_fn(|_| AtomicU32::new(0.0_f32.to_bits())),
            pad_start_frame: AtomicU64::new(0),
            pad_end_frame: AtomicU64::new(0),
            pad_source_sample_rate: AtomicU32::new(0),
            pad_playback_frame_count: AtomicU64::new(0),
            pad_sample_count: AtomicU32::new(0),
            pad_loop_enabled: AtomicBool::new(false),
            pad_playback_rate_bits: AtomicU32::new(1.0_f32.to_bits()),
            pad_reverse: AtomicBool::new(false),
            pad_gate_step_fraction_bits: AtomicU32::new(0.0_f32.to_bits()),
            pad_loop_crossfade_sample_count: AtomicU32::new(0),
            pad_chop_slice_count: AtomicU32::new(0),
            pad_chop_slice_starts: std::array::from_fn(|_| AtomicU32::new(0)),
            pad_samples: std::array::from_fn(|_| AtomicU32::new(0.0_f32.to_bits())),
            music_bus_level_bits: AtomicU32::new(0),
            grit_level_bits: AtomicU32::new(0),
            is_transport_running: AtomicBool::new(false),
            tempo_bpm_bits: AtomicU32::new(0),
            position_beats_bits: AtomicU64::new(0),
        };
        shared.update(render_state);
        shared
    }

    pub(super) fn update(&self, render_state: &W30PreviewRenderState) {
        begin_coherent_snapshot_update(&self.revision);
        self.mode
            .store(w30_mode_to_u32(render_state.mode), Ordering::Relaxed);
        self.routing
            .store(w30_routing_to_u32(render_state.routing), Ordering::Relaxed);
        self.source_profile.store(
            w30_source_profile_to_u32(render_state.source_profile),
            Ordering::Relaxed,
        );
        self.trigger_revision
            .store(render_state.trigger_revision, Ordering::Relaxed);
        self.trigger_velocity_bits
            .store(render_state.trigger_velocity.to_bits(), Ordering::Relaxed);
        self.update_source_window_preview(render_state.source_window_preview.as_ref());
        self.update_pad_playback(render_state.pad_playback.as_ref());
        self.music_bus_level_bits
            .store(render_state.music_bus_level.to_bits(), Ordering::Relaxed);
        self.grit_level_bits
            .store(render_state.grit_level.to_bits(), Ordering::Relaxed);
        self.is_transport_running
            .store(render_state.is_transport_running, Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(render_state.tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(render_state.position_beats.to_bits(), Ordering::Relaxed);
        finish_coherent_snapshot_update(&self.revision);
    }

    pub(super) fn snapshot(&self) -> RealtimeW30PreviewRenderState {
        coherent_snapshot(&self.revision, || self.read_snapshot_fields())
    }

    pub(super) fn snapshot_or_previous(
        &self,
        previous: &RealtimeW30PreviewRenderState,
    ) -> RealtimeW30PreviewRenderState {
        coherent_snapshot_or(&self.revision, previous, || self.read_snapshot_fields())
    }

    fn read_snapshot_fields(&self) -> RealtimeW30PreviewRenderState {
        RealtimeW30PreviewRenderState {
            mode: w30_mode_from_u32(self.mode.load(Ordering::Relaxed)),
            routing: w30_routing_from_u32(self.routing.load(Ordering::Relaxed)),
            source_profile: w30_source_profile_from_u32(
                self.source_profile.load(Ordering::Relaxed),
            ),
            trigger_revision: self.trigger_revision.load(Ordering::Relaxed),
            trigger_velocity: f32::from_bits(self.trigger_velocity_bits.load(Ordering::Relaxed)),
            source_window_preview: self.source_window_preview_snapshot(),
            pad_playback: self.pad_playback_snapshot(),
            music_bus_level: f32::from_bits(self.music_bus_level_bits.load(Ordering::Relaxed)),
            grit_level: f32::from_bits(self.grit_level_bits.load(Ordering::Relaxed)),
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
            tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
            position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
        }
    }

    fn update_source_window_preview(&self, source_window: Option<&W30PreviewSampleWindow>) {
        if let Some(source_window) = source_window {
            let sample_count = source_window
                .sample_count
                .min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
            self.source_start_frame
                .store(source_window.source_start_frame, Ordering::Relaxed);
            self.source_end_frame
                .store(source_window.source_end_frame, Ordering::Relaxed);
            for (index, sample) in source_window.samples.iter().copied().enumerate() {
                self.source_samples[index].store(sample.to_bits(), Ordering::Relaxed);
            }
            self.source_sample_count
                .store(sample_count as u32, Ordering::Relaxed);
        } else {
            self.source_sample_count.store(0, Ordering::Relaxed);
            self.source_start_frame.store(0, Ordering::Relaxed);
            self.source_end_frame.store(0, Ordering::Relaxed);
        }
    }

    fn source_window_preview_snapshot(&self) -> RealtimeW30PreviewSampleWindow {
        let sample_count = (self.source_sample_count.load(Ordering::Relaxed) as usize)
            .min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
        let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
        for (index, sample) in samples.iter_mut().enumerate() {
            *sample = f32::from_bits(self.source_samples[index].load(Ordering::Relaxed));
        }

        RealtimeW30PreviewSampleWindow {
            source_start_frame: self.source_start_frame.load(Ordering::Relaxed),
            source_end_frame: self.source_end_frame.load(Ordering::Relaxed),
            sample_count,
            samples,
        }
    }

    fn update_pad_playback(&self, pad_playback: Option<&W30PadPlaybackSampleWindow>) {
        if let Some(pad_playback) = pad_playback {
            let sample_count = pad_playback
                .sample_count
                .min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
            self.pad_start_frame
                .store(pad_playback.source_start_frame, Ordering::Relaxed);
            self.pad_end_frame
                .store(pad_playback.source_end_frame, Ordering::Relaxed);
            self.pad_source_sample_rate
                .store(pad_playback.source_sample_rate, Ordering::Relaxed);
            self.pad_playback_frame_count
                .store(pad_playback.playback_frame_count, Ordering::Relaxed);
            self.pad_loop_enabled
                .store(pad_playback.loop_enabled, Ordering::Relaxed);
            self.pad_playback_rate_bits
                .store(pad_playback.playback_rate.to_bits(), Ordering::Relaxed);
            self.pad_reverse
                .store(pad_playback.reverse, Ordering::Relaxed);
            self.pad_gate_step_fraction_bits
                .store(pad_playback.gate_step_fraction.to_bits(), Ordering::Relaxed);
            self.pad_loop_crossfade_sample_count.store(
                pad_playback.loop_crossfade_sample_count.min(sample_count) as u32,
                Ordering::Relaxed,
            );
            let chop_slice_count = pad_playback.chop_slice_count.min(W30_PAD_CHOP_SLICE_COUNT);
            for (index, start) in pad_playback.chop_slice_starts.iter().copied().enumerate() {
                self.pad_chop_slice_starts[index].store(start, Ordering::Relaxed);
            }
            self.pad_chop_slice_count
                .store(chop_slice_count as u32, Ordering::Relaxed);
            for (index, sample) in pad_playback.samples.iter().copied().enumerate() {
                self.pad_samples[index].store(sample.to_bits(), Ordering::Relaxed);
            }
            self.pad_sample_count
                .store(sample_count as u32, Ordering::Relaxed);
        } else {
            self.pad_sample_count.store(0, Ordering::Relaxed);
            self.pad_start_frame.store(0, Ordering::Relaxed);
            self.pad_end_frame.store(0, Ordering::Relaxed);
            self.pad_source_sample_rate.store(0, Ordering::Relaxed);
            self.pad_playback_frame_count.store(0, Ordering::Relaxed);
            self.pad_loop_enabled.store(false, Ordering::Relaxed);
            self.pad_playback_rate_bits
                .store(1.0_f32.to_bits(), Ordering::Relaxed);
            self.pad_reverse.store(false, Ordering::Relaxed);
            self.pad_gate_step_fraction_bits
                .store(0.0_f32.to_bits(), Ordering::Relaxed);
            self.pad_loop_crossfade_sample_count
                .store(0, Ordering::Relaxed);
            self.pad_chop_slice_count.store(0, Ordering::Relaxed);
        }
    }

    fn pad_playback_snapshot(&self) -> RealtimeW30PadPlaybackSampleWindow {
        let sample_count = (self.pad_sample_count.load(Ordering::Relaxed) as usize)
            .min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
        let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
        for (index, sample) in samples.iter_mut().enumerate() {
            *sample = f32::from_bits(self.pad_samples[index].load(Ordering::Relaxed));
        }
        let chop_slice_count = (self.pad_chop_slice_count.load(Ordering::Relaxed) as usize)
            .min(W30_PAD_CHOP_SLICE_COUNT);
        let chop_slice_starts = std::array::from_fn(|index| {
            self.pad_chop_slice_starts[index]
                .load(Ordering::Relaxed)
                .min(sample_count.saturating_sub(1) as u32)
        });

        RealtimeW30PadPlaybackSampleWindow {
            source_start_frame: self.pad_start_frame.load(Ordering::Relaxed),
            source_end_frame: self.pad_end_frame.load(Ordering::Relaxed),
            source_sample_rate: self.pad_source_sample_rate.load(Ordering::Relaxed),
            playback_frame_count: self.pad_playback_frame_count.load(Ordering::Relaxed),
            sample_count,
            loop_enabled: self.pad_loop_enabled.load(Ordering::Relaxed),
            playback_rate: f32::from_bits(self.pad_playback_rate_bits.load(Ordering::Relaxed)),
            reverse: self.pad_reverse.load(Ordering::Relaxed),
            gate_step_fraction: f32::from_bits(
                self.pad_gate_step_fraction_bits.load(Ordering::Relaxed),
            ),
            loop_crossfade_sample_count: (self
                .pad_loop_crossfade_sample_count
                .load(Ordering::Relaxed) as usize)
                .min(sample_count),
            chop_slice_count,
            chop_slice_starts,
            samples,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct RealtimeW30ResampleTapState {
    pub(super) mode: W30ResampleTapMode,
    pub(super) routing: W30ResampleTapRouting,
    pub(super) source_profile: Option<W30ResampleTapSourceProfile>,
    pub(super) source_audio: RealtimeW30ResampleSourceWindow,
    pub(super) lineage_capture_count: u8,
    pub(super) generation_depth: u8,
    pub(super) variation: W30ResampleTapVariation,
    pub(super) variation_revision: u64,
    pub(super) variation_intensity: f32,
    pub(super) music_bus_level: f32,
    pub(super) grit_level: f32,
    pub(super) is_transport_running: bool,
    pub(super) tempo_bpm: f32,
    pub(super) position_beats: f64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct RealtimeW30ResampleSourceWindow {
    pub(super) source_start_frame: u64,
    pub(super) source_sample_rate: u32,
    pub(super) source_frame_count: u64,
    pub(super) sample_count: usize,
    pub(super) samples: [f32; W30_RESAMPLE_SOURCE_WINDOW_LEN],
    pub(super) attack_start_frame: u64,
    pub(super) attack_sample_count: usize,
    pub(super) attack_samples: [f32; W30_RESAMPLE_ATTACK_WINDOW_LEN],
}

impl Default for RealtimeW30ResampleSourceWindow {
    fn default() -> Self {
        Self {
            source_start_frame: 0,
            source_sample_rate: 0,
            source_frame_count: 0,
            sample_count: 0,
            samples: [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN],
            attack_start_frame: 0,
            attack_sample_count: 0,
            attack_samples: [0.0; W30_RESAMPLE_ATTACK_WINDOW_LEN],
        }
    }
}

pub(super) struct SharedW30ResampleTapState {
    revision: AtomicU64,
    mode: AtomicU32,
    routing: AtomicU32,
    source_profile: AtomicU32,
    source_start_frame: AtomicU64,
    source_sample_rate: AtomicU32,
    source_frame_count: AtomicU64,
    source_sample_count: AtomicU32,
    source_samples: [AtomicU32; W30_RESAMPLE_SOURCE_WINDOW_LEN],
    attack_start_frame: AtomicU64,
    attack_sample_count: AtomicU32,
    attack_samples: [AtomicU32; W30_RESAMPLE_ATTACK_WINDOW_LEN],
    lineage_capture_count: AtomicU32,
    generation_depth: AtomicU32,
    variation: AtomicU32,
    variation_revision: AtomicU64,
    variation_intensity_bits: AtomicU32,
    music_bus_level_bits: AtomicU32,
    grit_level_bits: AtomicU32,
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
}

impl SharedW30ResampleTapState {
    pub(super) fn new(render_state: &W30ResampleTapState) -> Self {
        let shared = Self {
            revision: AtomicU64::new(0),
            mode: AtomicU32::new(0),
            routing: AtomicU32::new(0),
            source_profile: AtomicU32::new(0),
            source_start_frame: AtomicU64::new(0),
            source_sample_rate: AtomicU32::new(0),
            source_frame_count: AtomicU64::new(0),
            source_sample_count: AtomicU32::new(0),
            source_samples: std::array::from_fn(|_| AtomicU32::new(0.0_f32.to_bits())),
            attack_start_frame: AtomicU64::new(0),
            attack_sample_count: AtomicU32::new(0),
            attack_samples: std::array::from_fn(|_| AtomicU32::new(0.0_f32.to_bits())),
            lineage_capture_count: AtomicU32::new(0),
            generation_depth: AtomicU32::new(0),
            variation: AtomicU32::new(0),
            variation_revision: AtomicU64::new(0),
            variation_intensity_bits: AtomicU32::new(0),
            music_bus_level_bits: AtomicU32::new(0),
            grit_level_bits: AtomicU32::new(0),
            is_transport_running: AtomicBool::new(false),
            tempo_bpm_bits: AtomicU32::new(0),
            position_beats_bits: AtomicU64::new(0),
        };
        shared.update(render_state);
        shared
    }

    pub(super) fn update(&self, render_state: &W30ResampleTapState) {
        begin_coherent_snapshot_update(&self.revision);
        self.mode.store(
            w30_resample_mode_to_u32(render_state.mode),
            Ordering::Relaxed,
        );
        self.routing.store(
            w30_resample_routing_to_u32(render_state.routing),
            Ordering::Relaxed,
        );
        self.source_profile.store(
            w30_resample_source_profile_to_u32(render_state.source_profile),
            Ordering::Relaxed,
        );
        self.update_source_audio(render_state.source_audio.as_deref());
        self.lineage_capture_count.store(
            u32::from(render_state.lineage_capture_count),
            Ordering::Relaxed,
        );
        self.generation_depth
            .store(u32::from(render_state.generation_depth), Ordering::Relaxed);
        self.variation.store(
            w30_resample_variation_to_u32(render_state.variation),
            Ordering::Relaxed,
        );
        self.variation_revision
            .store(render_state.variation_revision, Ordering::Relaxed);
        self.variation_intensity_bits.store(
            render_state.variation_intensity.to_bits(),
            Ordering::Relaxed,
        );
        self.music_bus_level_bits
            .store(render_state.music_bus_level.to_bits(), Ordering::Relaxed);
        self.grit_level_bits
            .store(render_state.grit_level.to_bits(), Ordering::Relaxed);
        self.is_transport_running
            .store(render_state.is_transport_running, Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(render_state.tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(render_state.position_beats.to_bits(), Ordering::Relaxed);
        finish_coherent_snapshot_update(&self.revision);
    }

    pub(super) fn snapshot(&self) -> RealtimeW30ResampleTapState {
        coherent_snapshot(&self.revision, || self.read_snapshot_fields())
    }

    pub(super) fn snapshot_or_previous(
        &self,
        previous: &RealtimeW30ResampleTapState,
    ) -> RealtimeW30ResampleTapState {
        coherent_snapshot_or(&self.revision, previous, || self.read_snapshot_fields())
    }

    fn read_snapshot_fields(&self) -> RealtimeW30ResampleTapState {
        RealtimeW30ResampleTapState {
            mode: w30_resample_mode_from_u32(self.mode.load(Ordering::Relaxed)),
            routing: w30_resample_routing_from_u32(self.routing.load(Ordering::Relaxed)),
            source_profile: w30_resample_source_profile_from_u32(
                self.source_profile.load(Ordering::Relaxed),
            ),
            source_audio: self.source_audio_snapshot(),
            lineage_capture_count: self.lineage_capture_count.load(Ordering::Relaxed) as u8,
            generation_depth: self.generation_depth.load(Ordering::Relaxed) as u8,
            variation: w30_resample_variation_from_u32(self.variation.load(Ordering::Relaxed)),
            variation_revision: self.variation_revision.load(Ordering::Relaxed),
            variation_intensity: f32::from_bits(
                self.variation_intensity_bits.load(Ordering::Relaxed),
            ),
            music_bus_level: f32::from_bits(self.music_bus_level_bits.load(Ordering::Relaxed)),
            grit_level: f32::from_bits(self.grit_level_bits.load(Ordering::Relaxed)),
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
            tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
            position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
        }
    }

    fn update_source_audio(&self, source_audio: Option<&W30ResampleSourceWindow>) {
        if let Some(source_audio) = source_audio {
            let sample_count = source_audio
                .sample_count
                .min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
            self.source_start_frame
                .store(source_audio.source_start_frame, Ordering::Relaxed);
            self.source_sample_rate
                .store(source_audio.source_sample_rate, Ordering::Relaxed);
            self.source_frame_count
                .store(source_audio.source_frame_count, Ordering::Relaxed);
            for (index, sample) in source_audio.samples.iter().copied().enumerate() {
                self.source_samples[index].store(sample.to_bits(), Ordering::Relaxed);
            }
            self.source_sample_count
                .store(sample_count as u32, Ordering::Relaxed);
            let attack_sample_count = source_audio
                .attack_sample_count
                .min(W30_RESAMPLE_ATTACK_WINDOW_LEN);
            self.attack_start_frame
                .store(source_audio.attack_start_frame, Ordering::Relaxed);
            for (index, sample) in source_audio.attack_samples.iter().copied().enumerate() {
                self.attack_samples[index].store(sample.to_bits(), Ordering::Relaxed);
            }
            self.attack_sample_count
                .store(attack_sample_count as u32, Ordering::Relaxed);
        } else {
            self.source_start_frame.store(0, Ordering::Relaxed);
            self.source_sample_rate.store(0, Ordering::Relaxed);
            self.source_frame_count.store(0, Ordering::Relaxed);
            self.source_sample_count.store(0, Ordering::Relaxed);
            self.attack_start_frame.store(0, Ordering::Relaxed);
            self.attack_sample_count.store(0, Ordering::Relaxed);
        }
    }

    fn source_audio_snapshot(&self) -> RealtimeW30ResampleSourceWindow {
        let sample_count = (self.source_sample_count.load(Ordering::Relaxed) as usize)
            .min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
        let mut samples = [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN];
        for (index, sample) in samples.iter_mut().enumerate() {
            *sample = f32::from_bits(self.source_samples[index].load(Ordering::Relaxed));
        }
        let attack_sample_count = (self.attack_sample_count.load(Ordering::Relaxed) as usize)
            .min(W30_RESAMPLE_ATTACK_WINDOW_LEN);
        let mut attack_samples = [0.0; W30_RESAMPLE_ATTACK_WINDOW_LEN];
        for (index, sample) in attack_samples.iter_mut().enumerate() {
            *sample = f32::from_bits(self.attack_samples[index].load(Ordering::Relaxed));
        }
        RealtimeW30ResampleSourceWindow {
            source_start_frame: self.source_start_frame.load(Ordering::Relaxed),
            source_sample_rate: self.source_sample_rate.load(Ordering::Relaxed),
            source_frame_count: self.source_frame_count.load(Ordering::Relaxed),
            sample_count,
            samples,
            attack_start_frame: self.attack_start_frame.load(Ordering::Relaxed),
            attack_sample_count,
            attack_samples,
        }
    }
}

fn w30_resample_mode_to_u32(mode: W30ResampleTapMode) -> u32 {
    match mode {
        W30ResampleTapMode::Idle => 0,
        W30ResampleTapMode::CaptureLineageReady => 1,
    }
}

fn w30_resample_mode_from_u32(value: u32) -> W30ResampleTapMode {
    match value {
        1 => W30ResampleTapMode::CaptureLineageReady,
        _ => W30ResampleTapMode::Idle,
    }
}

fn w30_resample_routing_to_u32(routing: W30ResampleTapRouting) -> u32 {
    match routing {
        W30ResampleTapRouting::Silent => 0,
        W30ResampleTapRouting::InternalCaptureTap => 1,
    }
}

fn w30_resample_routing_from_u32(value: u32) -> W30ResampleTapRouting {
    match value {
        1 => W30ResampleTapRouting::InternalCaptureTap,
        _ => W30ResampleTapRouting::Silent,
    }
}

fn w30_resample_source_profile_to_u32(profile: Option<W30ResampleTapSourceProfile>) -> u32 {
    match profile {
        None => 0,
        Some(W30ResampleTapSourceProfile::RawCapture) => 1,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 2,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 3,
    }
}

fn w30_resample_source_profile_from_u32(value: u32) -> Option<W30ResampleTapSourceProfile> {
    match value {
        1 => Some(W30ResampleTapSourceProfile::RawCapture),
        2 => Some(W30ResampleTapSourceProfile::PromotedCapture),
        3 => Some(W30ResampleTapSourceProfile::PinnedCapture),
        _ => None,
    }
}

fn w30_resample_variation_to_u32(variation: W30ResampleTapVariation) -> u32 {
    match variation {
        W30ResampleTapVariation::Base => 0,
        W30ResampleTapVariation::HardDamage => 1,
    }
}

fn w30_resample_variation_from_u32(value: u32) -> W30ResampleTapVariation {
    match value {
        1 => W30ResampleTapVariation::HardDamage,
        _ => W30ResampleTapVariation::Base,
    }
}

#[derive(Debug, Default)]
pub(super) struct Tr909CallbackState {
    pub(super) beat_position: f64,
    pub(super) oscillator_phase: f32,
    pub(super) oscillator_hz: f32,
    pub(super) envelope: f32,
    pub(super) last_step: i64,
    pub(super) was_running: bool,
    pub(super) fill_voices: Tr909FillVoiceState,
}

#[derive(Debug, Default)]
pub(super) struct TransportTimingCallbackState {
    pub(super) beat_position: f64,
    pub(super) last_control_position_beats: f64,
    pub(super) was_running: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct CallbackTimingSnapshot {
    pub(super) is_transport_running: bool,
    pub(super) tempo_bpm: f32,
    pub(super) render_position_beats: f64,
    pub(super) completed_position_beats: f64,
}

#[derive(Debug, Default)]
pub(super) struct W30PreviewCallbackState {
    pub(super) beat_position: f64,
    pub(super) oscillator_phase: f32,
    pub(super) lfo_phase: f32,
    pub(super) source_sample_cursor: f32,
    pub(super) pad_playback_cursor: f32,
    pub(super) pad_playback_age_frames: u64,
    pub(super) last_character_input: f32,
    pub(super) character_edge_memory: f32,
    pub(super) last_source_window_signature: u64,
    pub(super) last_pad_playback_signature: u64,
    pub(super) envelope: f32,
    pub(super) last_step: i64,
    pub(super) last_trigger_revision: u64,
    pub(super) was_active: bool,
    pub(super) last_mode: Option<W30PreviewRenderMode>,
    pub(super) last_routing: Option<W30PreviewRenderRouting>,
    pub(super) last_source_profile: Option<W30PreviewSourceProfile>,
    pub(super) last_music_bus_level: f32,
    pub(super) last_grit_level: f32,
    pub(super) last_transport_running: bool,
    pub(super) transport_stop_latched: bool,
    pub(super) transport_stop_fade_frames_remaining: u32,
    pub(super) last_position_beats: f64,
}

#[derive(Debug, Default)]
pub(super) struct W30ResampleTapCallbackState {
    pub(super) beat_position: f64,
    pub(super) source_sample_cursor: f32,
    pub(super) attack_sample_cursor: f32,
    pub(super) last_character_input: f32,
    pub(super) character_edge_memory: f32,
    pub(super) envelope: f32,
    pub(super) last_step: i64,
    pub(super) last_variation_revision: u64,
    pub(super) variation_transition_frames_remaining: u32,
    pub(super) variation_transition_total_frames: u32,
    pub(super) variation_transition_start_sample: f32,
    pub(super) last_output_sample: f32,
    pub(super) was_active: bool,
    pub(super) last_transport_running: bool,
    pub(super) transport_stop_latched: bool,
    pub(super) transport_stop_fade_frames_remaining: u32,
}

pub(super) struct W30MixRenderState<'a> {
    pub(super) preview_render: &'a RealtimeW30PreviewRenderState,
    pub(super) preview_state: &'a mut W30PreviewCallbackState,
    pub(super) resample_render: &'a RealtimeW30ResampleTapState,
    pub(super) resample_state: &'a mut W30ResampleTapCallbackState,
}

fn sync_w30_preview_state(
    render: &RealtimeW30PreviewRenderState,
    state: &mut W30PreviewCallbackState,
) {
    state.last_mode = (!matches!(render.mode, W30PreviewRenderMode::Idle)).then_some(render.mode);
    state.last_routing =
        (!matches!(render.routing, W30PreviewRenderRouting::Silent)).then_some(render.routing);
    state.last_source_profile = render.source_profile;
    state.last_music_bus_level = render.music_bus_level;
    state.last_grit_level = render.grit_level;
    state.last_position_beats = render.position_beats;
}

pub(super) fn render_mix_buffer(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    tr909_render: &RealtimeTr909RenderState,
    mc202_render: &RealtimeMc202RenderState,
    tr909_state: &mut Tr909CallbackState,
    w30: &mut W30MixRenderState<'_>,
) {
    data.fill(0.0);
    let fill_focus = FillFocusRenderState::from_tr909(tr909_render);
    if fill_focus.is_active() {
        render_non_tr909_bed(data, sample_rate, channel_count, mc202_render, w30);
        apply_fill_focus_to_non_tr909_bed(data, sample_rate, channel_count, fill_focus);
        render_tr909_buffer(data, sample_rate, channel_count, tr909_render, tr909_state);
        return;
    }

    // Preserve the established summation order (and therefore non-fill output hashes).
    render_tr909_buffer(data, sample_rate, channel_count, tr909_render, tr909_state);
    render_non_tr909_bed(data, sample_rate, channel_count, mc202_render, w30);
}

fn render_non_tr909_bed(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    mc202_render: &RealtimeMc202RenderState,
    w30: &mut W30MixRenderState<'_>,
) {
    render_mc202_buffer(data, sample_rate, channel_count, &(*mc202_render).into());
    sync_w30_preview_state(w30.preview_render, w30.preview_state);
    render_w30_preview_buffer(
        data,
        sample_rate,
        channel_count,
        w30.preview_render,
        w30.preview_state,
    );
    render_w30_resample_tap_buffer(
        data,
        sample_rate,
        channel_count,
        w30.resample_render,
        w30.resample_state,
    );
}

pub(super) fn advance_transport_timing(
    control: &RealtimeTransportTimingState,
    state: &mut TransportTimingCallbackState,
    sample_rate: u32,
    frame_count: usize,
) -> CallbackTimingSnapshot {
    let transport_running = control.is_transport_running && control.tempo_bpm > 0.0;
    if !transport_running {
        state.was_running = false;
        state.beat_position = control.position_beats;
        state.last_control_position_beats = control.position_beats;
        return CallbackTimingSnapshot {
            is_transport_running: false,
            tempo_bpm: control.tempo_bpm,
            render_position_beats: control.position_beats,
            completed_position_beats: control.position_beats,
        };
    }

    let control_position_changed =
        (state.last_control_position_beats - control.position_beats).abs() > f64::EPSILON;
    if !state.was_running
        || (control_position_changed
            && (state.beat_position - control.position_beats).abs() > 0.125)
    {
        state.beat_position = control.position_beats;
        state.was_running = true;
    }
    state.last_control_position_beats = control.position_beats;

    let render_position_beats = state.beat_position;
    let beats_per_sample = f64::from(control.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1));
    let completed_position_beats = render_position_beats + (beats_per_sample * frame_count as f64);
    state.beat_position = completed_position_beats;

    CallbackTimingSnapshot {
        is_transport_running: true,
        tempo_bpm: control.tempo_bpm,
        render_position_beats,
        completed_position_beats,
    }
}
