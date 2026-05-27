#[derive(Clone, Copy, Debug, PartialEq)]
struct Mc202BassPressureProof {
    applied: bool,
    mode: Mc202RenderMode,
    phrase_shape: Mc202PhraseShape,
    note_budget: Mc202NoteBudget,
    phrase_variation_applied: bool,
    distinct_bar_profile_count: usize,
    bar_similarity: f32,
    identical_bar_run_length: usize,
    touch: f32,
    music_bus_level: f32,
    signal_rms: f32,
    low_band_rms: f32,
    active_sample_ratio: f32,
    peak_abs: f32,
    reason: &'static str,
}

#[derive(Serialize)]
struct ManifestMc202BassPressureProof {
    pattern_origin: &'static str,
    applied: bool,
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
    signal_rms: f32,
    low_band_rms: f32,
    active_sample_ratio: f32,
    peak_abs: f32,
    reason: &'static str,
}

const MC202_BASS_PRESSURE_MAX_BAR_SIMILARITY: f32 = 0.985;

fn manifest_mc202_bass_pressure_proof(
    proof: Mc202BassPressureProof,
) -> ManifestMc202BassPressureProof {
    ManifestMc202BassPressureProof {
        pattern_origin: "primitive_renderer",
        applied: proof.applied,
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
        signal_rms: proof.signal_rms,
        low_band_rms: proof.low_band_rms,
        active_sample_ratio: proof.active_sample_ratio,
        peak_abs: proof.peak_abs,
        reason: proof.reason,
    }
}

fn render_mc202_bass_pressure(
    grid: &Grid,
    tr909_profile: SourceAwareTr909Profile,
) -> (Vec<f32>, Mc202BassPressureProof) {
    let mut samples = vec![0.0; grid.total_frames * usize::from(CHANNEL_COUNT)];
    let channel_count = usize::from(CHANNEL_COUNT);
    let primary_state = mc202_bass_pressure_state(grid, tr909_profile, 0);

    for bar in 0..grid.bars {
        let start = grid.bar_start_frame(bar).saturating_mul(channel_count);
        let end = grid.bar_end_frame(bar).saturating_mul(channel_count);
        let mut state = mc202_bass_pressure_state(grid, tr909_profile, bar);
        state.position_beats = f64::from(bar.saturating_mul(grid.beats_per_bar));
        render_mc202_buffer(&mut samples[start..end], SAMPLE_RATE, channel_count, &state);
    }

    let metrics = render_metrics(&samples, grid);
    let low_band_metrics = metrics.low_band;
    let phrase_variation_applied = grid.bars > 1;
    let distinct_bar_profile_count = if phrase_variation_applied { 2 } else { 1 };
    let applied =
        metrics.signal.rms > MIN_SIGNAL_RMS && metrics.low_band.rms > 0.0 && metrics.signal.peak_abs > 0.0;
    let active_sample_ratio = if samples.is_empty() {
        0.0
    } else {
        metrics.signal.active_samples as f32 / samples.len() as f32
    };

    (
        samples,
        Mc202BassPressureProof {
            applied,
            mode: primary_state.mode,
            phrase_shape: primary_state.phrase_shape,
            note_budget: primary_state.note_budget,
            phrase_variation_applied,
            distinct_bar_profile_count,
            bar_similarity: metrics.bar_variation.bar_similarity,
            identical_bar_run_length: metrics.bar_variation.identical_bar_run_length,
            touch: primary_state.touch,
            music_bus_level: primary_state.music_bus_level,
            signal_rms: metrics.signal.rms,
            low_band_rms: low_band_metrics.rms,
            active_sample_ratio,
            peak_abs: metrics.signal.peak_abs,
            reason: if applied {
                "mc202_source_grid_proof_renderer"
            } else {
                "mc202_source_grid_proof_too_weak"
            },
        },
    )
}

fn mc202_bass_pressure_state(
    grid: &Grid,
    profile: SourceAwareTr909Profile,
    bar: u32,
) -> Mc202RenderState {
    let (mode, primary_shape, note_budget, touch, music_bus_level, contour_hint) =
        match profile.support_profile {
            Tr909SourceSupportProfile::DropDrive => (
                Mc202RenderMode::Pressure,
                Mc202PhraseShape::FollowerDrive,
                Mc202NoteBudget::Balanced,
                0.54,
                0.30,
                Mc202ContourHint::Drop,
            ),
            Tr909SourceSupportProfile::BreakLift => (
                Mc202RenderMode::Follower,
                Mc202PhraseShape::FollowerDrive,
                Mc202NoteBudget::Sparse,
                0.48,
                0.26,
                Mc202ContourHint::Lift,
            ),
            Tr909SourceSupportProfile::SteadyPulse => (
                Mc202RenderMode::Follower,
                Mc202PhraseShape::RootPulse,
                Mc202NoteBudget::Balanced,
                0.44,
                0.24,
                Mc202ContourHint::Neutral,
            ),
        };
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
