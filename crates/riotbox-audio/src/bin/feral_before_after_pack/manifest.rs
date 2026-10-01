use super::args::Args;
use super::artifact_io::metrics_path_for;
use super::config::{
    CHANNEL_COUNT, MIN_AFTER_RMS, MIN_DELTA_RMS, MIN_SOURCE_RMS, PACK_ID, SAMPLE_RATE,
    SILENCE_SECONDS,
};
use super::output_paths::PackOutputPaths;
use riotbox_audio::listening_manifest::{
    LISTENING_MANIFEST_SCHEMA_VERSION, ListeningPackArtifact as ManifestArtifact,
    ListeningPackSignalMetrics as ManifestSignalMetrics, write_manifest_json,
};
use serde::Serialize;

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
    args: &Args,
    paths: &PackOutputPaths,
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
        artifacts: manifest_artifacts(paths),
        thresholds: ManifestThresholds {
            min_source_rms: MIN_SOURCE_RMS,
            min_after_rms: MIN_AFTER_RMS,
            min_delta_rms: MIN_DELTA_RMS,
        },
        metrics,
        result: "pass",
    };

    write_manifest_json(&paths.manifest, &manifest)?;
    Ok(())
}

fn manifest_artifacts(paths: &PackOutputPaths) -> Vec<ManifestArtifact> {
    let source_metrics_path = metrics_path_for(&paths.source_excerpt);
    let after_metrics_path = metrics_path_for(&paths.after);
    let w30_metrics_path = metrics_path_for(&paths.w30);
    let tr909_metrics_path = metrics_path_for(&paths.tr909);
    let mc202_metrics_path = metrics_path_for(&paths.mc202);

    vec![
        ManifestArtifact::audio_wav(
            "source_excerpt",
            &paths.source_excerpt,
            Some(&source_metrics_path),
        ),
        ManifestArtifact::audio_wav("riotbox_after", &paths.after, Some(&after_metrics_path)),
        ManifestArtifact::audio_wav("before_then_after", &paths.before_after, None),
        ManifestArtifact::audio_wav("w30_source_chop", &paths.w30, Some(&w30_metrics_path)),
        ManifestArtifact::audio_wav("tr909_fill", &paths.tr909, Some(&tr909_metrics_path)),
        ManifestArtifact::audio_wav("mc202_instigator", &paths.mc202, Some(&mc202_metrics_path)),
        ManifestArtifact::markdown_report("comparison", &paths.comparison),
        ManifestArtifact::markdown_readme("readme", &paths.readme),
    ]
}
