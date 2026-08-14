use crate::tr909::{Tr909RenderMode, Tr909RenderRouting};

use super::{
    shared_transport_tr909::RealtimeTr909RenderState,
    tr909_fill_recipe::{
        BASE_FILL_FOCUS, Tr909FillFocusProfile, fill_bar_aligned_position_beats, fill_focus_profile,
    },
};

/// Callback-local view of the deterministic arrangement articulation owned by a TR-909 fill.
///
/// This is derived from existing typed render state. It is deliberately not session state:
/// identical mode, transport, and output-frame positions always produce identical gains.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct FillFocusRenderState {
    mode: Tr909RenderMode,
    routing: Tr909RenderRouting,
    profile: Tr909FillFocusProfile,
    drum_bus_level: f32,
    is_transport_running: bool,
    tempo_bpm: f32,
    position_beats: f64,
}

impl FillFocusRenderState {
    pub(super) fn from_tr909(render: &RealtimeTr909RenderState) -> Self {
        Self {
            mode: render.mode,
            routing: render.routing,
            profile: fill_focus_profile(render),
            drum_bus_level: render.drum_bus_level,
            is_transport_running: render.is_transport_running,
            tempo_bpm: render.tempo_bpm,
            position_beats: fill_bar_aligned_position_beats(render, render.position_beats),
        }
    }

    pub(super) const fn inactive() -> Self {
        Self {
            mode: Tr909RenderMode::Idle,
            routing: Tr909RenderRouting::SourceOnly,
            profile: BASE_FILL_FOCUS,
            drum_bus_level: 0.0,
            is_transport_running: false,
            tempo_bpm: 0.0,
            position_beats: 0.0,
        }
    }

    pub(super) fn is_active(self) -> bool {
        matches!(self.mode, Tr909RenderMode::Fill)
            && matches!(self.routing, Tr909RenderRouting::DrumBusSupport)
            && self.drum_bus_level.is_finite()
            && self.drum_bus_level > 0.0
            && self.is_transport_running
            && self.tempo_bpm.is_finite()
            && self.tempo_bpm > 0.0
            && self.position_beats.is_finite()
    }

    pub(super) fn gain_at_frame(self, sample_rate: u32, frame_index: usize) -> f32 {
        if !self.is_active() || sample_rate == 0 {
            return 1.0;
        }

        let beats_per_frame = f64::from(self.tempo_bpm) / 60.0 / f64::from(sample_rate);
        // Quantize the callback start to the transport's sample grid before advancing by an
        // integer frame count. This makes the same transport span bit-identical whether it is
        // rendered as one offline block or partitioned into device callback blocks.
        let start_frame = (self.position_beats / beats_per_frame).round();
        let position = (start_frame + frame_index as f64) * beats_per_frame;
        let nearest_beat = position.round();
        let position = if (position - nearest_beat).abs() < 1.0e-9 {
            nearest_beat
        } else {
            position
        };
        let beat_in_bar = position.rem_euclid(self.profile.bar_beats);
        if beat_in_bar < self.profile.start_beat {
            return 1.0;
        }

        let base_gain = if beat_in_bar < self.profile.start_beat + self.profile.attack_beats {
            let phase =
                ((beat_in_bar - self.profile.start_beat) / self.profile.attack_beats) as f32;
            lerp(1.0, self.profile.min_gain, smoothstep(phase))
        } else if beat_in_bar < self.profile.release_start_beat {
            self.profile.min_gain
        } else {
            let phase = ((beat_in_bar - self.profile.release_start_beat)
                / (self.profile.bar_beats - self.profile.release_start_beat))
                as f32;
            lerp(self.profile.min_gain, 1.0, smoothstep(phase))
        };

        let Some(hole) = self.profile.signature_hole else {
            return base_gain;
        };
        if beat_in_bar < hole.start_beat {
            return base_gain;
        }
        if beat_in_bar < hole.start_beat + hole.attack_beats {
            let phase = ((beat_in_bar - hole.start_beat) / hole.attack_beats) as f32;
            return lerp(base_gain, hole.min_gain, smoothstep(phase));
        }
        if beat_in_bar < hole.stomp_beat {
            return hole.min_gain;
        }
        if beat_in_bar < hole.stomp_beat + hole.release_beats {
            let phase = ((beat_in_bar - hole.stomp_beat) / hole.release_beats) as f32;
            return lerp(hole.min_gain, base_gain, smoothstep(phase));
        }
        base_gain
    }
}

pub(super) fn apply_fill_focus_to_non_tr909_bed(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    fill_focus: FillFocusRenderState,
) {
    if channel_count == 0 {
        return;
    }
    for (frame_index, frame) in data.chunks_exact_mut(channel_count).enumerate() {
        let gain = fill_focus.gain_at_frame(sample_rate, frame_index);
        for sample in frame {
            *sample *= gain;
        }
    }
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    start + (end - start) * amount
}
