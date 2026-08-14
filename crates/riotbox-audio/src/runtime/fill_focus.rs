use crate::tr909::{Tr909RenderMode, Tr909RenderRouting, Tr909SourceSupportProfile};

use super::{
    shared_transport_tr909::RealtimeTr909RenderState,
    tr909_fill_recipe::{
        BASE_FILL_FOCUS, Tr909FillFocusProfile, fill_bar_aligned_position_beats, fill_focus_profile,
    },
    w30_tr909_signal_helpers::{render_subdivision, should_trigger_step},
};

const IMPACT_POCKET_BAR_BEATS: f64 = 4.0;
const IMPACT_POCKET_PRE_ROLL_BEATS: f64 = 0.0625;
const IMPACT_POCKET_HOLD_BEATS: f64 = 0.03125;
const IMPACT_POCKET_RELEASE_BEATS: f64 = 0.1875;
const IMPACT_POCKET_MIN_GAIN: f32 = 0.30;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Tr909ImpactOwner {
    KickDownbeat,
    SnareBackbeat,
}

/// Callback-local collision policy owned by the committed TR-909 Slam gesture.
///
/// It creates room around one named, actually rendered drum impact. It does not alter the drum
/// lane, persist separate state, or claim that an isolated source event became harder.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct ImpactPocketRenderState {
    render: RealtimeTr909RenderState,
    owner: Option<Tr909ImpactOwner>,
}

impl ImpactPocketRenderState {
    pub(super) fn from_tr909(render: &RealtimeTr909RenderState) -> Self {
        let owner = match render.source_support_profile {
            Some(Tr909SourceSupportProfile::BreakLift) => Some(Tr909ImpactOwner::SnareBackbeat),
            Some(Tr909SourceSupportProfile::DropDrive | Tr909SourceSupportProfile::SteadyPulse) => {
                Some(Tr909ImpactOwner::KickDownbeat)
            }
            None => None,
        };
        Self {
            render: *render,
            owner,
        }
    }

    pub(super) fn is_active(self) -> bool {
        matches!(
            self.render.mode,
            Tr909RenderMode::SourceSupport | Tr909RenderMode::BreakReinforce
        ) && matches!(self.render.routing, Tr909RenderRouting::DrumBusSupport)
            && self.render.slam_enabled
            && self.render.drum_bus_level.is_finite()
            && self.render.drum_bus_level > 0.0
            && self.render.is_transport_running
            && self.render.tempo_bpm.is_finite()
            && self.render.tempo_bpm > 0.0
            && self.render.position_beats.is_finite()
            && self.owner.is_some()
            && self.owner_step_sounds()
    }

    #[cfg(test)]
    fn owner(self) -> Option<Tr909ImpactOwner> {
        if self.is_active() { self.owner } else { None }
    }

    pub(super) fn gain_at_frame(self, sample_rate: u32, frame_index: usize) -> f32 {
        if !self.is_active() || sample_rate == 0 {
            return 1.0;
        }
        let beats_per_frame = f64::from(self.render.tempo_bpm) / 60.0 / f64::from(sample_rate);
        let start_frame = (self.render.position_beats / beats_per_frame).round();
        let position = (start_frame + frame_index as f64) * beats_per_frame;
        self.gain_at_position(position)
    }

    fn owner_step_sounds(self) -> bool {
        let subdivision = i64::from(render_subdivision(&self.render)).max(1);
        let owner_step = match self.owner {
            Some(Tr909ImpactOwner::KickDownbeat) => 0,
            Some(Tr909ImpactOwner::SnareBackbeat) => subdivision * 2,
            None => return false,
        };
        should_trigger_step(&self.render, owner_step)
    }

