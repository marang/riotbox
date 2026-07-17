use super::{
    shared_transport_tr909::RealtimeTr909RenderState,
    shared_w30_resample_callback::Tr909CallbackState,
    tr909_fill_recipe::{
        Tr909FillStep, Tr909FillVoiceTrigger, fill_bar_aligned_position_beats, fill_focus_profile,
        prepared_fill_step,
    },
    w30_tr909_signal_helpers::{
        fill_performance_slam, render_gain, render_subdivision, tr909_deterministic_noise,
        trigger_envelope,
    },
};

const VOICE_SILENCE_THRESHOLD: f32 = 0.000_01;

#[derive(Clone, Copy, Debug)]
struct FillVoiceRetriggerProfile {
    kick_retained_tail: f32,
    snare_body_retained_tail: f32,
    snare_noise_retained_tail: f32,
    hat_retained_tail: f32,
    envelope_ceiling: f32,
}

const FILL_VOICE_RETRIGGER_V1: FillVoiceRetriggerProfile = FillVoiceRetriggerProfile {
    kick_retained_tail: 0.16,
    snare_body_retained_tail: 0.12,
    snare_noise_retained_tail: 0.06,
    hat_retained_tail: 0.08,
    envelope_ceiling: 0.96,
};

#[derive(Clone, Copy, Debug)]
struct FillKickProfile {
    floor_hz: f32,
    pitch_span_hz: f32,
    pressure_pitch_span_hz: f32,
    pitch_decay_per_second: f32,
    envelope_decay_per_second: f32,
    live_slam_decay_relief_per_second: f32,
    fundamental_gain: f32,
    body_harmonic_ratio: f32,
    body_harmonic_gain: f32,
    attack_harmonic_ratio: f32,
    attack_gain: f32,
}

const STANDARD_FILL_KICK_V1: FillKickProfile = FillKickProfile {
    floor_hz: 50.0,
    pitch_span_hz: 82.0,
    pressure_pitch_span_hz: 18.0,
    pitch_decay_per_second: 34.0,
    envelope_decay_per_second: 4.8,
    live_slam_decay_relief_per_second: 1.2,
    fundamental_gain: 0.82,
    body_harmonic_ratio: 2.0,
    body_harmonic_gain: 0.30,
    attack_harmonic_ratio: 5.0,
    attack_gain: 0.13,
};

const DIVE_STOMP_KICK_V1: FillKickProfile = FillKickProfile {
    floor_hz: 50.0,
    pitch_span_hz: 150.0,
    pressure_pitch_span_hz: 0.0,
    pitch_decay_per_second: 70.0,
    envelope_decay_per_second: 3.8,
    live_slam_decay_relief_per_second: 0.8,
    fundamental_gain: 0.82,
    body_harmonic_ratio: 2.0,
    body_harmonic_gain: 0.30,
    attack_harmonic_ratio: 5.0,
    attack_gain: 0.13,
};

#[derive(Clone, Copy, Debug)]
struct FillSnareProfile {
    body_hz: f32,
    pressure_body_hz: f32,
    primary_body_gain: f32,
    overtone_body_gain: f32,
    body_output_gain: f32,
    standard_noise_gain: f32,
    signature_noise_gain: f32,
    overtone_ratio: f32,
    noise_phase_increment: f32,
    body_decay_per_second: f32,
    noise_decay_per_second: f32,
}

const FILL_SNARE_V1: FillSnareProfile = FillSnareProfile {
    body_hz: 184.0,
    pressure_body_hz: 16.0,
    primary_body_gain: 0.68,
    overtone_body_gain: 0.32,
    body_output_gain: 0.64,
    standard_noise_gain: 1.20,
    signature_noise_gain: 1.50,
    overtone_ratio: 1.76,
    noise_phase_increment: 0.754_877_7,
    body_decay_per_second: 9.2,
    noise_decay_per_second: 31.0,
};

#[derive(Clone, Copy, Debug)]
struct FillHatProfile {
    partial_a_gain: f32,
    partial_b_gain: f32,
    partial_c_gain: f32,
    output_gain: f32,
    base_hz: f32,
    pressure_hz: f32,
    partial_b_ratio: f32,
    partial_c_ratio: f32,
    decay_per_second: f32,
}

