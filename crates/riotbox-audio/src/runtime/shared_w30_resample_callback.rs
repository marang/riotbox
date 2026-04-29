impl SharedW30PreviewRenderState {
    fn new(render_state: &W30PreviewRenderState) -> Self {
        let shared = Self {
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
            pad_sample_count: AtomicU32::new(0),
            pad_loop_enabled: AtomicBool::new(false),
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

    fn update(&self, render_state: &W30PreviewRenderState) {
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
    }

    fn snapshot(&self) -> RealtimeW30PreviewRenderState {
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
            self.pad_loop_enabled
                .store(pad_playback.loop_enabled, Ordering::Relaxed);
            for (index, sample) in pad_playback.samples.iter().copied().enumerate() {
                self.pad_samples[index].store(sample.to_bits(), Ordering::Relaxed);
            }
            self.pad_sample_count
                .store(sample_count as u32, Ordering::Relaxed);
        } else {
            self.pad_sample_count.store(0, Ordering::Relaxed);
            self.pad_start_frame.store(0, Ordering::Relaxed);
            self.pad_end_frame.store(0, Ordering::Relaxed);
            self.pad_loop_enabled.store(false, Ordering::Relaxed);
        }
    }

    fn pad_playback_snapshot(&self) -> RealtimeW30PadPlaybackSampleWindow {
        let sample_count = (self.pad_sample_count.load(Ordering::Relaxed) as usize)
            .min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
        let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
        for (index, sample) in samples.iter_mut().enumerate() {
            *sample = f32::from_bits(self.pad_samples[index].load(Ordering::Relaxed));
        }

        RealtimeW30PadPlaybackSampleWindow {
            source_start_frame: self.pad_start_frame.load(Ordering::Relaxed),
            source_end_frame: self.pad_end_frame.load(Ordering::Relaxed),
            sample_count,
            loop_enabled: self.pad_loop_enabled.load(Ordering::Relaxed),
            samples,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeW30ResampleTapState {
    mode: W30ResampleTapMode,
    routing: W30ResampleTapRouting,
    source_profile: Option<W30ResampleTapSourceProfile>,
    lineage_capture_count: u8,
    generation_depth: u8,
    music_bus_level: f32,
    grit_level: f32,
    is_transport_running: bool,
}

struct SharedW30ResampleTapState {
    mode: AtomicU32,
    routing: AtomicU32,
    source_profile: AtomicU32,
    lineage_capture_count: AtomicU32,
    generation_depth: AtomicU32,
    music_bus_level_bits: AtomicU32,
    grit_level_bits: AtomicU32,
    is_transport_running: AtomicBool,
}

impl SharedW30ResampleTapState {
    fn new(render_state: &W30ResampleTapState) -> Self {
        let shared = Self {
            mode: AtomicU32::new(0),
            routing: AtomicU32::new(0),
            source_profile: AtomicU32::new(0),
            lineage_capture_count: AtomicU32::new(0),
            generation_depth: AtomicU32::new(0),
            music_bus_level_bits: AtomicU32::new(0),
            grit_level_bits: AtomicU32::new(0),
            is_transport_running: AtomicBool::new(false),
        };
        shared.update(render_state);
        shared
    }

    fn update(&self, render_state: &W30ResampleTapState) {
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
        self.lineage_capture_count.store(
            u32::from(render_state.lineage_capture_count),
            Ordering::Relaxed,
        );
        self.generation_depth
            .store(u32::from(render_state.generation_depth), Ordering::Relaxed);
        self.music_bus_level_bits
            .store(render_state.music_bus_level.to_bits(), Ordering::Relaxed);
        self.grit_level_bits
            .store(render_state.grit_level.to_bits(), Ordering::Relaxed);
        self.is_transport_running
            .store(render_state.is_transport_running, Ordering::Relaxed);
    }

    fn snapshot(&self) -> RealtimeW30ResampleTapState {
        RealtimeW30ResampleTapState {
            mode: w30_resample_mode_from_u32(self.mode.load(Ordering::Relaxed)),
            routing: w30_resample_routing_from_u32(self.routing.load(Ordering::Relaxed)),
            source_profile: w30_resample_source_profile_from_u32(
                self.source_profile.load(Ordering::Relaxed),
            ),
            lineage_capture_count: self.lineage_capture_count.load(Ordering::Relaxed) as u8,
            generation_depth: self.generation_depth.load(Ordering::Relaxed) as u8,
            music_bus_level: f32::from_bits(self.music_bus_level_bits.load(Ordering::Relaxed)),
            grit_level: f32::from_bits(self.grit_level_bits.load(Ordering::Relaxed)),
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
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

#[derive(Debug, Default)]
struct Tr909CallbackState {
    beat_position: f64,
    oscillator_phase: f32,
    oscillator_hz: f32,
    envelope: f32,
    last_step: i64,
    was_running: bool,
}

#[derive(Debug, Default)]
struct TransportTimingCallbackState {
    beat_position: f64,
    was_running: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct CallbackTimingSnapshot {
    is_transport_running: bool,
    tempo_bpm: f32,
    render_position_beats: f64,
    completed_position_beats: f64,
}

#[derive(Debug, Default)]
struct W30PreviewCallbackState {
    beat_position: f64,
    oscillator_phase: f32,
    lfo_phase: f32,
    source_sample_cursor: f32,
    pad_playback_cursor: f32,
    last_source_window_signature: u64,
    last_pad_playback_signature: u64,
    envelope: f32,
    last_step: i64,
    last_trigger_revision: u64,
    was_active: bool,
    last_mode: Option<W30PreviewRenderMode>,
    last_routing: Option<W30PreviewRenderRouting>,
    last_source_profile: Option<W30PreviewSourceProfile>,
    last_music_bus_level: f32,
    last_grit_level: f32,
    last_transport_running: bool,
    last_position_beats: f64,
}

#[derive(Debug, Default)]
struct W30ResampleTapCallbackState {
    beat_position: f64,
    oscillator_phase: f32,
    shimmer_phase: f32,
    envelope: f32,
    last_step: i64,
    was_active: bool,
}

struct W30MixRenderState<'a> {
    preview_render: &'a RealtimeW30PreviewRenderState,
    preview_state: &'a mut W30PreviewCallbackState,
    resample_render: &'a RealtimeW30ResampleTapState,
    resample_state: &'a mut W30ResampleTapCallbackState,
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
    state.last_transport_running = render.is_transport_running;
    state.last_position_beats = render.position_beats;
}

fn render_mix_buffer(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    tr909_render: &RealtimeTr909RenderState,
    mc202_render: &RealtimeMc202RenderState,
    tr909_state: &mut Tr909CallbackState,
    w30: &mut W30MixRenderState<'_>,
) {
    data.fill(0.0);
    render_tr909_buffer(data, sample_rate, channel_count, tr909_render, tr909_state);
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

fn advance_transport_timing(
    control: &RealtimeTransportTimingState,
    state: &mut TransportTimingCallbackState,
    sample_rate: u32,
    frame_count: usize,
) -> CallbackTimingSnapshot {
    let transport_running = control.is_transport_running && control.tempo_bpm > 0.0;
    if !transport_running {
        state.was_running = false;
        state.beat_position = control.position_beats;
        return CallbackTimingSnapshot {
            is_transport_running: false,
            tempo_bpm: control.tempo_bpm,
            render_position_beats: control.position_beats,
            completed_position_beats: control.position_beats,
        };
    }

    if !state.was_running || (state.beat_position - control.position_beats).abs() > 0.125 {
        state.beat_position = control.position_beats;
        state.was_running = true;
    }

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

