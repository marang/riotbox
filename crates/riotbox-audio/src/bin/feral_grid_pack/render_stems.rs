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
             - TR-909 source reason: `{}`\n\
             - TR-909 support profile: `{}` / pattern `{}` / phrase `{}`\n\
             - TR-909 source low/high energy: `{:.6}` / `{:.6}`\n\
             - W-30 source-chop reason: `{}`\n\
             - W-30 source-chop preview RMS: `{:.6}` from source RMS `{:.6}` with gain `{:.6}`\n\
             - Source-first generated/source RMS ratio: `{:.6}` (max `{MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO:.6}`)\n\
             - Support generated/source RMS ratio: `{:.6}` (max `{MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO:.6}`)\n\
             - Generated-support mix low-band RMS: `{:.6}`\n\
             - Minimum full mix low-band RMS: `{MIN_LOW_BAND_RMS:.6}`\n\
             - Result: `pass`\n\n\
             | Stem | RMS | Peak abs | Low-band RMS | Active samples | Bar similarity | Identical bar run | Low energy | Mid energy | High energy |\n\
             | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n\
             | TR-909 source support | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n\
             | W-30 Feral source chop | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n\
             | Source-first mix | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n\
             | Generated-support mix | {:.6} | {:.6} | {:.6} | {} | {:.6} | {} | {:.6} | {:.6} | {:.6} |\n",
            args.source_path.display(),
            grid.bpm,
            grid.bars,
            grid.beats_per_bar,
            grid.total_beats,
            grid.total_frames,
            grid.duration_seconds(),
            report.tr909_source_profile.reason,
            report.tr909_source_profile.support_profile.label(),
            report.tr909_source_profile.pattern_adoption.label(),
            report.tr909_source_profile.phrase_variation.label(),
            report.tr909_source_profile.low_band_energy_ratio,
            report.tr909_source_profile.high_band_energy_ratio,
            report.w30_source_chop_profile.reason,
            report.w30_source_chop_profile.preview_rms,
            report.w30_source_chop_profile.source_window_rms,
            report.w30_source_chop_profile.gain,
            report.source_first_generated_to_source_rms_ratio,
            report.support_generated_to_source_rms_ratio,
            report.full_mix.low_band.rms,
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
            report.source_first_mix.signal.rms,
            report.source_first_mix.signal.peak_abs,
            report.source_first_mix.low_band.rms,
            report.source_first_mix.signal.active_samples,
            report.source_first_mix.bar_variation.bar_similarity,
            report.source_first_mix.bar_variation.identical_bar_run_length,
            report.source_first_mix.spectral_energy.low_band_energy_ratio,
            report.source_first_mix.spectral_energy.mid_band_energy_ratio,
            report.source_first_mix.spectral_energy.high_band_energy_ratio,
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
             - `stems/01_tr909_beat_fill.wav`: source-aware TR-909 support rendered on the same grid.\n\
             - `stems/02_w30_feral_source_chop.wav`: W-30 source-backed Feral chop with articulate source-window selection and bounded loudness normalization.\n\
             - `03_riotbox_source_first_mix.wav`: listen here first; source-backed W-30 leads and generated drums stay secondary.\n\
             - `04_riotbox_generated_support_mix.wav`: generated-support mix; TR-909 adds low-end and movement without proving source extraction by itself.\n\
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
