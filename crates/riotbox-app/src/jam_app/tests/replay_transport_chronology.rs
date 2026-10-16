//! Public reference-free save/load/restore, not an audio or source qualification.
use std::path::Path;

use crate::jam_app::JamAppState;
use crate::jam_app::{SourceMapNavigationIntent, SourceMapNavigationResult};
use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, CommitBoundary,
        Quantization, SourceMonitorMode,
    },
    ids::SnapshotId,
    persistence::save_session_json,
    queue::ActionQueue,
    replay::{build_replay_target_plan, upgrade_replay_policy_to_committed_order},
    session::{ReplayPolicy, SessionFile, Snapshot, SnapshotPayload},
    source_graph::{
        DecodeProfile, GraphProvenance, ManualSourceTimingGrid, SourceDescriptor, SourceGraph,
        install_manual_source_timing_grid,
    },
};

fn commit(app: &mut JamAppState, at: u64) {
    let boundary = app
        .runtime
        .transport
        .boundary_state(CommitBoundary::Immediate);
    assert_eq!(app.commit_ready_actions(boundary, at).unwrap().len(), 1);
}

fn graph_only_app() -> JamAppState {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: "generated-metadata-only".into(),
            path: "/nonexistent-generated-metadata-only.wav".into(),
            content_hash: "generated-metadata-only".into(),
            duration_seconds: 16.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "generated-test".into(),
            provider_set: vec![],
            generated_at: "2026-10-16".into(),
            source_hash: "generated-metadata-only".into(),
            analysis_seed: 0,
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
    let mut session = SessionFile::new("generated-public-navigation", "test", "2026-10-16");
    session.runtime_state.transport.position_beats = 12.0;
    JamAppState::from_parts(session, Some(graph), ActionQueue::new())
}

#[test]
fn public_source_map_backward_navigation_roundtrips_without_source_hydration() {
    let mut app = graph_only_app();
    app.queue_source_monitor_mode(SourceMonitorMode::Blend, 10)
        .unwrap();
    commit(&mut app, 11);
    assert!(matches!(
        app.queue_source_map_navigation(SourceMapNavigationIntent::PreviousBar, 20)
            .unwrap(),
        SourceMapNavigationResult::Enqueued {
            target_position_beats: 8,
            ..
        }
    ));
    commit(&mut app, 21);
    app.queue_source_monitor_mode(SourceMonitorMode::Source, 30)
        .unwrap();
    commit(&mut app, 31);
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("session.json");
    assert!(app.source_audio_cache.is_none() && app.capture_audio_cache.is_empty());
    assert!(app.session.source_refs.is_empty() && app.session.source_graph_refs.is_empty());
    save_session_json(&path, &app.session).unwrap();
    let mut loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    loaded.apply_restore_target_suffix(3).unwrap();
    assert_eq!(
        loaded.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Source
    );
    assert_eq!(loaded.session.runtime_state.transport.position_beats, 8.0);
    assert!(loaded.source_audio_cache.is_none() && loaded.capture_audio_cache.is_empty());
}

#[test]
fn legacy_backward_gesture_refuses_before_enqueue_then_explicit_safe_upgrade_admits_it() {
    let mut app = graph_only_app();
    app.session.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    let before = app.session.clone();
    assert!(matches!(
        app.queue_source_map_navigation(SourceMapNavigationIntent::PreviousBar, 10)
            .unwrap(),
        SourceMapNavigationResult::Unavailable { .. }
    ));
    assert_eq!(app.session, before);
    assert!(app.queue.pending_actions().is_empty());
    upgrade_replay_policy_to_committed_order(&mut app.session.action_log).unwrap();
    assert!(matches!(
        app.queue_source_map_navigation(SourceMapNavigationIntent::PreviousBar, 20)
            .unwrap(),
        SourceMapNavigationResult::Enqueued {
            target_position_beats: 8,
            ..
        }
    ));
    assert_eq!(
        app.queue.pending_actions()[0].id.0,
        1,
        "refusal must not consume an ActionId"
    );
    commit(&mut app, 21);
    assert_eq!(app.runtime.transport.position_beats, 8.0);
}