    fn gain_at_position(self, position_beats: f64) -> f32 {
        let owner_beat = match self.owner {
            Some(Tr909ImpactOwner::KickDownbeat) => 0.0,
            Some(Tr909ImpactOwner::SnareBackbeat) => 2.0,
            None => return 1.0,
        };
        let mut relative = (position_beats - owner_beat).rem_euclid(IMPACT_POCKET_BAR_BEATS);
        if relative > IMPACT_POCKET_BAR_BEATS - IMPACT_POCKET_PRE_ROLL_BEATS {
            relative -= IMPACT_POCKET_BAR_BEATS;
        }
        if !(-IMPACT_POCKET_PRE_ROLL_BEATS..IMPACT_POCKET_HOLD_BEATS + IMPACT_POCKET_RELEASE_BEATS)
            .contains(&relative)
        {
            return 1.0;
        }
        if relative < 0.0 {
            let phase =
                ((relative + IMPACT_POCKET_PRE_ROLL_BEATS) / IMPACT_POCKET_PRE_ROLL_BEATS) as f32;
            return lerp(1.0, IMPACT_POCKET_MIN_GAIN, smoothstep(phase));
        }
        if relative < IMPACT_POCKET_HOLD_BEATS {
            return IMPACT_POCKET_MIN_GAIN;
        }
        let phase = ((relative - IMPACT_POCKET_HOLD_BEATS) / IMPACT_POCKET_RELEASE_BEATS) as f32;
        lerp(IMPACT_POCKET_MIN_GAIN, 1.0, smoothstep(phase))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct BedFocusRenderState {
    fill: FillFocusRenderState,
    impact: ImpactPocketRenderState,
}

impl BedFocusRenderState {
    pub(super) fn from_tr909(render: &RealtimeTr909RenderState) -> Self {
        Self {
            fill: FillFocusRenderState::from_tr909(render),
            impact: ImpactPocketRenderState::from_tr909(render),
        }
    }

    pub(super) fn inactive() -> Self {
        let render = RealtimeTr909RenderState {
            mode: Tr909RenderMode::Idle,
            routing: Tr909RenderRouting::SourceOnly,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: None,
            phrase_variation: None,
            takeover_profile: None,
            drum_bus_level: 0.0,
            slam_enabled: false,
            slam_intensity: 0.0,
            is_transport_running: false,
            tempo_bpm: 0.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        };
        Self {
            fill: FillFocusRenderState::inactive(),
            impact: ImpactPocketRenderState::from_tr909(&render),
        }
    }

    pub(super) fn is_active(self) -> bool {
        self.fill.is_active() || self.impact.is_active()
    }

    pub(super) fn gain_at_frame(self, sample_rate: u32, frame_index: usize) -> f32 {
        if self.fill.is_active() {
            self.fill.gain_at_frame(sample_rate, frame_index)
        } else {
            self.impact.gain_at_frame(sample_rate, frame_index)
        }
    }
}

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

#[cfg(test)]
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

pub(super) fn apply_bed_focus_to_non_tr909_bed(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    bed_focus: BedFocusRenderState,
) {
    if channel_count == 0 {
        return;
    }
    for (frame_index, frame) in data.chunks_exact_mut(channel_count).enumerate() {
        let gain = bed_focus.gain_at_frame(sample_rate, frame_index);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tr909::{Tr909PatternAdoption, Tr909PhraseVariation};

    fn source_support_render(
        profile: Option<Tr909SourceSupportProfile>,
    ) -> RealtimeTr909RenderState {
        RealtimeTr909RenderState {
            mode: Tr909RenderMode::SourceSupport,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: profile,
            source_support_context: None,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseAnchor),
            takeover_profile: None,
            drum_bus_level: 0.72,
            slam_enabled: true,
            slam_intensity: 0.85,
            is_transport_running: true,
            tempo_bpm: 120.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        }
    }

    #[test]
    fn impact_pocket_selects_one_named_owner_from_trusted_source_support() {
        let kick = ImpactPocketRenderState::from_tr909(&source_support_render(Some(
            Tr909SourceSupportProfile::DropDrive,
        )));
        let steady = ImpactPocketRenderState::from_tr909(&source_support_render(Some(
            Tr909SourceSupportProfile::SteadyPulse,
        )));
        let snare = ImpactPocketRenderState::from_tr909(&source_support_render(Some(
            Tr909SourceSupportProfile::BreakLift,
        )));
        let break_reinforce = ImpactPocketRenderState::from_tr909(&RealtimeTr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            ..source_support_render(Some(Tr909SourceSupportProfile::DropDrive))
        });

        assert_eq!(kick.owner(), Some(Tr909ImpactOwner::KickDownbeat));
        assert_eq!(steady.owner(), Some(Tr909ImpactOwner::KickDownbeat));
        assert_eq!(snare.owner(), Some(Tr909ImpactOwner::SnareBackbeat));
        assert_eq!(
            break_reinforce.owner(),
            Some(Tr909ImpactOwner::KickDownbeat)
        );
    }

    #[test]
    fn impact_pocket_refuses_every_unowned_or_inaudible_path() {
        let render = source_support_render(Some(Tr909SourceSupportProfile::DropDrive));
        for refused in [
            RealtimeTr909RenderState {
                source_support_profile: None,
                ..render
            },
            RealtimeTr909RenderState {
                slam_enabled: false,
                ..render
            },
            RealtimeTr909RenderState {
                mode: Tr909RenderMode::Fill,
                ..render
            },
            RealtimeTr909RenderState {
                mode: Tr909RenderMode::Takeover,
                ..render
            },
            RealtimeTr909RenderState {
                routing: Tr909RenderRouting::SourceOnly,
                ..render
            },
            RealtimeTr909RenderState {
                drum_bus_level: 0.0,
                ..render
            },
            RealtimeTr909RenderState {
                is_transport_running: false,
                ..render
            },
        ] {
            let pocket = ImpactPocketRenderState::from_tr909(&refused);
            assert!(!pocket.is_active());
            assert_eq!(pocket.gain_at_frame(48_000, 0), 1.0);
        }
    }

    #[test]
    fn impact_pocket_v1_hits_frozen_landmarks_and_recovers_smoothly() {
        let sample_rate = 48_000;
        let frames_per_beat = 24_000;
        let pocket = ImpactPocketRenderState::from_tr909(&source_support_render(Some(
            Tr909SourceSupportProfile::DropDrive,
        )));

        assert_eq!(pocket.gain_at_position(-IMPACT_POCKET_PRE_ROLL_BEATS), 1.0);
        assert_eq!(pocket.gain_at_position(0.0), IMPACT_POCKET_MIN_GAIN);
        assert_eq!(
            pocket.gain_at_position(IMPACT_POCKET_HOLD_BEATS),
            IMPACT_POCKET_MIN_GAIN
        );
        assert_eq!(
            pocket.gain_at_position(IMPACT_POCKET_HOLD_BEATS + IMPACT_POCKET_RELEASE_BEATS),
            1.0
        );
        assert_eq!(pocket.gain_at_position(1.0), 1.0);

        let frame_count = frames_per_beat * 4;
        let mut previous = pocket.gain_at_frame(sample_rate, 0);
        let mut max_adjacent_delta = 0.0_f32;
        for frame in 1..=frame_count {
            let current = pocket.gain_at_frame(sample_rate, frame);
            assert!((IMPACT_POCKET_MIN_GAIN..=1.0).contains(&current));
            max_adjacent_delta = max_adjacent_delta.max((current - previous).abs());
            previous = current;
        }
        assert!(
            max_adjacent_delta < 0.001,
            "impact-pocket envelope stepped too abruptly: {max_adjacent_delta}"
        );
    }

    #[test]
    fn impact_pocket_is_sample_exact_across_callback_partitions() {
        let sample_rate = 48_000;
        let frame_count = 48_000 * 2;
        let render = source_support_render(Some(Tr909SourceSupportProfile::BreakLift));
        let mut full_block = vec![1.0_f32; frame_count * 2];
        apply_bed_focus_to_non_tr909_bed(
            &mut full_block,
            sample_rate,
            2,
            BedFocusRenderState::from_tr909(&render),
        );

        let mut partitioned = Vec::with_capacity(full_block.len());
        let beats_per_frame = 120.0_f64 / 60.0 / f64::from(sample_rate);
        for frame_offset in (0..frame_count).step_by(127) {
            let block_frames = (frame_count - frame_offset).min(127);
            let mut block = vec![1.0_f32; block_frames * 2];
            let block_render = RealtimeTr909RenderState {
                position_beats: frame_offset as f64 * beats_per_frame,
                ..render
            };
            apply_bed_focus_to_non_tr909_bed(
                &mut block,
                sample_rate,
                2,
                BedFocusRenderState::from_tr909(&block_render),
            );
            partitioned.extend(block);
        }

        assert_eq!(full_block, partitioned);
    }
}
