use super::{
    config::{CHANNEL_COUNT, PATTERN_ORIGIN_SOURCE_DERIVED, SAMPLE_RATE},
    grid::Grid,
    mc202_low_body_policy::apply_mc202_low_body_emphasis,
    mc202_source_contour::{Mc202SourceContourProfile, mc202_source_low_dominance},
    mc202_source_phrase::{mc202_bass_pressure_state, mc202_source_expression_role},
    render_measurements::{render_metrics, rms_delta},
    source_aware_tr909::SourceAwareTr909Profile,
};
use riotbox_audio::{
    mc202::{
        Mc202ContourHint, Mc202NoteBudget, Mc202PhraseShape, Mc202RenderMode, render_mc202_buffer,
    },
    tr909::Tr909SourceSupportProfile,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Mc202BassPressureProof {
    pub(super) pattern_origin: Mc202PatternOrigin,
    pub(super) applied: bool,
    pub(super) pressure_role: &'static str,
    pub(super) source_expression_render_plan_applied: bool,
    pub(super) source_expression_role: &'static str,
    pub(super) source_failure_fallback: bool,
    pub(super) mode: Mc202RenderMode,
    pub(super) phrase_shape: Mc202PhraseShape,
    pub(super) note_budget: Mc202NoteBudget,
    pub(super) phrase_variation_applied: bool,
    pub(super) distinct_bar_profile_count: usize,
    pub(super) bar_similarity: f32,
    pub(super) identical_bar_run_length: usize,
    pub(super) touch: f32,
    pub(super) music_bus_level: f32,
    pub(super) low_body_emphasis: f32,
    pub(super) pressure_reinforcement_gain: f32,
    pub(super) signal_rms: f32,
    pub(super) low_band_rms: f32,
    pub(super) low_to_mid_energy_ratio: f32,
    pub(super) low_to_high_energy_ratio: f32,
    pub(super) active_sample_ratio: f32,
    pub(super) peak_abs: f32,
    pub(super) reason: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mc202PatternOrigin {
    SourceDerived,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Mc202SourceContourProof {
    pub(super) applied: bool,
    pub(super) contour_hint: Mc202ContourHint,
    pub(super) note_budget: Mc202NoteBudget,
    pub(super) touch_boost: f32,
    pub(super) music_bus_boost: f32,
    pub(super) low_band_energy_ratio: f32,
    pub(super) mid_band_energy_ratio: f32,
    pub(super) high_band_energy_ratio: f32,
    pub(super) event_density_per_bar: f32,
    pub(super) source_contour_delta_rms: f32,
    pub(super) min_required_delta_rms: f32,
    pub(super) reason: &'static str,
}

pub(super) const MC202_BASS_PRESSURE_MAX_BAR_SIMILARITY: f32 = 0.985;

pub(super) const MC202_BASS_PRESSURE_MIN_SIGNAL_RMS: f32 = 0.003;

pub(super) const MC202_BASS_PRESSURE_MIN_LOW_BAND_RMS: f32 = 0.001;

pub(super) const MC202_BASS_PRESSURE_MIN_LOW_TO_MID_ENERGY_RATIO: f32 = 1.20;

const MC202_BASS_PRESSURE_DIAGNOSTIC_MAX_PEAK: f32 = 0.085;

const MC202_DENSE_DROP_DIAGNOSTIC_MAX_PEAK: f32 = 0.050;

const MC202_DENSE_DROP_MIN_EVENTS_PER_BAR: f32 = 70.0;

pub(super) const MC202_SOURCE_CONTOUR_MIN_DELTA_RMS: f32 = 0.00025;

pub(super) const MC202_PRESSURE_ROLE_WITH_SOURCE_CONTOUR: &str =
    "bass_pressure_with_source_contour";

const MC202_PRESSURE_ROLE_WITHOUT_PRESSURE: &str = "bass_phrase_without_pressure";

pub(super) const MC202_REASON_SOURCE_GRID_PROOF_RENDERER: &str = "mc202_source_grid_proof_renderer";

const MC202_REASON_SOURCE_GRID_PROOF_TOO_WEAK: &str = "mc202_source_grid_proof_too_weak";

pub(super) fn render_mc202_bass_pressure_with_source_contour(
    grid: &Grid,
    tr909_profile: SourceAwareTr909Profile,
    source_contour: Mc202SourceContourProfile,
) -> (Vec<f32>, Mc202BassPressureProof, Mc202SourceContourProof) {
    let mut samples = vec![0.0; grid.total_frames * usize::from(CHANNEL_COUNT)];
    let mut control_samples = vec![0.0; grid.total_frames * usize::from(CHANNEL_COUNT)];
    let channel_count = usize::from(CHANNEL_COUNT);
    let primary_state = mc202_bass_pressure_state(grid, tr909_profile, Some(source_contour), 0);

    for bar in 0..grid.bars {
        let start = grid.bar_start_frame(bar).saturating_mul(channel_count);
        let end = grid.bar_end_frame(bar).saturating_mul(channel_count);
        let mut state = mc202_bass_pressure_state(grid, tr909_profile, Some(source_contour), bar);
        state.position_beats = f64::from(bar.saturating_mul(grid.beats_per_bar));
        render_mc202_buffer(&mut samples[start..end], SAMPLE_RATE, channel_count, &state);

        let mut control_state = mc202_bass_pressure_state(grid, tr909_profile, None, bar);
        control_state.position_beats = f64::from(bar.saturating_mul(grid.beats_per_bar));
        render_mc202_buffer(
            &mut control_samples[start..end],
            SAMPLE_RATE,
            channel_count,
            &control_state,
        );
    }
    let pressure_reinforcement_gain =
        mc202_pressure_reinforcement_gain(source_contour, tr909_profile.support_profile);
    let low_body_emphasis = apply_mc202_low_body_emphasis(&mut samples, source_contour);
    add_mc202_pressure_reinforcement(
        &mut samples,
        grid,
        source_contour,
        pressure_reinforcement_gain,
    );
    apply_mc202_diagnostic_headroom(&mut samples, &mut control_samples, source_contour);

    let metrics = render_metrics(&samples, grid);
    let low_band_metrics = metrics.low_band;
    let low_to_mid_energy_ratio = band_ratio(
        metrics.spectral_energy.low_band_energy_ratio,
        metrics.spectral_energy.mid_band_energy_ratio,
    );
    let low_to_high_energy_ratio = band_ratio(
        metrics.spectral_energy.low_band_energy_ratio,
        metrics.spectral_energy.high_band_energy_ratio,
    );
    let phrase_variation_applied = grid.bars > 1;
    let distinct_bar_profile_count = if phrase_variation_applied { 2 } else { 1 };
    let source_expression_render_plan_applied = primary_state
        .source_phrase_plan
        .is_some_and(|plan| !plan.is_empty());
    let applied = metrics.signal.rms >= MC202_BASS_PRESSURE_MIN_SIGNAL_RMS
        && metrics.low_band.rms >= MC202_BASS_PRESSURE_MIN_LOW_BAND_RMS
        && low_to_mid_energy_ratio >= MC202_BASS_PRESSURE_MIN_LOW_TO_MID_ENERGY_RATIO
        && pressure_reinforcement_gain > 0.0
        && source_expression_render_plan_applied
        && metrics.signal.peak_abs > 0.0;
    let active_sample_ratio = if samples.is_empty() {
        0.0
    } else {
        metrics.signal.active_samples as f32 / samples.len() as f32
    };
    let source_contour_delta_rms = rms_delta(&samples, &control_samples, grid);
    let source_contour_applied = source_contour_delta_rms >= MC202_SOURCE_CONTOUR_MIN_DELTA_RMS;
    let pattern_origin =
        if applied && source_expression_render_plan_applied && source_contour_applied {
            Mc202PatternOrigin::SourceDerived
        } else {
            Mc202PatternOrigin::Unavailable
        };

    (
        samples,
        Mc202BassPressureProof {
            pattern_origin,
            applied,
            pressure_role: if applied {
                MC202_PRESSURE_ROLE_WITH_SOURCE_CONTOUR
            } else {
                MC202_PRESSURE_ROLE_WITHOUT_PRESSURE
            },
            source_expression_render_plan_applied,
            source_expression_role: mc202_source_expression_role(source_contour),
            source_failure_fallback: false,
            mode: primary_state.mode,
            phrase_shape: primary_state.phrase_shape,
            note_budget: primary_state.note_budget,
            phrase_variation_applied,
            distinct_bar_profile_count,
            bar_similarity: metrics.bar_variation.bar_similarity,
            identical_bar_run_length: metrics.bar_variation.identical_bar_run_length,
            touch: primary_state.touch,
            music_bus_level: primary_state.music_bus_level,
            low_body_emphasis,
            pressure_reinforcement_gain,
            signal_rms: metrics.signal.rms,
            low_band_rms: low_band_metrics.rms,
            low_to_mid_energy_ratio,
            low_to_high_energy_ratio,
            active_sample_ratio,
            peak_abs: metrics.signal.peak_abs,
            reason: if applied {
                MC202_REASON_SOURCE_GRID_PROOF_RENDERER
            } else {
                MC202_REASON_SOURCE_GRID_PROOF_TOO_WEAK
            },
        },
        Mc202SourceContourProof {
            applied: source_contour_applied,
            contour_hint: source_contour.contour_hint,
            note_budget: source_contour.note_budget,
            touch_boost: source_contour.touch_boost,
            music_bus_boost: source_contour.music_bus_boost,
            low_band_energy_ratio: source_contour.low_band_energy_ratio,
            mid_band_energy_ratio: source_contour.mid_band_energy_ratio,
            high_band_energy_ratio: source_contour.high_band_energy_ratio,
            event_density_per_bar: source_contour.event_density_per_bar,
            source_contour_delta_rms,
            min_required_delta_rms: MC202_SOURCE_CONTOUR_MIN_DELTA_RMS,
            reason: if source_contour_applied {
                source_contour.reason
            } else {
                "mc202_source_contour_too_weak"
            },
        },
    )
}

fn apply_mc202_diagnostic_headroom(
    samples: &mut [f32],
    control_samples: &mut [f32],
    source_contour: Mc202SourceContourProfile,
) {
    let max_peak = if source_contour.contour_hint == Mc202ContourHint::Drop
        && source_contour.event_density_per_bar >= MC202_DENSE_DROP_MIN_EVENTS_PER_BAR
    {
        MC202_DENSE_DROP_DIAGNOSTIC_MAX_PEAK
    } else {
        MC202_BASS_PRESSURE_DIAGNOSTIC_MAX_PEAK
    };
    let peak = samples.iter().copied().map(f32::abs).fold(0.0, f32::max);
    if peak <= max_peak || peak <= f32::EPSILON {
        return;
    }

    let gain = max_peak / peak;
    for sample in samples.iter_mut().chain(control_samples.iter_mut()) {
        *sample *= gain;
    }
}

fn band_ratio(numerator: f32, denominator: f32) -> f32 {
    numerator / denominator.max(0.000_001)
}

fn mc202_pressure_reinforcement_gain(
    source_contour: Mc202SourceContourProfile,
    support_profile: Tr909SourceSupportProfile,
) -> f32 {
    let low_dominance = mc202_source_low_dominance(source_contour);
    let profile_gain = match support_profile {
        Tr909SourceSupportProfile::DropDrive => 0.024,
        Tr909SourceSupportProfile::BreakLift => 0.014,
        Tr909SourceSupportProfile::SteadyPulse => 0.018,
    };
    let contour_gain = match source_contour.contour_hint {
        Mc202ContourHint::Drop => 0.018,
        Mc202ContourHint::Lift => 0.010,
        Mc202ContourHint::Hold | Mc202ContourHint::Neutral => 0.020,
    };

    (profile_gain + contour_gain + low_dominance * 0.032).clamp(0.010, 0.074)
}

fn add_mc202_pressure_reinforcement(
    samples: &mut [f32],
    grid: &Grid,
    source_contour: Mc202SourceContourProfile,
    gain: f32,
) {
    if gain <= 0.0 {
        return;
    }

    let channel_count = usize::from(CHANNEL_COUNT);
    let sample_rate = SAMPLE_RATE as f32;
    let beat_frames = sample_rate * 60.0 / grid.bpm.max(1.0);
    let base_frequency_hz = match source_contour.contour_hint {
        Mc202ContourHint::Drop => 38.75 + (1.0 - source_contour.low_band_energy_ratio) * 5.25,
        Mc202ContourHint::Lift => 55.00,
        Mc202ContourHint::Hold | Mc202ContourHint::Neutral => 49.00,
    };
    let low_source_weight = source_contour.low_band_energy_ratio.clamp(0.0, 1.0);

    for bar in 0..grid.bars {
        let bar_start_frame = grid.bar_start_frame(bar);
        let bar_end_frame = grid.bar_end_frame(bar);
        let bar_frames = bar_end_frame.saturating_sub(bar_start_frame);
        if bar_frames == 0 {
            continue;
        }

        for frame in 0..bar_frames {
            let beat_in_bar = frame as f32 / beat_frames;
            let pressure_envelope = pressure_pulse_envelope(beat_in_bar, low_source_weight);
            if pressure_envelope <= 0.0 {
                continue;
            }

            let phase =
                (frame as f32 / sample_rate * base_frequency_hz * std::f32::consts::TAU).sin();
            let bar_push = if bar.is_multiple_of(2) { 1.0 } else { 0.82 };
            let sample = (phase * pressure_envelope * gain * bar_push).tanh();
            let frame_start = (bar_start_frame + frame) * channel_count;
            for channel in 0..channel_count {
                samples[frame_start + channel] =
                    (samples[frame_start + channel] + sample).clamp(-0.98, 0.98);
            }
        }
    }
}

fn pressure_pulse_envelope(beat_in_bar: f32, low_source_weight: f32) -> f32 {
    const PULSES: [f32; 4] = [0.0, 1.5, 2.0, 3.5];
    PULSES
        .iter()
        .enumerate()
        .filter_map(|(index, pulse_beat)| {
            let distance = (beat_in_bar - pulse_beat).abs();
            let width = if index.is_multiple_of(2) { 0.44 } else { 0.26 };
            if distance > width {
                return None;
            }
            let strength = if index.is_multiple_of(2) {
                1.0
            } else {
                0.38 + low_source_weight * 0.22
            };
            Some((1.0 - distance / width).powf(2.4) * strength)
        })
        .fold(0.0, f32::max)
}

impl Mc202PatternOrigin {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::SourceDerived => PATTERN_ORIGIN_SOURCE_DERIVED,
            Self::Unavailable => "unavailable",
        }
    }
}
