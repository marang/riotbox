//! Existing committed performer prefixes, without legacy pack/gesture execution.

use std::error::Error;

use riotbox_app::jam_app::{JamAppState, QueueControlResult};
use riotbox_audio::{
    mc202::Mc202RenderRouting,
    runtime::{AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, SourceMonitorRenderState},
    tr909::Tr909RenderMode,
    w30::{W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewSourceProfile},
};
use riotbox_core::{
    action::{ActionCommand, ActionStatus, CaptureLengthIntent, CommitBoundary, SourceMonitorMode},
    live_performance_policy::{
        LivePerformanceBassOwner, LivePerformanceCharacter, LivePerformanceMc202Intent,
        LivePerformanceTr909Intent, derive_live_performance_policy,
    },
    session::W30HookSelectionPolicy,
    style::PerformancePresetId,
    transport::CommitBoundaryState,
};
use serde_json::{Value, json};

use crate::{calibration::Case, live_flow};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub(super) fn prepare(state: &mut JamAppState, case: Case) -> Result<RuntimeMixRenderPlan> {
    if !state.session.action_log.actions.is_empty()
        || !state.session.captures.is_empty()
        || !state.queue.pending_actions().is_empty()
    {
        return Err("calibration preparation requires a fresh ingest Session".into());
    }
    // Dense's graph estimate is 130.28494; the existing user-confirmed value is
    // 130.0. Reuse the explicit confirmation owner, never mutate graph metadata.
    state.confirm_limiter_calibration_source_bpm(case.bpm(), 10)?;
    state.set_transport_playing(true);
    let plan = if case == Case::Sparse {
        prepare_sparse(state)?
    } else {
        prepare_w30(state, case)?
    };
    require_committed_prefix(state, case)?;
    require_ordinary_capture(state, &plan)?;
    Ok(plan)
}

fn require_queued(result: QueueControlResult) -> Result<()> {
    if result != QueueControlResult::Enqueued {
        return Err(format!("calibration preparation action was not enqueued: {result:?}").into());
    }
    Ok(())
}

fn require_pad_queued(result: Option<QueueControlResult>) -> Result<()> {
    require_queued(result.ok_or("calibration W-30 action was unavailable")?)
}

fn require_promotion(state: &mut JamAppState, timestamp: u64) -> Result<()> {
    if !state.queue_promote_last_capture(timestamp) {
        return Err("calibration capture promotion was unavailable".into());
    }
    Ok(())
}

// Preserve w30_live_path_render's exact boundary adapter: unlike live_flow's
// helper it does not also move the current runtime transport clock.
fn w30_commit(
    state: &mut JamAppState,
    kind: CommitBoundary,
    beat: u64,
    bar: u64,
    timestamp: u64,
) -> Result<()> {
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind,
            beat_index: beat,
            bar_index: bar,
            phrase_index: 1,
            scene_id: state.runtime.transport.current_scene.clone(),
        },
        timestamp,
    );
    if committed.len() != 1 {
        return Err("calibration W-30 boundary did not commit exactly one action".into());
    }
    Ok(())
}

fn prepare_w30(state: &mut JamAppState, case: Case) -> Result<RuntimeMixRenderPlan> {
    require_queued(state.queue_performance_preset(PerformancePresetId::FeralBreakAlphaV2, 90))?;
    w30_commit(state, CommitBoundary::Immediate, 0, 1, 95)?;
    state.queue_capture_length_intent(CaptureLengthIntent::OneBar, 96);
    w30_commit(state, CommitBoundary::Immediate, 0, 1, 97)?;
    state.queue_capture_bar(100);
    w30_commit(state, CommitBoundary::Bar, 0, 1, 200)?;
    require_promotion(state, 210)?;
    w30_commit(state, CommitBoundary::Bar, 5, 2, 300)?;
    require_pad_queued(state.queue_w30_trigger_pad(310))?;
    w30_commit(state, CommitBoundary::Beat, 6, 2, 400)?;
    // This is the established six-action ordinary W-30 projection, not the
    // dense_break_live_path_render multi-lane preparation that failed V3.
    Ok(RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: case.bpm(),
            position_beats: case.start_beat(),
        },
        w30_preview_render: state.runtime.w30_preview.clone(),
        tr909_render: Default::default(),
        mc202_render: Default::default(),
        w30_resample_tap: Default::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
    })
}

