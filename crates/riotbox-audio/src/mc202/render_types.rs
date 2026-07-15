use super::source_phrase_sound_design::{
    mc202_source_phrase_sample, mc202_source_phrase_sound_design,
};

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
}

impl Mc202HookResponse {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mc202SourcePhraseRenderPlan {
    pub active_mask: u16,
    pub semitones: [i8; 16],
    pub accent_mask: u16,
    pub destructive_mask: u16,
    pub pressure: f32,
    pub contrast: f32,
    pub bass_weight: f32,
    pub stab_bite: f32,
    pub gate_snap: f32,
}

impl Mc202SourcePhraseRenderPlan {
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.active_mask == 0
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
    pub source_phrase_plan: Option<Mc202SourcePhraseRenderPlan>,
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
            source_phrase_plan: None,
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
    let Some(source_phrase_plan) = render.source_phrase_plan.filter(|plan| !plan.is_empty()) else {
        return;
    };

    let sample_rate = sample_rate.max(1) as f64;
    let tempo_bpm = render.tempo_bpm.max(1.0) as f64;
    let touch = render.touch.clamp(0.0, 1.0);

    for frame in 0..buffer.len() / channel_count {
        let beat = render.position_beats + frame as f64 * tempo_bpm / 60.0 / sample_rate;
        let sixteenth = (beat * 4.0).floor() as usize;
        let step_phase = (beat * 4.0).fract() as f32;
        let Some(source_step) = source_plan_voice(source_phrase_plan, render.mode, sixteenth)
        else {
            continue;
        };
        if !within_hook_response(render.hook_response, sixteenth) {
            continue;
        }
        if should_omit_source_step_for_phrase_variation(
            render.mode,
            source_phrase_plan,
            beat,
            source_step.origin_sixteenth,
        ) {
            continue;
        }
        let source_step_phase = source_step.distance as f32 + step_phase;
        let source_step_in_bar = source_step.origin_sixteenth % 16;
        let destructive_step =
            source_phrase_plan.destructive_mask & (1_u16 << source_step_in_bar) != 0;
        let semitone = source_step.semitone
            + contour_offset(render.contour_hint, source_step.origin_sixteenth)
            + hook_response_offset(render.hook_response, source_step.origin_sixteenth)
            + pressure_two_bar_turnaround(
                render.mode,
                source_phrase_plan,
                beat,
                source_step_in_bar,
            );

        let sound_design =
            mc202_source_phrase_sound_design(render, source_phrase_plan, destructive_step);
        let destructive_pitch_dive = if render.mode == Mc202RenderMode::Pressure {
            sound_design.destructive_dive
                * f64::from((source_step_phase / sound_design.gate_len).clamp(0.0, 1.0))
        } else {
            sound_design.destructive_dive * f64::from(source_step_phase)
        };
        let frequency = 110.0_f64
            * 2.0_f64
                .powf((semitone as f64 + sound_design.octave_drop + destructive_pitch_dive) / 12.0);
        if source_step_phase > sound_design.gate_len {
            continue;
        }

        let source_accent = source_phrase_plan.accent_mask & (1_u16 << source_step_in_bar) != 0;
        let accent = if source_accent {
            1.18 + touch * 0.45 + source_phrase_plan.pressure.clamp(0.0, 1.0) * 0.45
        } else if sixteenth.is_multiple_of(8) {
            1.0 + touch * 0.55
        } else if sixteenth % 4 == 2 {
            1.0 + touch * 0.25
        } else {
            0.82
        };
        let note_seconds = f64::from(source_step_phase) * 60.0 / tempo_bpm / 4.0;
        let phase = if render.mode == Mc202RenderMode::Pressure && !destructive_step {
            pressure_pitch_punch_phase(
                frequency,
                note_seconds,
                tempo_bpm,
                source_phrase_plan.pressure,
            )
        } else {
            note_seconds * frequency
        };
        let phase_increment = (frequency / sample_rate).clamp(0.0, 0.5) as f32;
        let sample = mc202_source_phrase_sample(
            phase,
            phase_increment,
            source_step_phase,
            accent,
            sound_design,
        );

        for channel in 0..channel_count {
            buffer[frame * channel_count + channel] += sample;
        }
    }
}

fn pressure_pitch_punch_phase(
    target_frequency: f64,
    note_seconds: f64,
    tempo_bpm: f64,
    pressure: f32,
) -> f64 {
    let punch_semitones = 2.0 + f64::from(pressure.clamp(0.0, 1.0)) * 2.0;
    let start_frequency = target_frequency * 2.0_f64.powf(punch_semitones / 12.0);
    let sixteenth_seconds = 60.0 / tempo_bpm.max(1.0) / 4.0;
    let punch_seconds = (sixteenth_seconds * 0.48).clamp(0.035, 0.075);
    if note_seconds <= punch_seconds {
        return start_frequency * note_seconds
            + 0.5 * (target_frequency - start_frequency) * note_seconds.powi(2) / punch_seconds;
    }

    0.5 * (start_frequency + target_frequency) * punch_seconds
        + target_frequency * (note_seconds - punch_seconds)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct SourcePlanVoice {
    semitone: i8,
    origin_sixteenth: usize,
    distance: usize,
}

fn source_plan_voice(
    plan: Mc202SourcePhraseRenderPlan,
    mode: Mc202RenderMode,
    sixteenth: usize,
) -> Option<SourcePlanVoice> {
    let max_hold_steps = if mode == Mc202RenderMode::Pressure {
        2
    } else {
        0
    };
    for distance in 0..=max_hold_steps.min(sixteenth) {
        let origin_sixteenth = sixteenth - distance;
        let step = origin_sixteenth % 16;
        if plan.active_mask & (1_u16 << step) != 0 {
            return Some(SourcePlanVoice {
                semitone: plan.semitones[step],
                origin_sixteenth,
                distance,
            });
        }
    }

    None
}

fn pressure_two_bar_turnaround(
    mode: Mc202RenderMode,
    plan: Mc202SourcePhraseRenderPlan,
    beat: f64,
    source_step_in_bar: usize,
) -> i8 {
    let bar_index = (beat / 4.0).floor() as u64;
    if mode != Mc202RenderMode::Pressure || bar_index.is_multiple_of(2) || source_step_in_bar < 8 {
        return 0;
    }

    (plan.contrast.clamp(0.0, 1.0) * 12.0)
        .round()
        .clamp(2.0, 7.0) as i8
}

fn should_omit_source_step_for_phrase_variation(
    mode: Mc202RenderMode,
    plan: Mc202SourcePhraseRenderPlan,
    beat: f64,
    sixteenth: usize,
) -> bool {
    if !matches!(
        mode,
        Mc202RenderMode::Answer | Mc202RenderMode::Pressure | Mc202RenderMode::Instigator
    ) {
        return false;
    }

    let bar_index = (beat / 4.0).floor() as u64;
    let source_marks_step_as_destructive = plan.destructive_mask & (1_u16 << (sixteenth % 16)) != 0;
    bar_index % 2 == 1 && source_marks_step_as_destructive
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

fn within_hook_response(response: Mc202HookResponse, _sixteenth: usize) -> bool {
    match response {
        Mc202HookResponse::Direct => true,
    }
}

fn hook_response_offset(response: Mc202HookResponse, _sixteenth: usize) -> i8 {
    match response {
        Mc202HookResponse::Direct => 0,
    }
}
