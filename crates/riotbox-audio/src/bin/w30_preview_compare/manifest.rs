use super::args::Args;
use super::comparison::ComparisonReport;
use super::config::{CASE_ID, DriftLimits, PACK_ID};
use super::metrics_input::SmokeMetrics;
use riotbox_audio::listening_manifest::{
    LISTENING_MANIFEST_SCHEMA_VERSION, ListeningPackArtifact as ManifestArtifact,
    write_manifest_json,
};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct W30PreviewSmokeManifest {
    schema_version: u32,
    pack_id: &'static str,
    case_id: &'static str,
    artifacts: Vec<ManifestArtifact>,
    limits: DriftLimits,
    metrics: ManifestMetrics,
    result: &'static str,
}

#[derive(Serialize)]
struct ManifestMetrics {
    baseline: SmokeMetrics,
    candidate: SmokeMetrics,
    deltas: ManifestMetricDeltas,
}

#[derive(Serialize)]
struct ManifestMetricDeltas {
    active_samples: usize,
    peak_abs: f64,
    rms: f64,
    sum: f64,
    mean_abs: f64,
    zero_crossings: usize,
    crest_factor: f64,
    active_sample_ratio: f64,
    silence_ratio: f64,
    dc_offset: f64,
    onset_count: usize,
    event_density_per_bar: f64,
}

pub(super) fn write_manifest(
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
                onset_count: report.onset_count.delta,
                event_density_per_bar: report.event_density_per_bar.delta,
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

pub(super) fn manifest_path_for_report_path(report_path: &Path) -> PathBuf {
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

pub(super) fn audio_path_for_metrics_path(metrics_path: &Path) -> PathBuf {
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
