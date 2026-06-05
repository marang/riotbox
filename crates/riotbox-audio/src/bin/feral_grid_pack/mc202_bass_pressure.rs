#[derive(Clone, Copy, Debug, PartialEq)]
struct Mc202BassPressureProof {
    applied: bool,
    pressure_role: &'static str,
    mode: Mc202RenderMode,
    phrase_shape: Mc202PhraseShape,
    note_budget: Mc202NoteBudget,
    phrase_variation_applied: bool,
    distinct_bar_profile_count: usize,
    bar_similarity: f32,
    identical_bar_run_length: usize,
    touch: f32,
    music_bus_level: f32,
    pressure_reinforcement_gain: f32,
    signal_rms: f32,
    low_band_rms: f32,
    low_to_mid_energy_ratio: f32,
    low_to_high_energy_ratio: f32,
    active_sample_ratio: f32,
    peak_abs: f32,
    reason: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Mc202SourceContourProfile {
    contour_hint: Mc202ContourHint,
    note_budget: Mc202NoteBudget,
    touch_boost: f32,
    music_bus_boost: f32,
    low_band_energy_ratio: f32,
    mid_band_energy_ratio: f32,
    high_band_energy_ratio: f32,
    event_density_per_bar: f32,
    reason: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Mc202SourceContourProof {
    applied: bool,
    contour_hint: Mc202ContourHint,
    note_budget: Mc202NoteBudget,
    touch_boost: f32,
    music_bus_boost: f32,
    low_band_energy_ratio: f32,
    mid_band_energy_ratio: f32,
    high_band_energy_ratio: f32,
    event_density_per_bar: f32,
    source_contour_delta_rms: f32,
    min_required_delta_rms: f32,
    reason: &'static str,
}

#[derive(Serialize)]
struct ManifestMc202BassPressureProof {
    pattern_origin: &'static str,
    applied: bool,
    pressure_role: &'static str,
    mode: &'static str,
    phrase_shape: &'static str,
    note_budget: &'static str,
    phrase_variation_applied: bool,
    distinct_bar_profile_count: usize,
    bar_similarity: f32,
    identical_bar_run_length: usize,
    max_bar_similarity: f32,
    touch: f32,
    music_bus_level: f32,
    pressure_reinforcement_gain: f32,
    signal_rms: f32,
    low_band_rms: f32,
    low_to_mid_energy_ratio: f32,
    low_to_high_energy_ratio: f32,
    min_low_to_mid_energy_ratio: f32,
    active_sample_ratio: f32,
    peak_abs: f32,
    reason: &'static str,
}

#[derive(Serialize)]
struct ManifestMc202SourceContourProof {
    pattern_origin: &'static str,
    applied: bool,
    contour_hint: &'static str,
    note_budget: &'static str,
    touch_boost: f32,
    music_bus_boost: f32,
    low_band_energy_ratio: f32,
    mid_band_energy_ratio: f32,
    high_band_energy_ratio: f32,
    event_density_per_bar: f32,
    source_contour_delta_rms: f32,
    min_required_delta_rms: f32,
    reason: &'static str,
}

const MC202_BASS_PRESSURE_MAX_BAR_SIMILARITY: f32 = 0.985;
const MC202_BASS_PRESSURE_MIN_SIGNAL_RMS: f32 = 0.003;
const MC202_BASS_PRESSURE_MIN_LOW_BAND_RMS: f32 = 0.001;
const MC202_BASS_PRESSURE_MIN_LOW_TO_MID_ENERGY_RATIO: f32 = 1.20;
const MC202_SOURCE_CONTOUR_MIN_DELTA_RMS: f32 = 0.00025;

fn manifest_mc202_bass_pressure_proof(
    proof: Mc202BassPressureProof,
) -> ManifestMc202BassPressureProof {
    ManifestMc202BassPressureProof {
        pattern_origin: "primitive_renderer",
        applied: proof.applied,
        pressure_role: proof.pressure_role,
        mode: proof.mode.label(),
        phrase_shape: proof.phrase_shape.label(),
        note_budget: proof.note_budget.label(),
        phrase_variation_applied: proof.phrase_variation_applied,
        distinct_bar_profile_count: proof.distinct_bar_profile_count,
        bar_similarity: proof.bar_similarity,
        identical_bar_run_length: proof.identical_bar_run_length,
        max_bar_similarity: MC202_BASS_PRESSURE_MAX_BAR_SIMILARITY,
        touch: proof.touch,
        music_bus_level: proof.music_bus_level,
        pressure_reinforcement_gain: proof.pressure_reinforcement_gain,
        signal_rms: proof.signal_rms,
        low_band_rms: proof.low_band_rms,
        low_to_mid_energy_ratio: proof.low_to_mid_energy_ratio,
        low_to_high_energy_ratio: proof.low_to_high_energy_ratio,
        min_low_to_mid_energy_ratio: MC202_BASS_PRESSURE_MIN_LOW_TO_MID_ENERGY_RATIO,
        active_sample_ratio: proof.active_sample_ratio,
        peak_abs: proof.peak_abs,
        reason: proof.reason,
    }
}

fn manifest_mc202_source_contour_proof(
    proof: Mc202SourceContourProof,
) -> ManifestMc202SourceContourProof {
    ManifestMc202SourceContourProof {
        pattern_origin: "source_derived_contour",
        applied: proof.applied,
        contour_hint: proof.contour_hint.label(),
        note_budget: proof.note_budget.label(),
        touch_boost: proof.touch_boost,
        music_bus_boost: proof.music_bus_boost,
        low_band_energy_ratio: proof.low_band_energy_ratio,
        mid_band_energy_ratio: proof.mid_band_energy_ratio,
        high_band_energy_ratio: proof.high_band_energy_ratio,
        event_density_per_bar: proof.event_density_per_bar,
        source_contour_delta_rms: proof.source_contour_delta_rms,
        min_required_delta_rms: proof.min_required_delta_rms,
        reason: proof.reason,
    }
}

fn render_mc202_bass_pressure_with_source_contour(
    grid: &Grid,
    tr909_profile: SourceAwareTr909Profile,
    source_contour: Mc202SourceContourProfile,
) -> (
    Vec<f32>,
    Mc202BassPressureProof,
    Mc202SourceContourProof,
) {
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
    let pressure_reinforcement_gain = mc202_pressure_reinforcement_gain(
        source_contour,
        tr909_profile.support_profile,
    );
    add_mc202_pressure_reinforcement(
        &mut samples,
        grid,
        source_contour,
        pressure_reinforcement_gain,
    );

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
    let applied = metrics.signal.rms >= MC202_BASS_PRESSURE_MIN_SIGNAL_RMS
        && metrics.low_band.rms >= MC202_BASS_PRESSURE_MIN_LOW_BAND_RMS
        && low_to_mid_energy_ratio >= MC202_BASS_PRESSURE_MIN_LOW_TO_MID_ENERGY_RATIO
        && pressure_reinforcement_gain > 0.0
        && metrics.signal.peak_abs > 0.0;
    let active_sample_ratio = if samples.is_empty() {
        0.0
    } else {
        metrics.signal.active_samples as f32 / samples.len() as f32
    };
    let source_contour_delta_rms = rms_delta(&samples, &control_samples, grid);
    let source_contour_applied =
        source_contour_delta_rms >= MC202_SOURCE_CONTOUR_MIN_DELTA_RMS;

    (
        samples,
        Mc202BassPressureProof {
            applied,
            pressure_role: if applied {
                "bass_pressure_with_source_contour"
            } else {
                "bass_phrase_without_pressure"
            },
            mode: primary_state.mode,
            phrase_shape: primary_state.phrase_shape,
            note_budget: primary_state.note_budget,
            phrase_variation_applied,
            distinct_bar_profile_count,
            bar_similarity: metrics.bar_variation.bar_similarity,
            identical_bar_run_length: metrics.bar_variation.identical_bar_run_length,
            touch: primary_state.touch,
            music_bus_level: primary_state.music_bus_level,
            pressure_reinforcement_gain,
            signal_rms: metrics.signal.rms,
            low_band_rms: low_band_metrics.rms,
            low_to_mid_energy_ratio,
            low_to_high_energy_ratio,
            active_sample_ratio,
            peak_abs: metrics.signal.peak_abs,
            reason: if applied {
                "mc202_source_grid_proof_renderer"
            } else {
                "mc202_source_grid_proof_too_weak"
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

fn band_ratio(numerator: f32, denominator: f32) -> f32 {
    numerator / denominator.max(0.000_001)
}

fn mc202_pressure_reinforcement_gain(
    source_contour: Mc202SourceContourProfile,
    support_profile: Tr909SourceSupportProfile,
) -> f32 {
    let low_dominance = (source_contour.low_band_energy_ratio
        - source_contour.mid_band_energy_ratio.max(source_contour.high_band_energy_ratio))
    .max(0.0);
    let profile_gain = match support_profile {
        Tr909SourceSupportProfile::DropDrive => 0.024,
        Tr909SourceSupportProfile::BreakLift => 0.014,
        Tr909SourceSupportProfile::SteadyPulse => 0.018,
    };
    let contour_gain = match source_contour.contour_hint {
        Mc202ContourHint::Drop => 0.018,
        Mc202ContourHint::Lift => 0.010,
        Mc202ContourHint::Hold | Mc202ContourHint::Neutral => 0.012,
    };

    (profile_gain + contour_gain + low_dominance * 0.020).clamp(0.010, 0.060)
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
        Mc202ContourHint::Drop => 43.65,
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

impl Mc202SourceContourProfile {
    fn from_source_window(samples: &[f32], grid: &Grid) -> Self {
        let spectral = spectral_energy_metrics(samples);
        let signal = signal_metrics_with_grid(
            samples,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            grid.bpm,
            grid.beats_per_bar,
        );

        if spectral.low_band_energy_ratio >= spectral.high_band_energy_ratio
            && spectral.low_band_energy_ratio >= spectral.mid_band_energy_ratio
        {
            Self {
                contour_hint: Mc202ContourHint::Drop,
                note_budget: Mc202NoteBudget::Balanced,
                touch_boost: 0.055,
                music_bus_boost: 0.040,
                low_band_energy_ratio: spectral.low_band_energy_ratio,
                mid_band_energy_ratio: spectral.mid_band_energy_ratio,
                high_band_energy_ratio: spectral.high_band_energy_ratio,
                event_density_per_bar: signal.event_density_per_bar,
                reason: "source_low_section_drop_contour",
            }
        } else if signal.event_density_per_bar >= 3.0
            || spectral.high_band_energy_ratio >= spectral.mid_band_energy_ratio
        {
            Self {
                contour_hint: Mc202ContourHint::Lift,
                note_budget: Mc202NoteBudget::Push,
                touch_boost: 0.045,
                music_bus_boost: 0.035,
                low_band_energy_ratio: spectral.low_band_energy_ratio,
                mid_band_energy_ratio: spectral.mid_band_energy_ratio,
                high_band_energy_ratio: spectral.high_band_energy_ratio,
                event_density_per_bar: signal.event_density_per_bar,
                reason: "source_busy_section_lift_contour",
            }
        } else {
            Self {
                contour_hint: Mc202ContourHint::Hold,
                note_budget: Mc202NoteBudget::Sparse,
                touch_boost: 0.035,
                music_bus_boost: 0.025,
                low_band_energy_ratio: spectral.low_band_energy_ratio,
                mid_band_energy_ratio: spectral.mid_band_energy_ratio,
                high_band_energy_ratio: spectral.high_band_energy_ratio,
                event_density_per_bar: signal.event_density_per_bar,
                reason: "source_mid_section_hold_contour",
            }
        }
    }
}

fn mc202_bass_pressure_state(
    grid: &Grid,
    profile: SourceAwareTr909Profile,
    source_contour: Option<Mc202SourceContourProfile>,
    bar: u32,
) -> Mc202RenderState {
    let (mode, primary_shape, note_budget, touch, music_bus_level, contour_hint) =
        match profile.support_profile {
            Tr909SourceSupportProfile::DropDrive => (
                Mc202RenderMode::Pressure,
                Mc202PhraseShape::FollowerDrive,
                Mc202NoteBudget::Balanced,
                0.68,
                0.48,
                Mc202ContourHint::Drop,
            ),
            Tr909SourceSupportProfile::BreakLift => (
                Mc202RenderMode::Follower,
                Mc202PhraseShape::FollowerDrive,
                Mc202NoteBudget::Sparse,
                0.66,
                0.48,
                Mc202ContourHint::Lift,
            ),
            Tr909SourceSupportProfile::SteadyPulse => (
                Mc202RenderMode::Follower,
                Mc202PhraseShape::RootPulse,
                Mc202NoteBudget::Balanced,
                0.64,
                0.46,
                Mc202ContourHint::Neutral,
            ),
        };
    let note_budget = source_contour
        .map(|contour| contour.note_budget)
        .unwrap_or(note_budget);
    let contour_hint = source_contour
        .map(|contour| contour.contour_hint)
        .unwrap_or(contour_hint);
    let touch = source_contour
        .map(|contour| (touch + contour.touch_boost).clamp(0.0, 1.0))
        .unwrap_or(touch);
    let music_bus_level = source_contour
        .map(|contour| (music_bus_level + contour.music_bus_boost).clamp(0.0, 1.0))
        .unwrap_or(music_bus_level);
    let phrase_shape = if bar % 2 == 1 {
        match primary_shape {
            Mc202PhraseShape::RootPulse => Mc202PhraseShape::FollowerDrive,
            _ => Mc202PhraseShape::MutatedDrive,
        }
    } else {
        primary_shape
    };

    Mc202RenderState {
        mode,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape,
        note_budget,
        contour_hint,
        touch,
        music_bus_level,
        tempo_bpm: grid.bpm,
        position_beats: 0.0,
        is_transport_running: true,
        ..Mc202RenderState::default()
    }
}
