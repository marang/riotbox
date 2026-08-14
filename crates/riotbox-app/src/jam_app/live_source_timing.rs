use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use riotbox_audio::{
    source_audio::SourceAudioCache,
    source_timing_probe::{SourceTimingProbeConfig, analyze_source_timing_probe},
    w30_hook_analysis::analyze_w30_hook_candidates,
};
use riotbox_core::{
    action::CommitBoundary,
    source_graph::{
        ManualSourceTimingGrid, MeterHint, QualityClass, SourceGraph,
        SourceTimingProbeBpmCandidatePolicy, TimingQuality, install_manual_source_timing_grid,
        timing_model_from_probe_bpm_candidates,
    },
    transport::CommitBoundaryState,
};

use super::{JamAppError, JamAppState, QueueControlResult};

const RUST_TIMING_PROVIDER: &str = "riotbox-rust-source-timing-probe";
const EXPLICIT_BPM_MATCH_TOLERANCE: f32 = 1.0;

pub(super) fn install_explicit_manual_source_grid(
    graph: &mut SourceGraph,
    explicit_source_bpm: f32,
    explicit_downbeat_seconds: f32,
) -> Result<(), JamAppError> {
    install_manual_source_timing_grid(
        graph,
        ManualSourceTimingGrid {
            bpm: explicit_source_bpm,
            downbeat_seconds: explicit_downbeat_seconds,
        },
    )
    .map_err(JamAppError::InvalidSession)?;

    let note = format!(
        "musician declared manual source grid at {explicit_source_bpm:.3} BPM with downbeat {explicit_downbeat_seconds:.6}s"
    );
    graph.provenance.run_notes = Some(match graph.provenance.run_notes.take() {
        Some(existing) if !existing.is_empty() => format!("{existing}; {note}"),
        _ => note,
    });
    Ok(())
}

pub(super) fn enrich_graph_with_rust_source_timing(
    graph: &mut SourceGraph,
    source_path: &Path,
) -> Result<SourceAudioCache, JamAppError> {
    let source = SourceAudioCache::load_pcm_wav(source_path).map_err(|error| {
        JamAppError::InvalidSession(format!(
            "live source timing could not decode {}: {error}",
            source_path.display()
        ))
    })?;
    let probe = analyze_source_timing_probe(&source, SourceTimingProbeConfig::default());
    let meter = graph.timing.meter_hint.unwrap_or(MeterHint {
        beats_per_bar: 4,
        beat_unit: 4,
    });
    let input = probe.bpm_candidate_input(graph.source.source_id.to_string(), meter);
    graph.timing = timing_model_from_probe_bpm_candidates(
        &input,
        SourceTimingProbeBpmCandidatePolicy::dance_loop_auto_readiness(),
    );
    graph.analysis_summary.timing_quality = match graph.timing.effective_timing_quality() {
        TimingQuality::High => QualityClass::High,
        TimingQuality::Medium => QualityClass::Medium,
        TimingQuality::Low => QualityClass::Low,
        TimingQuality::Unknown => QualityClass::Unknown,
    };
    if !graph
        .provenance
        .provider_set
        .iter()
        .any(|provider| provider == RUST_TIMING_PROVIDER)
    {
        graph
            .provenance
            .provider_set
            .push(RUST_TIMING_PROVIDER.into());
    }
    let note = "live timing replaced by deterministic Rust source-timing probe";
    graph.provenance.run_notes = Some(match graph.provenance.run_notes.take() {
        Some(existing) if !existing.is_empty() => format!("{existing}; {note}"),
        _ => note.into(),
    });
    Ok(source)
}

pub(super) fn attach_w30_hook_candidate_evidence(
    graph: &mut SourceGraph,
    source: &SourceAudioCache,
) {
    let Some(primary) = graph.timing.primary_hypothesis() else {
        graph.w30_hook_candidates.clear();
        return;
    };
    graph.w30_hook_candidates =
        analyze_w30_hook_candidates(source, &primary.bar_grid, primary.meter.beats_per_bar);
}

pub(super) fn confirm_explicit_source_bpm(
    state: &mut JamAppState,
    explicit_source_bpm: f32,
) -> Result<(), JamAppError> {
    let graph = state.source_graph.as_ref().ok_or_else(|| {
        JamAppError::InvalidSession(
            "explicit source BPM cannot confirm timing without a Source Graph".into(),
        )
    })?;
    validate_explicit_source_bpm(graph, explicit_source_bpm)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    if state.queue_source_timing_grid_confirmation(timestamp) != QueueControlResult::Enqueued {
        return Err(JamAppError::InvalidSession(
            "explicit source BPM confirmation could not be queued".into(),
        ));
    }
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Immediate,
            beat_index: state.runtime.transport.beat_index,
            bar_index: state.runtime.transport.bar_index,
            phrase_index: state.runtime.transport.phrase_index,
            scene_id: state.runtime.transport.current_scene.clone(),
        },
        timestamp,
    );
    if committed.len() != 1 {
        return Err(JamAppError::InvalidSession(
            "explicit source BPM confirmation did not commit exactly one action".into(),
        ));
    }
    Ok(())
}

pub(super) fn validate_explicit_source_bpm(
    graph: &SourceGraph,
    explicit_source_bpm: f32,
) -> Result<(), JamAppError> {
    if !explicit_source_bpm.is_finite() || explicit_source_bpm <= 0.0 {
        return Err(JamAppError::InvalidSession(format!(
            "explicit source BPM must be finite and positive, got {explicit_source_bpm}"
        )));
    }
    let detected_bpm = graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.bpm)
        .ok_or_else(|| {
            JamAppError::InvalidSession(
                "explicit source BPM cannot confirm timing because the Rust probe produced no primary grid"
                    .into(),
            )
        })?;
    if (detected_bpm - explicit_source_bpm).abs() > EXPLICIT_BPM_MATCH_TOLERANCE {
        return Err(JamAppError::InvalidSession(format!(
            "explicit source BPM {explicit_source_bpm:.2} does not match Rust timing candidate {detected_bpm:.2} within {EXPLICIT_BPM_MATCH_TOLERANCE:.2} BPM"
        )));
    }

    Ok(())
}
