#[test]
fn w30_hook_dawproject_exports_byte_identical_audio_through_action_session_and_receipt() {
    use dawproject::prelude::project::{ClipTypeContent, LanesTypeContent, TimeUnitType};
    use dawproject::DawprojectReader;
    use std::io::Read as _;

    let temp = tempdir().expect("tempdir");
    let package_root = temp.path().join("hook-package");
    let destination = temp.path().join("riotbox-hook.dawproject");
    let mut state = w30_hook_export_state();
    let source_receipt = state
        .commit_stem_package_export_w30_hook_loop(&package_root, 1_300)
        .expect("commit V4 hook package");
    let source_hook = source_receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role == ExportArtifactRole::W30HookLoop)
        .expect("source hook artifact");
    let source_path = PathBuf::from(source_hook.location_identity());
    let source_bytes = fs::read(&source_path).expect("read source hook");

    let receipt = state
        .commit_w30_hook_dawproject_export(None, &destination, 1_400)
        .expect("commit W-30 DAWproject");

    assert_eq!(receipt.export_scope, ExportScope::DawSession);
    assert_eq!(receipt.pack_id, "w30-hook-dawproject");
    assert_eq!(
        receipt.export_boundary,
        ProductExportBoundary::DawSessionW30HookDawprojectV1
    );
    assert_eq!(receipt.export_role, ProductExportRole::ArrangementManifest);
    assert_eq!(state.session.export_receipts.len(), 2);
    assert!(destination.is_file());
    let archive_artifact = receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role == ExportArtifactRole::DawProjectFile)
        .expect("DAWproject artifact");
    assert_eq!(archive_artifact.location_identity(), destination.to_string_lossy());
    assert_eq!(
        receipt.qa_gates[0].gate_id,
        riotbox_core::session::DAWPROJECT_ARCHIVE_QA_GATE_ID
    );
    assert!(
        receipt
            .artifact_set
            .iter()
            .any(|artifact| artifact.role == ExportArtifactRole::DawProjectProof)
    );
    assert!(receipt.arrangement_export_placement_report().ready());
    assert!(receipt.daw_tempo_map_report().ready());
    super::product_export::preflight_export_receipt_artifacts(&receipt, None)
        .expect("DAWproject receipt hydration preflight");

    let mut reader = DawprojectReader::open(&destination).expect("open DAWproject");
    reader.read_dawproject().expect("parse DAWproject XML");
    let project = reader.build_dawproject().expect("typed DAWproject");
    assert_eq!(project.project.version, "1.0");
    assert_eq!(
        project
            .project
            .transport
            .as_ref()
            .and_then(|transport| transport.tempo.as_ref())
            .and_then(|tempo| tempo.value.as_deref()),
        Some("120.000000")
    );
    let arrangement_lanes = project
        .project
        .arrangement
        .as_ref()
        .and_then(|arrangement| arrangement.lanes.as_ref())
        .expect("arrangement lanes");
    let LanesTypeContent::Lanes(hook_lanes) = &arrangement_lanes.content[0] else {
        panic!("first arrangement lane must own the W-30 hook");
    };
    let LanesTypeContent::Clips(hook_clips) = &hook_lanes.content[0] else {
        panic!("W-30 hook lane must contain clips");
    };
    let hook_clip = &hook_clips.clip[0];
    assert_eq!(hook_clip.time, 0.0);
    assert_eq!(hook_clip.duration, Some(8.0));
    assert_eq!(hook_clip.content_time_unit, Some(TimeUnitType::Seconds));
    let Some(ClipTypeContent::Audio(hook_audio)) = hook_clip.content.as_ref() else {
        panic!("W-30 hook clip must contain audio");
    };
    assert_eq!(hook_audio.file.path, "audio/w30_hook_loop.wav");
    assert_eq!(hook_audio.file.external, Some(false));
    let mut embedded = Vec::new();
    reader
        .by_name("audio/w30_hook_loop.wav")
        .expect("embedded hook")
        .read_to_end(&mut embedded)
        .expect("read embedded hook");
    assert_eq!(embedded, source_bytes);
    assert_eq!(receipt.artifact_set.len(), 4);
    let action = state
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| action.command == ActionCommand::ExportDawSession)
        .expect("committed DAWproject action");
    assert_eq!(action.status, ActionStatus::Committed);
    assert!(matches!(
        &action.params,
        ActionParams::DawSessionExport {
            boundary: riotbox_core::action::DawSessionExportBoundary::W30HookDawprojectV1,
            destination_kind: ProductExportDestinationKind::LocalFilePath,
            receipt_id: Some(receipt_id),
            ..
        } if receipt_id == source_receipt.receipt_id.as_str()
    ));
    assert!(state
        .session
        .action_log
        .commit_records
        .iter()
        .any(|record| record.action_id == action.id));
    let roundtrip: SessionFile = serde_json::from_slice(
        &serde_json::to_vec(&state.session).expect("serialize Session"),
    )
    .expect("restore Session");
    assert_eq!(roundtrip.export_receipts, state.session.export_receipts);
    assert_eq!(roundtrip.action_log, state.session.action_log);
}

