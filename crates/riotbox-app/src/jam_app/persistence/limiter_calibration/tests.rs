use std::{fs, path::Path};

use riotbox_audio::{
    runtime::render_w30_preview_offline,
    source_audio::{SourceAudioCache, pcm16_wave_bytes},
};
use riotbox_core::{
    action::{ActionCommand, ActionParams, CommitBoundary},
    session::CaptureAudioIdentityProvenance,
    source_graph::{
        DecodeProfile, GraphProvenance, ManualSourceTimingGrid, SourceDescriptor, SourceGraph,
        TimingHypothesisKind, install_manual_source_timing_grid,
    },
    transport::CommitBoundaryState,
};
use sha2::{Digest, Sha256};

use crate::jam_app::{CaptureAudioStatus, JamAppState, JamFileSet, QueueControlResult};

fn fixture(root: &Path, frame_count: usize) -> (SourceGraph, Vec<u8>, JamFileSet) {
    let samples: Vec<f32> = (0..frame_count * 2)
        .map(|index| (index % 97) as f32 / 194.0 - 0.25)
        .collect();
    let bytes = pcm16_wave_bytes(44_100, 2, &samples).unwrap();
    let hash = format!("sha256:{:x}", Sha256::digest(&bytes));
    let graph = SourceGraph::new(
        SourceDescriptor {
            source_id: "synthetic-calibration-source".into(),
            path: root
                .join("never-created/source.wav")
                .to_string_lossy()
                .into(),
            content_hash: hash.clone(),
            duration_seconds: frame_count as f32 / 44_100.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "synthetic-test".into(),
            provider_set: Vec::new(),
            generated_at: "synthetic".into(),
            source_hash: hash,
            analysis_seed: 1,
            run_notes: None,
        },
    );
    let files = JamFileSet {
        session_path: root.join("output/session.json"),
        source_graph_path: Some(root.join("output/graph.json")),
    };
    (graph, bytes, files)
}

#[test]
fn admitted_bytes_construct_without_original_source_or_metadata_file_io() {
    let dir = tempfile::tempdir().unwrap();
    let (graph, bytes, files) = fixture(dir.path(), 176_400);
    let expected_cache = SourceAudioCache::from_pcm_wav_bytes(&graph.source.path, &bytes).unwrap();
    let state =
        JamAppState::from_limiter_calibration_graph_bytes(graph.clone(), &bytes, files.clone())
            .unwrap();

    assert_eq!(state.source_graph.as_ref(), Some(&graph));
    assert_eq!(state.source_audio_cache.as_ref(), Some(&expected_cache));
    assert_eq!(state.files.as_ref(), Some(&files));
    assert!(state.session.captures.is_empty());
    assert!(state.session.action_log.actions.is_empty());
    assert!(
        state
            .session
            .runtime_state
            .source_timing
            .confirmed_grid
            .is_none()
    );
    assert_eq!(
        state.session.source_refs[0].content_hash,
        graph.source.content_hash
    );
    assert_eq!(state.session.source_refs[0].path_hint, graph.source.path);
    assert_eq!(state.session.runtime_state.mixer_state.music_level, 0.64);
    assert!(!Path::new(&graph.source.path).exists());
    assert!(!files.session_path.exists());
    assert!(!files.source_graph_path.unwrap().exists());
}

#[test]
fn rounded_sidecar_duration_is_accepted_without_rewriting_graph() {
    let dir = tempfile::tempdir().unwrap();
    let (mut graph, bytes, files) = fixture(dir.path(), 162_830);
    assert_ne!(graph.source.duration_seconds, 3.692);
    graph.source.duration_seconds = 3.692;
    let state =
        JamAppState::from_limiter_calibration_graph_bytes(graph.clone(), &bytes, files).unwrap();
    assert_eq!(state.source_graph.as_ref(), Some(&graph));
    assert_eq!(state.session.source_refs[0].duration_seconds, 3.692);
    assert_eq!(state.source_audio_cache.unwrap().frame_count(), 162_830);
}

#[test]
fn rejects_hash_provenance_format_profile_and_duration_mismatches() {
    let dir = tempfile::tempdir().unwrap();
    let (graph, bytes, files) = fixture(dir.path(), 176_400);
    let mutations: [fn(&mut SourceGraph); 7] = [
        |graph| graph.source.content_hash = format!("sha256:{}", "0".repeat(64)),
        |graph| graph.provenance.source_hash = format!("sha256:{}", "0".repeat(64)),
        |graph| graph.source.sample_rate = 48_000,
        |graph| graph.source.channel_count = 1,
        |graph| graph.source.decode_profile = DecodeProfile::NormalizedStereo,
        |graph| graph.source.duration_seconds += 0.001,
        |graph| graph.source.duration_seconds = f32::NAN,
    ];
    for mutate in mutations {
        let mut changed = graph.clone();
        mutate(&mut changed);
        assert!(
            JamAppState::from_limiter_calibration_graph_bytes(changed, &bytes, files.clone())
                .is_err()
        );
    }
    let mut wrong_pcm_bytes = bytes.clone();
    let last = wrong_pcm_bytes.last_mut().unwrap();
    *last ^= 1;
    assert!(
        JamAppState::from_limiter_calibration_graph_bytes(graph, &wrong_pcm_bytes, files).is_err()
    );
}

