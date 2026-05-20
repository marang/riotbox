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

const MC202_BASS_PRESSURE_MIN_RMS: f32 = 0.003;
const MC202_BASS_PRESSURE_MIN_LOW_BAND_RMS: f32 = 0.001;
const MC202_BASS_PRESSURE_MAX_PEAK_ABS: f32 = 0.95;
const MC202_BASS_PRESSURE_MIN_DISTINCT_BAR_PROFILES: usize = 2;
const MC202_BASS_PRESSURE_MAX_BAR_SIMILARITY: f32 = 0.985;

fn manifest_mc202_bass_pressure_proof(
    proof: Mc202BassPressureProof,
) -> ManifestMc202BassPressureProof {
    ManifestMc202BassPressureProof {
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
    let plan = mc202_bass_pressure_plan(grid, tr909_profile);
    let samples = render_mc202_bass_pressure_plan(grid, &plan);
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
    let bar_variation = bar_variation_metrics(&samples, grid);
    let distinct_bar_profile_count = mc202_distinct_bar_profile_count(&plan);
    let phrase_variation_applied = distinct_bar_profile_count >= MC202_BASS_PRESSURE_MIN_DISTINCT_BAR_PROFILES
        && bar_variation.bar_similarity <= MC202_BASS_PRESSURE_MAX_BAR_SIMILARITY
        && bar_variation.identical_bar_run_length < grid.bars as usize;
    let applied = metrics.rms >= MC202_BASS_PRESSURE_MIN_RMS
        && low_band.rms >= MC202_BASS_PRESSURE_MIN_LOW_BAND_RMS
        && metrics.peak_abs <= MC202_BASS_PRESSURE_MAX_PEAK_ABS
        && phrase_variation_applied;
    let reason = if applied {
        mc202_bass_pressure_reason(tr909_profile)
    } else if !phrase_variation_applied {
        "mc202_bass_pressure_too_static"
    } else {
        "mc202_bass_pressure_too_weak"
    };
    let primary_state = plan[0];

    (
        samples,
        Mc202BassPressureProof {
            applied,
            mode: primary_state.mode,
            phrase_shape: primary_state.phrase_shape,
            note_budget: primary_state.note_budget,
            phrase_variation_applied,
            distinct_bar_profile_count,
            bar_similarity: bar_variation.bar_similarity,
            identical_bar_run_length: bar_variation.identical_bar_run_length,
            touch: primary_state.touch,
            music_bus_level: primary_state.music_bus_level,
            signal_rms: metrics.rms,
            low_band_rms: low_band.rms,
            active_sample_ratio: metrics.active_sample_ratio,
            peak_abs: metrics.peak_abs,
            reason,
        },
    )
}

fn mc202_bass_pressure_plan(
    grid: &Grid,
    tr909_profile: SourceAwareTr909Profile,
) -> Vec<Mc202RenderState> {
    (0..grid.bars)
        .map(|bar| mc202_bass_pressure_state_for_bar(grid, tr909_profile, bar))
        .collect()
}

fn render_mc202_bass_pressure_plan(grid: &Grid, plan: &[Mc202RenderState]) -> Vec<f32> {
    let mut output = vec![0.0; grid.total_frames * usize::from(CHANNEL_COUNT)];
    let channel_count = usize::from(CHANNEL_COUNT);

    for (bar, state) in plan.iter().enumerate() {
        let bar = bar as u32;
        let start_frame = grid.bar_start_frame(bar);
        let end_frame = grid.bar_end_frame(bar).min(grid.total_frames);
        let frame_count = end_frame.saturating_sub(start_frame);
        if frame_count == 0 {
            continue;
        }

        let rendered = render_mc202_offline(state, SAMPLE_RATE, CHANNEL_COUNT, frame_count);
        let start = start_frame.saturating_mul(channel_count);
        let end = start.saturating_add(rendered.len()).min(output.len());
        output[start..end].copy_from_slice(&rendered[..end.saturating_sub(start)]);
    }

    output
}

fn mc202_distinct_bar_profile_count(plan: &[Mc202RenderState]) -> usize {
    plan.iter()
        .map(mc202_bar_profile_key)
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

fn mc202_bar_profile_key(
    state: &Mc202RenderState,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    (
        state.mode.label(),
        state.phrase_shape.label(),
        state.note_budget.label(),
        state.contour_hint.label(),
        state.hook_response.label(),
    )
}

fn mc202_bass_pressure_state_for_bar(
    grid: &Grid,
    tr909_profile: SourceAwareTr909Profile,
    bar: u32,
) -> Mc202RenderState {
    let (mode, phrase_shape, note_budget, contour_hint, hook_response, touch, music_bus_level) =
        match (tr909_profile.support_profile, bar % 4) {
            (Tr909SourceSupportProfile::DropDrive, 1) => mc202_bar_profile(
                Mc202RenderMode::Pressure,
                Mc202PhraseShape::FollowerDrive,
                Mc202NoteBudget::Sparse,
                Mc202ContourHint::Drop,
                Mc202HookResponse::Direct,
                0.86,
                0.82,
            ),
            (Tr909SourceSupportProfile::DropDrive, 3) => mc202_bar_profile(
                Mc202RenderMode::Pressure,
                Mc202PhraseShape::MutatedDrive,
                Mc202NoteBudget::Sparse,
                Mc202ContourHint::Hold,
                Mc202HookResponse::AnswerSpace,
                0.88,
                0.84,
            ),
            (Tr909SourceSupportProfile::DropDrive, _) => mc202_bar_profile(
                Mc202RenderMode::Pressure,
                Mc202PhraseShape::PressureCell,
                Mc202NoteBudget::Sparse,
                Mc202ContourHint::Hold,
                Mc202HookResponse::AnswerSpace,
                0.90,
                0.88,
            ),
            (Tr909SourceSupportProfile::BreakLift, 1 | 2) => mc202_bar_profile(
                Mc202RenderMode::Follower,
                Mc202PhraseShape::FollowerDrive,
                Mc202NoteBudget::Balanced,
                Mc202ContourHint::Lift,
                Mc202HookResponse::Direct,
                0.84,
                0.82,
            ),
            (Tr909SourceSupportProfile::BreakLift, _) => mc202_bar_profile(
                Mc202RenderMode::Follower,
                Mc202PhraseShape::MutatedDrive,
                Mc202NoteBudget::Push,
                Mc202ContourHint::Lift,
                Mc202HookResponse::AnswerSpace,
                0.86,
                0.84,
            ),
            (Tr909SourceSupportProfile::SteadyPulse, 2) => mc202_bar_profile(
                Mc202RenderMode::Pressure,
                Mc202PhraseShape::PressureCell,
                Mc202NoteBudget::Sparse,
                Mc202ContourHint::Hold,
                Mc202HookResponse::AnswerSpace,
                0.80,
                0.78,
            ),
            (Tr909SourceSupportProfile::SteadyPulse, _) => mc202_bar_profile(
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
        position_beats: f64::from(bar.saturating_mul(grid.beats_per_bar)),
        is_transport_running: true,
    }
}

fn mc202_bar_profile(
    mode: Mc202RenderMode,
    phrase_shape: Mc202PhraseShape,
    note_budget: Mc202NoteBudget,
    contour_hint: Mc202ContourHint,
    hook_response: Mc202HookResponse,
    touch: f32,
    music_bus_level: f32,
) -> (
    Mc202RenderMode,
    Mc202PhraseShape,
    Mc202NoteBudget,
    Mc202ContourHint,
    Mc202HookResponse,
    f32,
    f32,
) {
    (
        mode,
        phrase_shape,
        note_budget,
        contour_hint,
        hook_response,
        touch,
        music_bus_level,
    )
}

fn mc202_bass_pressure_reason(profile: SourceAwareTr909Profile) -> &'static str {
    match profile.support_profile {
        Tr909SourceSupportProfile::DropDrive => "mc202_sparse_pressure_cell_variation",
        Tr909SourceSupportProfile::BreakLift => "mc202_mutated_lift_pressure_variation",
        Tr909SourceSupportProfile::SteadyPulse => "mc202_follower_support_pressure_variation",
    }
}