fn sparse_commit(
    state: &mut JamAppState,
    kind: CommitBoundary,
    beat: u64,
    timestamp: u64,
) -> Result<()> {
    live_flow::commit(
        state,
        kind,
        beat,
        live_flow::current_scene(state),
        timestamp,
        1,
    )?;
    Ok(())
}

fn prepare_sparse(state: &mut JamAppState) -> Result<RuntimeMixRenderPlan> {
    let hypothesis = state
        .source_graph
        .as_ref()
        .and_then(|graph| graph.timing.primary_hypothesis())
        .ok_or("calibration sparse preparation requires source timing")?;
    let anchor = hypothesis
        .transport_bar_grid_anchor()
        .ok_or("calibration sparse preparation requires a bar anchor")?;
    if anchor.beat_cursor != 0 || anchor.bar_index != 1 || hypothesis.meter.beats_per_bar != 4 {
        return Err(
            "calibration sparse preparation requires the registered zero-phase 4/4 grid".into(),
        );
    }
    if state.session.runtime_state.source_monitor.mode != SourceMonitorMode::Source {
        return Err("calibration sparse prefix must begin in Source monitor mode".into());
    }
    state.queue_capture_length_intent(CaptureLengthIntent::OneBar, 90);
    sparse_commit(state, CommitBoundary::Immediate, 0, 95)?;
    state.queue_capture_bar(100);
    sparse_commit(state, CommitBoundary::Bar, 0, 200)?;
    require_pad_queued(state.queue_w30_raw_capture_audition(210))?;
    sparse_commit(state, CommitBoundary::Bar, 4, 220)?;
    require_promotion(state, 230)?;
    sparse_commit(state, CommitBoundary::Bar, 8, 300)?;
    require_queued(state.queue_performance_preset(PerformancePresetId::FeralBreakAlphaV2, 305))?;
    sparse_commit(state, CommitBoundary::Immediate, 8, 306)?;
    require_queued(state.queue_mc202_generate_instigator(311))?;
    sparse_commit(state, CommitBoundary::Phrase, 16, 400)?;
    let policy = state
        .source_graph
        .as_ref()
        .and_then(|graph| derive_live_performance_policy(&state.session, graph))
        .ok_or("calibration sparse live policy is unavailable")?;
    if policy.character != LivePerformanceCharacter::SparsePressure
        || policy.tr909_intent != LivePerformanceTr909Intent::Lead
        || policy.mc202_intent != LivePerformanceMc202Intent::Punctuate
        || policy.bass_owner != LivePerformanceBassOwner::Unassigned
    {
        return Err("calibration sparse source-derived contributor policy changed".into());
    }
    for (mode, timestamp) in [
        (SourceMonitorMode::Source, 420),
        (SourceMonitorMode::Blend, 430),
        (SourceMonitorMode::Riotbox, 440),
    ] {
        require_queued(state.queue_source_monitor_mode(mode, timestamp))?;
        sparse_commit(state, CommitBoundary::Immediate, 16, timestamp + 1)?;
    }
    require_pad_queued(state.queue_w30_trigger_pad(500))?;
    sparse_commit(state, CommitBoundary::Beat, 17, 510)?;
    let plan = live_flow::render_plan(state, 120.0, 17.0);
    if plan.tr909_render.mode == Tr909RenderMode::Idle
        || plan.mc202_render.routing == Mc202RenderRouting::Silent
        || plan.source_monitor_render.mode != SourceMonitorMode::Riotbox
        || plan.w30_resample_tap != Default::default()
    {
        return Err(
            "calibration sparse multi-lane projection is unavailable or has an extra contributor"
                .into(),
        );
    }
    Ok(plan)
}