#[test]
fn rejects_empty_or_invalid_wav_bytes_even_when_hashes_match() {
    let dir = tempfile::tempdir().unwrap();
    for bytes in [
        pcm16_wave_bytes(44_100, 2, &[]).unwrap(),
        b"not WAV".to_vec(),
    ] {
        let (mut graph, _, files) = fixture(dir.path(), 176_400);
        let hash = format!("sha256:{:x}", Sha256::digest(&bytes));
        graph.source.content_hash = hash.clone();
        graph.provenance.source_hash = hash;
        assert!(JamAppState::from_limiter_calibration_graph_bytes(graph, &bytes, files).is_err());
    }
}

#[test]
fn rejects_source_conflicts_shared_outputs_and_existing_destinations() {
    let dir = tempfile::tempdir().unwrap();
    let (graph, bytes, files) = fixture(dir.path(), 176_400);
    for conflict in [
        JamFileSet {
            session_path: graph.source.path.clone().into(),
            ..files.clone()
        },
        JamFileSet {
            source_graph_path: Some(graph.source.path.clone().into()),
            ..files.clone()
        },
        JamFileSet {
            source_graph_path: Some(files.session_path.clone()),
            ..files.clone()
        },
        JamFileSet {
            source_graph_path: Some(files.session_path.join("graph.json")),
            ..files.clone()
        },
        JamFileSet {
            session_path: dir.path().join("never-created/nested/../source.wav"),
            ..files.clone()
        },
    ] {
        assert!(
            JamAppState::from_limiter_calibration_graph_bytes(graph.clone(), &bytes, conflict)
                .is_err()
        );
    }
    fs::create_dir_all(files.session_path.parent().unwrap()).unwrap();
    fs::write(&files.session_path, b"preserved output").unwrap();
    assert!(
        JamAppState::from_limiter_calibration_graph_bytes(graph, &bytes, files.clone()).is_err()
    );
    assert_eq!(fs::read(&files.session_path).unwrap(), b"preserved output");
}

fn commit(state: &mut JamAppState, kind: CommitBoundary, beat_index: u64) {
    let actions = state.commit_ready_actions(
        CommitBoundaryState {
            kind,
            beat_index,
            bar_index: beat_index / 4 + 1,
            phrase_index: 0,
            scene_id: None,
        },
        100 + beat_index,
    );
    assert_eq!(actions.len(), 1);
    assert!(
        state
            .session
            .action_log
            .actions
            .iter()
            .find(|action| action.id == actions[0].action_id)
            .unwrap()
            .result
            .as_ref()
            .is_some_and(|result| result.accepted)
    );
}

#[test]
fn ordinary_commits_write_and_hydrate_capture_without_original_source_io() {
    let dir = tempfile::tempdir().unwrap();
    let (mut graph, bytes, files) = fixture(dir.path(), 176_400);
    install_manual_source_timing_grid(
        &mut graph,
        ManualSourceTimingGrid {
            bpm: 120.0,
            downbeat_seconds: 0.0,
        },
    )
    .unwrap();
    let original_path = graph.source.path.clone();
    let mut state =
        JamAppState::from_limiter_calibration_graph_bytes(graph, &bytes, files.clone()).unwrap();
    assert_eq!(
        state.queue_source_timing_grid_confirmation(10),
        QueueControlResult::Enqueued
    );
    commit(&mut state, CommitBoundary::Immediate, 0);
    state.queue_capture_bar(20);
    commit(&mut state, CommitBoundary::Phrase, 0);

    let capture = &state.session.captures[0];
    let capture_id = capture.capture_id.clone();
    let artifact_path = files
        .session_path
        .parent()
        .unwrap()
        .join(&capture.storage_path);
    let artifact_bytes = fs::read(&artifact_path).unwrap();
    assert_eq!(
        capture.audio_identity.as_ref().unwrap().provenance,
        CaptureAudioIdentityProvenance::CreatedFromEncodedBytesV1
    );
    assert_eq!(
        capture.audio_identity.as_ref().unwrap().sha256,
        format!("sha256:{:x}", Sha256::digest(&artifact_bytes))
    );
    assert_eq!(
        state.runtime.capture_audio_status.get(&capture_id),
        Some(&CaptureAudioStatus::Loaded)
    );
    assert_eq!(
        state.capture_audio_cache[&capture_id],
        SourceAudioCache::from_pcm_wav_bytes(&artifact_path, &artifact_bytes).unwrap()
    );
    state.source_audio_cache = None;
    assert!(state.queue_promote_last_capture(30));
    commit(&mut state, CommitBoundary::Bar, 0);
    assert_eq!(
        state.queue_w30_trigger_pad(40),
        Some(QueueControlResult::Enqueued)
    );
    commit(&mut state, CommitBoundary::Beat, 0);
    state.set_transport_playing(true);
    let rendered = render_w30_preview_offline(&state.runtime.w30_preview, 44_100, 2, 4_410);
    assert!(rendered.iter().any(|sample| sample.abs() > 0.001));
    state.save().unwrap();
    assert!(files.session_path.is_file());
    assert!(files.source_graph_path.unwrap().is_file());
    assert!(!Path::new(&original_path).exists());
}

