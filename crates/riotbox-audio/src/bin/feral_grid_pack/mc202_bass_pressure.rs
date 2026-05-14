#[derive(Clone, Copy, Debug, PartialEq)]
struct Mc202BassPressureProof {
    applied: bool,
    mode: Mc202RenderMode,
    phrase_shape: Mc202PhraseShape,
    note_budget: Mc202NoteBudget,
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
    applied: bool,
    mode: &'static str,
    phrase_shape: &'static str,
    note_budget: &'static str,
    touch: f32,
    music_bus_level: f32,
    signal_rms: f32,
    low_band_rms: f32,
    active_sample_ratio: f32,
    peak_abs: f32,
    reason: &'static str,
}

const MC202_BASS_PRESSURE_MIN_RMS: f32 = 0.003;
const MC202_BASS_PRESSURE_MIN_LOW_BAND_RMS: f32 = 0.001;
const MC202_BASS_PRESSURE_MAX_PEAK_ABS: f32 = 0.95;

fn manifest_mc202_bass_pressure_proof(
    proof: Mc202BassPressureProof,
) -> ManifestMc202BassPressureProof {
    ManifestMc202BassPressureProof {
        applied: proof.applied,
        mode: proof.mode.label(),
        phrase_shape: proof.phrase_shape.label(),
        note_budget: proof.note_budget.label(),
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
    let state = mc202_bass_pressure_state(grid, tr909_profile);
    let samples = render_mc202_offline(&state, SAMPLE_RATE, CHANNEL_COUNT, grid.total_frames);
    let metrics = signal_metrics_with_grid(
        &samples,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    );
    let low_band = signal_metrics_with_grid(
        &one_pole_lowpass(&samples, 165.0),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    );
    let applied = metrics.rms >= MC202_BASS_PRESSURE_MIN_RMS
        && low_band.rms >= MC202_BASS_PRESSURE_MIN_LOW_BAND_RMS
        && metrics.peak_abs <= MC202_BASS_PRESSURE_MAX_PEAK_ABS;
    let reason = if applied {
        mc202_bass_pressure_reason(tr909_profile)
    } else {
        "mc202_bass_pressure_too_weak"
    };

    (
        samples,
        Mc202BassPressureProof {
            applied,
            mode: state.mode,
            phrase_shape: state.phrase_shape,
            note_budget: state.note_budget,
            touch: state.touch,
            music_bus_level: state.music_bus_level,
            signal_rms: metrics.rms,
            low_band_rms: low_band.rms,
            active_sample_ratio: metrics.active_sample_ratio,
            peak_abs: metrics.peak_abs,
            reason,
        },
    )
}

fn mc202_bass_pressure_state(
    grid: &Grid,
    tr909_profile: SourceAwareTr909Profile,
) -> Mc202RenderState {
    let (mode, phrase_shape, note_budget, contour_hint, hook_response, touch, music_bus_level) =
        match tr909_profile.support_profile {
            Tr909SourceSupportProfile::DropDrive => (
                Mc202RenderMode::Pressure,
                Mc202PhraseShape::PressureCell,
                Mc202NoteBudget::Sparse,
                Mc202ContourHint::Hold,
                Mc202HookResponse::AnswerSpace,
                0.90,
                0.88,
            ),
            Tr909SourceSupportProfile::BreakLift => (
                Mc202RenderMode::Follower,
                Mc202PhraseShape::MutatedDrive,
                Mc202NoteBudget::Push,
                Mc202ContourHint::Lift,
                Mc202HookResponse::AnswerSpace,
                0.86,
                0.84,
            ),
            Tr909SourceSupportProfile::SteadyPulse => (
                Mc202RenderMode::Follower,
                Mc202PhraseShape::FollowerDrive,
                Mc202NoteBudget::Balanced,
                Mc202ContourHint::Neutral,
                Mc202HookResponse::Direct,
                0.82,
                0.80,
            ),
        };

    Mc202RenderState {
        mode,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape,
        note_budget,
        contour_hint,
        hook_response,
        touch,
        music_bus_level,
        tempo_bpm: grid.bpm,
        position_beats: 0.0,
        is_transport_running: true,
    }
}

fn mc202_bass_pressure_reason(profile: SourceAwareTr909Profile) -> &'static str {
    match profile.support_profile {
        Tr909SourceSupportProfile::DropDrive => "mc202_sparse_pressure_cell",
        Tr909SourceSupportProfile::BreakLift => "mc202_mutated_lift_pressure",
        Tr909SourceSupportProfile::SteadyPulse => "mc202_follower_support_pressure",
    }
}
