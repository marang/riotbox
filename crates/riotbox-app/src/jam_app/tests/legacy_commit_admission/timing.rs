//! Generated transport observations only; no audio runtime or references.
use std::{fs, path::Path};

use riotbox_audio::runtime::AudioRuntimeTimingSnapshot;
use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, CommitBoundary,
        Quantization,
    },
    queue::CommittedActionRef,
    session::ReplayPolicy,
};

use super::{assert_refusal_unchanged, enqueue, loaded_app};
use crate::jam_app::{JamAppError, JamAppState};

fn timing(position_beats: f64, running: bool) -> AudioRuntimeTimingSnapshot {
    AudioRuntimeTimingSnapshot {
        is_transport_running: running,
        tempo_bpm: 128.0,
        position_beats,
    }
}

fn enqueue_seek(app: &mut JamAppState, quantization: Quantization, position: u64) {
    let mut draft = ActionDraft::new(
        ActorType::User,
        ActionCommand::TransportSeek,
        quantization,
        ActionTarget::default(),
    );
    draft.params = ActionParams::Transport {
        position_beats: Some(position),
    };
    app.queue.enqueue(draft, 100).unwrap();
}

#[test]
fn crossed_legacy_unsafe_commits_refuse_before_local_or_observed_timing_mutates() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
    ] {
        for seek in [false, true] {
            for (quantization, position) in [
                (Quantization::NextBeat, 13.0),
                (Quantization::NextBar, 16.0),
            ] {
                for observed in [false, true] {
                    let path = owner
                        .path()
                        .join(format!("{policy:?}-{seek}-{position}-{observed}.json"));
                    let mut app = loaded_app(&path, policy);
                    app.set_transport_playing(true);
                    if seek {
                        enqueue_seek(&mut app, quantization, 1);
                    } else {
                        enqueue(&mut app, ActionCommand::TransportStop, quantization);
                    }
                    assert_eq!(
                        app.runtime.transport_driver.pending_audio_is_playing,
                        Some(true)
                    );
                    assert_refusal_unchanged(&mut app, &path, |app| {
                        if observed {
                            app.apply_audio_timing_snapshot(Some(timing(position, true)), 110)
                        } else {
                            app.advance_transport_by(position - 12.0, 110)
                        }
                    });
                    assert_eq!(
                        app.runtime.transport_driver.pending_audio_is_playing,
                        Some(true)
                    );
                    app.save().unwrap();
                    assert_eq!(
                        JamAppState::from_json_files(&path, None::<&Path>)
                            .unwrap()
                            .session,
                        app.session
                    );
                }
            }
        }
    }
}

#[test]
fn valid_observations_empty_crossings_and_quantized_pause_remain_available_for_all_policies() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
        ReplayPolicy::DeterministicCommittedOrderV2,
    ] {
        for observed in [false, true] {
            let path = owner
                .path()
                .join(format!("valid-{policy:?}-{observed}.json"));
            let mut app = loaded_app(&path, policy);
            app.set_transport_playing(true);
            // Empty crossing still publishes ordinary progress and acknowledges
            // matching observed Play without creating an action or upgrade.
            let empty = if observed {
                app.apply_audio_timing_snapshot(Some(timing(13.0, true)), 110)
            } else {
                app.advance_transport_by(1.0, 110)
            }
            .unwrap();
            assert!(empty.is_empty());
            assert_eq!(app.runtime.transport.position_beats, 13.0);
            assert!(app.session.action_log.actions.is_empty());
            if observed {
                assert_eq!(app.runtime.transport_driver.pending_audio_is_playing, None);
                assert_eq!(
                    app.runtime.transport_driver.last_audio_position_beats,
                    Some(13)
                );
            }
            enqueue(
                &mut app,
                ActionCommand::TransportPause,
                Quantization::NextBeat,
            );
            let committed = if observed {
                app.apply_audio_timing_snapshot(Some(timing(14.0, true)), 120)
            } else {
                app.advance_transport_by(1.0, 120)
            }
            .unwrap();
            assert_eq!(committed.len(), 1);
            assert_eq!(committed[0].boundary.beat_index, 14);
            assert_eq!(app.runtime.transport.position_beats, 14.0);
            assert!(!app.runtime.transport.is_playing);
            assert_eq!(app.session.action_log.replay_policy, policy);
            app.save().unwrap();
            assert_eq!(
                JamAppState::from_json_files(&path, None::<&Path>)
                    .unwrap()
                    .session,
                app.session
            );
        }
    }
}

fn assert_noop(
    app: &mut JamAppState,
    operation: impl FnOnce(&mut JamAppState) -> Result<Vec<CommittedActionRef>, JamAppError>,
) {
    let session = app.session.clone();
    let runtime = app.runtime.clone();
    let view = app.jam_view.clone();
    let runtime_view = app.runtime_view.clone();
    let pending: Vec<_> = app.queue.pending_actions().into_iter().cloned().collect();
    let history = app.queue.history().to_vec();
    let next_id = app.queue.clone().allocate_action_id();
    assert!(operation(app).unwrap().is_empty());
    assert_eq!(app.session, session);
    assert_eq!(app.runtime, runtime);
    assert_eq!(app.jam_view, view);
    assert_eq!(app.runtime_view, runtime_view);
    assert_eq!(
        app.queue
            .pending_actions()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>(),
        pending
    );
    assert_eq!(app.queue.history(), history);
    assert_eq!(app.queue.clone().allocate_action_id(), next_id);
}

