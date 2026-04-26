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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mc202RenderState {
    pub mode: Mc202RenderMode,
    pub routing: Mc202RenderRouting,
    pub phrase_shape: Mc202PhraseShape,
    pub note_budget: Mc202NoteBudget,
    pub contour_hint: Mc202ContourHint,
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
        let semitone = semitone + contour_offset(render.contour_hint, sixteenth);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(buffer: &[f32]) -> (usize, f32, f32) {
        let active = buffer.iter().filter(|sample| sample.abs() > 0.0001).count();
        let peak = buffer
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        let rms =
            (buffer.iter().map(|sample| sample * sample).sum::<f32>() / buffer.len() as f32).sqrt();
        (active, peak, rms)
    }

    #[test]
    fn follower_and_answer_shapes_are_audible_and_distinct() {
        let mut follower = vec![0.0; 44_100 * 2];
        let mut answer = vec![0.0; 44_100 * 2];

        render_mc202_buffer(
            &mut follower,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Follower,
                routing: Mc202RenderRouting::MusicBusBass,
                phrase_shape: Mc202PhraseShape::FollowerDrive,
                touch: 0.62,
                is_transport_running: true,
                ..Mc202RenderState::default()
            },
        );
        render_mc202_buffer(
            &mut answer,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Answer,
                routing: Mc202RenderRouting::MusicBusBass,
                phrase_shape: Mc202PhraseShape::AnswerHook,
                touch: 0.78,
                is_transport_running: true,
                ..Mc202RenderState::default()
            },
        );

        let follower_metrics = metrics(&follower);
        let answer_metrics = metrics(&answer);

        assert!(follower_metrics.0 > 10_000);
        assert!(answer_metrics.0 > 10_000);
        assert!((follower_metrics.2 - answer_metrics.2).abs() > 0.001);
    }

    #[test]
    fn touch_changes_render_energy_on_same_phrase() {
        let mut low_touch = vec![0.0; 44_100 * 2];
        let mut high_touch = vec![0.0; 44_100 * 2];
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(
            &mut low_touch,
            44_100,
            2,
            &Mc202RenderState {
                touch: 0.08,
                ..base
            },
        );
        render_mc202_buffer(
            &mut high_touch,
            44_100,
            2,
            &Mc202RenderState {
                touch: 0.92,
                ..base
            },
        );

        let low_metrics = metrics(&low_touch);
        let high_metrics = metrics(&high_touch);
        let max_delta = low_touch
            .iter()
            .zip(high_touch.iter())
            .map(|(low, high)| (low - high).abs())
            .fold(0.0_f32, f32::max);

        assert!(low_metrics.0 > 10_000);
        assert!(high_metrics.0 > 10_000);
        assert!(
            high_metrics.2 > low_metrics.2 + 0.006,
            "low RMS {:.6}, high RMS {:.6}",
            low_metrics.2,
            high_metrics.2
        );
        assert!(max_delta > 0.02, "max touch delta {max_delta}");
    }

    #[test]
    fn mutated_phrase_differs_from_follower_drive() {
        let mut follower = vec![0.0; 44_100 * 2];
        let mut mutated = vec![0.0; 44_100 * 2];
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(
            &mut follower,
            44_100,
            2,
            &Mc202RenderState {
                phrase_shape: Mc202PhraseShape::FollowerDrive,
                ..base
            },
        );
        render_mc202_buffer(
            &mut mutated,
            44_100,
            2,
            &Mc202RenderState {
                phrase_shape: Mc202PhraseShape::MutatedDrive,
                ..base
            },
        );

        let follower_metrics = metrics(&follower);
        let mutated_metrics = metrics(&mutated);
        let delta_rms = (follower
            .iter()
            .zip(mutated.iter())
            .map(|(follower, mutated)| (follower - mutated).powi(2))
            .sum::<f32>()
            / follower.len() as f32)
            .sqrt();
        let max_delta = follower
            .iter()
            .zip(mutated.iter())
            .map(|(follower, mutated)| (follower - mutated).abs())
            .fold(0.0_f32, f32::max);

        assert!(follower_metrics.0 > 10_000);
        assert!(mutated_metrics.0 > 10_000);
        assert!(delta_rms > 0.005, "mutated phrase delta RMS {delta_rms}");
        assert!(max_delta > 0.02, "mutated phrase max delta {max_delta}");
    }

    #[test]
    fn pressure_cell_differs_from_follower_drive() {
        let mut follower = vec![0.0; 44_100 * 2];
        let mut pressure = vec![0.0; 44_100 * 2];
        let base = Mc202RenderState {
            routing: Mc202RenderRouting::MusicBusBass,
            touch: 0.84,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(
            &mut follower,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Follower,
                phrase_shape: Mc202PhraseShape::FollowerDrive,
                ..base
            },
        );
        render_mc202_buffer(
            &mut pressure,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Pressure,
                phrase_shape: Mc202PhraseShape::PressureCell,
                ..base
            },
        );

        let follower_metrics = metrics(&follower);
        let pressure_metrics = metrics(&pressure);
        let delta_rms = (follower
            .iter()
            .zip(pressure.iter())
            .map(|(follower, pressure)| (follower - pressure).powi(2))
            .sum::<f32>()
            / follower.len() as f32)
            .sqrt();
        let max_delta = follower
            .iter()
            .zip(pressure.iter())
            .map(|(follower, pressure)| (follower - pressure).abs())
            .fold(0.0_f32, f32::max);

        assert!(follower_metrics.0 > 10_000);
        assert!(pressure_metrics.0 > 10_000);
        assert!(delta_rms > 0.004, "pressure phrase delta RMS {delta_rms}");
        assert!(max_delta > 0.02, "pressure phrase max delta {max_delta}");
    }

    #[test]
    fn note_budget_reduces_density_without_silencing_phrase() {
        let mut wide = vec![0.0; 44_100 * 2 * 2];
        let mut balanced = vec![0.0; 44_100 * 2 * 2];
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(
            &mut wide,
            44_100,
            2,
            &Mc202RenderState {
                note_budget: Mc202NoteBudget::Wide,
                ..base
            },
        );
        render_mc202_buffer(
            &mut balanced,
            44_100,
            2,
            &Mc202RenderState {
                note_budget: Mc202NoteBudget::Balanced,
                ..base
            },
        );

        let wide_metrics = metrics(&wide);
        let balanced_metrics = metrics(&balanced);
        let delta_rms = (wide
            .iter()
            .zip(balanced.iter())
            .map(|(wide, balanced)| (wide - balanced).powi(2))
            .sum::<f32>()
            / wide.len() as f32)
            .sqrt();

        assert!(wide_metrics.0 > 10_000);
        assert!(balanced_metrics.0 > 10_000);
        assert!(
            balanced_metrics.2 < wide_metrics.2,
            "balanced RMS {} should stay below wide RMS {}",
            balanced_metrics.2,
            wide_metrics.2
        );
        assert!(delta_rms > 0.001, "note-budget delta RMS {delta_rms}");
    }

    #[test]
    fn contour_hint_changes_phrase_without_silencing_it() {
        let mut neutral = vec![0.0; 44_100 * 2];
        let mut lift = vec![0.0; 44_100 * 2];
        let base = Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            note_budget: Mc202NoteBudget::Balanced,
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(&mut neutral, 44_100, 2, &base);
        render_mc202_buffer(
            &mut lift,
            44_100,
            2,
            &Mc202RenderState {
                contour_hint: Mc202ContourHint::Lift,
                ..base
            },
        );

        let neutral_metrics = metrics(&neutral);
        let lift_metrics = metrics(&lift);
        let delta_rms = (neutral
            .iter()
            .zip(lift.iter())
            .map(|(neutral, lift)| (neutral - lift).powi(2))
            .sum::<f32>()
            / neutral.len() as f32)
            .sqrt();

        assert!(neutral_metrics.0 > 10_000);
        assert!(lift_metrics.0 > 10_000);
        assert!(delta_rms > 0.004, "contour hint delta RMS {delta_rms}");
    }

    #[test]
    fn instigator_spike_differs_from_follower_drive() {
        let mut follower = vec![0.0; 44_100 * 2];
        let mut instigator = vec![0.0; 44_100 * 2];
        let base = Mc202RenderState {
            routing: Mc202RenderRouting::MusicBusBass,
            touch: 0.90,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };

        render_mc202_buffer(
            &mut follower,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Follower,
                phrase_shape: Mc202PhraseShape::FollowerDrive,
                ..base
            },
        );
        render_mc202_buffer(
            &mut instigator,
            44_100,
            2,
            &Mc202RenderState {
                mode: Mc202RenderMode::Instigator,
                phrase_shape: Mc202PhraseShape::InstigatorSpike,
                ..base
            },
        );

        let follower_metrics = metrics(&follower);
        let instigator_metrics = metrics(&instigator);
        let delta_rms = (follower
            .iter()
            .zip(instigator.iter())
            .map(|(follower, instigator)| (follower - instigator).powi(2))
            .sum::<f32>()
            / follower.len() as f32)
            .sqrt();
        let max_delta = follower
            .iter()
            .zip(instigator.iter())
            .map(|(follower, instigator)| (follower - instigator).abs())
            .fold(0.0_f32, f32::max);

        assert!(follower_metrics.0 > 10_000);
        assert!(instigator_metrics.0 > 8_000);
        assert!(delta_rms > 0.010, "instigator phrase delta RMS {delta_rms}");
        assert!(max_delta > 0.04, "instigator phrase max delta {max_delta}");
    }

    #[test]
    fn render_is_stable_across_callback_chunk_boundaries() {
        let render = Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        };
        let mut whole = vec![0.0; 44_100 * 2];
        let mut chunked = vec![0.0; 44_100 * 2];
        let split_frames = 2_048;
        let split_samples = split_frames * 2;

        render_mc202_buffer(&mut whole, 44_100, 2, &render);
        render_mc202_buffer(&mut chunked[..split_samples], 44_100, 2, &render);

        let mut second_render = render;
        second_render.position_beats +=
            split_frames as f64 * f64::from(render.tempo_bpm) / 60.0 / 44_100.0;
        render_mc202_buffer(&mut chunked[split_samples..], 44_100, 2, &second_render);

        let max_delta = whole
            .iter()
            .zip(chunked.iter())
            .map(|(whole, chunked)| (whole - chunked).abs())
            .fold(0.0_f32, f32::max);
        assert!(max_delta < 0.0001, "max chunk boundary delta {max_delta}");
    }
}
