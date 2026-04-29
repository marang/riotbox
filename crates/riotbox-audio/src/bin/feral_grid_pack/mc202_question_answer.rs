fn mc202_question_answer_state(question: bool, bpm: f32, position_beats: f64) -> Mc202RenderState {
    if question {
        Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            note_budget: Mc202NoteBudget::Balanced,
            contour_hint: Mc202ContourHint::Lift,
            hook_response: Mc202HookResponse::Direct,
            touch: 0.76,
            music_bus_level: 0.86,
            tempo_bpm: bpm,
            position_beats,
            is_transport_running: true,
        }
    } else {
        Mc202RenderState {
            mode: Mc202RenderMode::Answer,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::AnswerHook,
            note_budget: Mc202NoteBudget::Sparse,
            contour_hint: Mc202ContourHint::Drop,
            hook_response: Mc202HookResponse::AnswerSpace,
            touch: 0.92,
            music_bus_level: 0.88,
            tempo_bpm: bpm,
            position_beats,
            is_transport_running: true,
        }
    }
}

fn render_tr909_beat_fill(grid: &Grid) -> Vec<f32> {
    render_tr909_offline(
        &Tr909RenderState {
            mode: Tr909RenderMode::Fill,
            routing: Tr909RenderRouting::DrumBusSupport,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            drum_bus_level: 0.94,
            slam_intensity: 0.32,
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

fn render_w30_source_chop(grid: &Grid, source_window_preview: W30PreviewSampleWindow) -> Vec<f32> {
    render_w30_preview_offline(
        &W30PreviewRenderState {
            mode: W30PreviewRenderMode::RawCaptureAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
            active_bank_id: Some("bank-a".into()),
            focused_pad_id: Some("pad-01".into()),
            capture_id: Some("cap-feral-grid".into()),
            trigger_revision: 1,
            trigger_velocity: 0.82,
            source_window_preview: Some(source_window_preview),
            pad_playback: None,
            music_bus_level: 0.72,
            grit_level: 0.46,
            is_transport_running: true,
            tempo_bpm: grid.bpm,
            position_beats: 0.0,
        },
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.total_frames,
    )
}

fn render_full_mix(mc202: &[f32], tr909: &[f32], w30: &[f32]) -> Vec<f32> {
    debug_assert_eq!(mc202.len(), tr909.len());
    debug_assert_eq!(mc202.len(), w30.len());

    let tr909_low = one_pole_lowpass(tr909, 165.0);
    mc202
        .iter()
        .zip(tr909.iter())
        .zip(tr909_low.iter())
        .zip(w30.iter())
        .map(|(((mc202, tr909), tr909_low), w30)| {
            let mixed = mc202 * 1.12 + tr909 * 10.0 + tr909_low * 18.0 + w30 * 0.94;
            (mixed * 1.7).tanh() * 0.92
        })
        .collect()
}

fn one_pole_lowpass(samples: &[f32], cutoff_hz: f32) -> Vec<f32> {
    let dt = 1.0 / SAMPLE_RATE as f32;
    let rc = 1.0 / (std::f32::consts::TAU * cutoff_hz.max(1.0));
    let alpha = dt / (rc + dt);
    let mut state = [0.0_f32; CHANNEL_COUNT as usize];
    let mut output = Vec::with_capacity(samples.len());

    for frame in samples.chunks_exact(usize::from(CHANNEL_COUNT)) {
        for (channel, sample) in frame.iter().enumerate() {
            state[channel] += alpha * (*sample - state[channel]);
            output.push(state[channel]);
        }
    }

    output
}

fn assert_grid_len(name: &str, samples: &[f32], grid: &Grid) {
    assert_eq!(
        samples.len(),
        grid.total_frames.saturating_mul(usize::from(CHANNEL_COUNT)),
        "{name} must match grid length"
    );
}

fn render_metrics(samples: &[f32], grid: &Grid) -> RenderMetrics {
    RenderMetrics {
        signal: signal_metrics_with_grid(
            samples,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            grid.bpm,
            grid.beats_per_bar,
        ),
        low_band: signal_metrics_with_grid(
            &one_pole_lowpass(samples, 165.0),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            grid.bpm,
            grid.beats_per_bar,
        ),
        bar_variation: bar_variation_metrics(samples, grid),
        spectral_energy: spectral_energy_metrics(samples),
    }
}

fn mc202_first_question_answer_delta(mc202: &[f32], grid: &Grid) -> OfflineAudioMetrics {
    if grid.bars < 2 {
        return OfflineAudioMetrics {
            active_samples: 0,
            peak_abs: 0.0,
            rms: 0.0,
            sum: 0.0,
            ..OfflineAudioMetrics::default()
        };
    }

    let channels = usize::from(CHANNEL_COUNT);
    let question_start = grid.bar_start_frame(0) * channels;
    let question_end = grid.bar_end_frame(0) * channels;
    let answer_start = grid.bar_start_frame(1) * channels;
    let answer_end = grid.bar_end_frame(1) * channels;
    let question = &mc202[question_start..question_end];
    let answer = &mc202[answer_start..answer_end];
    let delta = question
        .iter()
        .zip(answer.iter())
        .map(|(question, answer)| question - answer)
        .collect::<Vec<_>>();
    signal_metrics_with_grid(
        &delta,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    )
}

fn validate_report(report: &PackReport) -> Result<(), Box<dyn std::error::Error>> {
    for (name, metrics) in [
        ("mc202", report.mc202),
        ("tr909", report.tr909),
        ("w30", report.w30),
        ("full_mix", report.full_mix),
    ] {
        if metrics.signal.rms <= MIN_SIGNAL_RMS {
            return Err(format!("{name} rendered near silence").into());
        }
    }

    if report.mc202_question_answer_delta.rms <= MIN_MC202_BAR_DELTA_RMS {
        return Err(format!(
            "MC-202 question/answer bars are too similar: delta RMS {:.6}",
            report.mc202_question_answer_delta.rms
        )
        .into());
    }

    if report.full_mix.low_band.rms <= MIN_LOW_BAND_RMS {
        return Err(format!(
            "full mix low-band support is too weak: low-band RMS {:.6}",
            report.full_mix.low_band.rms
        )
        .into());
    }

    Ok(())
}

fn write_audio_with_metrics(
    path: &Path,
    samples: &[f32],
    grid: &Grid,
) -> Result<(), SourceAudioError> {
    write_interleaved_pcm16_wav(path, SAMPLE_RATE, CHANNEL_COUNT, samples)?;
    write_metrics_markdown(&metrics_path_for(path), render_metrics(samples, grid))
        .map_err(|error| SourceAudioError::Io(error.to_string()))
}

fn metrics_path_for(path: &Path) -> PathBuf {
    let mut metrics_path = path.to_path_buf();
    metrics_path.set_file_name(match path.file_stem().and_then(|stem| stem.to_str()) {
        Some(stem) => format!("{stem}.metrics.md"),
        None => "metrics.md".to_string(),
    });
    metrics_path
}

fn write_metrics_markdown(path: &Path, metrics: RenderMetrics) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Grid Demo Metrics\n\n\
             - Pack: `{PACK_ID}`\n\
             - Peak abs: `{:.6}`\n\
             - RMS: `{:.6}`\n\
             - Active samples: `{}`\n\
             - Sum: `{:.6}`\n\
             - Mean abs: `{:.6}`\n\
             - Zero crossings: `{}`\n\
             - Crest factor: `{:.6}`\n\
             - Active sample ratio: `{:.6}`\n\
             - Silence ratio: `{:.6}`\n\
             - DC offset: `{:.6}`\n\
             - Onset count: `{}`\n\
             - Event density per bar: `{:.6}`\n\
             - Low-band peak abs: `{:.6}`\n\
             - Low-band RMS: `{:.6}`\n",
            metrics.signal.peak_abs,
            metrics.signal.rms,
            metrics.signal.active_samples,
            metrics.signal.sum,
            metrics.signal.mean_abs,
            metrics.signal.zero_crossings,
            metrics.signal.crest_factor,
            metrics.signal.active_sample_ratio,
            metrics.signal.silence_ratio,
            metrics.signal.dc_offset,
            metrics.signal.onset_count,
            metrics.signal.event_density_per_bar,
            metrics.low_band.peak_abs,
            metrics.low_band.rms
        ),
    )
}

