fn render_tr909_source_support(grid: &Grid, profile: SourceAwareTr909Profile) -> Vec<f32> {
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

fn validate_report(report: &PackReport) -> Result<(), Box<dyn std::error::Error>> {
    for (name, metrics) in [
        ("tr909", report.tr909),
        ("w30", report.w30),
        ("source_first_mix", report.source_first_mix),
        ("full_mix", report.full_mix),
    ] {
        if metrics.signal.rms <= MIN_SIGNAL_RMS {
            return Err(format!("{name} rendered near silence").into());
        }
    }

    if report.source_first_generated_to_source_rms_ratio
        >= MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO
    {
        return Err(format!(
            "source-first mix generated/source RMS ratio {:.6} exceeds {:.6}",
            report.source_first_generated_to_source_rms_ratio,
            MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO
        )
        .into());
    }

    if report.support_generated_to_source_rms_ratio >= MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO {
        return Err(format!(
            "generated-support mix generated/source RMS ratio {:.6} exceeds {:.6}",
            report.support_generated_to_source_rms_ratio,
            MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO
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

    if report.tr909_source_grid_alignment.hit_ratio < SOURCE_GRID_OUTPUT_MIN_HIT_RATIO {
        return Err(format!(
            "TR-909 source-grid alignment hit ratio {:.6} is below {:.6}",
            report.tr909_source_grid_alignment.hit_ratio, SOURCE_GRID_OUTPUT_MIN_HIT_RATIO
        )
        .into());
    }

    if report.w30_source_grid_alignment.hit_ratio < SOURCE_GRID_OUTPUT_MIN_HIT_RATIO {
        return Err(format!(
            "W-30 source-grid alignment hit ratio {:.6} is below {:.6}",
            report.w30_source_grid_alignment.hit_ratio, SOURCE_GRID_OUTPUT_MIN_HIT_RATIO
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

fn verification_command(args: &Args, grid: &Grid, source_window_seconds: f32) -> String {
    let bpm_arg = if args.bpm_overridden {
        format!(" {:.3}", grid.bpm)
    } else {
        " auto".to_string()
    };
    format!(
        "just feral-grid-pack \"{}\" {}{} {} {:.3} {:.3}",
        args.source_path.display(),
        args.date,
        bpm_arg,
        grid.bars,
        source_window_seconds,
        args.source_start_seconds
    )
}

fn write_manifest(
    path: &Path,
    args: &Args,
    grid: &Grid,
    report: PackReport,
    source_timing_analysis: &SourceTimingAnalysisForManifest,
    grid_bpm: GridBpmDecision,
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
        grid_bpm_source: grid_bpm_source_label(grid_bpm.source),
        grid_bpm_decision_reason: grid_bpm_decision_reason_label(grid_bpm.reason),
        source_timing_bpm_delta: grid_bpm.source_delta_bpm,
        beats_per_bar: grid.beats_per_bar,
        bars: grid.bars,
        total_beats: grid.total_beats,
        total_frames: grid.total_frames,
        duration_seconds: grid.duration_seconds(),
        source_start_seconds: args.source_start_seconds,
        source_window_seconds,
        artifacts: manifest_artifacts(&output_dir),
        feral_scorecard: manifest_feral_scorecard(),
        source_timing: manifest_source_timing_readiness(
            &source_timing_analysis.readiness,
            grid_bpm,
            &source_timing_analysis.anchor_evidence,
            &source_timing_analysis.groove_evidence,
        ),
        thresholds: ManifestThresholds {
            min_signal_rms: MIN_SIGNAL_RMS,
            min_low_band_rms: MIN_LOW_BAND_RMS,
            max_source_first_generated_to_source_rms_ratio:
                MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
            max_support_generated_to_source_rms_ratio: MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
        },
        metrics: ManifestPackMetrics {
            tr909_source_profile: manifest_tr909_source_profile(report.tr909_source_profile),
            w30_source_chop_profile: manifest_w30_source_chop_profile(
                report.w30_source_chop_profile,
            ),
            tr909_beat_fill: manifest_render_metrics(report.tr909),
            w30_feral_source_chop: manifest_render_metrics(report.w30),
            source_first_mix: manifest_render_metrics(report.source_first_mix),
            full_grid_mix: manifest_render_metrics(report.full_mix),
            mix_balance: ManifestMixBalanceMetrics {
                source_first_generated_to_source_rms_ratio:
                    report.source_first_generated_to_source_rms_ratio,
                support_generated_to_source_rms_ratio: report.support_generated_to_source_rms_ratio,
            },
            tr909_source_grid_alignment: report.tr909_source_grid_alignment,
            w30_source_grid_alignment: report.w30_source_grid_alignment,
            source_grid_output_drift: report.source_grid_output_drift,
            bar_variation: ManifestBarVariationMetrics {
                tr909_beat_fill: report.tr909.bar_variation,
                w30_feral_source_chop: report.w30.bar_variation,
                source_first_mix: report.source_first_mix.bar_variation,
                full_grid_mix: report.full_mix.bar_variation,
            },
            spectral_energy: ManifestSpectralEnergyMetrics {
                tr909_beat_fill: report.tr909.spectral_energy,
                w30_feral_source_chop: report.w30.spectral_energy,
                source_first_mix: report.source_first_mix.spectral_energy,
                full_grid_mix: report.full_mix.spectral_energy,
            },
        },
        verification_command: verification_command(args, grid, source_window_seconds),
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
        lane_gestures: ["tr909 beat/fill", "w30 source chop"],
        material_sources: ["generated tr909", "source-backed w30 window"],
        warnings: ["offline QA pack, not live arranger"],
    }
}

fn manifest_artifacts(output_dir: &Path) -> Vec<ManifestArtifact> {
    vec![
        manifest_audio_artifact(
            "tr909_beat_fill",
            output_dir.join("stems/01_tr909_beat_fill.wav"),
        ),
        manifest_audio_artifact(
            "w30_feral_source_chop",
            output_dir.join("stems/02_w30_feral_source_chop.wav"),
        ),
        manifest_audio_artifact(
            "source_first_mix",
            output_dir.join("03_riotbox_source_first_mix.wav"),
        ),
        manifest_audio_artifact(
            "full_grid_mix",
            output_dir.join("04_riotbox_generated_support_mix.wav"),
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
