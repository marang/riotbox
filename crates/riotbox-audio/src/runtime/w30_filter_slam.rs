use super::*;

#[derive(Copy, Clone, Debug, Default)]
struct BiquadDelayState {
    z1: f64,
    z2: f64,
}

#[derive(Debug)]
pub(super) struct W30FilterSlamCallbackState {
    channels: Vec<BiquadDelayState>,
    started_at_beat: Option<u64>,
}

impl W30FilterSlamCallbackState {
    pub(super) fn with_channel_count(channel_count: usize) -> Self {
        Self {
            channels: vec![BiquadDelayState::default(); channel_count.max(1)],
            started_at_beat: None,
        }
    }

    pub(super) fn reset(&mut self) {
        if self.started_at_beat.is_none() {
            return;
        }
        self.channels.fill(BiquadDelayState::default());
        self.started_at_beat = None;
    }

    pub(super) fn prepare(&mut self, started_at_beat: u64) {
        if self.started_at_beat != Some(started_at_beat) {
            self.reset();
            self.started_at_beat = Some(started_at_beat);
        }
    }
}

impl Default for W30FilterSlamCallbackState {
    fn default() -> Self {
        Self::with_channel_count(2)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct W30FilterSlamFrame {
    pub(super) cutoff_hz: f64,
    pub(super) q: f64,
    pub(super) wet_gain: f64,
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
}

pub(super) fn w30_filter_slam_frame(
    render: &RealtimeW30PreviewRenderState,
    position_beats: f64,
    sample_rate: u32,
) -> Option<W30FilterSlamFrame> {
    if render.pad_playback.hook_articulation_profile
        != Some(W30HookArticulationProfile::FilterSlamV1)
        || sample_rate == 0
        || !render.tempo_bpm.is_finite()
        || render.tempo_bpm <= 0.0
    {
        return None;
    }

    let relative_beat =
        position_beats - render.pad_playback.hook_articulation_started_at_beat as f64;
    if !relative_beat.is_finite() {
        return None;
    }
    let nearest_beat = relative_beat.round();
    let relative_beat = if (relative_beat - nearest_beat).abs() <= 1.0e-9 {
        nearest_beat
    } else {
        relative_beat
    };
    if !(0.0..8.0).contains(&relative_beat) {
        return None;
    }

    let (cutoff_hz, q, wet_gain) = if relative_beat < 4.0 {
        let shaped = smoothstep(relative_beat / 4.0);
        (
            exponential_interpolation(14_000.0, 1_800.0, shaped),
            linear_interpolation(0.707, 0.85, shaped),
            1.0,
        )
    } else if relative_beat < 6.0 {
        let shaped = smoothstep((relative_beat - 4.0) / 2.0);
        (
            exponential_interpolation(1_800.0, 280.0, shaped),
            linear_interpolation(0.85, 1.2, shaped),
            1.0,
        )
    } else if relative_beat < 7.0 {
        (280.0, 1.2, 1.0)
    } else {
        let elapsed_seconds = (relative_beat - 7.0) * 60.0 / f64::from(render.tempo_bpm);
        let return_seconds = 0.02;
        let half_sample_seconds = 0.5 / f64::from(sample_rate);
        if elapsed_seconds >= return_seconds - half_sample_seconds {
            return None;
        }
        let wet_gain = (1.0 - elapsed_seconds / return_seconds).clamp(0.0, 1.0);
        (280.0, 1.2, wet_gain)
    };

    let omega = std::f64::consts::TAU * cutoff_hz / f64::from(sample_rate);
    let cosine = omega.cos();
    let alpha = omega.sin() / (2.0 * q);
    let a0 = 1.0 + alpha;
    Some(W30FilterSlamFrame {
        cutoff_hz,
        q,
        wet_gain,
        b0: ((1.0 - cosine) * 0.5) / a0,
        b1: (1.0 - cosine) / a0,
        b2: ((1.0 - cosine) * 0.5) / a0,
        a1: (-2.0 * cosine) / a0,
        a2: (1.0 - alpha) / a0,
    })
}

pub(super) fn w30_filter_slam_sample(
    control_sample: f32,
    channel: usize,
    frame: W30FilterSlamFrame,
    state: &mut W30FilterSlamCallbackState,
) -> f32 {
    let Some(delay) = state.channels.get_mut(channel) else {
        return 0.0;
    };
    let input = f64::from(control_sample);
    let filtered = frame.b0 * input + delay.z1;
    delay.z1 = frame.b1 * input - frame.a1 * filtered + delay.z2;
    delay.z2 = frame.b2 * input - frame.a2 * filtered;
    (filtered * frame.wet_gain + input * (1.0 - frame.wet_gain)) as f32
}

fn smoothstep(progress: f64) -> f64 {
    let progress = progress.clamp(0.0, 1.0);
    progress * progress * (3.0 - 2.0 * progress)
}

fn exponential_interpolation(start: f64, end: f64, progress: f64) -> f64 {
    start * (end / start).powf(progress)
}

fn linear_interpolation(start: f64, end: f64, progress: f64) -> f64 {
    start + (end - start) * progress
}