fn require_committed_prefix(state: &JamAppState, case: Case) -> Result<()> {
    use ActionCommand::{
        CaptureBarGroup, CaptureSetLength, Mc202GenerateInstigator, PresetActivate,
        PromoteCaptureToPad, SourceMonitorSetMode, SourceTimingConfirmGrid, W30AuditionRawCapture,
        W30TriggerPad,
    };
    let expected: &[ActionCommand] = if case == Case::Sparse {
        &[
            SourceTimingConfirmGrid,
            CaptureSetLength,
            CaptureBarGroup,
            W30AuditionRawCapture,
            PromoteCaptureToPad,
            PresetActivate,
            Mc202GenerateInstigator,
            SourceMonitorSetMode,
            SourceMonitorSetMode,
            SourceMonitorSetMode,
            W30TriggerPad,
        ]
    } else {
        &[
            SourceTimingConfirmGrid,
            PresetActivate,
            CaptureSetLength,
            CaptureBarGroup,
            PromoteCaptureToPad,
            W30TriggerPad,
        ]
    };
    let log = &state.session.action_log;
    if log.actions.len() != expected.len()
        || log.commit_records.len() != expected.len()
        || !state.queue.pending_actions().is_empty()
        || log.actions.iter().zip(expected).any(|(action, command)| {
            action.command != *command
                || action.status != ActionStatus::Committed
                || !action.result.as_ref().is_some_and(|result| result.accepted)
        })
    {
        return Err(
            "calibration action prefix contains a missing, rejected, or extra action".into(),
        );
    }
    if state.session.runtime_state.source_timing.confirmed_bpm != Some(case.bpm())
        || state.session.runtime_state.style.w30_hook_selection_policy
            != W30HookSelectionPolicy::TransportBoundaryV1
    {
        return Err("calibration committed BPM or ordinary hook policy changed".into());
    }
    Ok(())
}

fn require_ordinary_capture(state: &JamAppState, plan: &RuntimeMixRenderPlan) -> Result<()> {
    let render = &plan.w30_preview_render;
    let pad = render
        .pad_playback
        .as_ref()
        .ok_or("calibration lost ordinary W-30 pad playback")?;
    if state.session.captures.len() != 1
        || render.mode != W30PreviewRenderMode::LiveRecall
        || render.routing != W30PreviewRenderRouting::MusicBusPreview
        || render.source_profile != Some(W30PreviewSourceProfile::PromotedRecall)
        || render.tempo_bpm != plan.transport.tempo_bpm
        || pad.playback_rate != 1.0
        || pad.reverse
        || pad.gate_step_fraction != 0.0
        || pad.hook_articulation.is_some()
    {
        return Err("calibration W-30 owner is not the ordinary promoted capture".into());
    }
    let capture = &state.session.captures[0];
    if capture.audio_identity.is_none()
        || !state.capture_audio_cache.contains_key(&capture.capture_id)
    {
        return Err("calibration capture did not retain artifact-backed playback identity".into());
    }
    Ok(())
}

