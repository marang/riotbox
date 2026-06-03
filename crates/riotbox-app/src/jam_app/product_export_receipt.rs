use std::path::Path;

use riotbox_audio::{runtime::signal_metrics, source_audio::SourceAudioCache};
use riotbox_core::session::{
    ExportArtifactAudioMetrics, ExportArtifactLocation, ExportArtifactMediaType,
    ExportArtifactRole, ExportArtifactSourceGraphRef, ExportArtifactTimingGridRef,
    ExportReceiptState, SessionFile,
};

pub(super) fn attach_product_export_artifact_lineage(
    receipt: &mut ExportReceiptState,
    session: &SessionFile,
) {
    if let Some(source_graph_ref) = export_artifact_source_graph_ref(session) {
        receipt.attach_artifact_source_graph_ref(ExportArtifactRole::FullGridMix, source_graph_ref);
    }
    if let Some(timing_grid_ref) = export_artifact_timing_grid_ref(session) {
        receipt.attach_artifact_timing_grid_ref(ExportArtifactRole::FullGridMix, timing_grid_ref);
    }
}

pub(super) fn attach_product_export_artifact_audio_metrics(receipt: &mut ExportReceiptState) {
    for artifact in &mut receipt.artifact_set {
        if artifact.role != ExportArtifactRole::FullGridMix
            || artifact.media_type != ExportArtifactMediaType::AudioWav
        {
            continue;
        }

        let ExportArtifactLocation::LocalPath { path } = &artifact.location else {
            continue;
        };
        let Some(evidence) = local_wav_audio_evidence(path) else {
            continue;
        };

        artifact.audio_metrics = Some(evidence.audio_metrics);
        artifact.sample_rate_hz = Some(evidence.sample_rate_hz);
        artifact.channel_count = Some(evidence.channel_count);
        artifact.duration_ms = Some(evidence.duration_ms);
    }
}

fn export_artifact_source_graph_ref(session: &SessionFile) -> Option<ExportArtifactSourceGraphRef> {
    session
        .source_graph_refs
        .first()
        .map(|graph_ref| ExportArtifactSourceGraphRef {
            source_id: graph_ref.source_id.clone(),
            graph_version: graph_ref.graph_version,
            graph_hash: graph_ref.graph_hash.clone(),
        })
}

fn export_artifact_timing_grid_ref(session: &SessionFile) -> Option<ExportArtifactTimingGridRef> {
    session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .map(|confirmed_grid| ExportArtifactTimingGridRef {
            source_id: confirmed_grid.source_id.clone(),
            hypothesis_id: confirmed_grid.hypothesis_id.clone(),
            confirmed_by_action: confirmed_grid.confirmed_by_action,
            confirmed_at: confirmed_grid.confirmed_at,
        })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LocalWavAudioEvidence {
    pub(super) audio_metrics: ExportArtifactAudioMetrics,
    pub(super) sample_rate_hz: u32,
    pub(super) channel_count: u16,
    pub(super) duration_ms: u64,
}

pub(super) fn local_wav_audio_evidence(path: impl AsRef<Path>) -> Option<LocalWavAudioEvidence> {
    let cache = SourceAudioCache::load_pcm_wav(path).ok()?;
    let samples = cache.interleaved_samples();
    let metrics = signal_metrics(samples);
    let frame_count = cache.frame_count() as u64;
    let channel_count = usize::from(cache.channel_count);
    let silent_frame_count = samples
        .chunks(channel_count)
        .filter(|frame| frame.iter().all(|sample| sample.abs() <= 0.0001))
        .count() as u64;

    Some(LocalWavAudioEvidence {
        audio_metrics: ExportArtifactAudioMetrics {
            peak_milli_dbfs: amplitude_to_milli_dbfs(metrics.peak_abs),
            rms_milli_dbfs: amplitude_to_milli_dbfs(metrics.rms),
            peak_amplitude_micros: Some(amplitude_to_micros(metrics.peak_abs)),
            rms_amplitude_micros: Some(amplitude_to_micros(metrics.rms)),
            silent_frame_count: Some(silent_frame_count),
            total_frame_count: Some(frame_count),
        },
        sample_rate_hz: cache.sample_rate,
        channel_count: cache.channel_count,
        duration_ms: frame_count.saturating_mul(1000) / u64::from(cache.sample_rate),
    })
}

fn amplitude_to_milli_dbfs(amplitude: f32) -> Option<i32> {
    (amplitude > 0.0).then(|| (20.0 * amplitude.log10() * 1000.0).round() as i32)
}

fn amplitude_to_micros(amplitude: f32) -> u32 {
    (amplitude.max(0.0) * 1_000_000.0)
        .round()
        .min(u32::MAX as f32) as u32
}
