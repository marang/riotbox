fn write_metrics_markdown(path: &Path, metrics: OfflineAudioMetrics) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Before / After Metrics\n\n\
             - Peak abs: `{:.6}`\n\
             - RMS: `{:.6}`\n\
             - Active samples: `{}`\n\
             - Sum: `{:.6}`\n",
            metrics.peak_abs, metrics.rms, metrics.active_samples, metrics.sum
        ),
    )
}

fn signal_delta_metrics(baseline: &[f32], candidate: &[f32]) -> OfflineAudioMetrics {
    let delta = baseline
        .iter()
        .zip(candidate.iter())
        .map(|(baseline, candidate)| baseline - candidate)
        .collect::<Vec<_>>();
    signal_metrics(&delta)
}

fn write_comparison_markdown(
    path: &Path,
    source: OfflineAudioMetrics,
    after: OfflineAudioMetrics,
    delta: OfflineAudioMetrics,
) -> std::io::Result<()> {
    fs::write(
        path,
        format!(
            "# Feral Before / After Comparison\n\n\
             - Pack: `{PACK_ID}`\n\
             - Source RMS: `{:.6}`\n\
             - Riotbox after RMS: `{:.6}`\n\
             - Signal delta RMS: `{:.6}`\n\
             - Source peak abs: `{:.6}`\n\
             - Riotbox after peak abs: `{:.6}`\n\
             - Signal delta peak abs: `{:.6}`\n\
             - Result: `{}`\n",
            source.rms,
            after.rms,
            delta.rms,
            source.peak_abs,
            after.peak_abs,
            delta.peak_abs,
            if source.rms > MIN_SOURCE_RMS && after.rms > MIN_AFTER_RMS && delta.rms > MIN_DELTA_RMS
            {
                "pass"
            } else {
                "fail"
            }
        ),
    )
}

fn write_readme(output_dir: &Path, args: &Args, source_excerpt_path: &Path) -> std::io::Result<()> {
    fs::write(
        output_dir.join("README.md"),
        format!(
            "# Feral Before / After Pack\n\n\
             - Pack: `{PACK_ID}`\n\
             - Source: `{}`\n\
             - Source window: `{:.3}s` to `{:.3}s`\n\
             - Source preview window for W-30: `{:.3}s`\n\n\
             ## Files\n\n\
             - `01_source_excerpt.wav`: direct source excerpt.\n\
             - `02_riotbox_feral_changed.wav`: Riotbox-rendered Feral preview mix.\n\
             - `03_before_then_after.wav`: source excerpt, short silence, then Riotbox after render.\n\
             - `comparison.md`: source-vs-after metrics.\n\n\
             - `manifest.json`: machine-readable artifact paths, thresholds, and metrics.\n\n\
             ## Stems\n\n\
             - `stems/w30_source_chop.wav`: source-backed W-30 preview render.\n\
             - `stems/tr909_fill.wav`: TR-909 fill render.\n\
             - `stems/mc202_instigator.wav`: MC-202 instigator render.\n\n\
             ## Current Limit\n\n\
             This pack proves a current offline listening/QA path. It does not claim the live TUI mixer can perform this combined result directly yet.\n\n\
             ## Source Excerpt\n\n\
             `{}`\n",
            args.source_path.display(),
            args.source_start_seconds,
            args.source_start_seconds + args.duration_seconds,
            args.source_window_seconds.min(args.duration_seconds),
            source_excerpt_path.display()
        ),
    )
}

#[derive(Serialize)]
struct ListeningPackManifest {
    schema_version: u32,
    pack_id: &'static str,
    source: String,
    sample_rate: u32,
    channel_count: u16,
    duration_seconds: f32,
    source_start_seconds: f32,
    source_window_seconds: f32,
    silence_seconds: f32,
    artifacts: Vec<ManifestArtifact>,
    thresholds: ManifestThresholds,
    metrics: ManifestMetrics,
    result: &'static str,
}

