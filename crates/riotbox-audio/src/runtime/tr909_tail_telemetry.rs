fn envelope_decay(render: &RealtimeTr909RenderState) -> f32 {
    let slam = render.slam_intensity.clamp(0.0, 1.0);
    let base = match render.mode {
        Tr909RenderMode::Idle => 0.0,
        Tr909RenderMode::SourceSupport => match render.source_support_profile {
            Some(Tr909SourceSupportProfile::SteadyPulse) | None => 0.992 - (slam * 0.002),
            Some(Tr909SourceSupportProfile::BreakLift) => 0.989 - (slam * 0.003),
            Some(Tr909SourceSupportProfile::DropDrive) => 0.986 - (slam * 0.004),
        },
        Tr909RenderMode::Fill => 0.988 - (slam * 0.003),
        Tr909RenderMode::BreakReinforce => 0.989 - (slam * 0.003),
        Tr909RenderMode::Takeover => match render.takeover_profile {
            Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => 0.986 - (slam * 0.004),
            Some(Tr909TakeoverRenderProfile::SceneLock) => 0.982 - (slam * 0.005),
        },
    };
    let pattern_decay = match render.pattern_adoption {
        Some(Tr909PatternAdoption::SupportPulse) | None => 0.0,
        Some(Tr909PatternAdoption::MainlineDrive) => 0.002,
        Some(Tr909PatternAdoption::TakeoverGrid) => 0.004,
    };
    let phrase_decay = match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => 0.0,
        Some(Tr909PhraseVariation::PhraseLift) => -0.001,
        Some(Tr909PhraseVariation::PhraseDrive) => -0.003,
        Some(Tr909PhraseVariation::PhraseRelease) => 0.01,
    };
    (base - pattern_decay - phrase_decay).clamp(0.0, 1.0)
}

const fn mode_to_u32(mode: Tr909RenderMode) -> u32 {
    match mode {
        Tr909RenderMode::Idle => 0,
        Tr909RenderMode::SourceSupport => 1,
        Tr909RenderMode::Fill => 2,
        Tr909RenderMode::BreakReinforce => 3,
        Tr909RenderMode::Takeover => 4,
    }
}

const fn mode_from_u32(value: u32) -> Tr909RenderMode {
    match value {
        1 => Tr909RenderMode::SourceSupport,
        2 => Tr909RenderMode::Fill,
        3 => Tr909RenderMode::BreakReinforce,
        4 => Tr909RenderMode::Takeover,
        _ => Tr909RenderMode::Idle,
    }
}

const fn routing_to_u32(routing: Tr909RenderRouting) -> u32 {
    match routing {
        Tr909RenderRouting::SourceOnly => 0,
        Tr909RenderRouting::DrumBusSupport => 1,
        Tr909RenderRouting::DrumBusTakeover => 2,
    }
}

const fn routing_from_u32(value: u32) -> Tr909RenderRouting {
    match value {
        1 => Tr909RenderRouting::DrumBusSupport,
        2 => Tr909RenderRouting::DrumBusTakeover,
        _ => Tr909RenderRouting::SourceOnly,
    }
}

const fn support_profile_to_u32(profile: Option<Tr909SourceSupportProfile>) -> u32 {
    match profile {
        None => 0,
        Some(Tr909SourceSupportProfile::SteadyPulse) => 1,
        Some(Tr909SourceSupportProfile::BreakLift) => 2,
        Some(Tr909SourceSupportProfile::DropDrive) => 3,
    }
}

const fn support_profile_from_u32(value: u32) -> Option<Tr909SourceSupportProfile> {
    match value {
        1 => Some(Tr909SourceSupportProfile::SteadyPulse),
        2 => Some(Tr909SourceSupportProfile::BreakLift),
        3 => Some(Tr909SourceSupportProfile::DropDrive),
        _ => None,
    }
}

const fn support_context_to_u32(context: Option<Tr909SourceSupportContext>) -> u32 {
    match context {
        None => 0,
        Some(Tr909SourceSupportContext::SceneTarget) => 1,
        Some(Tr909SourceSupportContext::TransportBar) => 2,
    }
}

const fn support_context_from_u32(value: u32) -> Option<Tr909SourceSupportContext> {
    match value {
        1 => Some(Tr909SourceSupportContext::SceneTarget),
        2 => Some(Tr909SourceSupportContext::TransportBar),
        _ => None,
    }
}

const fn pattern_adoption_to_u32(pattern: Option<Tr909PatternAdoption>) -> u32 {
    match pattern {
        None => 0,
        Some(Tr909PatternAdoption::SupportPulse) => 1,
        Some(Tr909PatternAdoption::MainlineDrive) => 2,
        Some(Tr909PatternAdoption::TakeoverGrid) => 3,
    }
}

const fn pattern_adoption_from_u32(value: u32) -> Option<Tr909PatternAdoption> {
    match value {
        1 => Some(Tr909PatternAdoption::SupportPulse),
        2 => Some(Tr909PatternAdoption::MainlineDrive),
        3 => Some(Tr909PatternAdoption::TakeoverGrid),
        _ => None,
    }
}

const fn phrase_variation_to_u32(variation: Option<Tr909PhraseVariation>) -> u32 {
    match variation {
        None => 0,
        Some(Tr909PhraseVariation::PhraseAnchor) => 1,
        Some(Tr909PhraseVariation::PhraseLift) => 2,
        Some(Tr909PhraseVariation::PhraseDrive) => 3,
        Some(Tr909PhraseVariation::PhraseRelease) => 4,
    }
}

