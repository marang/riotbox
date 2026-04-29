fn tr909_support_state(
    profile: Tr909SourceSupportProfile,
    context: Tr909SourceSupportContext,
    adoption: Tr909PatternAdoption,
    variation: Tr909PhraseVariation,
) -> Tr909RenderState {
    Tr909RenderState {
        mode: Tr909RenderMode::SourceSupport,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: Some(profile),
        source_support_context: Some(context),
        pattern_adoption: Some(adoption),
        phrase_variation: Some(variation),
        drum_bus_level: 0.72,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        current_scene_id: (context == Tr909SourceSupportContext::SceneTarget)
            .then(|| "scene-02-break".into()),
        ..Tr909RenderState::default()
    }
}

fn mc202_state(mode: Mc202RenderMode, shape: Mc202PhraseShape, touch: f32) -> Mc202RenderState {
    mc202_state_with_contour(mode, shape, touch, Mc202ContourHint::Neutral)
}

fn mc202_state_with_contour(
    mode: Mc202RenderMode,
    shape: Mc202PhraseShape,
    touch: f32,
    contour_hint: Mc202ContourHint,
) -> Mc202RenderState {
    mc202_state_with_policy(mode, shape, touch, contour_hint, Mc202HookResponse::Direct)
}

fn mc202_state_with_policy(
    mode: Mc202RenderMode,
    shape: Mc202PhraseShape,
    touch: f32,
    contour_hint: Mc202ContourHint,
    hook_response: Mc202HookResponse,
) -> Mc202RenderState {
    Mc202RenderState {
        mode,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: shape,
        note_budget: mc202_note_budget_for_shape_and_hook_response(shape, hook_response),
        contour_hint,
        hook_response,
        touch,
        music_bus_level: 0.74,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
    }
}

fn mc202_note_budget_for_shape_and_hook_response(
    shape: Mc202PhraseShape,
    hook_response: Mc202HookResponse,
) -> riotbox_audio::mc202::Mc202NoteBudget {
    if hook_response == Mc202HookResponse::AnswerSpace {
        return riotbox_audio::mc202::Mc202NoteBudget::Sparse;
    }

    match shape {
        Mc202PhraseShape::PressureCell => riotbox_audio::mc202::Mc202NoteBudget::Sparse,
        Mc202PhraseShape::InstigatorSpike => riotbox_audio::mc202::Mc202NoteBudget::Push,
        Mc202PhraseShape::MutatedDrive => riotbox_audio::mc202::Mc202NoteBudget::Wide,
        Mc202PhraseShape::RootPulse
        | Mc202PhraseShape::FollowerDrive
        | Mc202PhraseShape::AnswerHook => riotbox_audio::mc202::Mc202NoteBudget::Balanced,
    }
}

fn render_case(
    output_dir: &Path,
    case: PackCase,
    frame_count: usize,
    duration_seconds: f32,
) -> Result<CaseReport, Box<dyn std::error::Error>> {
    let case_dir = output_dir.join(case.id);
    fs::create_dir_all(&case_dir)?;

    let (baseline, candidate) = render_pair(&case.render_pair, frame_count);
    let baseline_metrics = signal_metrics(&baseline);
    let candidate_metrics = signal_metrics(&candidate);
    let signal_delta_metrics = signal_delta_metrics(&baseline, &candidate);
    let report = CaseReport {
        id: case.id,
        title: case.title,
        recipe_refs: case.recipe_refs,
        baseline_label: case.baseline_label,
        candidate_label: case.candidate_label,
        baseline_metrics,
        candidate_metrics,
        signal_delta_metrics,
        min_rms_delta: case.min_rms_delta,
        min_signal_delta_rms: case.min_signal_delta_rms,
        passed: rms_delta(baseline_metrics, candidate_metrics) >= case.min_rms_delta
            && signal_delta_metrics.rms >= case.min_signal_delta_rms,
    };

    let baseline_path = case_dir.join("baseline.wav");
    let candidate_path = case_dir.join("candidate.wav");
    write_pcm16_wav(&baseline_path, SAMPLE_RATE, CHANNEL_COUNT, &baseline)?;
    write_pcm16_wav(&candidate_path, SAMPLE_RATE, CHANNEL_COUNT, &candidate)?;

    fs::write(
        case_dir.join("baseline.metrics.md"),
        render_metrics_markdown(&case, "baseline", duration_seconds, baseline_metrics),
    )?;
    fs::write(
        case_dir.join("candidate.metrics.md"),
        render_metrics_markdown(&case, "candidate", duration_seconds, candidate_metrics),
    )?;
    fs::write(
        case_dir.join("comparison.md"),
        render_comparison_markdown(&case, &report),
    )?;

    if !report.passed {
        return Err(format!(
            "{} output delta failed: RMS delta {:.6} / min {:.6}, signal delta RMS {:.6} / min {:.6}",
            report.id,
            rms_delta(report.baseline_metrics, report.candidate_metrics),
            report.min_rms_delta,
            report.signal_delta_metrics.rms,
            report.min_signal_delta_rms
        )
        .into());
    }

    Ok(report)
}

