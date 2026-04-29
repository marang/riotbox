fn render_report(baseline_path: &Path, candidate_path: &Path, report: &ComparisonReport) -> String {
    format!(
        "W-30 preview smoke metrics comparison\n\
         baseline: {}\n\
         candidate: {}\n\
         active_samples: {} -> {} | delta {} | min {} | max {} | {}\n\
         peak_abs: {:.6} -> {:.6} | delta {:.6} | min {:.6} | max {:.6} | {}\n\
         rms: {:.6} -> {:.6} | delta {:.6} | min {:.6} | max {:.6} | {}\n\
         sum: {:.6} -> {:.6} | delta {:.6} | min {:.6} | max {:.6} | {}\n\
         mean_abs: {:.6} -> {:.6} | delta {:.6} | diagnostic\n\
         zero_crossings: {} -> {} | delta {} | diagnostic\n\
         crest_factor: {:.6} -> {:.6} | delta {:.6} | diagnostic\n\
         active_sample_ratio: {:.6} -> {:.6} | delta {:.6} | diagnostic\n\
         silence_ratio: {:.6} -> {:.6} | delta {:.6} | diagnostic\n\
         dc_offset: {:.6} -> {:.6} | delta {:.6} | diagnostic\n\
         result: {}",
        baseline_path.display(),
        candidate_path.display(),
        report.active_samples.baseline,
        report.active_samples.candidate,
        report.active_samples.delta,
        report.active_samples.min_delta,
        report.active_samples.max_delta,
        status_label(report.active_samples.passed),
        report.peak_abs.baseline,
        report.peak_abs.candidate,
        report.peak_abs.delta,
        report.peak_abs.min_delta,
        report.peak_abs.max_delta,
        status_label(report.peak_abs.passed),
        report.rms.baseline,
        report.rms.candidate,
        report.rms.delta,
        report.rms.min_delta,
        report.rms.max_delta,
        status_label(report.rms.passed),
        report.sum.baseline,
        report.sum.candidate,
        report.sum.delta,
        report.sum.min_delta,
        report.sum.max_delta,
        status_label(report.sum.passed),
        report.mean_abs.baseline,
        report.mean_abs.candidate,
        report.mean_abs.delta,
        report.zero_crossings.baseline,
        report.zero_crossings.candidate,
        report.zero_crossings.delta,
        report.crest_factor.baseline,
        report.crest_factor.candidate,
        report.crest_factor.delta,
        report.active_sample_ratio.baseline,
        report.active_sample_ratio.candidate,
        report.active_sample_ratio.delta,
        report.silence_ratio.baseline,
        report.silence_ratio.candidate,
        report.silence_ratio.delta,
        report.dc_offset.baseline,
        report.dc_offset.candidate,
        report.dc_offset.delta,
        if report.has_failures() {
            "fail"
        } else {
            "pass"
        }
    )
}

fn write_report_markdown(path: &Path, report: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{report}\n"))
}

fn write_manifest(
    args: &Args,
    baseline: SmokeMetrics,
    candidate: SmokeMetrics,
    report: &ComparisonReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = manifest_path_for_report_path(&args.report_path);
    let artifacts = manifest_artifacts(
        &args.baseline_metrics_path,
        &args.candidate_metrics_path,
        &args.report_path,
    );
    ensure_manifest_artifacts_exist(&artifacts)?;

    let manifest = W30PreviewSmokeManifest {
        schema_version: LISTENING_MANIFEST_SCHEMA_VERSION,
        pack_id: PACK_ID,
        case_id: CASE_ID,
        artifacts,
        limits: args.limits,
        metrics: ManifestMetrics {
            baseline,
            candidate,
            deltas: ManifestMetricDeltas {
                active_samples: report.active_samples.delta,
                peak_abs: report.peak_abs.delta,
                rms: report.rms.delta,
                sum: report.sum.delta,
                mean_abs: report.mean_abs.delta,
                zero_crossings: report.zero_crossings.delta,
                crest_factor: report.crest_factor.delta,
                active_sample_ratio: report.active_sample_ratio.delta,
                silence_ratio: report.silence_ratio.delta,
                dc_offset: report.dc_offset.delta,
            },
        },
        result: if report.has_failures() {
            "fail"
        } else {
            "pass"
        },
    };

    write_manifest_json(&manifest_path, &manifest)?;
    Ok(())
}

fn manifest_path_for_report_path(report_path: &Path) -> PathBuf {
    report_path.with_file_name("manifest.json")
}

fn manifest_artifacts(
    baseline_metrics_path: &Path,
    candidate_metrics_path: &Path,
    report_path: &Path,
) -> Vec<ManifestArtifact> {
    let baseline_audio_path = audio_path_for_metrics_path(baseline_metrics_path);
    let candidate_audio_path = audio_path_for_metrics_path(candidate_metrics_path);
    vec![
        ManifestArtifact::audio_wav(
            "baseline",
            &baseline_audio_path,
            Some(baseline_metrics_path),
        ),
        ManifestArtifact::audio_wav(
            "candidate",
            &candidate_audio_path,
            Some(candidate_metrics_path),
        ),
        ManifestArtifact::markdown_report("comparison", report_path),
    ]
}

fn audio_path_for_metrics_path(metrics_path: &Path) -> PathBuf {
    let mut audio_path = metrics_path.to_path_buf();
    let stem = metrics_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.strip_suffix(".metrics").or(Some(stem)))
        .unwrap_or("audio");
    audio_path.set_file_name(format!("{stem}.wav"));
    audio_path
}

fn ensure_manifest_artifacts_exist(
    artifacts: &[ManifestArtifact],
) -> Result<(), Box<dyn std::error::Error>> {
    for artifact in artifacts {
        let path = Path::new(&artifact.path);
        if !path.is_file() {
            return Err(format!("manifest artifact does not exist: {}", path.display()).into());
        }
        if let Some(metrics_path) = artifact.metrics_path.as_deref() {
            let metrics_path = Path::new(metrics_path);
            if !metrics_path.is_file() {
                return Err(format!(
                    "manifest metrics artifact does not exist: {}",
                    metrics_path.display()
                )
                .into());
            }
        }
    }
    Ok(())
}

const fn status_label(passed: bool) -> &'static str {
    if passed { "ok" } else { "drift" }
}
