//! Existing primitive support render, kick-pressure policy and accent evidence for QA.

use super::config::{
    CHANNEL_COUNT, PATTERN_ORIGIN_PRIMITIVE_RENDERER, PATTERN_ORIGIN_SOURCE_DERIVED, SAMPLE_RATE,
};
use super::grid::{Grid, frames_for_beat_position};
use super::signal_filter::one_pole_lowpass;
use super::source_aware_tr909::SourceAwareTr909Profile;
use riotbox_audio::runtime::{render_tr909_offline, signal_metrics_with_grid};
use riotbox_audio::tr909::{
    Tr909RenderMode, Tr909RenderRouting, Tr909RenderState, Tr909SourceSupportProfile,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Tr909KickPressureProof {
    pub(super) pattern_origin: &'static str,
    pub(super) source_evidence_role: &'static str,
    pub(super) source_profile_reason: &'static str,
    pub(super) applied: bool,
    pub(super) anchor_count: usize,
    pub(super) pressure_gain: f32,
    pub(super) pre_low_band_rms: f32,
    pub(super) post_low_band_rms: f32,
    pub(super) low_band_rms_delta: f32,
    pub(super) low_band_rms_ratio: f32,
    pub(super) post_peak_abs: f32,
    pub(super) reason: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Tr909SourceAccentDynamicsProof {
    pub(super) pattern_origin: &'static str,
    pub(super) applied: bool,
    pub(super) anchor_count: usize,
    pub(super) distinct_accent_count: usize,
    pub(super) min_accent: f32,
    pub(super) max_accent: f32,
    pub(super) accent_span: f32,
    pub(super) min_required_accent_span: f32,
    pub(super) source_energy_span: f32,
    pub(super) reason: &'static str,
}

pub(super) const TR909_KICK_PRESSURE_MIN_LOW_BAND_RATIO: f32 = 1.06;

pub(super) const TR909_KICK_PRESSURE_MAX_PEAK_ABS: f32 = 0.95;

pub(super) const TR909_SOURCE_ACCENT_MIN_ACCENT_SPAN: f32 = 0.22;

pub(super) const TR909_SOURCE_ACCENT_MIN_DISTINCT_ACCENTS: usize = 3;

pub(super) const TR909_SOURCE_EVIDENCE_ROLE_PROFILE_AND_ACCENT_DYNAMICS: &str =
    "tr909_source_profile_and_accent_dynamics";

const TR909_SOURCE_EVIDENCE_ROLE_PRIMITIVE_CONTROL_ONLY: &str = "tr909_primitive_control_only";

#[allow(dead_code)]
pub(super) fn render_tr909_source_support(
    grid: &Grid,
    profile: SourceAwareTr909Profile,
) -> Vec<f32> {
    render_tr909_source_support_with_pressure(grid, profile).0
}

fn render_tr909_source_support_with_pressure(
    grid: &Grid,
    profile: SourceAwareTr909Profile,
) -> (Vec<f32>, Tr909KickPressureProof) {
    let (samples, kick_pressure, _) =
        render_tr909_source_support_with_pressure_and_accents(grid, profile);
    (samples, kick_pressure)
}

pub(super) fn render_tr909_source_support_with_pressure_and_accents(
    grid: &Grid,
    profile: SourceAwareTr909Profile,
) -> (
    Vec<f32>,
    Tr909KickPressureProof,
    Tr909SourceAccentDynamicsProof,
) {
    let mut samples = render_tr909_source_support_legacy(grid, profile);
    let (kick_pressure, accent_dynamics) = apply_tr909_kick_pressure(&mut samples, grid, profile);
    (samples, kick_pressure, accent_dynamics)
}

fn apply_tr909_kick_pressure(
    samples: &mut [f32],
    grid: &Grid,
    profile: SourceAwareTr909Profile,
) -> (Tr909KickPressureProof, Tr909SourceAccentDynamicsProof) {
    let pre = tr909_kick_pressure_metrics(samples, grid);
    let policy = tr909_kick_pressure_policy(profile);
    let mut accents = Vec::new();

    for beat in 0..grid.total_beats {
        let beat_in_bar = beat % grid.beats_per_bar;
        let accent = tr909_source_accent_for_beat(profile, beat_in_bar);
        if accent <= 0.0 {
            continue;
        }

        render_tr909_kick_pressure_anchor(samples, grid, beat as f32, policy, accent);
        accents.push(accent);
    }

    for sample in &mut *samples {
        *sample = sample.clamp(
            -TR909_KICK_PRESSURE_MAX_PEAK_ABS,
            TR909_KICK_PRESSURE_MAX_PEAK_ABS,
        );
    }

    let post = tr909_kick_pressure_metrics(samples, grid);
    let low_band_rms_delta = post.low_band_rms - pre.low_band_rms;
    let low_band_rms_ratio = post.low_band_rms / pre.low_band_rms.max(f32::EPSILON);
    let accent_dynamics = tr909_source_accent_dynamics_proof(&accents, profile);
    let applied = accent_dynamics.applied
        && !accents.is_empty()
        && low_band_rms_ratio >= TR909_KICK_PRESSURE_MIN_LOW_BAND_RATIO
        && post.peak_abs <= TR909_KICK_PRESSURE_MAX_PEAK_ABS;

    (
        Tr909KickPressureProof {
            pattern_origin: if accent_dynamics.applied {
                PATTERN_ORIGIN_SOURCE_DERIVED
            } else {
                PATTERN_ORIGIN_PRIMITIVE_RENDERER
            },
            source_evidence_role: if accent_dynamics.applied {
                TR909_SOURCE_EVIDENCE_ROLE_PROFILE_AND_ACCENT_DYNAMICS
            } else {
                TR909_SOURCE_EVIDENCE_ROLE_PRIMITIVE_CONTROL_ONLY
            },
            source_profile_reason: profile.reason,
            applied,
            anchor_count: accents.len(),
            pressure_gain: policy.gain,
            pre_low_band_rms: pre.low_band_rms,
            post_low_band_rms: post.low_band_rms,
            low_band_rms_delta,
            low_band_rms_ratio,
            post_peak_abs: post.peak_abs,
            reason: if applied {
                policy.reason
            } else if !accent_dynamics.applied {
                accent_dynamics.reason
            } else {
                "tr909_kick_pressure_too_weak"
            },
        },
        accent_dynamics,
    )
}

fn tr909_source_accent_for_beat(profile: SourceAwareTr909Profile, beat_in_bar: u32) -> f32 {
    let density = tr909_source_density_weight(profile);
    let low = profile.low_band_energy_ratio.clamp(0.0, 1.0);
    let mid = profile.mid_band_energy_ratio.clamp(0.0, 1.0);
    let high = profile.high_band_energy_ratio.clamp(0.0, 1.0);
    let base = match beat_in_bar {
        0 => 1.0,
        2 => 0.58,
        _ if profile.support_profile == Tr909SourceSupportProfile::DropDrive => 0.22,
        _ if profile.support_profile == Tr909SourceSupportProfile::BreakLift => 0.18,
        _ if profile.support_profile == Tr909SourceSupportProfile::SteadyPulse => 0.12,
        _ => 0.0,
    };
    if base <= 0.0 {
        return 0.0;
    }

    let source_factor = match beat_in_bar {
        0 => 1.0 + low * 0.18 + density * 0.08,
        2 => 0.88 + mid * 0.18,
        _ => 0.70 + high * 0.28 + density * 0.14,
    };
    (base * source_factor).clamp(0.05, 1.35)
}

fn tr909_source_accent_dynamics_proof(
    accents: &[f32],
    profile: SourceAwareTr909Profile,
) -> Tr909SourceAccentDynamicsProof {
    if accents.is_empty() {
        return Tr909SourceAccentDynamicsProof {
            pattern_origin: PATTERN_ORIGIN_SOURCE_DERIVED,
            applied: false,
            anchor_count: 0,
            distinct_accent_count: 0,
            min_accent: 0.0,
            max_accent: 0.0,
            accent_span: 0.0,
            min_required_accent_span: TR909_SOURCE_ACCENT_MIN_ACCENT_SPAN,
            source_energy_span: 0.0,
            reason: "tr909_source_accent_no_anchors",
        };
    }

    let min_accent = accents.iter().copied().fold(f32::INFINITY, f32::min);
    let max_accent = accents.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let accent_span = max_accent - min_accent;
    let distinct_accent_count = accents
        .iter()
        .map(|accent| (accent * 100.0).round() as i32)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let source_energy_span = profile
        .low_band_energy_ratio
        .max(profile.mid_band_energy_ratio)
        .max(profile.high_band_energy_ratio)
        - profile
            .low_band_energy_ratio
            .min(profile.mid_band_energy_ratio)
            .min(profile.high_band_energy_ratio);
    let applied = distinct_accent_count >= TR909_SOURCE_ACCENT_MIN_DISTINCT_ACCENTS
        && accent_span >= TR909_SOURCE_ACCENT_MIN_ACCENT_SPAN;

    Tr909SourceAccentDynamicsProof {
        pattern_origin: PATTERN_ORIGIN_SOURCE_DERIVED,
        applied,
        anchor_count: accents.len(),
        distinct_accent_count,
        min_accent,
        max_accent,
        accent_span,
        min_required_accent_span: TR909_SOURCE_ACCENT_MIN_ACCENT_SPAN,
        source_energy_span,
        reason: if applied {
            "tr909_source_accented_support_dynamics"
        } else {
            "tr909_source_accent_dynamics_too_flat"
        },
    }
}

#[derive(Clone, Copy, Debug)]
struct Tr909KickPressurePolicy {
    gain: f32,
    body_hz: f32,
    tail_seconds: f32,
    click_gain: f32,
    reason: &'static str,
}

fn tr909_kick_pressure_policy(profile: SourceAwareTr909Profile) -> Tr909KickPressurePolicy {
    let density = tr909_source_density_weight(profile);
    let low = profile.low_band_energy_ratio.clamp(0.0, 1.0);
    let high = profile.high_band_energy_ratio.clamp(0.0, 1.0);
    match profile.support_profile {
        Tr909SourceSupportProfile::DropDrive => Tr909KickPressurePolicy {
            gain: 0.018,
            body_hz: 49.0 + low * 7.0 + density * 8.5,
            tail_seconds: 0.180,
            click_gain: 0.005 + high * 0.006,
            reason: "tr909_low_drive_pressure",
        },
        Tr909SourceSupportProfile::BreakLift => Tr909KickPressurePolicy {
            gain: 0.0245,
            body_hz: 49.0 + density * 4.5,
            tail_seconds: 0.185,
            click_gain: 0.011,
            reason: "tr909_break_lift_pressure",
        },
        Tr909SourceSupportProfile::SteadyPulse => Tr909KickPressurePolicy {
            gain: 0.0175,
            body_hz: 49.0 + density * 4.5,
            tail_seconds: 0.175,
            click_gain: 0.005,
            reason: "tr909_steady_pulse_pressure",
        },
    }
}

fn tr909_source_density_weight(profile: SourceAwareTr909Profile) -> f32 {
    (profile.event_density_per_bar / 48.0).clamp(0.0, 1.0)
}

fn render_tr909_kick_pressure_anchor(
    samples: &mut [f32],
    grid: &Grid,
    beat_position: f32,
    policy: Tr909KickPressurePolicy,
    accent: f32,
) {
    let start_frame = frames_for_beat_position(grid.bpm, beat_position);
    let frame_count = (policy.tail_seconds * SAMPLE_RATE as f32).round() as usize;
    for offset in 0..frame_count {
        let frame = start_frame.saturating_add(offset);
        if frame >= grid.total_frames {
            break;
        }

        let position = offset as f32 / frame_count.max(1) as f32;
        let pitch_drop = 1.0 - position.min(1.0) * 0.28;
        let phase = offset as f32 * policy.body_hz * pitch_drop / SAMPLE_RATE as f32;
        let body_envelope = (1.0 - position).clamp(0.0, 1.0).powf(2.15);
        let body = (std::f32::consts::TAU * phase).sin() * body_envelope * policy.gain * accent;
        let click = if offset < 72 {
            let click_position = offset as f32 / 72.0;
            (1.0 - click_position) * policy.click_gain * accent
        } else {
            0.0
        };
        let output_index = frame.saturating_mul(usize::from(CHANNEL_COUNT));
        samples[output_index] += body + click;
        samples[output_index + 1] += body * 0.98 + click;
    }
}

#[derive(Clone, Copy, Debug)]
struct Tr909KickPressureMetrics {
    low_band_rms: f32,
    peak_abs: f32,
}

fn tr909_kick_pressure_metrics(samples: &[f32], grid: &Grid) -> Tr909KickPressureMetrics {
    let low_band = signal_metrics_with_grid(
        &one_pole_lowpass(samples, 120.0),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    );
    let signal = signal_metrics_with_grid(
        samples,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    );
    Tr909KickPressureMetrics {
        low_band_rms: low_band.rms,
        peak_abs: signal.peak_abs,
    }
}

#[allow(dead_code)]
pub(super) fn render_tr909_source_support_legacy(
    grid: &Grid,
    profile: SourceAwareTr909Profile,
) -> Vec<f32> {
    render_tr909_offline(
        &Tr909RenderState {
            mode: Tr909RenderMode::SourceSupport,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: Some(profile.support_profile),
            source_support_context: Some(profile.support_context),
            pattern_adoption: Some(profile.pattern_adoption),
            phrase_variation: Some(profile.phrase_variation),
            drum_bus_level: profile.drum_bus_level,
            slam_intensity: profile.slam_intensity,
            is_transport_running: true,
            tempo_bpm: grid.bpm,
            position_beats: 0.0,
            ..Tr909RenderState::default()
        },
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.total_frames,
    )
}