const FILL_HAT_V1: FillHatProfile = FillHatProfile {
    partial_a_gain: 0.52,
    partial_b_gain: 0.28,
    partial_c_gain: 0.20,
    output_gain: 1.70,
    base_hz: 5_900.0,
    pressure_hz: 720.0,
    partial_b_ratio: 1.447,
    partial_c_ratio: 1.793,
    decay_per_second: 38.0,
};

#[derive(Clone, Copy, Debug)]
struct DiveStompGestureProfile {
    choke_seconds: f32,
    flam_delay_seconds: f32,
    flam_level_ratio: f32,
    flam_level_ceiling: f32,
    flam_body_level_ratio: f32,
}

const DIVE_STOMP_GESTURE_V1: DiveStompGestureProfile = DiveStompGestureProfile {
    choke_seconds: 0.006,
    flam_delay_seconds: 0.011,
    flam_level_ratio: 0.52,
    flam_level_ceiling: 0.80,
    flam_body_level_ratio: 0.72,
};

/// Fixed callback-local voices for the playable TR-909 fill.
///
/// Each drum family owns its oscillator and envelope, so a hat or snare retrigger no longer
/// truncates the kick body that is already in flight. The state is fixed-size and performs no
/// allocation, locking, or I/O in the audio callback.
#[derive(Debug, Default)]
pub(super) struct Tr909FillVoiceState {
    kick: FillKickVoice,
    snare: FillSnareVoice,
    hat: FillHatVoice,
    choke_gain: f32,
    choke_active: bool,
    active: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Tr909FillVoiceSample {
    pub(super) kick: f32,
    pub(super) snare: f32,
    pub(super) hat: f32,
}

impl Tr909FillVoiceSample {
    fn sum(self) -> f32 {
        self.kick + self.snare + self.hat
    }
}

#[derive(Debug, Default)]
struct FillKickVoice {
    phase: f32,
    envelope: f32,
    pitch_envelope: f32,
    dive_stomp: bool,
}

#[derive(Debug, Default)]
struct FillSnareVoice {
    body_phase: f32,
    overtone_phase: f32,
    noise_phase: f32,
    body_envelope: f32,
    noise_envelope: f32,
    noise_seed: i64,
    flam_delay_seconds: f32,
    flam_level: f32,
    flam_seed: i64,
    signature_crack: bool,
}

#[derive(Debug, Default)]
struct FillHatVoice {
    phase_a: f32,
    phase_b: f32,
    phase_c: f32,
    envelope: f32,
}

impl Tr909FillVoiceState {
    pub(super) fn start(&mut self) {
        self.kick = FillKickVoice::default();
        self.snare = FillSnareVoice::default();
        self.hat = FillHatVoice::default();
        self.choke_gain = 1.0;
        self.choke_active = false;
        self.active = true;
    }

    pub(super) fn deactivate(&mut self) {
        self.start();
        self.active = false;
    }

    pub(super) fn is_active(&self) -> bool {
        self.active
    }

    pub(super) fn trigger(&mut self, trigger: Tr909FillVoiceTrigger, envelope: f32, step: i64) {
        self.choke_gain = 1.0;
        self.choke_active = false;
        self.snare.flam_delay_seconds = 0.0;
        self.snare.flam_level = 0.0;
        if trigger.kick > 0.0 {
            self.kick.phase = 0.0;
            self.kick.pitch_envelope = 1.0;
            self.kick.dive_stomp = false;
            self.kick.envelope = retriggered_envelope(
                self.kick.envelope,
                envelope * trigger.kick,
                FILL_VOICE_RETRIGGER_V1.kick_retained_tail,
            );
        }
        if trigger.snare > 0.0 {
            self.snare.signature_crack = false;
            self.trigger_snare_now(envelope * trigger.snare, step);
        }
        if trigger.hat > 0.0 {
            let seed_phase = (step.rem_euclid(17) as f32 * 0.017).fract();
            self.hat.phase_a = seed_phase;
            self.hat.phase_b = (seed_phase * 1.37).fract();
            self.hat.phase_c = (seed_phase * 1.79).fract();
            self.hat.envelope = retriggered_envelope(
                self.hat.envelope,
                envelope * trigger.hat,
                FILL_VOICE_RETRIGGER_V1.hat_retained_tail,
            );
        }
    }