#[test]
fn ordinary_legacy_load_and_save_preserve_policy_record_order_and_file_bytes() {
    let mut app = JamAppState::from_parts(
        SessionFile::new("legacy-roundtrip", "test", "2026-10-16"),
        None,
        ActionQueue::new(),
    );
    app.queue_source_monitor_mode(SourceMonitorMode::Blend, 10)
        .unwrap();
    commit(&mut app, 11);
    app.queue_source_monitor_mode(SourceMonitorMode::Source, 20)
        .unwrap();
    commit(&mut app, 21);
    app.session.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    // Declared historical record permutations remain legacy, not append proof.
    app.session.action_log.commit_records.reverse();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session.json");
    save_session_json(&path, &app.session).unwrap();
    let before = std::fs::read(&path).unwrap();
    let loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert_eq!(loaded.session.action_log, app.session.action_log);
    loaded.save().unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn unsupported_legacy_backward_history_load_fails_without_rewriting_the_file() {
    let mut app = graph_only_app();
    app.queue_source_map_navigation(SourceMapNavigationIntent::PreviousBar, 10)
        .unwrap();
    commit(&mut app, 11);
    app.session.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("legacy.json");
    save_session_json(&path, &app.session).unwrap();
    let before = std::fs::read(&path).unwrap();
    let error = JamAppState::from_json_files(&path, None::<&Path>)
        .err()
        .expect("legacy navigation needs explicit upgrade");
    assert!(
        error
            .to_string()
            .contains("LegacyNavigationRequiresChronologyUpgrade")
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn backward_navigation_save_load_restore_preserves_committed_monitor_state() {
    let mut session = SessionFile::new("generated-backward-restore", "test", "2026-10-16");
    session.runtime_state.transport.position_beats = 12.0;
    let mut app = JamAppState::from_parts(session, None, ActionQueue::new());
    app.queue_source_monitor_mode(SourceMonitorMode::Blend, 10)
        .unwrap();
    commit(&mut app, 11);
    let mut seek = ActionDraft::new(
        ActorType::User,
        ActionCommand::TransportSeek,
        Quantization::Immediate,
        ActionTarget::default(),
    );
    seek.params = ActionParams::Transport {
        position_beats: Some(0),
    };
    app.queue.enqueue(seek, 20).unwrap();
    commit(&mut app, 21);
    app.queue_source_monitor_mode(SourceMonitorMode::Source, 30)
        .unwrap();
    commit(&mut app, 31);
    assert!(app.session.source_refs.is_empty());
    assert!(app.session.source_graph_refs.is_empty());
    assert!(app.session.captures.is_empty());

    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("session.json");
    save_session_json(&path, &app.session).unwrap();
    let saved = std::fs::read(&path).unwrap();
    let mut loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert_eq!(loaded.session.action_log, app.session.action_log);
    assert_eq!(
        loaded.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Source
    );
    loaded.apply_restore_target_suffix(3).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    assert_eq!(
        loaded.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Source
    );
    assert_eq!(loaded.session.runtime_state.transport.position_beats, 0.0);
}

#[test]
fn real_undo_clock_rollback_and_post_undo_snapshot_survive_save_load_restore() {
    let mut app = JamAppState::from_parts(
        SessionFile::new("fresh-undo-clock", "test", "2026-10-16"),
        None,
        ActionQueue::new(),
    );
    app.queue_source_monitor_mode(SourceMonitorMode::Blend, 900)
        .unwrap();
    commit(&mut app, 1000);
    assert!(app.undo_last_action(500).unwrap().is_some());
    let cursor = app.session.action_log.actions.len();
    let snapshot_id = SnapshotId::from("generated-post-undo");
    app.session.snapshots.push(Snapshot {
        snapshot_id: snapshot_id.clone(),
        created_at: "2026-10-16".into(),
        label: "generated post-undo anchor".into(),
        action_cursor: cursor,
        payload: Some(SnapshotPayload::from_runtime_state(
            &snapshot_id,
            cursor,
            &app.session.runtime_state,
        )),
    });
    app.queue_source_monitor_mode(SourceMonitorMode::Riotbox, 600)
        .unwrap();
    commit(&mut app, 700);
    let target = app.session.action_log.actions.len();
    let plan =
        build_replay_target_plan(&app.session.action_log, &app.session.snapshots, target).unwrap();
    assert_eq!(plan.anchor.unwrap().snapshot_id, snapshot_id);
    assert_eq!(plan.suffix.len(), 1);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session.json");
    save_session_json(&path, &app.session).unwrap();
    let mut loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    loaded
        .apply_restore_target_from_snapshot_payload(target)
        .unwrap();
    assert_eq!(
        loaded.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Riotbox
    );
    assert_eq!(loaded.session.action_log, app.session.action_log);
}

#[test]
fn real_domain_barrier_preserves_independent_monitor_undo_and_roundtrip() {
    let graph = super::fixtures::session_source::sample_graph();
    let mut app = JamAppState::from_parts(
        SessionFile::new("fresh-domain-undo", "test", "2026-10-16"),
        Some(graph),
        ActionQueue::new(),
    );
    let initial_monitor_mode = app.session.runtime_state.source_monitor.mode;
    app.queue_source_monitor_mode(SourceMonitorMode::Blend, 10)
        .unwrap();
    commit(&mut app, 11);
    let monitor_id = app.session.action_log.actions[0].id;
    app.queue_mc202_generate_follower(20).unwrap();
    super::fixtures::mc202_recipe::commit_mc202_recipe_step(&mut app, 1, 21);
    assert!(
        app.session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_some()
    );
    app.queue_source_timing_grid_revert(30).unwrap();
    commit(&mut app, 31);
    let marker = app.undo_last_action(40).unwrap().unwrap();
    assert!(
        matches!(marker.params, ActionParams::Undo { target_action_id } if target_action_id == monitor_id)
    );
    assert_eq!(
        app.session.runtime_state.source_monitor.mode,
        initial_monitor_mode
    );
    assert!(
        app.session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session.json");
    save_session_json(&path, &app.session).unwrap();
    let mut loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    loaded
        .apply_restore_target_suffix(app.session.action_log.actions.len())
        .unwrap();
    assert_eq!(
        loaded.session.runtime_state.source_monitor,
        app.session.runtime_state.source_monitor
    );
    assert!(
        loaded
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
}

#[test]
fn restored_backward_history_preserves_actual_generated_monitor_mix_not_only_mode() {
    use riotbox_audio::{
        runtime::render_source_monitor_mix_offline, source_audio::SourceAudioCache,
    };

    let mut app = JamAppState::from_parts(
        SessionFile::new("fresh-monitor-output", "test", "2026-10-16"),
        None,
        ActionQueue::new(),
    );
    for (command, position) in [
        (ActionCommand::TransportPlay, None),
        (ActionCommand::TransportSeek, Some(12)),
    ] {
        let mut draft = ActionDraft::new(
            ActorType::User,
            command,
            Quantization::Immediate,
            ActionTarget::default(),
        );
        if position.is_some() {
            draft.params = ActionParams::Transport {
                position_beats: position,
            };
        }
        app.queue.enqueue(draft, 10).unwrap();
        commit(&mut app, 11);
    }
    app.queue_source_monitor_mode(SourceMonitorMode::Blend, 20)
        .unwrap();
    commit(&mut app, 21);
    let mut seek = ActionDraft::new(
        ActorType::User,
        ActionCommand::TransportSeek,
        Quantization::Immediate,
        ActionTarget::default(),
    );
    seek.params = ActionParams::Transport {
        position_beats: Some(0),
    };
    app.queue.enqueue(seek, 30).unwrap();
    commit(&mut app, 31);
    app.queue_source_monitor_mode(SourceMonitorMode::Source, 40)
        .unwrap();
    commit(&mut app, 41);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session.json");
    save_session_json(&path, &app.session).unwrap();
    let mut loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    loaded
        .apply_restore_target_suffix(app.session.action_log.actions.len())
        .unwrap();
    assert!(app.source_audio_cache.is_none() && loaded.source_audio_cache.is_none());
    // Test-only generated PCM, attached after ordinary metadata restore. No file
    // is opened, no audio device is constructed and nothing is played to a person.
    let cache = SourceAudioCache::from_interleaved_samples(
        "never-opened-generated-pcm",
        1000,
        1,
        vec![0.2; 4096],
    )
    .unwrap();
    app.source_audio_cache = Some(cache.clone());
    loaded.source_audio_cache = Some(cache);
    let input = vec![0.3; 256];
    let live_render = app.source_monitor_render_state();
    let restored_render = loaded.source_monitor_render_state();
    let live = render_source_monitor_mix_offline(&input, 1000, 1, &live_render);
    let restored = render_source_monitor_mix_offline(&input, 1000, 1, &restored_render);
    assert!(live.iter().any(|sample| sample.abs() > 0.01));
    assert_eq!(live, restored);
    let mut old_wrong_blend = restored_render;
    old_wrong_blend.mode = SourceMonitorMode::Blend;
    assert_ne!(
        restored,
        render_source_monitor_mix_offline(&input, 1000, 1, &old_wrong_blend)
    );
}
