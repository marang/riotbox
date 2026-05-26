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
        pattern_origin: "compatibility_silent",
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
    _tr909_profile: SourceAwareTr909Profile,
) -> (Vec<f32>, Mc202BassPressureProof) {
    let samples = vec![0.0; grid.total_frames * usize::from(CHANNEL_COUNT)];

    (
        samples,
        Mc202BassPressureProof {
            applied: false,
            mode: Mc202RenderMode::Idle,
            phrase_shape: Mc202PhraseShape::RootPulse,
            note_budget: Mc202NoteBudget::Balanced,
            phrase_variation_applied: false,
            distinct_bar_profile_count: 0,
            bar_similarity: 1.0,
            identical_bar_run_length: grid.bars as usize,
            touch: 0.0,
            music_bus_level: 0.0,
            signal_rms: 0.0,
            low_band_rms: 0.0,
            active_sample_ratio: 0.0,
            peak_abs: 0.0,
            reason: "mc202_bass_pressure_removed_pending_source_derived_phrase_planner",
        },
    )
}
