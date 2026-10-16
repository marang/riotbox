use super::*;

#[test]
fn exhausted_writers_refuse_before_autoqueue_and_preserve_missing_or_foreign_paths() {
    let owner = tempdir().unwrap();
    for foreign in [false, true] {
        let root = owner
            .path()
            .join(if foreign { "foreign" } else { "missing" });
        fs::create_dir(&root).unwrap();
        let path = root.join("session.json");
        write_source_free_history(&path, u32::MAX);
        let mut state = load_source_free(&path);
        let proof = root.join("proof.json");
        let destination = root.join("export");
        let daw = root.join("export.dawproject");
        if foreign {
            fs::write(&proof, b"foreign proof").unwrap();
            fs::write(&destination, b"foreign destination").unwrap();
            fs::write(&daw, b"foreign archive").unwrap();
        }
        let saved = fs::read(&path).unwrap();
        assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
            state.commit_product_mix_export_from_proof(&proof, &destination, 20)
        });
        assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
            state.commit_stem_package_export_local_ci_package(&destination, 20, Vec::new())
        });
        assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
            state.commit_stem_package_export_from_product_handoff(&proof, &destination, 20)
        });
        assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
            state.commit_stem_package_export_w30_hook_loop(&destination, 20)
        });
        assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
            state.commit_w30_hook_dawproject_export(None, &daw, 20)
        });
        assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
            state.commit_live_master_dawproject_export(&daw, 20)
        });
        assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
            state.commit_and_save_live_master_dawproject_export(&daw, 20)
        });
        assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
            state.commit_daw_session_writer_export(None, &destination, 20)
        });
        assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
            state.commit_daw_session_host_import_proof_export(&proof, 20)
        });
        assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
            state.commit_daw_session_audible_output_proof_export(&proof, 20)
        });
        assert_eq!(fs::read(&path).unwrap(), saved);
        if foreign {
            assert_eq!(fs::read(&proof).unwrap(), b"foreign proof");
            assert_eq!(fs::read(&destination).unwrap(), b"foreign destination");
            assert_eq!(fs::read(&daw).unwrap(), b"foreign archive");
        } else {
            assert!(!proof.exists());
            assert!(!destination.exists());
            assert!(!daw.exists());
        }
    }
}

#[test]
fn queue_only_export_rejection_does_not_require_commit_sequence_capacity() {
    use riotbox_core::action::ActionStatus;

    let owner = tempdir().unwrap();
    let path = owner.path().join("queue-only-rejection.json");
    write_source_free_history(&path, u32::MAX);
    let mut state = load_source_free(&path);
    let session = state.session.clone();
    let saved_bytes = fs::read(&path).unwrap();
    let proof = owner.path().join("never-opened-proof.json");
    let destination = owner.path().join("never-created-export");

    let id = state
        .reject_product_mix_export_request(20, "explicit rejection")
        .unwrap();
    assert_eq!(id, ActionId(2));
    let rejected = state.queue.history_action(id).unwrap();
    assert_eq!(rejected.command, ActionCommand::ExportProductMix);
    assert_eq!(rejected.status, ActionStatus::Rejected);
    assert!(!rejected.result.as_ref().unwrap().accepted);
    assert_eq!(
        rejected.result.as_ref().unwrap().summary,
        "explicit rejection"
    );
    assert!(state.queue.pending_actions().is_empty());

    // An unavailable active source is a producer precondition, not a commit.
    // Its existing rejected queue trace remains legal at the full boundary.
    let error = state
        .commit_product_mix_export_from_active_source_proof(&proof, &destination, 30)
        .unwrap_err();
    assert!(matches!(error, JamAppError::InvalidSession(message)
        if message == "product mix export requires an active Source Graph identity"));
    assert_eq!(
        state.queue.history_action(ActionId(3)).unwrap().status,
        ActionStatus::Rejected
    );
    assert!(state.queue.pending_actions().is_empty());
    assert_eq!(state.session, session);
    assert_eq!(state.session.action_log.commit_records.len(), 1);
    assert_eq!(fs::read(&path).unwrap(), saved_bytes);
    assert!(!proof.exists());
    assert!(!destination.exists());
}

#[test]
fn recording_publication_refuses_before_creating_any_recording_or_proof_file() {
    use crate::jam_app::{
        LiveMasterRecordingQueueResult,
        tests::p016_live_master_recording::{
            live_master_test_health, live_master_test_outcome, live_master_test_output,
        },
    };
    use riotbox_core::ids::SceneId;

    let owner = tempdir().unwrap();
    let path = owner.path().join("recording-session.json");
    let scene = SceneId::from("generated-recording-scene");
    let mut boundary = imported_boundary();
    boundary.scene_id = Some(scene.clone());
    write_history_at(&path, u32::MAX, boundary.clone());
    let mut state = load_source_free(&path);
    state.session.runtime_state.source_timing.confirmed_bpm = Some(120.0);
    state.session.runtime_state.scene_state.scenes = vec![scene.clone()];
    state.session.runtime_state.scene_state.active_scene = Some(scene.clone());
    state.session.runtime_state.transport.current_scene = Some(scene.clone());
    state.runtime.transport.current_scene = Some(scene);
    state.refresh_view();

    let destination = owner.path().join("never-published.wav");
    let output = live_master_test_output();
    let plan = match state
        .queue_live_master_recording(10, &output, &destination)
        .unwrap()
    {
        LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        other => panic!("generated recording plan must be eligible: {other:?}"),
    };
    let outcome = live_master_test_outcome(&plan);
    let health = live_master_test_health(&output);
    assert_sequence_refusal(&mut state, &boundary, 1, |state| {
        state.commit_live_master_recording(&plan, &outcome, &health, 20)
    });
    assert_sequence_refusal(&mut state, &boundary, 1, |state| {
        state.commit_and_save_live_master_recording(&plan, &outcome, &health, 30)
    });
    assert!(!destination.exists());
    assert!(!plan.proof_path.exists());
    assert!(state.session.export_receipts.is_empty());
}

#[cfg(feature = "limiter-calibration")]
#[test]
fn explicit_bpm_sequence_refusal_precedes_confirmation_autoqueue() {
    use riotbox_core::source_graph::{ManualSourceTimingGrid, install_manual_source_timing_grid};
    let owner = tempdir().unwrap();
    let path = owner.path().join("bpm-session.json");
    write_source_free_history(&path, u32::MAX);
    let mut state = load_source_free(&path);
    let mut graph = crate::jam_app::tests::fixtures::session_source::sample_graph();
    install_manual_source_timing_grid(
        &mut graph,
        ManualSourceTimingGrid {
            bpm: 126.0,
            downbeat_seconds: 0.0,
        },
    )
    .unwrap();
    state.source_graph = Some(graph.clone());
    assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
        state.confirm_limiter_calibration_source_bpm(126.0, 10)
    });
    assert_eq!(state.source_graph.as_ref(), Some(&graph));
    assert!(state.source_audio_cache.is_none());
}