    pub(super) fn trigger_dive_stomp(
        &mut self,
        trigger: Tr909FillVoiceTrigger,
        envelope: f32,
        step: i64,
    ) {
        self.trigger(trigger, envelope, step);
        self.kick.dive_stomp = trigger.kick > 0.0;
        if trigger.snare > 0.0 {
            self.snare.signature_crack = true;
            self.snare.flam_delay_seconds = DIVE_STOMP_GESTURE_V1.flam_delay_seconds;
            self.snare.flam_level =
                (envelope * trigger.snare * DIVE_STOMP_GESTURE_V1.flam_level_ratio)
                    .clamp(0.0, DIVE_STOMP_GESTURE_V1.flam_level_ceiling);
            self.snare.flam_seed = step.saturating_add(97);
        }
    }

    pub(super) fn choke(&mut self) {
        self.choke_gain = 1.0;
        self.choke_active = true;
        self.snare.flam_delay_seconds = 0.0;
        self.snare.flam_level = 0.0;
    }

    fn trigger_snare_now(&mut self, level: f32, step: i64) {
        self.snare.body_phase = 0.0;
        self.snare.overtone_phase = 0.0;
        self.snare.noise_phase = (step.rem_euclid(29) as f32 * 0.031_25).fract();
        self.snare.noise_seed = step;
        self.snare.body_envelope = retriggered_envelope(
            self.snare.body_envelope,
            level,
            FILL_VOICE_RETRIGGER_V1.snare_body_retained_tail,
        );
        self.snare.noise_envelope = retriggered_envelope(
            self.snare.noise_envelope,
            level,
            FILL_VOICE_RETRIGGER_V1.snare_noise_retained_tail,
        );
    }

    fn trigger_snare_flam_now(&mut self, level: f32, step: i64) {
        self.snare.body_phase = 0.0;
        self.snare.overtone_phase = 0.0;
        self.snare.noise_phase = (step.rem_euclid(29) as f32 * 0.031_25).fract();
        self.snare.noise_seed = step;
        self.snare.body_envelope = self
            .snare
            .body_envelope
            .max(level * DIVE_STOMP_GESTURE_V1.flam_body_level_ratio);
        self.snare.noise_envelope = (self.snare.noise_envelope + level)
            .clamp(0.0, FILL_VOICE_RETRIGGER_V1.envelope_ceiling);
    }