#[test]
fn stopped_nonpositive_unavailable_and_stale_timing_still_skip_pending_unsafe_actions() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
        ReplayPolicy::DeterministicCommittedOrderV2,
    ] {
        let path = owner.path().join(format!("gates-{policy:?}.json"));
        let mut app = loaded_app(&path, policy);
        enqueue(
            &mut app,
            ActionCommand::TransportStop,
            Quantization::NextBeat,
        );
        let saved = fs::read(&path).unwrap();
        assert_noop(&mut app, |app| app.advance_transport_by(1.0, 110));
        app.set_transport_playing(true);
        assert_noop(&mut app, |app| app.advance_transport_by(0.0, 110));
        assert_noop(&mut app, |app| app.advance_transport_by(-1.0, 110));
        assert_noop(&mut app, |app| app.apply_audio_timing_snapshot(None, 110));
        assert_noop(&mut app, |app| {
            app.apply_audio_timing_snapshot(Some(timing(13.0, false)), 110)
        });
        // Matching observation at the same beat clears the pending Play ack.
        assert!(
            app.apply_audio_timing_snapshot(Some(timing(12.0, true)), 110)
                .unwrap()
                .is_empty()
        );
        assert_eq!(app.runtime.transport_driver.pending_audio_is_playing, None);
        assert_noop(&mut app, |app| {
            app.apply_audio_timing_snapshot(Some(timing(13.0, false)), 110)
        });
        app.set_transport_playing(false);
        assert!(
            app.apply_audio_timing_snapshot(Some(timing(14.0, false)), 120)
                .unwrap()
                .is_empty()
        );
        assert_eq!(app.runtime.transport.position_beats, 14.0);
        assert_eq!(app.runtime.transport_driver.pending_audio_is_playing, None);
        assert_eq!(app.runtime.transport_driver.last_audio_position_beats, None);
        assert_eq!(app.queue.pending_actions().len(), 1);
        assert!(app.session.action_log.actions.is_empty());
        assert_eq!(fs::read(&path).unwrap(), saved);
    }
}

#[test]
fn v2_crossed_stop_and_backward_seek_still_commit_from_local_and_observed_progress() {
    let owner = tempfile::tempdir().unwrap();
    for seek in [false, true] {
        for observed in [false, true] {
            let path = owner
                .path()
                .join(format!("v2-navigation-{seek}-{observed}.json"));
            let mut app = loaded_app(&path, ReplayPolicy::DeterministicCommittedOrderV2);
            app.set_transport_playing(true);
            if seek {
                enqueue_seek(&mut app, Quantization::NextBeat, 1);
            } else {
                enqueue(
                    &mut app,
                    ActionCommand::TransportStop,
                    Quantization::NextBeat,
                );
            }
            let committed = if observed {
                app.apply_audio_timing_snapshot(Some(timing(13.0, true)), 110)
            } else {
                app.advance_transport_by(1.0, 110)
            }
            .unwrap();
            assert_eq!(committed.len(), 1);
            assert_eq!(committed[0].boundary.beat_index, 13);
            assert_eq!(
                app.runtime.transport.position_beats,
                if seek { 1.0 } else { 0.0 }
            );
            assert!(app.queue.pending_actions().is_empty());
            app.save().unwrap();
            assert_eq!(
                JamAppState::from_json_files(&path, None::<&Path>)
                    .unwrap()
                    .session,
                app.session
            );
        }
    }
}

#[test]
fn committed_forward_seek_blocks_stale_observations_until_position_delivery_for_all_policies() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
        ReplayPolicy::DeterministicCommittedOrderV2,
    ] {
        let path = owner.path().join(format!("position-gate-{policy:?}.json"));
        let mut app = loaded_app(&path, policy);
        app.set_transport_playing(true);
        enqueue_seek(&mut app, Quantization::Immediate, 16);
        let boundary = app
            .runtime
            .transport
            .boundary_state(CommitBoundary::Immediate);
        assert_eq!(app.commit_ready_actions(boundary, 110).unwrap().len(), 1);
        assert!(
            app.runtime
                .transport_driver
                .pending_audio_position
                .is_some()
        );
        enqueue(
            &mut app,
            ActionCommand::TransportStop,
            Quantization::NextBeat,
        );
        let saved = fs::read(&path).unwrap();
        assert_noop(&mut app, |app| {
            app.apply_audio_timing_snapshot(Some(timing(17.0, true)), 120)
        });
        assert_eq!(fs::read(&path).unwrap(), saved);
    }
}
