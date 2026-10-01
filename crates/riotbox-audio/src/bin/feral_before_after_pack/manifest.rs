use super::args::Args;
use super::artifact_io::metrics_path_for;
use super::config::{
    CHANNEL_COUNT, MIN_AFTER_RMS, MIN_DELTA_RMS, MIN_SOURCE_RMS, PACK_ID, SAMPLE_RATE,
    SILENCE_SECONDS,
};
use riotbox_audio::listening_manifest::{
    LISTENING_MANIFEST_SCHEMA_VERSION, ListeningPackArtifact as ManifestArtifact,
    ListeningPackSignalMetrics as ManifestSignalMetrics, write_manifest_json,
};
use serde::Serialize;
use std::path::Path;

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
pub(super) struct ManifestMetrics {
    pub(super) source_excerpt: ManifestSignalMetrics,
    pub(super) riotbox_after: ManifestSignalMetrics,
    pub(super) source_after_delta: ManifestSignalMetrics,
    pub(super) w30_source_chop: ManifestSignalMetrics,
    pub(super) tr909_fill: ManifestSignalMetrics,
    pub(super) mc202_instigator: ManifestSignalMetrics,
}

pub(super) fn write_manifest(
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
