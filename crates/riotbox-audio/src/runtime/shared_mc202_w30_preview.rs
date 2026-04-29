impl SharedTr909RenderState {
    fn new(render_state: &Tr909RenderState) -> Self {
        let shared = Self {
            mode: AtomicU32::new(0),
            routing: AtomicU32::new(0),
            source_support_profile: AtomicU32::new(0),
            source_support_context: AtomicU32::new(0),
            pattern_adoption: AtomicU32::new(0),
            phrase_variation: AtomicU32::new(0),
            takeover_profile: AtomicU32::new(0),
            drum_bus_level_bits: AtomicU32::new(0),
            slam_intensity_bits: AtomicU32::new(0),
            is_transport_running: AtomicBool::new(false),
            tempo_bpm_bits: AtomicU32::new(0),
            position_beats_bits: AtomicU64::new(0),
        };
        shared.update(render_state);
        shared
    }

    fn update(&self, render_state: &Tr909RenderState) {
        self.mode
            .store(mode_to_u32(render_state.mode), Ordering::Relaxed);
        self.routing
            .store(routing_to_u32(render_state.routing), Ordering::Relaxed);
        self.source_support_profile.store(
            support_profile_to_u32(render_state.source_support_profile),
            Ordering::Relaxed,
        );
        self.source_support_context.store(
            support_context_to_u32(render_state.source_support_context),
            Ordering::Relaxed,
        );
        self.pattern_adoption.store(
            pattern_adoption_to_u32(render_state.pattern_adoption),
            Ordering::Relaxed,
        );
        self.phrase_variation.store(
            phrase_variation_to_u32(render_state.phrase_variation),
            Ordering::Relaxed,
        );
        self.takeover_profile.store(
            takeover_profile_to_u32(render_state.takeover_profile),
            Ordering::Relaxed,
        );
        self.drum_bus_level_bits
            .store(render_state.drum_bus_level.to_bits(), Ordering::Relaxed);
        self.slam_intensity_bits
            .store(render_state.slam_intensity.to_bits(), Ordering::Relaxed);
        self.is_transport_running
            .store(render_state.is_transport_running, Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(render_state.tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(render_state.position_beats.to_bits(), Ordering::Relaxed);
    }

    fn snapshot(&self) -> RealtimeTr909RenderState {
        RealtimeTr909RenderState {
            mode: mode_from_u32(self.mode.load(Ordering::Relaxed)),
            routing: routing_from_u32(self.routing.load(Ordering::Relaxed)),
            source_support_profile: support_profile_from_u32(
                self.source_support_profile.load(Ordering::Relaxed),
            ),
            source_support_context: support_context_from_u32(
                self.source_support_context.load(Ordering::Relaxed),
            ),
            pattern_adoption: pattern_adoption_from_u32(
                self.pattern_adoption.load(Ordering::Relaxed),
            ),
            phrase_variation: phrase_variation_from_u32(
                self.phrase_variation.load(Ordering::Relaxed),
            ),
            takeover_profile: takeover_profile_from_u32(
                self.takeover_profile.load(Ordering::Relaxed),
            ),
            drum_bus_level: f32::from_bits(self.drum_bus_level_bits.load(Ordering::Relaxed)),
            slam_intensity: f32::from_bits(self.slam_intensity_bits.load(Ordering::Relaxed)),
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
            tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
            position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeMc202RenderState {
    mode: Mc202RenderMode,
    routing: Mc202RenderRouting,
    phrase_shape: Mc202PhraseShape,
    note_budget: Mc202NoteBudget,
    contour_hint: Mc202ContourHint,
    hook_response: Mc202HookResponse,
    touch: f32,
    music_bus_level: f32,
    tempo_bpm: f32,
    position_beats: f64,
    is_transport_running: bool,
}

impl From<RealtimeMc202RenderState> for Mc202RenderState {
    fn from(render: RealtimeMc202RenderState) -> Self {
        Self {
            mode: render.mode,
            routing: render.routing,
            phrase_shape: render.phrase_shape,
            note_budget: render.note_budget,
            contour_hint: render.contour_hint,
            hook_response: render.hook_response,
            touch: render.touch,
            music_bus_level: render.music_bus_level,
            tempo_bpm: render.tempo_bpm,
            position_beats: render.position_beats,
            is_transport_running: render.is_transport_running,
        }
    }
}

struct SharedMc202RenderState {
    mode: AtomicU32,
    routing: AtomicU32,
    phrase_shape: AtomicU32,
    note_budget: AtomicU32,
    contour_hint: AtomicU32,
    hook_response: AtomicU32,
    touch_bits: AtomicU32,
    music_bus_level_bits: AtomicU32,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
    is_transport_running: AtomicBool,
}

impl SharedMc202RenderState {
    fn new(render_state: &Mc202RenderState) -> Self {
        let shared = Self {
            mode: AtomicU32::new(0),
            routing: AtomicU32::new(0),
            phrase_shape: AtomicU32::new(0),
            note_budget: AtomicU32::new(mc202_note_budget_to_u32(Mc202NoteBudget::Balanced)),
            contour_hint: AtomicU32::new(mc202_contour_hint_to_u32(Mc202ContourHint::Neutral)),
            hook_response: AtomicU32::new(mc202_hook_response_to_u32(Mc202HookResponse::Direct)),
            touch_bits: AtomicU32::new(0),
            music_bus_level_bits: AtomicU32::new(0),
            tempo_bpm_bits: AtomicU32::new(0),
            position_beats_bits: AtomicU64::new(0),
            is_transport_running: AtomicBool::new(false),
        };
        shared.update(render_state);
        shared
    }

    fn update(&self, render_state: &Mc202RenderState) {
        self.mode
            .store(mc202_mode_to_u32(render_state.mode), Ordering::Relaxed);
        self.routing.store(
            mc202_routing_to_u32(render_state.routing),
            Ordering::Relaxed,
        );
        self.phrase_shape.store(
            mc202_phrase_shape_to_u32(render_state.phrase_shape),
            Ordering::Relaxed,
        );
        self.note_budget.store(
            mc202_note_budget_to_u32(render_state.note_budget),
            Ordering::Relaxed,
        );
        self.contour_hint.store(
            mc202_contour_hint_to_u32(render_state.contour_hint),
            Ordering::Relaxed,
        );
        self.hook_response.store(
            mc202_hook_response_to_u32(render_state.hook_response),
            Ordering::Relaxed,
        );
        self.touch_bits
            .store(render_state.touch.to_bits(), Ordering::Relaxed);
        self.music_bus_level_bits
            .store(render_state.music_bus_level.to_bits(), Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(render_state.tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(render_state.position_beats.to_bits(), Ordering::Relaxed);
        self.is_transport_running
            .store(render_state.is_transport_running, Ordering::Relaxed);
    }

    fn snapshot(&self) -> RealtimeMc202RenderState {
        RealtimeMc202RenderState {
            mode: mc202_mode_from_u32(self.mode.load(Ordering::Relaxed)),
            routing: mc202_routing_from_u32(self.routing.load(Ordering::Relaxed)),
            phrase_shape: mc202_phrase_shape_from_u32(self.phrase_shape.load(Ordering::Relaxed)),
            note_budget: mc202_note_budget_from_u32(self.note_budget.load(Ordering::Relaxed)),
            contour_hint: mc202_contour_hint_from_u32(self.contour_hint.load(Ordering::Relaxed)),
            hook_response: mc202_hook_response_from_u32(self.hook_response.load(Ordering::Relaxed)),
            touch: f32::from_bits(self.touch_bits.load(Ordering::Relaxed)),
            music_bus_level: f32::from_bits(self.music_bus_level_bits.load(Ordering::Relaxed)),
            tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
            position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
        }
    }
}

fn mc202_mode_to_u32(mode: Mc202RenderMode) -> u32 {
    match mode {
        Mc202RenderMode::Idle => 0,
        Mc202RenderMode::Leader => 1,
        Mc202RenderMode::Follower => 2,
        Mc202RenderMode::Answer => 3,
        Mc202RenderMode::Pressure => 4,
        Mc202RenderMode::Instigator => 5,
    }
}

fn mc202_mode_from_u32(value: u32) -> Mc202RenderMode {
    match value {
        1 => Mc202RenderMode::Leader,
        2 => Mc202RenderMode::Follower,
        3 => Mc202RenderMode::Answer,
        4 => Mc202RenderMode::Pressure,
        5 => Mc202RenderMode::Instigator,
        _ => Mc202RenderMode::Idle,
    }
}

fn mc202_routing_to_u32(routing: Mc202RenderRouting) -> u32 {
    match routing {
        Mc202RenderRouting::Silent => 0,
        Mc202RenderRouting::MusicBusBass => 1,
    }
}

fn mc202_routing_from_u32(value: u32) -> Mc202RenderRouting {
    match value {
        1 => Mc202RenderRouting::MusicBusBass,
        _ => Mc202RenderRouting::Silent,
    }
}

fn mc202_phrase_shape_to_u32(shape: Mc202PhraseShape) -> u32 {
    match shape {
        Mc202PhraseShape::RootPulse => 0,
        Mc202PhraseShape::FollowerDrive => 1,
        Mc202PhraseShape::AnswerHook => 2,
        Mc202PhraseShape::MutatedDrive => 3,
        Mc202PhraseShape::PressureCell => 4,
        Mc202PhraseShape::InstigatorSpike => 5,
    }
}

fn mc202_phrase_shape_from_u32(value: u32) -> Mc202PhraseShape {
    match value {
        1 => Mc202PhraseShape::FollowerDrive,
        2 => Mc202PhraseShape::AnswerHook,
        3 => Mc202PhraseShape::MutatedDrive,
        4 => Mc202PhraseShape::PressureCell,
        5 => Mc202PhraseShape::InstigatorSpike,
        _ => Mc202PhraseShape::RootPulse,
    }
}

fn mc202_note_budget_to_u32(budget: Mc202NoteBudget) -> u32 {
    match budget {
        Mc202NoteBudget::Sparse => 0,
        Mc202NoteBudget::Balanced => 1,
        Mc202NoteBudget::Push => 2,
        Mc202NoteBudget::Wide => 3,
    }
}

fn mc202_note_budget_from_u32(value: u32) -> Mc202NoteBudget {
    match value {
        0 => Mc202NoteBudget::Sparse,
        2 => Mc202NoteBudget::Push,
        3 => Mc202NoteBudget::Wide,
        _ => Mc202NoteBudget::Balanced,
    }
}

fn mc202_contour_hint_to_u32(hint: Mc202ContourHint) -> u32 {
    match hint {
        Mc202ContourHint::Neutral => 0,
        Mc202ContourHint::Lift => 1,
        Mc202ContourHint::Drop => 2,
        Mc202ContourHint::Hold => 3,
    }
}

fn mc202_contour_hint_from_u32(value: u32) -> Mc202ContourHint {
    match value {
        1 => Mc202ContourHint::Lift,
        2 => Mc202ContourHint::Drop,
        3 => Mc202ContourHint::Hold,
        _ => Mc202ContourHint::Neutral,
    }
}

fn mc202_hook_response_to_u32(response: Mc202HookResponse) -> u32 {
    match response {
        Mc202HookResponse::Direct => 0,
        Mc202HookResponse::AnswerSpace => 1,
    }
}

fn mc202_hook_response_from_u32(value: u32) -> Mc202HookResponse {
    match value {
        1 => Mc202HookResponse::AnswerSpace,
        _ => Mc202HookResponse::Direct,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeW30PreviewRenderState {
    mode: W30PreviewRenderMode,
    routing: W30PreviewRenderRouting,
    source_profile: Option<W30PreviewSourceProfile>,
    trigger_revision: u64,
    trigger_velocity: f32,
    source_window_preview: RealtimeW30PreviewSampleWindow,
    pad_playback: RealtimeW30PadPlaybackSampleWindow,
    music_bus_level: f32,
    grit_level: f32,
    is_transport_running: bool,
    tempo_bpm: f32,
    position_beats: f64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeW30PreviewSampleWindow {
    source_start_frame: u64,
    source_end_frame: u64,
    sample_count: usize,
    samples: [f32; W30_PREVIEW_SAMPLE_WINDOW_LEN],
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeW30PadPlaybackSampleWindow {
    source_start_frame: u64,
    source_end_frame: u64,
    sample_count: usize,
    loop_enabled: bool,
    samples: [f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
}

impl Default for RealtimeW30PreviewSampleWindow {
    fn default() -> Self {
        Self {
            source_start_frame: 0,
            source_end_frame: 0,
            sample_count: 0,
            samples: [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN],
        }
    }
}

impl Default for RealtimeW30PadPlaybackSampleWindow {
    fn default() -> Self {
        Self {
            source_start_frame: 0,
            source_end_frame: 0,
            sample_count: 0,
            loop_enabled: false,
            samples: [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
        }
    }
}

struct SharedW30PreviewRenderState {
    mode: AtomicU32,
    routing: AtomicU32,
    source_profile: AtomicU32,
    trigger_revision: AtomicU64,
    trigger_velocity_bits: AtomicU32,
    source_start_frame: AtomicU64,
    source_end_frame: AtomicU64,
    source_sample_count: AtomicU32,
    source_samples: [AtomicU32; W30_PREVIEW_SAMPLE_WINDOW_LEN],
    pad_start_frame: AtomicU64,
    pad_end_frame: AtomicU64,
    pad_sample_count: AtomicU32,
    pad_loop_enabled: AtomicBool,
    pad_samples: [AtomicU32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
    music_bus_level_bits: AtomicU32,
    grit_level_bits: AtomicU32,
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
}