#[derive(Serialize)]
struct ManifestThresholds {
    min_source_rms: f32,
    min_after_rms: f32,
    min_delta_rms: f32,
}

#[derive(Clone, Copy, Serialize)]
struct ManifestMetrics {
    source_excerpt: ManifestSignalMetrics,
    riotbox_after: ManifestSignalMetrics,
    source_after_delta: ManifestSignalMetrics,
    w30_source_chop: ManifestSignalMetrics,
    tr909_fill: ManifestSignalMetrics,
    mc202_instigator: ManifestSignalMetrics,
}

fn write_manifest(
    path: &Path,
    args: &Args,
    output_dir: &Path,
    metrics: ManifestMetrics,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = ListeningPackManifest {
        schema_version: LISTENING_MANIFEST_SCHEMA_VERSION,
        pack_id: PACK_ID,
        source: args.source_path.display().to_string(),
        sample_rate: SAMPLE_RATE,
        channel_count: CHANNEL_COUNT,
        duration_seconds: args.duration_seconds,
        source_start_seconds: args.source_start_seconds,
        source_window_seconds: args.source_window_seconds.min(args.duration_seconds),
        silence_seconds: SILENCE_SECONDS,
        artifacts: manifest_artifacts(output_dir),
        thresholds: ManifestThresholds {
            min_source_rms: MIN_SOURCE_RMS,
            min_after_rms: MIN_AFTER_RMS,
            min_delta_rms: MIN_DELTA_RMS,
        },
        metrics,
        result: "pass",
    };

    write_manifest_json(path, &manifest)?;
    Ok(())
}