fn write_report(path: &Path, args: &Args, grid: &Grid, report: PackReport) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Grid Demo Report\n\n\
             - Pack: `{PACK_ID}`\n\
             - Source: `{}`\n\
             - BPM: `{:.3}`\n\
             - Bars: `{}`\n\
             - Beats per bar: `{}`\n\
             - Total beats: `{}`\n\
             - Total frames: `{}`\n\
             - Duration seconds: `{:.6}`\n\
             - MC-202 question/answer delta RMS: `{:.6}`\n\
             - Minimum MC-202 delta RMS: `{MIN_MC202_BAR_DELTA_RMS:.6}`\n\
             - Full mix low-band RMS: `{:.6}`\n\
             - Minimum full mix low-band RMS: `{MIN_LOW_BAND_RMS:.6}`\n\
             - Result: `pass`\n\n\
             | Stem | RMS | Peak abs | Low-band RMS | Active samples | Bar similarity | Identical bar run | Low energy | Mid energy | High energy |\n\
             | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n\
             | MC-202 question/answer | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n\
             | TR-909 beat/fill | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n\
             | W-30 Feral source chop | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n\
             | Full grid mix | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n",
            args.source_path.display(),
            grid.bpm,
            grid.bars,
            grid.beats_per_bar,
            grid.total_beats,
            grid.total_frames,
            grid.duration_seconds(),
            report.mc202_question_answer_delta.rms,
            report.full_mix.low_band.rms,
            report.mc202.signal.rms,
            report.mc202.signal.peak_abs,
            report.mc202.low_band.rms,
            report.mc202.signal.active_samples,
            report.mc202.bar_variation.bar_similarity,
            report.mc202.bar_variation.identical_bar_run_length,
            report.mc202.spectral_energy.low_band_energy_ratio,
            report.mc202.spectral_energy.mid_band_energy_ratio,
            report.mc202.spectral_energy.high_band_energy_ratio,
            report.tr909.signal.rms,
            report.tr909.signal.peak_abs,
            report.tr909.low_band.rms,
            report.tr909.signal.active_samples,
            report.tr909.bar_variation.bar_similarity,
            report.tr909.bar_variation.identical_bar_run_length,
            report.tr909.spectral_energy.low_band_energy_ratio,
            report.tr909.spectral_energy.mid_band_energy_ratio,
            report.tr909.spectral_energy.high_band_energy_ratio,
            report.w30.signal.rms,
            report.w30.signal.peak_abs,
            report.w30.low_band.rms,
            report.w30.signal.active_samples,
            report.w30.bar_variation.bar_similarity,
            report.w30.bar_variation.identical_bar_run_length,
            report.w30.spectral_energy.low_band_energy_ratio,
            report.w30.spectral_energy.mid_band_energy_ratio,
            report.w30.spectral_energy.high_band_energy_ratio,
            report.full_mix.signal.rms,
            report.full_mix.signal.peak_abs,
            report.full_mix.low_band.rms,
            report.full_mix.signal.active_samples,
            report.full_mix.bar_variation.bar_similarity,
            report.full_mix.bar_variation.identical_bar_run_length,
            report.full_mix.spectral_energy.low_band_energy_ratio,
            report.full_mix.spectral_energy.mid_band_energy_ratio,
            report.full_mix.spectral_energy.high_band_energy_ratio
        ),
    )
}

