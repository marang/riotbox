#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mc202RenderMode {
    Idle,
    Leader,
    Follower,
    Answer,
    Pressure,
    Instigator,
}

impl Mc202RenderMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Leader => "leader",
            Self::Follower => "follower",
            Self::Answer => "answer",
            Self::Pressure => "pressure",
            Self::Instigator => "instigator",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mc202RenderRouting {
    Silent,
    MusicBusBass,
}

impl Mc202RenderRouting {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::MusicBusBass => "music_bus_bass",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mc202PhraseShape {
    RootPulse,
    FollowerDrive,
    AnswerHook,
    MutatedDrive,
    PressureCell,
    InstigatorSpike,
}

impl Mc202PhraseShape {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::RootPulse => "root_pulse",
            Self::FollowerDrive => "follower_drive",
            Self::AnswerHook => "answer_hook",
            Self::MutatedDrive => "mutated_drive",
            Self::PressureCell => "pressure_cell",
            Self::InstigatorSpike => "instigator_spike",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mc202NoteBudget {
    Sparse,
    Balanced,
    Push,
    Wide,
}

impl Mc202NoteBudget {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Sparse => "sparse",
            Self::Balanced => "balanced",
            Self::Push => "push",
            Self::Wide => "wide",
        }
    }

    const fn max_active_steps(self) -> usize {
        match self {
            Self::Sparse => 7,
            Self::Balanced => 10,
            Self::Push => 8,
            Self::Wide => 12,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mc202ContourHint {
    Neutral,
    Lift,
    Drop,
    Hold,
}

impl Mc202ContourHint {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Lift => "lift",
            Self::Drop => "drop",
            Self::Hold => "hold",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mc202HookResponse {
    Direct,
    AnswerSpace,
}

impl Mc202HookResponse {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::AnswerSpace => "answer_space",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mc202RenderState {
    pub mode: Mc202RenderMode,
    pub routing: Mc202RenderRouting,
    pub phrase_shape: Mc202PhraseShape,
    pub note_budget: Mc202NoteBudget,
    pub contour_hint: Mc202ContourHint,
    pub hook_response: Mc202HookResponse,
    pub touch: f32,
    pub music_bus_level: f32,
    pub tempo_bpm: f32,
    pub position_beats: f64,
    pub is_transport_running: bool,
}

impl Default for Mc202RenderState {
    fn default() -> Self {
        Self {
            mode: Mc202RenderMode::Idle,
            routing: Mc202RenderRouting::Silent,
            phrase_shape: Mc202PhraseShape::RootPulse,
            note_budget: Mc202NoteBudget::Balanced,
            contour_hint: Mc202ContourHint::Neutral,
            hook_response: Mc202HookResponse::Direct,
            touch: 0.4,
            music_bus_level: 0.72,
            tempo_bpm: 128.0,
            position_beats: 0.0,
            is_transport_running: false,
        }
    }
}

pub fn render_mc202_buffer(
    buffer: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &Mc202RenderState,
) {
    if channel_count == 0
        || matches!(render.mode, Mc202RenderMode::Idle)
        || matches!(render.routing, Mc202RenderRouting::Silent)
        || !render.is_transport_running
        || render.music_bus_level <= 0.0
    {
        return;
    }

    let sample_rate = sample_rate.max(1) as f64;
    let tempo_bpm = render.tempo_bpm.max(1.0) as f64;
    let touch = render.touch.clamp(0.0, 1.0);
    let gain = render.music_bus_level.clamp(0.0, 1.0) * (0.08 + touch * 0.08);

    for frame in 0..buffer.len() / channel_count {
        let beat = render.position_beats + frame as f64 * tempo_bpm / 60.0 / sample_rate;
        let sixteenth = (beat * 4.0).floor() as usize;
        let step_phase = (beat * 4.0).fract() as f32;
        let Some(semitone) = step_semitone(render.phrase_shape, sixteenth) else {
            continue;
        };
        if !within_note_budget(render.phrase_shape, render.note_budget, sixteenth) {
            continue;
        }
        if !within_hook_response(render.hook_response, sixteenth) {
            continue;
        }
        let semitone = semitone
            + contour_offset(render.contour_hint, sixteenth)
            + hook_response_offset(render.hook_response, sixteenth);

        let octave_drop = match render.mode {
            Mc202RenderMode::Follower | Mc202RenderMode::Pressure => -12.0,
            Mc202RenderMode::Instigator => -2.0,
            _ => -5.0,
        };
        let frequency = 110.0_f64 * 2.0_f64.powf((semitone as f64 + octave_drop) / 12.0);

        let gate_len = match render.phrase_shape {
            Mc202PhraseShape::AnswerHook => 0.42,
            Mc202PhraseShape::PressureCell => 0.50,
            Mc202PhraseShape::InstigatorSpike => 0.30,
            _ => 0.62,
        };
        if step_phase > gate_len {
            continue;
        }

        let env = (1.0 - step_phase / gate_len).powf(1.8);
        let accent = if sixteenth.is_multiple_of(8) {
            1.0 + touch * 0.55
        } else if sixteenth % 4 == 2 {
            1.0 + touch * 0.25
        } else {
            0.82
        };
        let note_seconds = f64::from(step_phase) * 60.0 / tempo_bpm / 4.0;
        let phase = (note_seconds * frequency).fract();
        let saw = (phase as f32 * 2.0) - 1.0;
        let pulse = if phase < 0.42 { 1.0 } else { -1.0 };
        let bite = (saw * (0.58 + touch * 0.25)) + (pulse * (0.24 + touch * 0.18));
        let sample = (bite * env * accent * gain).tanh();

        for channel in 0..channel_count {
            buffer[frame * channel_count + channel] += sample;
        }
    }
}

fn step_semitone(shape: Mc202PhraseShape, sixteenth: usize) -> Option<i8> {
    let pattern = pattern_for_shape(shape);
    pattern[sixteenth % pattern.len()]
}

fn pattern_for_shape(shape: Mc202PhraseShape) -> &'static [Option<i8>; 16] {
    match shape {
        Mc202PhraseShape::RootPulse => &[
            Some(0),
            None,
            Some(0),
            None,
            Some(7),
            None,
            Some(0),
            None,
            Some(0),
            None,
            Some(5),
            None,
            Some(7),
            None,
            Some(0),
            None,
        ],
        Mc202PhraseShape::FollowerDrive => &[
            Some(0),
            Some(0),
            None,
            Some(3),
            Some(5),
            None,
            Some(7),
            Some(5),
            Some(0),
            None,
            Some(3),
            Some(5),
            Some(7),
            Some(10),
            Some(7),
            None,
        ],
        Mc202PhraseShape::AnswerHook => &[
            None,
            Some(12),
            Some(10),
            None,
            Some(7),
            None,
            Some(10),
            Some(12),
            None,
            Some(15),
            Some(12),
            None,
            Some(10),
            Some(7),
            None,
            Some(12),
        ],
        Mc202PhraseShape::MutatedDrive => &[
            Some(0),
            Some(7),
            Some(3),
            None,
            Some(10),
            Some(7),
            None,
            Some(5),
            Some(12),
            None,
            Some(10),
            Some(3),
            Some(7),
            Some(5),
            Some(0),
            Some(15),
        ],
        Mc202PhraseShape::PressureCell => &[
            None,
            Some(0),
            None,
            Some(0),
            None,
            Some(7),
            None,
            Some(0),
            Some(10),
            None,
            Some(7),
            None,
            None,
            Some(5),
            Some(7),
            None,
        ],
        Mc202PhraseShape::InstigatorSpike => &[
            Some(12),
            None,
            Some(15),
            Some(19),
            None,
            Some(12),
            None,
            Some(22),
            Some(24),
            Some(19),
            None,
            Some(15),
            Some(12),
            None,
            Some(27),
            None,
        ],
    }
}

fn within_note_budget(shape: Mc202PhraseShape, budget: Mc202NoteBudget, sixteenth: usize) -> bool {
    let pattern = pattern_for_shape(shape);
    let step = sixteenth % pattern.len();
    let active_index = pattern
        .iter()
        .take(step + 1)
        .filter(|semitone| semitone.is_some())
        .count()
        .saturating_sub(1);

    active_index < budget.max_active_steps()
}

fn contour_offset(hint: Mc202ContourHint, sixteenth: usize) -> i8 {
    let quarter = (sixteenth % 16) / 4;
    match hint {
        Mc202ContourHint::Neutral => 0,
        Mc202ContourHint::Lift => match quarter {
            0 => 0,
            1 => 2,
            2 => 5,
            _ => 7,
        },
        Mc202ContourHint::Drop => match quarter {
            0 => 7,
            1 => 5,
            2 => 2,
            _ => 0,
        },
        Mc202ContourHint::Hold => {
            if sixteenth % 4 >= 2 {
                -5
            } else {
                0
            }
        }
    }
}

fn within_hook_response(response: Mc202HookResponse, sixteenth: usize) -> bool {
    match response {
        Mc202HookResponse::Direct => true,
        Mc202HookResponse::AnswerSpace => matches!(sixteenth % 4, 1 | 3),
    }
}

fn hook_response_offset(response: Mc202HookResponse, sixteenth: usize) -> i8 {
    match response {
        Mc202HookResponse::Direct => 0,
        Mc202HookResponse::AnswerSpace => {
            if sixteenth % 8 >= 4 {
                12
            } else {
                7
            }
        }
    }
}