fn manifest_artifacts(output_dir: &Path) -> Vec<ManifestArtifact> {
    let source_path = output_dir.join("01_source_excerpt.wav");
    let source_metrics_path = output_dir.join("01_source_excerpt.metrics.md");
    let after_path = output_dir.join("02_riotbox_feral_changed.wav");
    let after_metrics_path = metrics_path_for(&after_path);
    let before_after_path = output_dir.join("03_before_then_after.wav");
    let w30_path = output_dir.join("stems/w30_source_chop.wav");
    let w30_metrics_path = metrics_path_for(&w30_path);
    let tr909_path = output_dir.join("stems/tr909_fill.wav");
    let tr909_metrics_path = metrics_path_for(&tr909_path);
    let mc202_path = output_dir.join("stems/mc202_instigator.wav");
    let mc202_metrics_path = metrics_path_for(&mc202_path);

    vec![
        ManifestArtifact::audio_wav("source_excerpt", &source_path, Some(&source_metrics_path)),
        ManifestArtifact::audio_wav("riotbox_after", &after_path, Some(&after_metrics_path)),
        ManifestArtifact::audio_wav("before_then_after", &before_after_path, None),
        ManifestArtifact::audio_wav("w30_source_chop", &w30_path, Some(&w30_metrics_path)),
        ManifestArtifact::audio_wav("tr909_fill", &tr909_path, Some(&tr909_metrics_path)),
        ManifestArtifact::audio_wav("mc202_instigator", &mc202_path, Some(&mc202_metrics_path)),
        ManifestArtifact::markdown_report("comparison", &output_dir.join("comparison.md")),
        ManifestArtifact::markdown_readme("readme", &output_dir.join("README.md")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_source_and_custom_output() {
        let parsed = Args::parse([
            "--source".to_string(),
            "input.wav".to_string(),
            "--output-dir".to_string(),
            "out".to_string(),
            "--duration-seconds".to_string(),
            "0.5".to_string(),
            "--source-window-seconds".to_string(),
            "0.25".to_string(),
        ])
        .expect("parse args");

        assert_eq!(parsed.source_path, PathBuf::from("input.wav"));
        assert_eq!(parsed.output_dir, Some(PathBuf::from("out")));
        assert_eq!(parsed.duration_seconds, 0.5);
        assert_eq!(parsed.source_window_seconds, 0.25);
    }

    #[test]
    fn rejects_missing_source() {
        assert!(Args::parse(Vec::<String>::new()).is_err());
    }

    #[test]
    fn renders_pack_files_and_distinct_after_audio() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source_path = temp.path().join("source.wav");
        let output_dir = temp.path().join("pack");
        write_interleaved_pcm16_wav(
            &source_path,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &synthetic_break_source(seconds_to_frames(0.5)),
        )
        .expect("write source");

        let args = Args {
            source_path,
            output_dir: Some(output_dir.clone()),
            date: "test".into(),
            source_start_seconds: 0.0,
            duration_seconds: 0.5,
            source_window_seconds: 0.25,
            show_help: false,
        };

        render_pack(&args).expect("render pack");

        assert!(output_dir.join("01_source_excerpt.wav").is_file());
        assert!(output_dir.join("02_riotbox_feral_changed.wav").is_file());
        assert!(output_dir.join("03_before_then_after.wav").is_file());
        assert!(output_dir.join("stems/w30_source_chop.wav").is_file());
        assert!(output_dir.join("stems/tr909_fill.wav").is_file());
        assert!(output_dir.join("stems/mc202_instigator.wav").is_file());
        assert!(output_dir.join("comparison.md").is_file());
        assert!(output_dir.join("manifest.json").is_file());

        let source = SourceAudioCache::load_pcm_wav(output_dir.join("01_source_excerpt.wav"))
            .expect("load source");
        let after = SourceAudioCache::load_pcm_wav(output_dir.join("02_riotbox_feral_changed.wav"))
            .expect("load after");
        let source_metrics = signal_metrics(source.interleaved_samples());
        let after_metrics = signal_metrics(after.interleaved_samples());
        let delta = signal_delta_metrics(source.interleaved_samples(), after.interleaved_samples());

        assert!(source_metrics.rms > 0.001);
        assert!(after_metrics.rms > 0.001);
        assert!(delta.rms > 0.005);

        let manifest = fs::read_to_string(output_dir.join("manifest.json")).expect("manifest");
        let manifest: serde_json::Value = serde_json::from_str(&manifest).expect("parse manifest");
        assert_eq!(
            manifest["schema_version"],
            LISTENING_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(manifest["pack_id"], PACK_ID);
        assert_eq!(manifest["result"], "pass");
        assert_eq!(
            manifest["artifacts"].as_array().expect("artifacts").len(),
            8
        );
        assert!(
            manifest["metrics"]["riotbox_after"]["rms"]
                .as_f64()
                .expect("after rms")
                > f64::from(MIN_AFTER_RMS)
        );
        assert!(
            manifest["metrics"]["source_after_delta"]["rms"]
                .as_f64()
                .expect("delta rms")
                > f64::from(MIN_DELTA_RMS)
        );
        for artifact in manifest["artifacts"].as_array().expect("artifacts") {
            let path = PathBuf::from(artifact["path"].as_str().expect("artifact path"));
            assert!(path.is_file(), "{} missing", path.display());
            if let Some(metrics_path) = artifact["metrics_path"].as_str() {
                let metrics_path = PathBuf::from(metrics_path);
                assert!(metrics_path.is_file(), "{} missing", metrics_path.display());
            }
        }
    }

    fn synthetic_break_source(frame_count: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(frame_count * usize::from(CHANNEL_COUNT));
        for frame in 0..frame_count {
            let phase = frame as f32 / SAMPLE_RATE as f32;
            let beat = frame % 11_025;
            let kick = if beat < 1_200 {
                ((1.0 - beat as f32 / 1_200.0).max(0.0) * 0.9)
                    * (phase * 80.0 * std::f32::consts::TAU).sin()
            } else {
                0.0
            };
            let grit = (phase * 730.0 * std::f32::consts::TAU).sin() * 0.08;
            let sample = kick + grit;
            samples.push(sample);
            samples.push(sample * 0.96);
        }
        samples
    }
}