fn write_manifest(
    path: &Path,
    args: &Args,
    grid: &Grid,
    report: PackReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = args.output_dir();
    let source_window_seconds = args.source_window_seconds.min(grid.duration_seconds());
    let manifest = ListeningPackManifest {
        schema_version: LISTENING_MANIFEST_SCHEMA_VERSION,
        pack_id: PACK_ID,
        source: args.source_path.display().to_string(),
        sample_rate: SAMPLE_RATE,
        channel_count: CHANNEL_COUNT,
        bpm: grid.bpm,
        beats_per_bar: grid.beats_per_bar,
        bars: grid.bars,
        total_beats: grid.total_beats,
        total_frames: grid.total_frames,
        duration_seconds: grid.duration_seconds(),
        source_start_seconds: args.source_start_seconds,
        source_window_seconds,
        artifacts: manifest_artifacts(&output_dir),
        feral_scorecard: manifest_feral_scorecard(),
        thresholds: ManifestThresholds {
            min_signal_rms: MIN_SIGNAL_RMS,
            min_mc202_bar_delta_rms: MIN_MC202_BAR_DELTA_RMS,
            min_low_band_rms: MIN_LOW_BAND_RMS,
        },
        metrics: ManifestPackMetrics {
            mc202_question_answer: manifest_render_metrics(report.mc202),
            tr909_beat_fill: manifest_render_metrics(report.tr909),
            w30_feral_source_chop: manifest_render_metrics(report.w30),
            full_grid_mix: manifest_render_metrics(report.full_mix),
            bar_variation: ManifestBarVariationMetrics {
                mc202_question_answer: report.mc202.bar_variation,
                tr909_beat_fill: report.tr909.bar_variation,
                w30_feral_source_chop: report.w30.bar_variation,
                full_grid_mix: report.full_mix.bar_variation,
            },
            spectral_energy: ManifestSpectralEnergyMetrics {
                mc202_question_answer: report.mc202.spectral_energy,
                tr909_beat_fill: report.tr909.spectral_energy,
                w30_feral_source_chop: report.w30.spectral_energy,
                full_grid_mix: report.full_mix.spectral_energy,
            },
            mc202_question_answer_delta: report.mc202_question_answer_delta.into(),
        },
        verification_command: format!(
            "just feral-grid-pack \"{}\" {} {:.3} {} {:.3} {:.3}",
            args.source_path.display(),
            args.date,
            grid.bpm,
            grid.bars,
            source_window_seconds,
            args.source_start_seconds
        ),
        result: "pass",
    };

    write_manifest_json(path, &manifest)?;
    Ok(())
}