#[test]
fn w30_hook_dawproject_is_deterministic_and_refuses_existing_destination() {
    let temp = tempdir().expect("tempdir");
    let package_root = temp.path().join("hook-package");
    let first = temp.path().join("first.dawproject");
    let second = temp.path().join("second.dawproject");
    let mut state = w30_hook_export_state();
    state
        .commit_stem_package_export_w30_hook_loop(&package_root, 1_300)
        .expect("commit V4 hook package");
    state
        .commit_w30_hook_dawproject_export(None, &first, 1_400)
        .expect("first DAWproject");
    state
        .commit_w30_hook_dawproject_export(None, &second, 1_500)
        .expect("second DAWproject");
    assert_eq!(
        fs::read(&first).expect("first bytes"),
        fs::read(&second).expect("second bytes")
    );

    let receipt_count = state.session.export_receipts.len();
    let action_count = state.session.action_log.actions.len();
    let error = state
        .commit_w30_hook_dawproject_export(None, &first, 1_600)
        .expect_err("existing destination must reject");
    assert!(error.to_string().contains("already exists"));
    assert_eq!(state.session.export_receipts.len(), receipt_count);
    assert_eq!(state.session.action_log.actions.len(), action_count);
    assert_eq!(
        state.queue.history().last().map(|action| action.status),
        Some(ActionStatus::Rejected)
    );
}

#[test]
fn w30_hook_dawproject_fails_closed_without_v4_receipt_or_after_source_hash_drift() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("blocked.dawproject");
    let mut missing = JamAppState::from_parts(
        SessionFile::new("missing-v4", "riotbox-test", "2026-08-28T00:00:00Z"),
        None,
        ActionQueue::new(),
    );
    let error = missing
        .commit_w30_hook_dawproject_export(None, &destination, 1_000)
        .expect_err("missing V4 receipt must reject");
    assert!(error.to_string().contains("V4 hook receipt"));
    assert!(!destination.exists());
    assert!(missing.session.export_receipts.is_empty());

    let package_root = temp.path().join("hook-package");
    let mut drift = w30_hook_export_state();
    let source_receipt = drift
        .commit_stem_package_export_w30_hook_loop(&package_root, 1_300)
        .expect("commit V4 hook package");
    let source_path = PathBuf::from(
        source_receipt
            .artifact_set
            .iter()
            .find(|artifact| artifact.role == ExportArtifactRole::W30HookLoop)
            .expect("hook artifact")
            .location_identity(),
    );
    drift.session.runtime_state.scene_state.active_scene = None;
    let no_scene_destination = temp.path().join("no-scene.dawproject");
    let error = drift
        .commit_w30_hook_dawproject_export(None, &no_scene_destination, 1_350)
        .expect_err("missing active scene must reject");
    assert!(error.to_string().contains("active Session-owned scene"));
    assert!(!no_scene_destination.exists());
    assert_eq!(drift.session.export_receipts.len(), 1);
    drift.session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-1"));

    let mut mutated = fs::read(&source_path).expect("read hook");
    *mutated.last_mut().expect("WAV byte") ^= 1;
    fs::write(&source_path, mutated).expect("mutate temp hook");
    let drift_destination = temp.path().join("drift.dawproject");
    let error = drift
        .commit_w30_hook_dawproject_export(None, &drift_destination, 1_400)
        .expect_err("hash drift must reject");
    assert!(error.to_string().contains("hash drift"));
    assert!(!drift_destination.exists());
    assert_eq!(drift.session.export_receipts.len(), 1);
}
