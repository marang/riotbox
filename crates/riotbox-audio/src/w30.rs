pub const W30_PREVIEW_SAMPLE_WINDOW_LEN: usize = 2_048;
pub const W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN: usize = 16_384;
/// Callback-safe full-duration mono proxy for the active internal resample artifact.
///
/// The source artifact may be longer than this fixed payload. Control-plane projection then keeps
/// evenly spaced PCM frames across the complete artifact, matching the existing committed-pad
/// playback seam instead of reducing the resample to one short transient grain.
pub const W30_RESAMPLE_SOURCE_WINDOW_LEN: usize = W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN;
pub const W30_PAD_CHOP_SLICE_COUNT: usize = 8;
pub const W30_RESAMPLE_HARD_SLICE_COUNT: usize = 8;
/// Frozen H12 whole-path gain whose 0–20 ms head level H14 preserves while
/// calibrating the rest of `source_hit_shaper_v3`.
pub const W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN: f32 = 0.94;
/// H14 keeps a selected `source_hit_shaper_v3` hit at unity while exact
/// callback calibration may lower only the material between selected hits.
pub const W30_RESAMPLE_HIT_SHAPER_PRESERVED_OUTPUT_GAIN: f32 = 1.0;
/// Maximum inverse gain needed when H14 chooses its minimum whole-path output
/// gain (`0.25`) but keeps the selected hit at unity.
pub const W30_RESAMPLE_HIT_SHAPER_MAX_WINDOW_COMPENSATION_GAIN: f32 = 4.0;
/// Minimum source-local body articulation for the versioned H13 gesture.
pub const W30_RESAMPLE_H13_MIN_BODY_GAIN: f32 = 1.12;
/// Maximum source-local body articulation for the versioned H13 gesture.
pub const W30_RESAMPLE_H13_MAX_BODY_GAIN: f32 = 1.45;
/// Lower bound for source-relative energy matching of the H13 impact window.
pub const W30_RESAMPLE_H13_MIN_IMPACT_LEVEL_COMPENSATION: f32 = 0.75;
/// Lower bound for source-relative reverse-pickup normalization.
pub const W30_RESAMPLE_H13_MIN_PICKUP_GAIN: f32 = 0.10;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30PreviewRenderMode {
    Idle,
    LiveRecall,
    RawCaptureAudition,
    PromotedAudition,
}

impl W30PreviewRenderMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::LiveRecall => "live_recall",
            Self::RawCaptureAudition => "raw_capture_audition",
            Self::PromotedAudition => "promoted_audition",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30PreviewRenderRouting {
    Silent,
    MusicBusPreview,
}