fn manifest_feral_scorecard() -> ManifestFeralScorecard {
    ManifestFeralScorecard {
        readiness: "ready",
        break_rebuild_potential: "high",
        hook_fragment_count: 1,
        break_support_count: 3,
        quote_risk_count: 0,
        capture_candidate_count: 1,
        top_reason: "grid-locked generated feral QA pack",
        source_backed: true,
        generated: true,
        fallback_like: false,
        lane_gestures: [
            "mc202 question/answer",
            "tr909 beat/fill",
            "w30 source chop",
        ],
        material_sources: [
            "generated mc202",
            "generated tr909",
            "source-backed w30 window",
        ],
        warnings: ["offline QA pack, not live arranger"],
    }
}

fn manifest_artifacts(output_dir: &Path) -> Vec<ManifestArtifact> {
    vec![
        manifest_audio_artifact(
            "mc202_question_answer",
            output_dir.join("stems/01_mc202_question_answer.wav"),
        ),
        manifest_audio_artifact(
            "tr909_beat_fill",
            output_dir.join("stems/02_tr909_beat_fill.wav"),
        ),
        manifest_audio_artifact(
            "w30_feral_source_chop",
            output_dir.join("stems/03_w30_feral_source_chop.wav"),
        ),
        manifest_audio_artifact(
            "full_grid_mix",
            output_dir.join("04_riotbox_grid_feral_mix.wav"),
        ),
        ManifestArtifact::markdown_report("grid_report", &output_dir.join("grid-report.md")),
        ManifestArtifact::markdown_readme("readme", &output_dir.join("README.md")),
    ]
}

fn manifest_audio_artifact(role: &'static str, path: PathBuf) -> ManifestArtifact {
    let metrics_path = metrics_path_for(&path);
    ManifestArtifact::audio_wav(role, &path, Some(&metrics_path))
}

fn manifest_render_metrics(metrics: RenderMetrics) -> ManifestRenderMetrics {
    ManifestRenderMetrics {
        signal: metrics.signal.into(),
        low_band: metrics.low_band.into(),
    }
}

fn write_readme(output_dir: &Path, args: &Args, grid: &Grid) -> std::io::Result<()> {
    fs::write(
        output_dir.join("README.md"),
        format!(
            "# Feral Grid Demo Pack\n\n\
             This pack is the current Riotbox offline QA path for checking a musical grid,\n\
             not only a log path. All stems use the same BPM, bar count, and frame count.\n\n\
             ## Grid\n\n\
             - Source: `{}`\n\
             - BPM: `{:.3}`\n\
             - Bars: `{}`\n\
             - Beats per bar: `{}`\n\
             - Duration: `{:.3}s`\n\
             - Source window start: `{:.3}s`\n\
             - W-30 source window length: `{:.3}s`\n\n\
             ## Files\n\n\
             - `stems/01_mc202_question_answer.wav`: one-bar question, one-bar answer, alternating across the grid.\n\
             - `stems/02_tr909_beat_fill.wav`: TR-909 beat/fill support rendered on the same grid.\n\
             - `stems/03_w30_feral_source_chop.wav`: W-30 source-backed Feral chop rendered on the same grid.\n\
             - `04_riotbox_grid_feral_mix.wav`: combined grid-locked listening mix with low-end support.\n\
             - `grid-report.md`: timing and output metrics.\n\
             - `manifest.json`: machine-readable pack metadata, artifact paths, thresholds, and key metrics.\n\
\n\
             ## Current Limit\n\n\
             This is an offline QA/listening pack. It proves the render seams can align musically,\n\
             but it does not yet mean the live TUI mixer exposes this whole arrangement path directly.\n",
            args.source_path.display(),
            grid.bpm,
            grid.bars,
            grid.beats_per_bar,
            grid.duration_seconds(),
            args.source_start_seconds,
            args.source_window_seconds.min(grid.duration_seconds())
        ),
    )
}