fn detected_timing_graph(mut graph: SourceGraph) -> SourceGraph {
    install_manual_source_timing_grid(
        &mut graph,
        ManualSourceTimingGrid {
            bpm: 130.284_94,
            downbeat_seconds: 0.0,
        },
    )
    .unwrap();
    let hypothesis = &mut graph.timing.hypotheses[0];
    hypothesis.kind = TimingHypothesisKind::Primary;
    hypothesis.hypothesis_id = "synthetic-probe-primary".into();
    hypothesis.provenance = vec!["generated-test-grid".into()];
    graph.timing.primary_hypothesis_id = Some(hypothesis.hypothesis_id.clone());
    graph
}

#[test]
fn explicit_bpm_uses_existing_validation_and_commits_without_changing_detected_grid() {
    let dir = tempfile::tempdir().unwrap();
    let (graph, bytes, files) = fixture(dir.path(), 176_400);
    let graph = detected_timing_graph(graph);
    let mut state =
        JamAppState::from_limiter_calibration_graph_bytes(graph.clone(), &bytes, files).unwrap();
    state
        .confirm_limiter_calibration_source_bpm(130.0, 10)
        .unwrap();

    assert_eq!(state.source_graph.as_ref(), Some(&graph));
    assert_eq!(
        state.session.runtime_state.source_timing.confirmed_bpm,
        Some(130.0)
    );
    assert_eq!(state.runtime.w30_preview.tempo_bpm, 130.0);
    let actions = &state.session.action_log.actions;
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].command, ActionCommand::SourceTimingConfirmGrid);
    assert_eq!(actions[0].requested_at, 10);
    assert_eq!(actions[0].committed_at, Some(10));
    assert_eq!(
        actions[0].params,
        ActionParams::SourceTimingGrid {
            source_id: Some(graph.source.source_id.clone()),
            hypothesis_id: graph.timing.primary_hypothesis_id.clone(),
            confirmed_bpm: Some(130.0),
        }
    );
    let commits = &state.session.action_log.commit_records;
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].boundary.kind, CommitBoundary::Immediate);
    assert!(state.queue.pending_actions().is_empty());
    assert!(!Path::new(&graph.source.path).exists());
    assert!(
        state
            .confirm_limiter_calibration_source_bpm(130.0, 11)
            .is_err()
    );
    assert_eq!(state.session.action_log.actions.len(), 1);
}

#[test]
fn explicit_bpm_mismatch_or_pending_actions_fail_without_committing() {
    let dir = tempfile::tempdir().unwrap();
    let (graph, bytes, files) = fixture(dir.path(), 176_400);
    let graph = detected_timing_graph(graph);
    let mut state =
        JamAppState::from_limiter_calibration_graph_bytes(graph.clone(), &bytes, files).unwrap();
    for rejected in [120.0, 0.0, f32::NAN, f32::INFINITY] {
        assert!(
            state
                .confirm_limiter_calibration_source_bpm(rejected, 10)
                .is_err()
        );
        assert!(state.queue.pending_actions().is_empty());
        assert!(state.session.action_log.actions.is_empty());
        assert!(
            state
                .session
                .runtime_state
                .source_timing
                .confirmed_bpm
                .is_none()
        );
        assert_eq!(state.source_graph.as_ref(), Some(&graph));
    }
    state.queue_capture_bar(9);
    assert!(
        state
            .confirm_limiter_calibration_source_bpm(130.0, 10)
            .is_err()
    );
    assert_eq!(state.queue.pending_actions().len(), 1);
    assert_eq!(
        state.queue.pending_actions()[0].command,
        ActionCommand::CaptureBarGroup
    );
    assert!(state.session.action_log.actions.is_empty());
}