pub(super) fn evidence(state: &JamAppState, case: Case) -> Result<Value> {
    let capture = state
        .session
        .captures
        .first()
        .ok_or("calibration capture missing")?;
    let window = capture
        .source_window
        .as_ref()
        .ok_or("calibration capture source window missing")?;
    let expected_end = if case == Case::Dense { 81_237 } else { 88_200 };
    if window.start_frame != 0 || window.end_frame != expected_end {
        return Err("calibration capture window differs from the frozen historical binding".into());
    }
    let contributors = if case == Case::Sparse {
        json!({"w30_preview": "source_transform", "tr909": "hardest_transient",
            "mc202": "punctuation", "source_monitor": "silent", "w30_resample_tap": "silent",
            "bass_owner": "unassigned"})
    } else {
        json!({"w30_preview": "source_transform_foundation", "tr909": "silent",
            "mc202": "silent", "source_monitor": "silent", "w30_resample_tap": "silent"})
    };
    Ok(json!({
        "recipe": if case == Case::Sparse { "sparse_existing_prefix_after_w_v1" } else { "six_action_w30_owner_v1" },
        "committed_bpm": case.bpm(), "start_beat": case.start_beat(), "duration_beats": 8,
        "committed_actions": state.session.action_log.actions,
        "commit_records": state.session.action_log.commit_records,
        "capture_window": window, "capture_audio_identity": capture.audio_identity,
        "source_timing": state.session.runtime_state.source_timing,
        "contributors": contributors,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use riotbox_app::jam_app::JamFileSet;
    use riotbox_audio::{
        runtime::{RuntimeMixRenderSequenceStep, limiter_calibration::render_sequence},
        source_audio::pcm16_wave_bytes,
    };
    use riotbox_core::source_graph::{
        DecodeProfile, EnergyClass, GraphProvenance, ManualSourceTimingGrid, PhraseAudioFeatures,
        QualityClass, Section, SectionLabelHint, SourceDescriptor, SourceGraph,
        install_manual_source_timing_grid,
    };
    use sha2::{Digest, Sha256};

    fn synthetic_graph_and_pcm() -> (SourceGraph, Vec<u8>) {
        let samples: Vec<f32> = (0..176_400 * 2)
            .map(|index| ((index % 97) as f32 / 194.0 - 0.25) * 0.25)
            .collect();
        let bytes = pcm16_wave_bytes(44_100, 2, &samples).unwrap();
        let hash = format!("sha256:{:x}", Sha256::digest(&bytes));
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: "synthetic-limiter-preparation".into(),
                path: "synthetic-never-created-limiter-preparation.wav".into(),
                content_hash: hash.clone(),
                duration_seconds: 4.0,
                sample_rate: 44_100,
                channel_count: 2,
                decode_profile: DecodeProfile::Native,
            },
            GraphProvenance {
                sidecar_version: "synthetic".into(),
                provider_set: Vec::new(),
                generated_at: "synthetic".into(),
                source_hash: hash,
                analysis_seed: 19,
                run_notes: None,
            },
        );
        install_manual_source_timing_grid(
            &mut graph,
            ManualSourceTimingGrid {
                bpm: 120.0,
                downbeat_seconds: 0.0,
            },
        )
        .unwrap();
        (graph, bytes)
    }

    fn synthetic_state(
        graph: SourceGraph,
        bytes: &[u8],
        directory: &std::path::Path,
    ) -> JamAppState {
        JamAppState::from_limiter_calibration_graph_bytes(
            graph,
            bytes,
            JamFileSet {
                session_path: directory.join("session.json"),
                source_graph_path: Some(directory.join("source-graph.json")),
            },
        )
        .unwrap()
    }

    #[test]
    fn synthetic_w30_prefix_uses_existing_commits_and_only_the_w30_projection() {
        let directory = tempfile::tempdir().unwrap();
        let (graph, bytes) = synthetic_graph_and_pcm();
        let original_graph = graph.clone();
        let mut state = synthetic_state(graph, &bytes, directory.path());
        // Exercise the core recipe with generated PCM, not registered-source admission.
        let plan = prepare(&mut state, Case::Tonal).unwrap();
        assert_eq!(state.source_graph.as_ref(), Some(&original_graph));
        assert_eq!(plan.transport.position_beats, 8.0);
        assert_eq!(plan.tr909_render, Default::default());
        assert_eq!(plan.mc202_render, Default::default());
        assert_eq!(plan.w30_resample_tap, Default::default());
        assert_eq!(state.session.action_log.actions.len(), 6);
        let records = &state.session.action_log.commit_records;
        assert_eq!(records[4].boundary.beat_index, 5);
        assert_eq!(records[5].boundary.beat_index, 6);
        let evidence = evidence(&state, Case::Tonal).unwrap();
        assert_eq!(evidence["capture_window"]["end_frame"], 88_200);
        let prepare_error = prepare(&mut state, Case::Tonal).unwrap_err();
        let evidence_error = super::evidence(&state, Case::Dense).unwrap_err();
        let snapshot = crate::calibration::preparation_snapshot(&state, Case::Tonal);
        for (stage, error) in [
            ("preparation", prepare_error),
            ("preparation_evidence", evidence_error),
        ] {
            let record = failure_record(error.as_ref(), stage, snapshot.clone());
            assert_eq!(record["stage"], stage);
            assert_eq!(
                record["preparation"]["action_log"]["actions"]
                    .as_array()
                    .unwrap()
                    .len(),
                6
            );
            assert_eq!(
                record["preparation"]["action_log"]["commit_records"]
                    .as_array()
                    .unwrap()
                    .len(),
                6
            );
            assert_eq!(
                record["preparation"]["captures"][0]["source_window"]["end_frame"],
                88_200
            );
            assert_eq!(
                record["preparation"]["source_timing"]["confirmed_bpm"],
                120.0
            );
        }
        // A synthetic silent projection exercises retention of the already validated
        // preparation on the comparison-failure route, without a source run.
        let mut silent_plan = plan.clone();
        silent_plan.w30_preview_render.music_bus_level = 0.0;
        let comparison_error = crate::calibration::compare_plan(&silent_plan, 4_096, Case::Tonal)
            .map(|_| ())
            .expect_err("synthetic silent projection must fail the clean gate");
        let record = failure_record(comparison_error.as_ref(), "comparison", evidence.clone());
        assert_eq!(record["preparation"], evidence);
        assert_eq!(record["stage"], "clean_silence");
        assert_eq!(
            record["diagnostics"]["clean_A"]["post"]["active_samples"],
            0
        );
    }

    fn failure_record(
        error: &(dyn Error + 'static),
        stage: &'static str,
        preparation: Value,
    ) -> Value {
        let mut encoded = Vec::new();
        crate::calibration::write_failure_record(error, stage, Some(preparation), &mut encoded)
            .unwrap();
        assert!(encoded.len() < 64 * 1024);
        let line = String::from_utf8(encoded).unwrap();
        assert!(!line.contains("\"graph\""));
        assert!(!line.contains("\"samples\""));
        serde_json::from_str(
            line.trim()
                .strip_prefix("RIOTBOX_LIMITER_CALIBRATION_FAILURE ")
                .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn synthetic_sparse_prefix_commits_all_eleven_actions_and_renders_stable_multilanes() {
        let directory = tempfile::tempdir().unwrap();
        let (mut graph, bytes) = synthetic_graph_and_pcm();
        // Reuse the measured_sparse_character_* Core regression's synthetic
        // contrasts, not features or metadata copied from a registered source.
        graph.analysis_summary.break_rebuild_potential = QualityClass::High;
        graph.analysis_summary.loop_candidate_count = 1;
        graph.analysis_summary.overall_confidence = 0.9;
        graph.timing.bpm_confidence = 0.95;
        graph.sections.push(Section {
            section_id: "synthetic-sparse-section".into(),
            label_hint: SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 4.0,
            bar_start: 1,
            bar_end: 2,
            energy_class: EnergyClass::High,
            confidence: 0.9,
            tags: vec!["synthetic".into()],
        });
        graph.phrase_audio_features.push(PhraseAudioFeatures {
            phrase_index: 1,
            start_seconds: 0.0,
            end_seconds: 4.0,
            start_bar: 1,
            end_bar: 2,
            low_band_rms: 0.1,
            low_mid_ratio: 0.70,
            low_band_movement: 0.1,
            transient_density: 1.0,
            offbeat_onset_density: 0.20,
            spectral_roughness: 0.05,
            spectral_brightness: 0.39,
            hook_restraint_hint: 0.38,
            confidence: 0.95,
            provenance_refs: vec!["synthetic.sparse-policy-fixture".into()],
        });
        let original_graph = graph.clone();
        let mut state = synthetic_state(graph, &bytes, directory.path());
        let plan = prepare(&mut state, Case::Sparse).unwrap_or_else(|error| {
            panic!(
                "{error}; source plan: {:?}",
                state
                    .session
                    .runtime_state
                    .lane_state
                    .mc202
                    .source_phrase_plan
            )
        });
        assert_eq!(state.source_graph.as_ref(), Some(&original_graph));
        assert_eq!(state.session.action_log.actions.len(), 11);
        let records = &state.session.action_log.commit_records;
        assert_eq!(
            records
                .iter()
                .map(|record| record.boundary.beat_index)
                .collect::<Vec<_>>(),
            [0, 0, 0, 4, 8, 8, 16, 16, 16, 16, 17]
        );
        assert_eq!(plan.transport.position_beats, 17.0);
        assert_eq!(plan.transport.tempo_bpm, 120.0);
        assert_ne!(plan.tr909_render.mode, Tr909RenderMode::Idle);
        assert_eq!(plan.mc202_render.routing, Mc202RenderRouting::MusicBusBass);
        assert_eq!(plan.source_monitor_render.mode, SourceMonitorMode::Riotbox);
        let details = evidence(&state, Case::Sparse).unwrap();
        assert_eq!(details["capture_window"]["end_frame"], 88_200);
        assert_eq!(details["contributors"]["tr909"], "hardest_transient");
        assert_eq!(details["contributors"]["mc202"], "punctuation");
        assert_eq!(details["contributors"]["bass_owner"], "unassigned");
        let steps = [RuntimeMixRenderSequenceStep::new(&plan, 4_096)];
        let primary = render_sequence(&steps, 48_000, 2, 128);
        assert!(
            primary[0]
                .baseline
                .samples
                .iter()
                .any(|sample| sample.abs() > 0.001)
        );
        for callback_frames in [128, 257] {
            let repeated = render_sequence(&steps, 48_000, 2, callback_frames);
            assert_eq!(primary[0].baseline.limiter, repeated[0].baseline.limiter);
            for (first, second) in [
                (&primary[0].pre_samples, &repeated[0].pre_samples),
                (&primary[0].baseline.samples, &repeated[0].baseline.samples),
            ] {
                assert_eq!(first.len(), second.len());
                assert!(
                    first
                        .iter()
                        .zip(second)
                        .all(|(left, right)| left.to_bits() == right.to_bits())
                );
            }
        }
    }
}