    pub(super) fn render_sample(
        &mut self,
        render: &RealtimeTr909RenderState,
        sample_rate: u32,
    ) -> Tr909FillVoiceSample {
        let sample_rate = sample_rate.max(1) as f32;
        let policy_pressure = render.slam_intensity.clamp(0.0, 1.0);
        let live_slam = fill_performance_slam(render);

        if self.snare.flam_delay_seconds > 0.0 {
            self.snare.flam_delay_seconds -= 1.0 / sample_rate;
            if self.snare.flam_delay_seconds <= 0.0 && self.snare.flam_level > 0.0 {
                let flam_level = self.snare.flam_level;
                let flam_seed = self.snare.flam_seed;
                self.snare.flam_level = 0.0;
                self.trigger_snare_flam_now(flam_level, flam_seed);
            }
        }

        let kick = if self.kick.envelope > VOICE_SILENCE_THRESHOLD {
            let profile = if self.kick.dive_stomp {
                DIVE_STOMP_KICK_V1
            } else {
                STANDARD_FILL_KICK_V1
            };
            let pitch_span_hz = if self.kick.dive_stomp {
                profile.pitch_span_hz
            } else {
                profile.pitch_span_hz + policy_pressure * profile.pressure_pitch_span_hz
            };
            let envelope_decay = profile.envelope_decay_per_second
                - live_slam * profile.live_slam_decay_relief_per_second;
            let frequency = profile.floor_hz + self.kick.pitch_envelope * pitch_span_hz;
            let phase = std::f32::consts::TAU * self.kick.phase;
            let fundamental = phase.sin();
            let body_harmonic =
                (phase * profile.body_harmonic_ratio).sin() * profile.body_harmonic_gain;
            let attack = (phase * profile.attack_harmonic_ratio).sin()
                * self.kick.pitch_envelope
                * profile.attack_gain;
            let sample = (fundamental * profile.fundamental_gain + body_harmonic + attack)
                * self.kick.envelope;
            self.kick.phase = (self.kick.phase + frequency / sample_rate).fract();
            self.kick.envelope *= (1.0 - envelope_decay / sample_rate).clamp(0.0, 1.0);
            self.kick.pitch_envelope *=
                (1.0 - profile.pitch_decay_per_second / sample_rate).clamp(0.0, 1.0);
            sample
        } else {
            self.kick.envelope = 0.0;
            0.0
        };

        let snare = if self.snare.body_envelope > VOICE_SILENCE_THRESHOLD
            || self.snare.noise_envelope > VOICE_SILENCE_THRESHOLD
        {
            let body_hz = FILL_SNARE_V1.body_hz + policy_pressure * FILL_SNARE_V1.pressure_body_hz;
            let body_phase = std::f32::consts::TAU * self.snare.body_phase;
            let overtone_phase = std::f32::consts::TAU * self.snare.overtone_phase;
            let body = body_phase.sin() * FILL_SNARE_V1.primary_body_gain
                + overtone_phase.sin() * FILL_SNARE_V1.overtone_body_gain;
            let noise = tr909_deterministic_noise(self.snare.noise_phase, self.snare.noise_seed);
            let noise_gain = if self.snare.signature_crack {
                FILL_SNARE_V1.signature_noise_gain
            } else {
                FILL_SNARE_V1.standard_noise_gain
            };
            let sample = body * FILL_SNARE_V1.body_output_gain * self.snare.body_envelope
                + noise * noise_gain * self.snare.noise_envelope;
            self.snare.body_phase = (self.snare.body_phase + body_hz / sample_rate).fract();
            self.snare.overtone_phase = (self.snare.overtone_phase
                + body_hz * FILL_SNARE_V1.overtone_ratio / sample_rate)
                .fract();
            self.snare.noise_phase =
                (self.snare.noise_phase + FILL_SNARE_V1.noise_phase_increment).fract();
            self.snare.body_envelope *=
                (1.0 - FILL_SNARE_V1.body_decay_per_second / sample_rate).clamp(0.0, 1.0);
            self.snare.noise_envelope *=
                (1.0 - FILL_SNARE_V1.noise_decay_per_second / sample_rate).clamp(0.0, 1.0);
            sample
        } else {
            self.snare.body_envelope = 0.0;
            self.snare.noise_envelope = 0.0;
            0.0
        };

        let hat = if self.hat.envelope > VOICE_SILENCE_THRESHOLD {
            let metal = (std::f32::consts::TAU * self.hat.phase_a).sin()
                * FILL_HAT_V1.partial_a_gain
                + (std::f32::consts::TAU * self.hat.phase_b).sin() * FILL_HAT_V1.partial_b_gain
                + (std::f32::consts::TAU * self.hat.phase_c).sin() * FILL_HAT_V1.partial_c_gain;
            let sample = metal * FILL_HAT_V1.output_gain * self.hat.envelope;
            let hat_hz = FILL_HAT_V1.base_hz + policy_pressure * FILL_HAT_V1.pressure_hz;
            self.hat.phase_a = (self.hat.phase_a + hat_hz / sample_rate).fract();
            self.hat.phase_b =
                (self.hat.phase_b + hat_hz * FILL_HAT_V1.partial_b_ratio / sample_rate).fract();
            self.hat.phase_c =
                (self.hat.phase_c + hat_hz * FILL_HAT_V1.partial_c_ratio / sample_rate).fract();
            self.hat.envelope *= (1.0 - FILL_HAT_V1.decay_per_second / sample_rate).clamp(0.0, 1.0);
            sample
        } else {
            self.hat.envelope = 0.0;
            0.0
        };

        let choke_gain = if self.choke_active {
            let gain = self.choke_gain;
            self.choke_gain = (self.choke_gain
                - 1.0 / (DIVE_STOMP_GESTURE_V1.choke_seconds * sample_rate).max(1.0))
            .max(0.0);
            if self.choke_gain == 0.0 {
                self.kick = FillKickVoice::default();
                self.snare = FillSnareVoice::default();
                self.hat = FillHatVoice::default();
                self.choke_active = false;
            }
            gain
        } else {
            1.0
        };

        Tr909FillVoiceSample {
            kick: kick * choke_gain,
            snare: snare * choke_gain,
            hat: hat * choke_gain,
        }
    }
}

fn retriggered_envelope(previous: f32, requested: f32, retained_tail: f32) -> f32 {
    (requested.max(0.0) + previous.max(0.0) * retained_tail)
        .clamp(0.0, FILL_VOICE_RETRIGGER_V1.envelope_ceiling)
}

fn fill_voice_tail_release_gain(render: &RealtimeTr909RenderState, position_beats: f64) -> f32 {
    let focus = fill_focus_profile(render);
    let beat_in_bar =
        fill_bar_aligned_position_beats(render, position_beats).rem_euclid(focus.bar_beats);
    if beat_in_bar < focus.release_start_beat {
        return 1.0;
    }
    let phase = ((beat_in_bar - focus.release_start_beat)
        / (focus.bar_beats - focus.release_start_beat)) as f32;
    let phase = phase.clamp(0.0, 1.0);
    let smooth = phase * phase * (3.0 - 2.0 * phase);
    1.0 - smooth
}

pub(super) fn render_tr909_fill_buffer(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeTr909RenderState,
    state: &mut Tr909CallbackState,
) {
    let subdivision = render_subdivision(render);
    let current_step = fill_transport_step(render, render.position_beats, subdivision);
    let step_beats = 1.0 / f64::from(subdivision.max(1));
    let transport_jump = (state.beat_position - render.position_beats).abs() + 1.0e-9 >= step_beats;
    if !state.was_running || !state.fill_voices.is_active() || transport_jump {
        state.beat_position = render.position_beats;
        state.last_step = current_step.saturating_sub(1);
        state.fill_voices.start();
        state.was_running = true;
    }

    let beats_per_sample = f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1));
    let frame_count = data.len() / channel_count.max(1);
    let gain = render_gain(render);

    for frame_index in 0..frame_count {
        let step = fill_transport_step(render, state.beat_position, subdivision);
        if step != state.last_step {
            let starts_new_bar = step.rem_euclid(i64::from(subdivision) * 4) == 0;
            state.last_step = step;
            if starts_new_bar {
                // The previous bar's tails reached zero through the bounded release below. Clear
                // their hidden envelopes before gain returns to one, then start the new downbeat.
                state.fill_voices.start();
            }
            match prepared_fill_step(render, subdivision, step, fill_performance_slam(render)) {
                Tr909FillStep::Hit(trigger) => {
                    state
                        .fill_voices
                        .trigger(trigger, trigger_envelope(render), step);
                }
                Tr909FillStep::Choke => state.fill_voices.choke(),
                Tr909FillStep::DiveStomp(trigger) => {
                    state
                        .fill_voices
                        .trigger_dive_stomp(trigger, trigger_envelope(render), step)
                }
                Tr909FillStep::Rest => {}
            }
        }

        let voices = state.fill_voices.render_sample(render, sample_rate);
        let sample =
            voices.sum() * gain * fill_voice_tail_release_gain(render, state.beat_position);
        let base = frame_index * channel_count;
        for channel in 0..channel_count {
            data[base + channel] += sample;
        }
        state.beat_position += beats_per_sample;
    }
}

fn fill_transport_step(
    render: &RealtimeTr909RenderState,
    position_beats: f64,
    subdivision: u32,
) -> i64 {
    let position = fill_bar_aligned_position_beats(render, position_beats) * f64::from(subdivision);
    let nearest_step = position.round();
    let position = if (position - nearest_step).abs() < 1.0e-9 {
        nearest_step
    } else {
        position
    };
    position.floor() as i64
}