impl W30PreviewRenderRouting {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::MusicBusPreview => "music_bus_preview",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30PreviewSourceProfile {
    PinnedRecall,
    PromotedRecall,
    SlicePoolBrowse,
    RawCaptureAudition,
    PromotedAudition,
}

impl W30PreviewSourceProfile {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PinnedRecall => "pinned_recall",
            Self::PromotedRecall => "promoted_recall",
            Self::SlicePoolBrowse => "slice_pool_browse",
            Self::RawCaptureAudition => "raw_capture_audition",
            Self::PromotedAudition => "promoted_audition",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30ResampleTapMode {
    Idle,
    CaptureLineageReady,
}

impl W30ResampleTapMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::CaptureLineageReady => "capture_lineage_ready",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30ResampleTapRouting {
    Silent,
    InternalCaptureTap,
}

impl W30ResampleTapRouting {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::InternalCaptureTap => "internal_capture_tap",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30ResampleTapSourceProfile {
    RawCapture,
    PromotedCapture,
    PinnedCapture,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30ResampleTapAvailability {
    Idle,
    SourceAudioUnavailable,
    SourceAudioReady,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum W30ResampleTapVariation {
    #[default]
    Base,
    HardDamage,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum W30ResampleTapHardPolicy {
    #[default]
    Unavailable,
    SourceTransientChop,
    SourceTextureBite,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum W30ResampleHardSuitability {
    #[default]
    Unavailable,
    Suitable,
    InsufficientLevel,
    InsufficientActivity,
}

impl W30ResampleHardSuitability {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::Suitable => "suitable",
            Self::InsufficientLevel => "insufficient_level",
            Self::InsufficientActivity => "insufficient_activity",
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct W30ResampleHardSuitabilityPlan {
    pub status: W30ResampleHardSuitability,
    pub source_rms: f32,
    pub active_frame_ratio: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct W30ResampleHardCalibrationPlan {
    /// Source-relative transform level before the calibrated output gain.
    pub predicted_raw_level_ratio: f32,
    /// Source-relative transform level after the calibrated output gain.
    pub predicted_compensated_level_ratio: f32,
    /// Predicted 20–100 ms body ratio after matching the analyzed hit recipe
    /// to its intended whole-hit level.
    ///
    /// Zero means that the selected policy does not claim hit-body ownership.
    pub predicted_level_matched_body_ratio: f32,
    /// Source-calibrated gain carried unchanged into the realtime callback.
    pub output_gain: f32,
    /// Local selected-hit compensation paired with `output_gain` for
    /// `source_hit_shaper_v3`.
    ///
    /// Exact callback calibration may reduce the whole Hard path while this
    /// keeps the already source-owned 0–200 ms hit from collapsing.
    pub hit_window_compensation_gain: f32,
    /// True when the hit-shaper level was measured through the exact callback
    /// at the trusted source tempo rather than only predicted from source
    /// windows.
    pub exact_callback_calibrated: bool,
    /// True after exact callback calibration has run for these typed inputs,
    /// including a bounded attempt that could not satisfy every gate.
    ///
    /// This lets the control plane cache a negative result instead of
    /// repeatedly rendering the same rejected source on every view refresh.
    pub exact_callback_evaluated: bool,
}

impl Default for W30ResampleHardCalibrationPlan {
    fn default() -> Self {
        Self {
            predicted_raw_level_ratio: 1.0,
            predicted_compensated_level_ratio: 1.0,
            predicted_level_matched_body_ratio: 0.0,
            output_gain: 1.0,
            hit_window_compensation_gain: 1.0,
            exact_callback_calibrated: false,
            exact_callback_evaluated: false,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum W30ResampleHardGestureRecipe {
    #[default]
    Unavailable,
    /// H13 source-backed performance gesture: reverse the selected impact into
    /// its forward grid return and articulate only that source hit's body.
    SourceReverseIntoImpactV1,
}

impl W30ResampleHardGestureRecipe {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::SourceReverseIntoImpactV1 => "source_reverse_into_impact_v1",
        }
    }

    /// Source material used by the reverse pickup before it reaches the impact.
    #[must_use]
    pub const fn pickup_duration_frames(self, sample_rate: u32) -> u32 {
        match self {
            Self::Unavailable => 0,
            Self::SourceReverseIntoImpactV1 => {
                let frames = sample_rate * 120 / 1_000;
                if frames == 0 { 1 } else { frames }
            }
        }
    }

    /// Start of the source-owned body articulation after the transient head.
    #[must_use]
    pub const fn body_start_frames(self, sample_rate: u32) -> u32 {
        match self {
            Self::Unavailable => 0,
            Self::SourceReverseIntoImpactV1 => sample_rate / 50,
        }
    }

    /// End of the source-owned body articulation window.
    #[must_use]
    pub const fn body_end_frames(self, sample_rate: u32) -> u32 {
        match self {
            Self::Unavailable => 0,
            Self::SourceReverseIntoImpactV1 => {
                let frames = sample_rate / 10;
                if frames == 0 { 1 } else { frames }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct W30ResampleHardGesturePlan {
    pub recipe: W30ResampleHardGestureRecipe,
    /// Strongest source-backed impact inside the eight-slot Hard cycle.
    pub impact_slot: u8,
    /// Grid slot immediately before `impact_slot`; its tail owns the pickup.
    pub pickup_slot: u8,
    /// Source-relative gain applied only to the selected impact's body window.
    pub body_gain: f32,
    /// Source-relative energy compensation for the articulated impact window.
    ///
    /// This is separate from H12 calibration so the new local body shape cannot
    /// pass by increasing whole-gesture level.
    pub impact_level_compensation: f32,
    /// Source-relative reverse level matched against the destination slot tail.
    pub pickup_gain: f32,
    /// Source evidence for the selected impact's 0–20 ms head.
    pub selected_head_rms: f32,
    /// Source evidence for the selected impact's 20–100 ms body.
    pub selected_body_rms: f32,
}

impl Default for W30ResampleHardGesturePlan {
    fn default() -> Self {
        Self {
            recipe: W30ResampleHardGestureRecipe::Unavailable,
            impact_slot: 0,
            pickup_slot: 0,
            body_gain: 1.0,
            impact_level_compensation: 1.0,
            pickup_gain: 1.0,
            selected_head_rms: 0.0,
            selected_body_rms: 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum W30ResampleHardGritRecipe {
    #[default]
    Unavailable,
    SourceGritSlamV1,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum W30ResampleLowImpactRecipe {
    #[default]
    Unavailable,
    /// Source-owned 45–180 Hz transient body returned in parallel after the
    /// destructive Hard chain. It never synthesizes a kick or missing low end.
    SourceLowTransientPunchV1,
    /// Source-owned low body plus short presence-band head, rendered as the
    /// Hard output before any Damage texture can mask the physical hit.
    SourceKickImpactV2,
    /// Source-local, phase-coherent direct-path hit shaper. It carries the
    /// existing source hit over a longer body window and gives its short head
    /// a clipped articulation without adding a synthetic kick or bass layer.
    SourceHitShaperV3,
}

impl W30ResampleLowImpactRecipe {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::SourceLowTransientPunchV1 => "source_low_transient_punch_v1",
            Self::SourceKickImpactV2 => "source_kick_impact_v2",
            Self::SourceHitShaperV3 => "source_hit_shaper_v3",
        }
    }

    #[must_use]
    pub const fn cutoff_hz(self) -> Option<(f32, f32)> {
        match self {
            Self::Unavailable => None,
            Self::SourceLowTransientPunchV1
            | Self::SourceKickImpactV2
            | Self::SourceHitShaperV3 => Some((45.0, 180.0)),
        }
    }

    #[must_use]
    pub const fn parallel_attack_gain(self) -> f32 {
        match self {
            Self::Unavailable => 0.0,
            Self::SourceLowTransientPunchV1 => 0.5,
            Self::SourceKickImpactV2 => 0.43,
            Self::SourceHitShaperV3 => 0.0,
        }
    }

    #[must_use]
    pub const fn presence_cutoff_hz(self) -> Option<(f32, f32)> {
        match self {
            Self::SourceKickImpactV2 | Self::SourceHitShaperV3 => Some((900.0, 3_600.0)),
            Self::Unavailable | Self::SourceLowTransientPunchV1 => None,
        }
    }

    #[must_use]
    pub const fn parallel_head_gain(self) -> f32 {
        match self {
            Self::SourceKickImpactV2 => 0.38,
            Self::Unavailable | Self::SourceLowTransientPunchV1 | Self::SourceHitShaperV3 => 0.0,
        }
    }

    /// Center of the source-owned body equalizer.
    #[must_use]
    pub const fn body_eq_center_hz(self) -> f32 {
        match self {
            Self::SourceHitShaperV3 => 90.0,
            Self::Unavailable | Self::SourceLowTransientPunchV1 | Self::SourceKickImpactV2 => 0.0,
        }
    }

    /// Q of the source-owned body equalizer.
    #[must_use]
    pub const fn body_eq_q(self) -> f32 {
        match self {
            Self::SourceHitShaperV3 => 0.8,
            Self::Unavailable | Self::SourceLowTransientPunchV1 | Self::SourceKickImpactV2 => 1.0,
        }
    }

    /// Maximum body-band lift while the source hit is owned by the Hard path.
    #[must_use]
    pub const fn body_eq_gain_db(self) -> f32 {
        match self {
            Self::SourceHitShaperV3 => 12.0,
            Self::Unavailable | Self::SourceLowTransientPunchV1 | Self::SourceKickImpactV2 => 0.0,
        }
    }

    /// Nonlinear drive for the short source-owned presence head.
    #[must_use]
    pub const fn head_drive(self) -> f32 {
        match self {
            Self::SourceHitShaperV3 => 3.5,
            Self::Unavailable | Self::SourceLowTransientPunchV1 | Self::SourceKickImpactV2 => 1.0,
        }
    }

    /// Wet share of the nonlinear attack-head replacement residual.
    #[must_use]
    pub const fn head_wet(self) -> f32 {
        match self {
            Self::SourceHitShaperV3 => 0.6,
            Self::Unavailable | Self::SourceLowTransientPunchV1 | Self::SourceKickImpactV2 => 0.0,
        }
    }

    /// Minimum source-local hit window. This carries the following body long
    /// enough to be judged as punch rather than a sub-20-ms click.
    #[must_use]
    pub const fn minimum_hit_window_frames(self, sample_rate: u32) -> u32 {
        match self {
            Self::SourceHitShaperV3 => {
                let frames = sample_rate / 10;
                if frames == 0 { 1 } else { frames }
            }
            Self::Unavailable | Self::SourceLowTransientPunchV1 | Self::SourceKickImpactV2 => 1,
        }
    }

    /// H14 preserves the selected source hit through the 0–200 ms QA window
    /// while exact callback calibration lowers only the surrounding material.
    #[must_use]
    pub const fn calibrated_hit_preservation_frames(self, sample_rate: u32) -> u32 {
        match self {
            Self::SourceHitShaperV3 => sample_rate / 5,
            Self::Unavailable | Self::SourceLowTransientPunchV1 | Self::SourceKickImpactV2 => 0,
        }
    }

    /// Smoothly leave the H14 hit-preservation gain after the owned hit window.
    #[must_use]
    pub const fn calibrated_hit_preservation_fade_frames(self, sample_rate: u32) -> u32 {
        match self {
            Self::SourceHitShaperV3 => sample_rate / 100,
            Self::Unavailable | Self::SourceLowTransientPunchV1 | Self::SourceKickImpactV2 => 0,
        }
    }

    /// Preserve the 20 ms immediately before a selected hit so the
    /// role-filter state does not inherit the attenuated between-hit level.
    #[must_use]
    pub const fn calibrated_hit_preroll_frames(self, sample_rate: u32) -> u32 {
        match self {
            Self::SourceHitShaperV3 => sample_rate / 50,
            Self::Unavailable | Self::SourceLowTransientPunchV1 | Self::SourceKickImpactV2 => 0,
        }
    }

    /// Click-safe lead-in before the full 20 ms unity pre-roll begins.
    #[must_use]
    pub const fn calibrated_hit_preroll_fade_frames(self, sample_rate: u32) -> u32 {
        match self {
            Self::SourceHitShaperV3 => sample_rate / 400,
            Self::Unavailable | Self::SourceLowTransientPunchV1 | Self::SourceKickImpactV2 => 0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct W30ResampleLowImpactPlan {
    pub recipe: W30ResampleLowImpactRecipe,
    /// RMS share of the 45–180 Hz band inside selected source attacks.
    pub low_band_attack_share: f32,
    /// Selected low-band attack RMS divided by its following body RMS.
    pub low_band_attack_over_body: f32,
    /// Selected low-band attack RMS divided by whole-source RMS.
    pub low_band_attack_over_source: f32,
}

impl Default for W30ResampleLowImpactPlan {
    fn default() -> Self {
        Self {
            recipe: W30ResampleLowImpactRecipe::Unavailable,
            low_band_attack_share: 0.0,
            low_band_attack_over_body: 0.0,
            low_band_attack_over_source: 0.0,
        }
    }
}

impl W30ResampleHardGritRecipe {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::SourceGritSlamV1 => "source_grit_slam_v1",
        }
    }

    #[must_use]
    pub const fn effective_sample_rate_hz(self) -> Option<u32> {
        match self {
            Self::Unavailable => None,
            Self::SourceGritSlamV1 => Some(8_000),
        }
    }

    #[must_use]
    pub const fn quantization_levels(self) -> Option<u16> {
        match self {
            Self::Unavailable => None,
            Self::SourceGritSlamV1 => Some(63),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum W30ResampleAttackBiteBand {
    #[default]
    Unavailable,
    LowMid,
    Presence,
}

impl W30ResampleAttackBiteBand {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::LowMid => "low_mid",
            Self::Presence => "presence",
        }
    }

    #[must_use]
    pub const fn cutoff_hz(self) -> Option<(f32, f32)> {
        match self {
            Self::Unavailable => None,
            Self::LowMid => Some((250.0, 900.0)),
            Self::Presence => Some((700.0, 2_150.0)),
        }
    }
}

pub const W30_RESAMPLE_HARD_BITE_NONLINEAR_DRIVE: f32 = 4.0;
/// A positive 20 ms envelope rise must exceed the source's mean envelope
/// before transient-chop and its source-grit recipe may claim ownership.
pub const W30_RESAMPLE_TRANSIENT_CHOP_MIN_RISE_TO_MEAN: f32 = 1.0;
/// A low transient must occupy a meaningful share of a selected attack before
/// Hard may claim source-owned kick/body impact.
pub const W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_SHARE: f32 = 0.18;
/// The source low band must rise above its following body, not merely contain a
/// sustained bass note or pad.
pub const W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_OVER_BODY: f32 = 1.10;
/// H10 ownership guard: a body-shaping recipe needs a decisive low-band attack,
/// not merely low-frequency content inside a tonal or sustained phrase.
pub const W30_RESAMPLE_HIT_SHAPER_MIN_ATTACK_OVER_BODY: f32 = 1.40;
/// The selected low attack must be material relative to the source itself.
/// This remains level-relative so a genuinely quiet source is not misclassified.
pub const W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_OVER_SOURCE: f32 = 0.30;
/// Absolute mono source level required before a Hard policy may claim output.
pub const W30_RESAMPLE_HARD_MIN_SOURCE_RMS: f32 = 0.04;
/// Share of mono source frames above the activity floor required for Hard.
pub const W30_RESAMPLE_HARD_MIN_ACTIVE_FRAME_RATIO: f32 = 0.60;
/// Absolute mono activity floor used only for Hard source suitability.
pub const W30_RESAMPLE_HARD_ACTIVE_FRAME_FLOOR: f32 = 0.001;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct W30ResampleAttackBitePlan {
    pub band: W30ResampleAttackBiteBand,
    /// Source-derived normalization applied before the fixed nonlinear recipe.
    pub input_gain: f32,
    /// Source-derived gain that restores the selected band's RMS on its nonlinear residual.
    pub output_gain: f32,
}

impl Default for W30ResampleAttackBitePlan {
    fn default() -> Self {
        Self {
            band: W30ResampleAttackBiteBand::Unavailable,
            input_gain: 1.0,
            output_gain: 1.0,
        }
    }
}

impl W30ResampleTapHardPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::SourceTransientChop => "source_transient_chop",
            Self::SourceTextureBite => "source_texture_bite",
        }
    }

    /// Fixed transform vocabulary selected by this source-derived hard policy.
    ///
    /// The recipe never activates without both source audio and a committed
    /// `HardDamage` variation; it is not a missing-source fallback.
    #[must_use]
    pub const fn grit_recipe(self) -> W30ResampleHardGritRecipe {
        match self {
            Self::SourceTransientChop => W30ResampleHardGritRecipe::SourceGritSlamV1,
            Self::Unavailable | Self::SourceTextureBite => W30ResampleHardGritRecipe::Unavailable,
        }
    }
}

/// Stateful source-character sample shared by projection calibration and the
/// live callback. Keeping the nonlinear topology here prevents a diagnostic
/// approximation from silently drifting away from product output.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn w30_resample_source_character_sample(
    sample: f32,
    grit_level: f32,
    variation_intensity: f32,
    enabled: bool,
    hard_texture: bool,
    last_input: &mut f32,
    edge_memory: &mut f32,
) -> f32 {
    const EDGE_MEMORY: f32 = 0.68;
    const EDGE_RANGE: f32 = 1.1;
    const DRIVE_RANGE: f32 = 4.2;
    const WET_RANGE: f32 = 0.78;
    const BODY_SHARE: f32 = 0.76;
    const EDGE_SHARE: f32 = 0.24;
    const HARD_SOURCE_DRIVE_RANGE: f32 = 5.2;
    const HARD_SOURCE_WET_RANGE: f32 = 0.92;
    const HARD_SOURCE_BODY_SHARE: f32 = 0.62;
    const HARD_SOURCE_EDGE_SHARE: f32 = 0.38;
    const HARD_SOURCE_EDGE_BASE: f32 = 7.0;
    const HARD_SOURCE_EDGE_RANGE: f32 = 20.0;
    const HARD_TEXTURE_FOLD_DRIVE: f32 = 2.2;
    const HARD_TEXTURE_QUANTIZATION_STEPS: f32 = 8.0;
    const HARD_TEXTURE_FOLD_CRUSH_MIX: f32 = 0.58;

    let grit = if !enabled {
        0.0
    } else if hard_texture {
        (grit_level.clamp(0.0, 1.0) * 0.22 + variation_intensity.clamp(0.0, 1.0) * 0.34)
            .clamp(0.0, 0.62)
    } else {
        grit_level.clamp(0.0, 1.0) * 0.22
    };
    let raw_edge = sample - *last_input;
    *last_input = sample;
    *edge_memory = *edge_memory * EDGE_MEMORY + raw_edge * (1.0 - EDGE_MEMORY);
    if grit <= f32::EPSILON {
        return sample;
    }

    let driven = sample + *edge_memory * grit * EDGE_RANGE;
    let drive_range = if hard_texture {
        HARD_SOURCE_DRIVE_RANGE
    } else {
        DRIVE_RANGE
    };
    let (body_share, edge_share, edge_base, edge_range, wet_range) = if hard_texture {
        (
            HARD_SOURCE_BODY_SHARE,
            HARD_SOURCE_EDGE_SHARE,
            HARD_SOURCE_EDGE_BASE,
            HARD_SOURCE_EDGE_RANGE,
            HARD_SOURCE_WET_RANGE,
        )
    } else {
        (BODY_SHARE, EDGE_SHARE, 5.0, 13.0, WET_RANGE)
    };
    let saturated = (driven * (1.0 + grit * drive_range)).tanh();
    let edge = (raw_edge * (edge_base + grit * edge_range)).tanh();
    let bitten = saturated * body_share + edge * edge_share;
    let bitten = if hard_texture {
        let folded = (bitten * HARD_TEXTURE_FOLD_DRIVE * std::f32::consts::FRAC_PI_2).sin();
        let crushed =
            (folded * HARD_TEXTURE_QUANTIZATION_STEPS).round() / HARD_TEXTURE_QUANTIZATION_STEPS;
        bitten * (1.0 - HARD_TEXTURE_FOLD_CRUSH_MIX) + crushed * HARD_TEXTURE_FOLD_CRUSH_MIX
    } else {
        bitten
    };
    let wet = grit * wet_range;
    (sample * (1.0 - wet) + bitten * wet).clamp(-0.98, 0.98)
}

impl W30ResampleTapVariation {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::HardDamage => "hard_damage",
        }
    }
}

impl W30ResampleTapAvailability {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::SourceAudioUnavailable => "source_audio_unavailable",
            Self::SourceAudioReady => "source_audio_ready",
        }
    }
}

impl W30ResampleTapSourceProfile {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::RawCapture => "raw_capture",
            Self::PromotedCapture => "promoted_capture",
            Self::PinnedCapture => "pinned_capture",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct W30PreviewRenderState {
    pub mode: W30PreviewRenderMode,
    pub routing: W30PreviewRenderRouting,
    pub source_profile: Option<W30PreviewSourceProfile>,
    pub active_bank_id: Option<String>,
    pub focused_pad_id: Option<String>,
    pub capture_id: Option<String>,
    pub trigger_revision: u64,
    pub trigger_velocity: f32,
    pub source_window_preview: Option<W30PreviewSampleWindow>,
    pub pad_playback: Option<W30PadPlaybackSampleWindow>,
    pub music_bus_level: f32,
    pub grit_level: f32,
    pub is_transport_running: bool,
    pub tempo_bpm: f32,
    pub position_beats: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct W30PreviewSampleWindow {
    pub source_start_frame: u64,
    pub source_end_frame: u64,
    pub sample_count: usize,
    pub samples: [f32; W30_PREVIEW_SAMPLE_WINDOW_LEN],
}

#[derive(Clone, Debug, PartialEq)]
pub struct W30PadPlaybackSampleWindow {
    pub source_start_frame: u64,
    pub source_end_frame: u64,
    pub source_sample_rate: u32,
    pub playback_frame_count: u64,
    pub sample_count: usize,
    pub loop_enabled: bool,
    pub playback_rate: f32,
    pub reverse: bool,
    /// Fraction of one W-30 trigger step kept audible before a short choke.
    /// Zero disables the gate and preserves the full slice playback path.
    pub gate_step_fraction: f32,
    pub loop_crossfade_sample_count: usize,
    pub chop_slice_count: usize,
    pub chop_slice_starts: [u32; W30_PAD_CHOP_SLICE_COUNT],
    pub samples: [f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
}

impl Default for W30PreviewRenderState {
    fn default() -> Self {
        Self {
            mode: W30PreviewRenderMode::Idle,
            routing: W30PreviewRenderRouting::Silent,
            source_profile: None,
            active_bank_id: None,
            focused_pad_id: None,
            capture_id: None,
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: None,
            pad_playback: None,
            music_bus_level: 0.0,
            grit_level: 0.0,
            is_transport_running: false,
            tempo_bpm: 0.0,
            position_beats: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct W30ResampleTapState {
    pub mode: W30ResampleTapMode,
    pub routing: W30ResampleTapRouting,
    pub availability: W30ResampleTapAvailability,
    pub source_profile: Option<W30ResampleTapSourceProfile>,
    pub source_capture_id: Option<String>,
    pub source_audio: Option<Box<W30ResampleSourceWindow>>,
    pub lineage_capture_count: u8,
    pub generation_depth: u8,
    pub variation: W30ResampleTapVariation,
    /// Stable action-log position of the committed gesture that activated the variation.
    ///
    /// Zero means that no post-resample variation gesture has committed.
    pub variation_revision: u64,
    pub variation_intensity: f32,
    pub hard_policy: W30ResampleTapHardPolicy,
    /// Typed source-level and activity evidence that permits or rejects Hard
    /// policy ownership before any audible transform reaches the callback.
    pub hard_suitability: W30ResampleHardSuitabilityPlan,
    /// Source-calibrated H12 transform prediction and realtime output gain.
    pub hard_calibration: W30ResampleHardCalibrationPlan,
    /// Eight source-derived eighth-note trigger decisions, least-significant bit first.
    pub hard_trigger_mask: u8,
    /// Source-proxy cursor for the detected local onset in each performed eighth-note slot.
    pub hard_slice_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    /// Source-adaptive perceptual attack length for each selected onset in proxy samples.
    pub hard_attack_lengths: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    /// Source-selected, level-compensated H2 nonlinear processing for the attack path.
    pub hard_attack_bite: W30ResampleAttackBitePlan,
    /// Source-evidence-gated low-transient impact returned after the destructive chain.
    pub hard_low_impact: W30ResampleLowImpactPlan,
    /// Source-selected H13 reverse pickup, impact slot, and local body articulation.
    pub hard_gesture: W30ResampleHardGesturePlan,
    /// Strongest positive 20 ms envelope rise divided by the mean source envelope.
    pub hard_transient_contrast: f32,
    pub music_bus_level: f32,
    pub grit_level: f32,
    pub is_transport_running: bool,
    pub tempo_bpm: f32,
    pub position_beats: f64,
}

impl W30ResampleTapState {
    /// Whether the typed product state owns the exact callback calibration
    /// used by the source-backed hit-shaper Hard recipe.
    ///
    /// Preflight and projection share this predicate so diagnostics cannot
    /// invent a second recipe selector or disagree about applicability.
    #[must_use]
    pub fn exact_hit_shaper_calibration_applicable(&self) -> bool {
        self.variation == W30ResampleTapVariation::HardDamage
            && self.hard_policy == W30ResampleTapHardPolicy::SourceTransientChop
            && self.hard_low_impact.recipe == W30ResampleLowImpactRecipe::SourceHitShaperV3
            && self.source_audio.is_some()
            && self.tempo_bpm.is_finite()
            && self.tempo_bpm > 0.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct W30ResampleSourceWindow {
    /// Stable fingerprint of the projected PCM and its source timing metadata.
    ///
    /// The callback uses this to reset or phase-map source-local state when
    /// active PCM changes without requiring a mode or variation change.
    pub source_revision: u64,
    pub source_start_frame: u64,
    pub source_sample_rate: u32,
    pub source_frame_count: u64,
    pub sample_count: usize,
    pub samples: [f32; W30_RESAMPLE_SOURCE_WINDOW_LEN],
}

impl Default for W30ResampleTapState {
    fn default() -> Self {
        Self {
            mode: W30ResampleTapMode::Idle,
            routing: W30ResampleTapRouting::Silent,
            availability: W30ResampleTapAvailability::Idle,
            source_profile: None,
            source_capture_id: None,
            source_audio: None,
            lineage_capture_count: 0,
            generation_depth: 0,
            variation: W30ResampleTapVariation::Base,
            variation_revision: 0,
            variation_intensity: 0.0,
            hard_policy: W30ResampleTapHardPolicy::Unavailable,
            hard_suitability: W30ResampleHardSuitabilityPlan::default(),
            hard_calibration: W30ResampleHardCalibrationPlan::default(),
            hard_trigger_mask: 0,
            hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_attack_bite: W30ResampleAttackBitePlan::default(),
            hard_low_impact: W30ResampleLowImpactPlan::default(),
            hard_gesture: W30ResampleHardGesturePlan::default(),
            hard_transient_contrast: 0.0,
            music_bus_level: 0.0,
            grit_level: 0.0,
            is_transport_running: false,
            tempo_bpm: 0.0,
            position_beats: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState,
        W30PreviewSourceProfile, W30ResampleTapAvailability, W30ResampleTapMode,
        W30ResampleTapRouting, W30ResampleTapSourceProfile, W30ResampleTapState,
    };

    #[test]
    fn default_preview_state_is_idle_and_silent() {
        let state = W30PreviewRenderState::default();

        assert_eq!(state.mode, W30PreviewRenderMode::Idle);
        assert_eq!(state.routing, W30PreviewRenderRouting::Silent);
        assert_eq!(state.source_profile, None);
        assert_eq!(state.active_bank_id, None);
        assert_eq!(state.focused_pad_id, None);
        assert_eq!(state.capture_id, None);
        assert_eq!(state.trigger_revision, 0);
        assert_eq!(state.trigger_velocity, 0.0);
        assert_eq!(state.source_window_preview, None);
        assert!(!state.is_transport_running);
    }

    #[test]
    fn preview_labels_stay_stable() {
        assert_eq!(W30PreviewRenderMode::Idle.label(), "idle");
        assert_eq!(W30PreviewRenderMode::LiveRecall.label(), "live_recall");
        assert_eq!(
            W30PreviewRenderMode::RawCaptureAudition.label(),
            "raw_capture_audition"
        );
        assert_eq!(
            W30PreviewRenderMode::PromotedAudition.label(),
            "promoted_audition"
        );
        assert_eq!(W30PreviewRenderRouting::Silent.label(), "silent");
        assert_eq!(
            W30PreviewRenderRouting::MusicBusPreview.label(),
            "music_bus_preview"
        );
        assert_eq!(
            W30PreviewSourceProfile::PinnedRecall.label(),
            "pinned_recall"
        );
        assert_eq!(
            W30PreviewSourceProfile::PromotedRecall.label(),
            "promoted_recall"
        );
        assert_eq!(
            W30PreviewSourceProfile::SlicePoolBrowse.label(),
            "slice_pool_browse"
        );
        assert_eq!(
            W30PreviewSourceProfile::RawCaptureAudition.label(),
            "raw_capture_audition"
        );
        assert_eq!(
            W30PreviewSourceProfile::PromotedAudition.label(),
            "promoted_audition"
        );
        assert_eq!(W30ResampleTapMode::Idle.label(), "idle");
        assert_eq!(
            W30ResampleTapMode::CaptureLineageReady.label(),
            "capture_lineage_ready"
        );
        assert_eq!(W30ResampleTapRouting::Silent.label(), "silent");
        assert_eq!(
            W30ResampleTapRouting::InternalCaptureTap.label(),
            "internal_capture_tap"
        );
        assert_eq!(W30ResampleTapAvailability::Idle.label(), "idle");
        assert_eq!(
            W30ResampleTapAvailability::SourceAudioUnavailable.label(),
            "source_audio_unavailable"
        );
        assert_eq!(
            W30ResampleTapAvailability::SourceAudioReady.label(),
            "source_audio_ready"
        );
        assert_eq!(
            W30ResampleTapSourceProfile::RawCapture.label(),
            "raw_capture"
        );
        assert_eq!(
            W30ResampleTapSourceProfile::PromotedCapture.label(),
            "promoted_capture"
        );
        assert_eq!(
            W30ResampleTapSourceProfile::PinnedCapture.label(),
            "pinned_capture"
        );
    }

    #[test]
    fn default_resample_tap_state_is_idle_and_silent() {
        let state = W30ResampleTapState::default();

        assert_eq!(state.mode, W30ResampleTapMode::Idle);
        assert_eq!(state.routing, W30ResampleTapRouting::Silent);
        assert_eq!(state.availability, W30ResampleTapAvailability::Idle);
        assert_eq!(state.source_profile, None);
        assert_eq!(state.source_capture_id, None);
        assert_eq!(state.source_audio, None);
        assert_eq!(state.lineage_capture_count, 0);
        assert_eq!(state.generation_depth, 0);
        assert!(!state.is_transport_running);
    }
}
