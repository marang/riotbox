#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mc202RenderMode {
    Idle,
    Leader,
    Follower,
    Answer,
}

impl Mc202RenderMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Leader => "leader",
            Self::Follower => "follower",
            Self::Answer => "answer",
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
}

impl Mc202PhraseShape {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::RootPulse => "root_pulse",
            Self::FollowerDrive => "follower_drive",
            Self::AnswerHook => "answer_hook",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mc202RenderState {
    pub mode: Mc202RenderMode,
    pub routing: Mc202RenderRouting,
    pub phrase_shape: Mc202PhraseShape,
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

        let octave_drop = if matches!(render.mode, Mc202RenderMode::Follower) {
            -12.0
        } else {
            -5.0
        };
        let frequency = 110.0_f64 * 2.0_f64.powf((semitone as f64 + octave_drop) / 12.0);

        let gate_len = if matches!(render.phrase_shape, Mc202PhraseShape::AnswerHook) {
            0.42
        } else {
            0.62
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
    let pattern = match shape {
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
    };
    pattern[sixteenth % pattern.len()]
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