fn render_pair(render_pair: &RenderPair, frame_count: usize) -> (Vec<f32>, Vec<f32>) {
    match render_pair {
        RenderPair::Tr909 {
            baseline,
            candidate,
        } => (
            render_tr909_offline(baseline, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
            render_tr909_offline(candidate, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
        ),
        RenderPair::Mc202 {
            baseline,
            candidate,
        } => (
            render_mc202_offline(baseline, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
            render_mc202_offline(candidate, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
        ),
    }
}

fn render_metrics_markdown(
    case: &PackCase,
    role: &str,
    duration_seconds: f32,
    metrics: OfflineAudioMetrics,
) -> String {
    let label = if role == "baseline" {
        case.baseline_label
    } else {
        case.candidate_label
    };
    format!(
        "# Lane Recipe Listening Metrics\n\n\
         - Pack: `{PACK_ID}`\n\
         - Case: `{}`\n\
         - Title: `{}`\n\
         - Recipes: `{}`\n\
         - Role: `{role}`\n\
         - Label: `{label}`\n\
         - Sample rate: `{SAMPLE_RATE}`\n\
         - Channels: `{CHANNEL_COUNT}`\n\
         - Duration seconds: `{duration_seconds:.3}`\n\
         - Active samples: `{}`\n\
         - Peak abs: `{:.6}`\n\
         - RMS: `{:.6}`\n\
         - Sum: `{:.6}`\n\
         - Mean abs: `{:.6}`\n\
         - Zero crossings: `{}`\n\
         - Crest factor: `{:.6}`\n\
         - Active sample ratio: `{:.6}`\n\
         - Silence ratio: `{:.6}`\n\
         - DC offset: `{:.6}`\n",
        case.id,
        case.title,
        case.recipe_refs,
        metrics.active_samples,
        metrics.peak_abs,
        metrics.rms,
        metrics.sum,
        metrics.mean_abs,
        metrics.zero_crossings,
        metrics.crest_factor,
        metrics.active_sample_ratio,
        metrics.silence_ratio,
        metrics.dc_offset
    )
}

fn render_comparison_markdown(case: &PackCase, report: &CaseReport) -> String {
    let baseline = report.baseline_metrics;
    let candidate = report.candidate_metrics;
    let active_delta = baseline.active_samples.abs_diff(candidate.active_samples);
    let peak_delta = (baseline.peak_abs - candidate.peak_abs).abs();
    let rms_delta = rms_delta(baseline, candidate);
    let sum_delta = (baseline.sum - candidate.sum).abs();
    let mean_abs_delta = (baseline.mean_abs - candidate.mean_abs).abs();
    let zero_crossings_delta = baseline.zero_crossings.abs_diff(candidate.zero_crossings);
    let crest_factor_delta = (baseline.crest_factor - candidate.crest_factor).abs();
    let active_ratio_delta =
        (baseline.active_sample_ratio - candidate.active_sample_ratio).abs();
    let silence_ratio_delta = (baseline.silence_ratio - candidate.silence_ratio).abs();
    let dc_offset_delta = (baseline.dc_offset - candidate.dc_offset).abs();
    let signal_delta = report.signal_delta_metrics;

    format!(
        "# Lane Recipe Listening Comparison\n\n\
         - Pack: `{PACK_ID}`\n\
         - Case: `{}`\n\
         - Title: `{}`\n\
         - Recipes: `{}`\n\
         - Baseline: `{}`\n\
         - Candidate: `{}`\n\
         - Minimum RMS delta: `{:.6}`\n\
         - Signal delta RMS: `{:.6}`\n\
         - Minimum signal delta RMS: `{:.6}`\n\
         - Signal delta peak abs: `{:.6}`\n\
         - Result: `{}`\n\
         - Note: {}\n\n\
         | Metric | Baseline | Candidate | Delta |\n\
         | --- | ---: | ---: | ---: |\n\
         | active_samples | {} | {} | {} |\n\
         | peak_abs | {:.6} | {:.6} | {:.6} |\n\
         | rms | {:.6} | {:.6} | {:.6} |\n\
         | sum | {:.6} | {:.6} | {:.6} |\n\
         | mean_abs | {:.6} | {:.6} | {:.6} |\n\
         | zero_crossings | {} | {} | {} |\n\
         | crest_factor | {:.6} | {:.6} | {:.6} |\n\
         | active_sample_ratio | {:.6} | {:.6} | {:.6} |\n\
         | silence_ratio | {:.6} | {:.6} | {:.6} |\n\
         | dc_offset | {:.6} | {:.6} | {:.6} |\n",
        case.id,
        case.title,
        case.recipe_refs,
        case.baseline_label,
        case.candidate_label,
        report.min_rms_delta,
        signal_delta.rms,
        report.min_signal_delta_rms,
        signal_delta.peak_abs,
        if report.passed { "pass" } else { "fail" },
        case.note,
        baseline.active_samples,
        candidate.active_samples,
        active_delta,
        baseline.peak_abs,
        candidate.peak_abs,
        peak_delta,
        baseline.rms,
        candidate.rms,
        rms_delta,
        baseline.sum,
        candidate.sum,
        sum_delta,
        baseline.mean_abs,
        candidate.mean_abs,
        mean_abs_delta,
        baseline.zero_crossings,
        candidate.zero_crossings,
        zero_crossings_delta,
        baseline.crest_factor,
        candidate.crest_factor,
        crest_factor_delta,
        baseline.active_sample_ratio,
        candidate.active_sample_ratio,
        active_ratio_delta,
        baseline.silence_ratio,
        candidate.silence_ratio,
        silence_ratio_delta,
        baseline.dc_offset,
        candidate.dc_offset,
        dc_offset_delta
    )
}

fn rms_delta(baseline: OfflineAudioMetrics, candidate: OfflineAudioMetrics) -> f32 {
    (baseline.rms - candidate.rms).abs()
}

fn signal_delta_metrics(baseline: &[f32], candidate: &[f32]) -> OfflineAudioMetrics {
    debug_assert_eq!(
        baseline.len(),
        candidate.len(),
        "baseline and candidate renders should use the same frame count"
    );
    let delta = baseline
        .iter()
        .zip(candidate.iter())
        .map(|(baseline, candidate)| baseline - candidate)
        .collect::<Vec<_>>();
    signal_metrics(&delta)
}

fn render_pack_summary(args: &Args, output_dir: &Path, reports: &[CaseReport]) -> String {
    let mut summary = format!(
        "# Lane Recipe Listening Pack\n\n\
         - Pack: `{PACK_ID}`\n\
         - Date: `{}`\n\
         - Output dir: `{}`\n\
         - Duration seconds: `{:.3}`\n\n\
         This pack is the first local recipe-level audio proof outside the W-30 source-preview path.\n\
         It renders bounded TR-909, MC-202, and Scene-coupled support comparisons as WAV files plus sibling metrics and `manifest.json`.\n\n\
         ## Cases\n\n\
         | Case | Active delta | Peak delta | RMS delta | Min RMS delta | Signal delta RMS | Min signal delta RMS | Sum delta |\n\
         | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n",
        args.date,
        output_dir.display(),
        args.duration_seconds
    );

    for report in reports {
        let active_delta = report
            .baseline_metrics
            .active_samples
            .abs_diff(report.candidate_metrics.active_samples);
        let peak_delta =
            (report.baseline_metrics.peak_abs - report.candidate_metrics.peak_abs).abs();
        let rms_delta = rms_delta(report.baseline_metrics, report.candidate_metrics);
        let sum_delta = (report.baseline_metrics.sum - report.candidate_metrics.sum).abs();
        summary.push_str(&format!(
            "| `{}` | {} | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} |\n",
            report.id,
            active_delta,
            peak_delta,
            rms_delta,
            report.min_rms_delta,
            report.signal_delta_metrics.rms,
            report.min_signal_delta_rms,
            sum_delta
        ));
    }

    summary.push_str(
        "\n## Current MC-202 Status\n\n\
         MC-202 now has explicit offline audio cases for follower-vs-answer, touch energy, pressure, instigator, phrase mutation, note budget, source-section contour hints, and hook-response restraint. These cases prove bounded renderable contrasts for the current `g`, `a`, `P`, `I`, `G`, `<`, and `>` gestures, not a finished MC-202 synth engine.\n\n\
         ## Current Scene Status\n\n\
         Scene Brain is represented here only through the current TR-909 `scene_target` support-accent seam. This does not claim a finished Scene transition engine.\n",
    );

    summary
}

#[derive(Serialize)]
struct ListeningPackManifest {
    schema_version: u32,
    pack_id: &'static str,
    date: String,
    sample_rate: u32,
    channel_count: u16,
    duration_seconds: f32,
    case_count: usize,
    artifacts: Vec<ManifestArtifact>,
    cases: Vec<ManifestCase>,
    result: &'static str,
}

#[derive(Serialize)]
struct ManifestCase {
    id: &'static str,
    title: &'static str,
    recipe_refs: &'static str,
    baseline_label: &'static str,
    candidate_label: &'static str,
    thresholds: ManifestThresholds,
    metrics: ManifestCaseMetrics,
    result: &'static str,
}

#[derive(Serialize)]
struct ManifestThresholds {
    min_rms_delta: f32,
    min_signal_delta_rms: f32,
}

#[derive(Serialize)]
struct ManifestCaseMetrics {
    baseline: ManifestSignalMetrics,
    candidate: ManifestSignalMetrics,
    signal_delta: ManifestSignalMetrics,
    rms_delta: f32,
}

fn write_manifest(
    path: &Path,
    args: &Args,
    output_dir: &Path,
    reports: &[CaseReport],
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = ListeningPackManifest {
        schema_version: LISTENING_MANIFEST_SCHEMA_VERSION,
        pack_id: PACK_ID,
        date: args.date.clone(),
        sample_rate: SAMPLE_RATE,
        channel_count: CHANNEL_COUNT,
        duration_seconds: args.duration_seconds,
        case_count: reports.len(),
        artifacts: manifest_artifacts(output_dir, reports),
        cases: reports.iter().map(ManifestCase::from).collect(),
        result: "pass",
    };

    write_manifest_json(path, &manifest)?;
    Ok(())
}

fn manifest_artifacts(output_dir: &Path, reports: &[CaseReport]) -> Vec<ManifestArtifact> {
    let mut artifacts = Vec::new();
    for report in reports {
        let case_dir = output_dir.join(report.id);
        let baseline_path = case_dir.join("baseline.wav");
        let baseline_metrics_path = case_dir.join("baseline.metrics.md");
        let candidate_path = case_dir.join("candidate.wav");
        let candidate_metrics_path = case_dir.join("candidate.metrics.md");
        let comparison_path = case_dir.join("comparison.md");
        artifacts.push(ManifestArtifact::case_audio_wav(
            report.id,
            "baseline",
            &baseline_path,
            Some(&baseline_metrics_path),
        ));
        artifacts.push(ManifestArtifact::case_audio_wav(
            report.id,
            "candidate",
            &candidate_path,
            Some(&candidate_metrics_path),
        ));
        artifacts.push(ManifestArtifact::case_markdown_report(
            report.id,
            "comparison",
            &comparison_path,
        ));
    }
    artifacts.push(ManifestArtifact::case_markdown_report(
        "pack",
        "summary",
        &output_dir.join("pack-summary.md"),
    ));
    artifacts
}

impl From<&CaseReport> for ManifestCase {
    fn from(report: &CaseReport) -> Self {
        Self {
            id: report.id,
            title: report.title,
            recipe_refs: report.recipe_refs,
            baseline_label: report.baseline_label,
            candidate_label: report.candidate_label,
            thresholds: ManifestThresholds {
                min_rms_delta: report.min_rms_delta,
                min_signal_delta_rms: report.min_signal_delta_rms,
            },
            metrics: ManifestCaseMetrics {
                baseline: report.baseline_metrics.into(),
                candidate: report.candidate_metrics.into(),
                signal_delta: report.signal_delta_metrics.into(),
                rms_delta: rms_delta(report.baseline_metrics, report.candidate_metrics),
            },
            result: if report.passed { "pass" } else { "fail" },
        }
    }
}