const fn phrase_variation_from_u32(value: u32) -> Option<Tr909PhraseVariation> {
    match value {
        1 => Some(Tr909PhraseVariation::PhraseAnchor),
        2 => Some(Tr909PhraseVariation::PhraseLift),
        3 => Some(Tr909PhraseVariation::PhraseDrive),
        4 => Some(Tr909PhraseVariation::PhraseRelease),
        _ => None,
    }
}

const fn takeover_profile_to_u32(profile: Option<Tr909TakeoverRenderProfile>) -> u32 {
    match profile {
        None => 0,
        Some(Tr909TakeoverRenderProfile::ControlledPhrase) => 1,
        Some(Tr909TakeoverRenderProfile::SceneLock) => 2,
    }
}

const fn takeover_profile_from_u32(value: u32) -> Option<Tr909TakeoverRenderProfile> {
    match value {
        1 => Some(Tr909TakeoverRenderProfile::ControlledPhrase),
        2 => Some(Tr909TakeoverRenderProfile::SceneLock),
        _ => None,
    }
}

const fn w30_mode_to_u32(mode: W30PreviewRenderMode) -> u32 {
    match mode {
        W30PreviewRenderMode::Idle => 0,
        W30PreviewRenderMode::LiveRecall => 1,
        W30PreviewRenderMode::RawCaptureAudition => 2,
        W30PreviewRenderMode::PromotedAudition => 3,
    }
}

const fn w30_mode_from_u32(value: u32) -> W30PreviewRenderMode {
    match value {
        1 => W30PreviewRenderMode::LiveRecall,
        2 => W30PreviewRenderMode::RawCaptureAudition,
        3 => W30PreviewRenderMode::PromotedAudition,
        _ => W30PreviewRenderMode::Idle,
    }
}

const fn w30_routing_to_u32(routing: W30PreviewRenderRouting) -> u32 {
    match routing {
        W30PreviewRenderRouting::Silent => 0,
        W30PreviewRenderRouting::MusicBusPreview => 1,
    }
}

const fn w30_routing_from_u32(value: u32) -> W30PreviewRenderRouting {
    match value {
        1 => W30PreviewRenderRouting::MusicBusPreview,
        _ => W30PreviewRenderRouting::Silent,
    }
}

const fn w30_source_profile_to_u32(profile: Option<W30PreviewSourceProfile>) -> u32 {
    match profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 1,
        Some(W30PreviewSourceProfile::PromotedRecall) => 2,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => 3,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => 4,
        Some(W30PreviewSourceProfile::PromotedAudition) => 5,
        None => 0,
    }
}

const fn w30_source_profile_from_u32(value: u32) -> Option<W30PreviewSourceProfile> {
    match value {
        1 => Some(W30PreviewSourceProfile::PinnedRecall),
        2 => Some(W30PreviewSourceProfile::PromotedRecall),
        3 => Some(W30PreviewSourceProfile::SlicePoolBrowse),
        4 => Some(W30PreviewSourceProfile::RawCaptureAudition),
        5 => Some(W30PreviewSourceProfile::PromotedAudition),
        _ => None,
    }
}

#[derive(Default)]
struct RuntimeTelemetrySnapshot {
    callback_count: u64,
    max_callback_gap_micros: Option<u64>,
    stream_error_count: u64,
    last_stream_error: Option<String>,
    timing: AudioRuntimeTimingSnapshot,
}

struct RuntimeTelemetry {
    callback_count: AtomicU64,
    max_callback_gap_micros: AtomicU64,
    last_callback_micros: AtomicU64,
    stream_error_count: AtomicU64,
    last_stream_error: Mutex<Option<String>>,
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
}

impl RuntimeTelemetry {
    fn new() -> Self {
        Self {
            callback_count: AtomicU64::new(0),
            max_callback_gap_micros: AtomicU64::new(0),
            last_callback_micros: AtomicU64::new(0),
            stream_error_count: AtomicU64::new(0),
            last_stream_error: Mutex::new(None),
            is_transport_running: AtomicBool::new(false),
            tempo_bpm_bits: AtomicU32::new(0.0_f32.to_bits()),
            position_beats_bits: AtomicU64::new(0.0_f64.to_bits()),
        }
    }

    fn record_callback_at(&self, now_micros: u64, timing: &CallbackTimingSnapshot) {
        let previous = self
            .last_callback_micros
            .swap(now_micros, Ordering::Relaxed);
        if previous != 0 {
            let gap = now_micros.saturating_sub(previous);
            self.max_callback_gap_micros
                .fetch_max(gap, Ordering::Relaxed);
        }
        self.callback_count.fetch_add(1, Ordering::Relaxed);
        self.is_transport_running
            .store(timing.is_transport_running, Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(timing.tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(timing.completed_position_beats.to_bits(), Ordering::Relaxed);
    }

    fn record_stream_error(&self, message: String) {
        self.stream_error_count.fetch_add(1, Ordering::Relaxed);
        *self
            .last_stream_error
            .lock()
            .expect("lock stream error buffer") = Some(message);
    }

    fn snapshot(&self) -> RuntimeTelemetrySnapshot {
        let callback_count = self.callback_count.load(Ordering::Relaxed);
        let max_gap_micros = self.max_callback_gap_micros.load(Ordering::Relaxed);

        RuntimeTelemetrySnapshot {
            callback_count,
            max_callback_gap_micros: (callback_count > 1).then_some(max_gap_micros),
            stream_error_count: self.stream_error_count.load(Ordering::Relaxed),
            last_stream_error: self
                .last_stream_error
                .lock()
                .expect("lock stream error buffer")
                .clone(),
            timing: AudioRuntimeTimingSnapshot {
                is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
                tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
                position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
            },
        }
    }

    fn timing_snapshot(&self) -> AudioRuntimeTimingSnapshot {
        self.snapshot().timing
    }
}
