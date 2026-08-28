use sha2::{Digest, Sha256};

use super::product_export::{
    STEM_PACKAGE_EXPORT_RESERVED_REASON, StemPackageExportQueueResult,
    StemPackageExportSurfaceBlocker, StemPackageExportSurfaceStatus,
};

#[test]
fn product_mix_export_writes_artifact_and_receipt_after_proof_success() {
    let temp = tempdir().expect("tempdir");
    let proof_dir = temp.path().join("proof");
    let destination = temp.path().join("export");
    fs::create_dir_all(&proof_dir).expect("create proof dir");
    let artifact_path = proof_dir.join("full_grid_mix.wav");
    write_pcm16_wave(&artifact_path, 1_000, 1, 0.01);
    let artifact_bytes = fs::read(&artifact_path).expect("read product artifact");
    let artifact_hash = sha256_bytes(&artifact_bytes);
    let proof_path = proof_dir.join("product_export_proof.json");
    write_product_export_proof(&proof_path, "full_grid_mix.wav", &artifact_hash);
    let proof_hash = sha256_bytes(&fs::read(&proof_path).expect("read product proof"));

    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.source_timing.confirmed_grid = Some(SourceTimingGridConfirmationState {
        source_id: SourceId::from("src-1"),
        hypothesis_id: Some("primary-grid".into()),
        confirmed_by_action: ActionId(1),
        confirmed_at: 850,
    });
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let receipt = state
        .commit_product_mix_export_from_proof(&proof_path, &destination, 900)
        .expect("commit product export");

    assert_eq!(receipt.created_by_action, ActionId(2));
    assert_eq!(receipt.export_role, ProductExportRole::FullGridMix);
    assert_eq!(
        receipt.export_boundary,
        ProductExportBoundary::FeralGridGeneratedSupport
    );
    assert_eq!(receipt.export_hash, artifact_hash);
    assert_eq!(receipt.artifact_set.len(), 2);
    let artifact = &receipt.artifact_set[0];
    assert_eq!(artifact.role, ExportArtifactRole::FullGridMix);
    assert_eq!(
        artifact.location,
        ExportArtifactLocation::LocalPath {
            path: destination.join("full_grid_mix.wav").to_string_lossy().into_owned()
        }
    );
    assert_eq!(artifact.media_type, ExportArtifactMediaType::AudioWav);
    assert_eq!(artifact.sha256, artifact_hash);
    assert_eq!(
        artifact.normalized_manifest_hash.as_deref(),
        Some("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd")
    );
    assert_eq!(
        artifact.source_graph_ref,
        Some(ExportArtifactSourceGraphRef {
        source_id: SourceId::from("src-1"),
        graph_version: SourceGraphVersion::V1,
        graph_hash: state.session.source_graph_refs[0].graph_hash.clone(),
        })
    );
    assert_eq!(
        artifact.timing_grid_ref,
        Some(ExportArtifactTimingGridRef {
        source_id: SourceId::from("src-1"),
        hypothesis_id: Some("primary-grid".into()),
        confirmed_by_action: ActionId(1),
        confirmed_at: 850,
        })
    );
    let metrics = artifact.audio_metrics.as_ref().expect("audio metrics");
    assert!(metrics.peak_milli_dbfs.expect("peak dbfs") < 0);
    assert!(metrics.rms_milli_dbfs.expect("rms dbfs") < 0);
    assert!(metrics.peak_amplitude_micros.expect("peak amplitude") > 200_000);
    assert!(metrics.rms_amplitude_micros.expect("rms amplitude") > 100_000);
    assert_eq!(metrics.silent_frame_count, Some(1));
    assert_eq!(metrics.total_frame_count, Some(10));
    assert_eq!(artifact.sample_rate_hz, Some(1_000));
    assert_eq!(artifact.channel_count, Some(1));
    assert_eq!(artifact.duration_ms, Some(10));
    let proof_artifact = &receipt.artifact_set[1];
    assert_eq!(proof_artifact.role, ExportArtifactRole::ProductExportProof);
    assert_eq!(
        proof_artifact.location,
        ExportArtifactLocation::LocalPath {
            path: destination
                .join("product_export_proof.json")
                .to_string_lossy()
                .into_owned()
        }
    );
    assert_eq!(proof_artifact.media_type, ExportArtifactMediaType::Json);
    assert_eq!(proof_artifact.sha256, proof_hash);
    assert_eq!(proof_artifact.audio_metrics, None);
    assert_eq!(
        receipt.unsupported_scopes,
        vec![
            UnsupportedExportScope::StemPackage,
            UnsupportedExportScope::LiveRecording,
            UnsupportedExportScope::DawExport,
            UnsupportedExportScope::HostAudioSoak,
        ]
    );
    assert!(destination.join("full_grid_mix.wav").is_file());
    assert!(destination.join("product_export_proof.json").is_file());
    assert_eq!(state.session.export_receipts, vec![receipt.clone()]);
    assert!(state.queue.pending_actions().is_empty());

    let action = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.command == ActionCommand::ExportProductMix)
        .expect("export action logged");
    assert_eq!(action.status, ActionStatus::Committed);
    assert_eq!(action.committed_at, Some(900));
    assert!(matches!(action.undo_policy, UndoPolicy::NotUndoable { .. }));
    assert!(
        action
            .result
            .as_ref()
            .expect("result")
            .summary
            .contains("exported full_grid_mix")
    );
    assert!(state.session.action_log.commit_records.iter().any(|record| {
        record.action_id == action.id
            && record.boundary.kind == CommitBoundary::Immediate
            && record.committed_at == 900
    }));
}

#[test]
fn product_mix_export_rejects_without_receipt_when_proof_artifact_hash_fails() {
    let temp = tempdir().expect("tempdir");
    let proof_dir = temp.path().join("proof");
    let destination = temp.path().join("export");
    fs::create_dir_all(&proof_dir).expect("create proof dir");
    let artifact_path = proof_dir.join("full_grid_mix.wav");
    fs::write(&artifact_path, b"changed product mix").expect("write product artifact");
    let proof_path = proof_dir.join("product_export_proof.json");
    write_product_export_proof(
        &proof_path,
        "full_grid_mix.wav",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );

    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let error = state
        .commit_product_mix_export_from_proof(&proof_path, &destination, 900)
        .expect_err("hash mismatch rejects export");

    assert!(error.to_string().contains("export artifact hash mismatch"));
    assert!(state.session.export_receipts.is_empty());
    assert!(!destination.join("full_grid_mix.wav").exists());
    assert!(
        state
            .session
            .action_log
            .actions
            .iter()
            .all(|action| action.command != ActionCommand::ExportProductMix)
    );
    let rejected = state
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportProductMix)
        .expect("rejected export action recorded in queue history");
    assert_eq!(rejected.status, ActionStatus::Rejected);
}

#[test]
fn product_mix_export_rejects_source_mismatch_before_writing_or_attaching_lineage() {
    let temp = tempdir().expect("tempdir");
    let proof_dir = temp.path().join("proof");
    let destination = temp.path().join("export");
    fs::create_dir_all(&proof_dir).expect("create proof dir");
    let artifact_path = proof_dir.join("full_grid_mix.wav");
    write_pcm16_wave(&artifact_path, 1_000, 1, 0.01);
    let artifact_hash = sha256_bytes(&fs::read(&artifact_path).expect("read artifact"));
    let proof_path = proof_dir.join("product_export_proof.json");
    write_product_export_proof(&proof_path, "full_grid_mix.wav", &artifact_hash);

    let mut graph = sample_graph();
    graph.source.content_hash = "sha256:different-source".into();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let error = state
        .commit_product_mix_export_from_active_source_proof(&proof_path, &destination, 901)
        .expect_err("source mismatch must reject export");

    assert!(error.to_string().contains("product mix export source mismatch"));
    assert!(!destination.exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
    let rejected = state
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportProductMix)
        .expect("rejected export action");
    assert_eq!(rejected.status, ActionStatus::Rejected);
    assert!(
        rejected
            .result
            .as_ref()
            .expect("rejection result")
            .summary
            .contains("source mismatch")
    );
}

#[test]
fn active_source_product_mix_export_rejects_missing_source_identity_without_pending_action() {
    let temp = tempdir().expect("tempdir");
    let mut state = JamAppState::from_parts(
        SessionFile::new("export-no-source", "0.1.0", "2026-08-26T00:00:00Z"),
        None,
        ActionQueue::new(),
    );

    let error = state
        .commit_product_mix_export_from_active_source_proof(
            temp.path().join("proof.json"),
            temp.path().join("export"),
            902,
        )
        .expect_err("missing active source must reject export");

    assert!(error.to_string().contains("active Source Graph identity"));
    assert!(state.queue.pending_actions().is_empty());
    assert!(state.session.export_receipts.is_empty());
    assert_eq!(state.queue.history().len(), 1);
    assert_eq!(state.queue.history()[0].status, ActionStatus::Rejected);
}

#[test]
fn explicit_product_mix_rejection_consumes_an_existing_pending_request() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.queue_product_mix_export(903, None);

    let rejected_action = state.reject_product_mix_export_request(
        904,
        "product mix export handoff is unavailable",
    );

    assert!(state.queue.pending_actions().is_empty());
    assert_eq!(state.queue.history().len(), 1);
    assert_eq!(state.queue.history()[0].id, rejected_action);
    assert_eq!(state.queue.history()[0].status, ActionStatus::Rejected);
}

#[test]
fn reserved_stem_package_export_queue_attempt_is_rejected_without_receipt() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let result = state.queue_stem_package_export_reserved(
        950,
        Some("exports/stem-package".into()),
        vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
    );

    let reason = match result {
        StemPackageExportQueueResult::Rejected { reason } => reason,
        other => panic!("expected reserved stem package export rejection, got {other:?}"),
    };
    assert!(reason.contains("stem package export is disabled for musicians"));
    assert!(reason.starts_with(STEM_PACKAGE_EXPORT_RESERVED_REASON));
    assert!(reason.contains("stem writer proof is missing"));
    assert!(reason.contains("DAW placement workflow is not ready"));
    assert!(state.queue.pending_actions().is_empty());
    assert!(state.session.export_receipts.is_empty());
    assert!(
        state
            .session
            .action_log
            .actions
            .iter()
            .all(|action| action.command != ActionCommand::ExportStemPackage)
    );

    let rejected = state
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportStemPackage)
        .expect("reserved stem package action recorded in queue history");
    assert_eq!(rejected.status, ActionStatus::Rejected);
    assert_eq!(
        rejected.result.as_ref().map(|result| result.summary.as_str()),
        Some(reason.as_str())
    );
    assert!(matches!(rejected.undo_policy, UndoPolicy::NotUndoable { .. }));
    assert_eq!(rejected.target.scope, Some(TargetScope::Session));
    match &rejected.params {
        ActionParams::StemPackageExport {
            export_scope,
            claimed_stem_roles,
            lineage_policy,
            fallback_comparison_policy,
            ..
        } => {
            assert_eq!(*export_scope, ExportScope::StemPackage);
            assert_eq!(
                claimed_stem_roles,
                &vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass]
            );
            assert_eq!(
                *lineage_policy,
                riotbox_core::action::StemPackageLineagePolicy::RequireAnyCoreLineage
            );
            assert_eq!(
                *fallback_comparison_policy,
                riotbox_core::action::StemPackageFallbackComparisonPolicy::Required
            );
        }
        other => panic!("expected stem package params, got {other:?}"),
    }
}

#[test]
fn stem_package_musician_surface_gate_is_disabled_until_product_gates_are_ready() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let gate = state.stem_package_export_surface_gate();

    assert_eq!(gate.status, StemPackageExportSurfaceStatus::Disabled);
    assert!(!gate.runnable());
    assert_eq!(
        gate.blockers,
        vec![
            StemPackageExportSurfaceBlocker::CiWriterProofMissing,
            StemPackageExportSurfaceBlocker::DeveloperProofOnly,
            StemPackageExportSurfaceBlocker::DawPlacementWorkflowMissing,
            StemPackageExportSurfaceBlocker::StructuredListeningReviewMissing,
        ]
    );
    assert!(gate.musician_summary().contains("disabled for musicians"));
}

#[test]
fn local_ci_stem_package_proof_still_keeps_musician_surface_disabled() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("stem-export");
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state
        .commit_stem_package_export_local_ci_package(
            &destination,
            1_131,
            vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        )
        .expect("commit local CI stem package export");

    let gate = state.stem_package_export_surface_gate();

    assert_eq!(gate.status, StemPackageExportSurfaceStatus::Disabled);
    assert!(!gate.runnable());
    assert_eq!(
        gate.blockers,
        vec![
            StemPackageExportSurfaceBlocker::DeveloperProofOnly,
            StemPackageExportSurfaceBlocker::DawPlacementWorkflowMissing,
            StemPackageExportSurfaceBlocker::StructuredListeningReviewMissing,
        ]
    );
}

#[test]
fn local_ci_stem_package_export_commits_writer_receipt_and_action() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("stem-export");
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let receipt = state
        .commit_stem_package_export_local_ci_package(
            &destination,
            1_131,
            vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        )
        .expect("commit local CI stem package export");

    assert_eq!(receipt.created_by_action, ActionId(2));
    assert_eq!(receipt.export_scope, ExportScope::StemPackage);
    assert!(receipt.stem_package_readiness_report().ready());
    assert!(receipt.unsupported_scopes.is_empty());
    assert_eq!(receipt.artifact_set.len(), 4);
    assert_eq!(receipt.qa_gates.len(), 5);
    assert!(destination.join("stem_package/stems/stem_drums.wav").is_file());
    assert!(destination.join("stem_package/stems/stem_bass.wav").is_file());
    assert!(
        destination
            .join("stem_package/stem_package_manifest.json")
            .is_file()
    );
    assert!(
        destination
            .join("stem_package/stem_package_proof.json")
            .is_file()
    );
    super::product_export::preflight_export_receipt_artifacts(&receipt, None)
        .expect("committed stem package receipt artifacts should preflight");
    assert_eq!(state.session.export_receipts, vec![receipt.clone()]);
    assert!(state.queue.pending_actions().is_empty());

    let action = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.command == ActionCommand::ExportStemPackage)
        .expect("stem package export action logged");
    assert_eq!(action.status, ActionStatus::Committed);
    assert_eq!(action.committed_at, Some(1_131));
    assert!(matches!(action.undo_policy, UndoPolicy::NotUndoable { .. }));
    assert!(
        action
            .result
            .as_ref()
            .expect("result")
            .summary
            .contains("exported stem_package")
    );
    match &action.params {
        ActionParams::StemPackageExport {
            boundary,
            claimed_stem_roles,
            ..
        } => {
            assert_eq!(
                *boundary,
                riotbox_core::action::StemPackageExportBoundary::LocalCiPackageV1
            );
            assert_eq!(
                claimed_stem_roles,
                &vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass]
            );
        }
        other => panic!("expected stem package params, got {other:?}"),
    }
    assert!(state.session.action_log.commit_records.iter().any(|record| {
        record.action_id == action.id
            && record.boundary.kind == CommitBoundary::Immediate
            && record.committed_at == 1_131
    }));
}

#[test]
fn local_ci_stem_package_export_rejects_unsupported_role_without_receipt() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("stem-export");
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let error = state
        .commit_stem_package_export_local_ci_package(
            &destination,
            1_131,
            vec![ExportArtifactRole::StemMusic],
        )
        .expect_err("unsupported stem role should reject local CI export");

    assert!(error.to_string().contains("unsupported local CI stem package role"));
    assert!(state.session.export_receipts.is_empty());
    assert!(!destination.join("stem_package").exists());
    assert!(
        state
            .session
            .action_log
            .actions
            .iter()
            .all(|action| action.command != ActionCommand::ExportStemPackage)
    );
    let rejected = state
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportStemPackage)
        .expect("rejected stem package action recorded in queue history");
    assert_eq!(rejected.status, ActionStatus::Rejected);
    assert!(
        rejected
            .result
            .as_ref()
            .expect("result")
            .summary
            .contains("unsupported local CI stem package role")
    );
}

#[test]
fn source_matched_stem_handoff_commits_three_real_stems_and_session_receipt() {
    let temp = tempdir().expect("tempdir");
    let source_hash = "a".repeat(64);
    let proof_path = write_source_matched_handoff_fixture(temp.path(), &source_hash);
    let destination = temp.path().join("export");
    let mut graph = sample_graph();
    graph.source.content_hash = source_hash.clone();
    let mut session = sample_session(&graph);
    session.runtime_state.source_timing.confirmed_grid = Some(SourceTimingGridConfirmationState {
        source_id: graph.source.source_id.clone(),
        hypothesis_id: Some("source-matched-grid".into()),
        confirmed_by_action: ActionId(1),
        confirmed_at: 1_200,
    });
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let receipt = state
        .commit_stem_package_export_from_product_handoff(&proof_path, &destination, 1_210)
        .expect("commit source-matched stem package");

    assert_eq!(
        receipt.export_boundary,
        ProductExportBoundary::StemPackageSourceMatchedHandoffV1
    );
    assert_eq!(
        receipt.pack_id,
        riotbox_core::export_readiness::STEM_PACKAGE_SOURCE_MATCHED_PACK_ID
    );
    assert!(receipt.stem_package_readiness_report().ready());
    assert_eq!(receipt.artifact_set.len(), 5);
    for role in [
        ExportArtifactRole::StemDrums,
        ExportArtifactRole::StemMusic,
        ExportArtifactRole::StemBass,
    ] {
        let artifact = receipt
            .artifact_set
            .iter()
            .find(|artifact| artifact.role == role)
            .expect("source-matched stem artifact");
        assert_eq!(
            artifact.source_graph_ref.as_ref().map(|value| &value.source_id),
            Some(&SourceId::from("src-1"))
        );
        assert!(artifact.timing_grid_ref.is_some());
        let fallback = artifact
            .fallback_comparison
            .as_ref()
            .expect("fail-closed silence comparison");
        assert!(
            fallback
                .reference_identity
                .contains("#fail-closed-silence-v1/")
        );
        assert!(fallback.rms_difference_micros.is_some_and(|value| value > 0));
        let output_path = destination
            .join("stem_package/stems")
            .join(format!("{}.wav", source_matched_role_file_stem(role)));
        let input_path = proof_path
            .parent()
            .expect("proof parent")
            .join("stems")
            .join(format!("{}.wav", source_matched_role_file_stem(role)));
        assert_eq!(
            fs::read(output_path).expect("read output stem"),
            fs::read(input_path).expect("read input stem")
        );
    }
    let manifest_path = destination.join("stem_package/stem_package_manifest.json");
    let on_disk_manifest: riotbox_core::stem_package_manifest::StemPackageManifest =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read package manifest"))
            .expect("parse package manifest");
    assert_eq!(
        on_disk_manifest.package_id,
        riotbox_core::export_readiness::STEM_PACKAGE_SOURCE_MATCHED_PACK_ID
    );
    assert_eq!(
        on_disk_manifest.export_boundary,
        ProductExportBoundary::StemPackageSourceMatchedHandoffV1
    );
    let package_proof_path = destination.join("stem_package/stem_package_proof.json");
    let on_disk_proof: riotbox_core::stem_package_proof::StemPackageProof =
        serde_json::from_slice(&fs::read(&package_proof_path).expect("read package proof"))
            .expect("parse package proof");
    assert_eq!(
        on_disk_proof.package_id,
        riotbox_core::export_readiness::STEM_PACKAGE_SOURCE_MATCHED_PACK_ID
    );
    assert_eq!(
        on_disk_proof.export_boundary,
        ProductExportBoundary::StemPackageSourceMatchedHandoffV1
    );
    assert_eq!(
        on_disk_proof.manifest_sha256,
        on_disk_manifest
            .normalized_json_sha256()
            .expect("hash package manifest")
    );
    assert!(state.queue.pending_actions().is_empty());
    let musician_gate = state.stem_package_export_surface_gate();
    assert_eq!(musician_gate.status, StemPackageExportSurfaceStatus::Disabled);
    assert_eq!(
        musician_gate.blockers,
        vec![
            StemPackageExportSurfaceBlocker::DeveloperProofOnly,
            StemPackageExportSurfaceBlocker::DawPlacementWorkflowMissing,
            StemPackageExportSurfaceBlocker::StructuredListeningReviewMissing,
        ]
    );
    assert_eq!(state.session.export_receipts, vec![receipt.clone()]);
    let action = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.command == ActionCommand::ExportStemPackage)
        .expect("committed source-matched action");
    assert_eq!(action.status, ActionStatus::Committed);
    match &action.params {
        ActionParams::StemPackageExport {
            boundary,
            handoff_proof_path,
            claimed_stem_roles,
            ..
        } => {
            assert_eq!(
                *boundary,
                riotbox_core::action::StemPackageExportBoundary::SourceMatchedHandoffV1
            );
            assert_eq!(
                handoff_proof_path.as_deref(),
                Some(proof_path.to_string_lossy().as_ref())
            );
            assert_eq!(
                claimed_stem_roles,
                &vec![
                    ExportArtifactRole::StemDrums,
                    ExportArtifactRole::StemMusic,
                    ExportArtifactRole::StemBass,
                ]
            );
        }
        other => panic!("expected source-matched stem package params, got {other:?}"),
    }
}

#[test]
fn source_matched_stem_handoff_rejects_stale_source_before_destination_write() {
    let temp = tempdir().expect("tempdir");
    let proof_path = write_source_matched_handoff_fixture(temp.path(), &"b".repeat(64));
    let destination = temp.path().join("export");
    let mut graph = sample_graph();
    graph.source.content_hash = "a".repeat(64);
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let error = state
        .commit_stem_package_export_from_product_handoff(&proof_path, &destination, 1_220)
        .expect_err("stale source identity must reject");

    assert!(error.to_string().contains("source mismatch"));
    assert!(!destination.join("stem_package").exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
    assert!(state.queue.history().iter().any(|action| {
        action.command == ActionCommand::ExportStemPackage
            && action.status == ActionStatus::Rejected
    }));
}

#[test]
fn source_matched_stem_handoff_requires_session_graph_lineage_before_write() {
    let temp = tempdir().expect("tempdir");
    let source_hash = "a".repeat(64);
    let proof_path = write_source_matched_handoff_fixture(temp.path(), &source_hash);
    let destination = temp.path().join("export");
    let mut graph = sample_graph();
    graph.source.content_hash = source_hash;
    let mut session = sample_session(&graph);
    session.source_graph_refs.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let error = state
        .commit_stem_package_export_from_product_handoff(&proof_path, &destination, 1_220)
        .expect_err("missing Session graph lineage must reject");

    assert!(error.to_string().contains("Session Source Graph lineage"));
    assert!(!destination.exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn source_matched_stem_handoff_rejects_stale_session_graph_ref_before_write() {
    let temp = tempdir().expect("tempdir");
    let source_hash = "a".repeat(64);
    let proof_path = write_source_matched_handoff_fixture(temp.path(), &source_hash);
    let destination = temp.path().join("export");
    let mut graph = sample_graph();
    graph.source.content_hash = source_hash;
    let mut session = sample_session(&graph);
    session.source_graph_refs[0].graph_hash = format!("sha256:{}", "f".repeat(64));
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let error = state
        .commit_stem_package_export_from_product_handoff(&proof_path, &destination, 1_220)
        .expect_err("stale Session graph ref must reject");

    assert!(error.to_string().contains("exact active Session Source Graph lineage"));
    assert!(!destination.exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
}

#[cfg(unix)]
#[test]
fn source_matched_stem_handoff_rejects_symlinked_artifact_before_write() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().expect("tempdir");
    let source_hash = "a".repeat(64);
    let proof_path = write_source_matched_handoff_fixture(temp.path(), &source_hash);
    let drums_path = proof_path
        .parent()
        .expect("proof parent")
        .join("stems/stem_drums.wav");
    let drums_target = proof_path
        .parent()
        .expect("proof parent")
        .join("stem_drums_target.wav");
    fs::rename(&drums_path, &drums_target).expect("move drums target");
    symlink(&drums_target, &drums_path).expect("symlink drums artifact");
    let destination = temp.path().join("export");
    let mut graph = sample_graph();
    graph.source.content_hash = source_hash;
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let error = state
        .commit_stem_package_export_from_product_handoff(&proof_path, &destination, 1_225)
        .expect_err("symlinked handoff artifact must reject");

    assert!(error.to_string().contains("regular file"));
    assert!(!destination.exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn source_matched_stem_handoff_does_not_displace_another_pending_stem_export() {
    let temp = tempdir().expect("tempdir");
    let source_hash = "d".repeat(64);
    let proof_path = write_source_matched_handoff_fixture(temp.path(), &source_hash);
    let destination = temp.path().join("export");
    let mut graph = sample_graph();
    graph.source.content_hash = source_hash;
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    assert_eq!(
        state.queue_stem_package_export_local_ci_package(
            1_205,
            Some(temp.path().join("local-ci").to_string_lossy().into_owned()),
            vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        ),
        QueueControlResult::Enqueued
    );

    let error = state
        .commit_stem_package_export_from_product_handoff(&proof_path, &destination, 1_210)
        .expect_err("a different pending stem export must block source-matched ingress");

    assert!(error.to_string().contains("another stem-package export is pending"));
    assert!(!destination.exists());
    assert!(state.session.export_receipts.is_empty());
    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert!(matches!(
        pending[0].params,
        ActionParams::StemPackageExport {
            boundary: riotbox_core::action::StemPackageExportBoundary::LocalCiPackageV1,
            ..
        }
    ));
}

#[test]
fn source_matched_stem_handoff_rejects_hash_and_reconstruction_mutations_fail_closed() {
    for mutation in ["hash", "reconstruction"] {
        let temp = tempdir().expect("tempdir");
        let source_hash = "a".repeat(64);
        let proof_path = write_source_matched_handoff_fixture(temp.path(), &source_hash);
        let mut proof: serde_json::Value = serde_json::from_slice(
            &fs::read(&proof_path).expect("read source-matched proof"),
        )
        .expect("parse source-matched proof");
        match mutation {
            "hash" => proof["artifacts"][0]["sha256"] = serde_json::json!("f".repeat(64)),
            "reconstruction" => {
                proof["reconstruction"]["max_abs_error"] = serde_json::json!(0.00008)
            }
            _ => unreachable!(),
        }
        fs::write(
            &proof_path,
            serde_json::to_vec_pretty(&proof).expect("serialize mutated proof"),
        )
        .expect("write mutated proof");
        let destination = temp.path().join("export");
        let mut graph = sample_graph();
        graph.source.content_hash = source_hash;
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert!(
            state
                .commit_stem_package_export_from_product_handoff(
                    &proof_path,
                    &destination,
                    1_230,
                )
                .is_err(),
            "{mutation} mutation must reject"
        );
        assert!(!destination.join("stem_package").exists());
        assert!(state.session.export_receipts.is_empty());
        assert!(state.queue.pending_actions().is_empty());
    }
}

#[test]
fn w30_hook_loop_export_commits_one_semantic_stem_through_the_existing_spine() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("w30-hook-export");
    let mut state = w30_hook_export_state();

    let receipt = state
        .commit_stem_package_export_w30_hook_loop(&destination, 1_300)
        .expect("commit semantic W-30 hook export");

    assert_eq!(receipt.export_scope, ExportScope::StemPackage);
    assert_eq!(receipt.pack_id, "stem-package-w30-hook-loop");
    assert_eq!(
        receipt.export_boundary,
        ProductExportBoundary::StemPackageW30HookLoopV4
    );
    assert!(receipt.stem_package_readiness_report().ready());
    let stem = receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role == ExportArtifactRole::W30HookLoop)
        .expect("semantic hook artifact");
    assert_eq!(stem.sample_rate_hz, Some(48_000));
    assert_eq!(stem.channel_count, Some(2));
    assert_eq!(stem.duration_ms, Some(4_000));
    assert_eq!(stem.source_capture_refs, vec![CaptureId::from("cap-01")]);
    assert!(stem.source_graph_ref.is_some());
    assert!(stem.timing_grid_ref.is_some());
    assert!(stem.fallback_comparison.is_some());
    assert!(
        destination
            .join("stem_package/stems/w30_hook_loop.wav")
            .is_file()
    );
    assert_eq!(state.session.export_receipts, vec![receipt.clone()]);
    let action = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| {
            action.command == ActionCommand::ExportStemPackage
                && matches!(
                    action.params,
                    ActionParams::StemPackageExport {
                        boundary: riotbox_core::action::StemPackageExportBoundary::W30HookLoopV4,
                        ..
                    }
                )
        })
        .expect("committed semantic hook action");
    assert_eq!(action.status, ActionStatus::Committed);
    let restored: riotbox_core::session::SessionFile = serde_json::from_value(
        serde_json::to_value(&state.session).expect("serialize semantic hook Session"),
    )
    .expect("restore semantic hook Session");
    assert_eq!(restored.export_receipts, vec![receipt]);
    assert!(restored.action_log.actions.iter().any(|action| {
        matches!(
            action.params,
            ActionParams::StemPackageExport {
                boundary: riotbox_core::action::StemPackageExportBoundary::W30HookLoopV4,
                ..
            }
        )
    }));
}

#[test]
fn w30_hook_loop_export_fails_closed_without_an_ordinary_focused_capture() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("w30-hook-export");
    let mut state = w30_hook_export_state();
    state.session.runtime_state.lane_state.w30.last_capture = None;
    state.session.runtime_state.lane_state.w30.focused_pad = None;
    state.refresh_view();

    let error = state
        .commit_stem_package_export_w30_hook_loop(&destination, 1_310)
        .expect_err("missing focused capture must fail closed");

    assert!(error.to_string().contains("focused W-30 capture"));
    assert!(!destination.join("stem_package").exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn w30_hook_loop_export_fails_closed_from_capture_scoped_damage_state() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("w30-hook-export");
    let mut state = w30_hook_export_state();
    assert_eq!(
        state.queue_w30_apply_damage_profile(1_305),
        Some(QueueControlResult::Enqueued)
    );
    let committed = state.commit_ready_actions(
        riotbox_core::transport::CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 8,
            bar_index: 3,
            phrase_index: 1,
            scene_id: Some(riotbox_core::ids::SceneId::from("scene-1")),
        },
        1_306,
    );
    assert_eq!(committed.len(), 1);

    let error = state
        .commit_stem_package_export_w30_hook_loop(&destination, 1_310)
        .expect_err("damaged focused capture must fail closed");

    assert!(error.to_string().contains("ordinary promoted or pinned"));
    assert!(!destination.join("stem_package").exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn w30_hook_loop_export_fails_closed_from_another_ordinary_state_owner() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("w30-hook-export");
    let mut state = w30_hook_export_state();
    state.session.action_log.actions.pop();

    let error = state
        .commit_stem_package_export_w30_hook_loop(&destination, 1_310)
        .expect_err("non-owner action history must fail closed");

    assert!(error.to_string().contains("exact six-action"));
    assert!(!destination.join("stem_package").exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn w30_hook_loop_export_fails_closed_from_wrong_six_action_params() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("w30-hook-export");
    let mut state = w30_hook_export_state();
    state.session.action_log.actions[5].params = ActionParams::Mutation {
        intensity: 0.40,
        target_id: Some("cap-01".into()),
    };

    let error = state
        .commit_stem_package_export_w30_hook_loop(&destination, 1_310)
        .expect_err("wrong exact-owner params must fail closed");

    assert!(error.to_string().contains("parameters, targets"));
    assert!(!destination.join("stem_package").exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn w30_hook_loop_export_fails_closed_from_stale_exact_owner_state() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("w30-hook-export");
    let mut state = w30_hook_export_state();
    state.session.captures[0].created_from_action = Some(ActionId(99));

    let error = state
        .commit_stem_package_export_w30_hook_loop(&destination, 1_310)
        .expect_err("stale exact-owner state must fail closed");

    assert!(error.to_string().contains("state lineage"));
    assert!(!destination.join("stem_package").exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
}

fn w30_hook_export_state() -> JamAppState {
    let mut graph = sample_graph();
    graph.source.content_hash = "a".repeat(64);
    graph.provenance.source_hash = graph.source.content_hash.clone();
    let mut session = sample_session(&graph);
    session.source_refs[0].content_hash = graph.source.content_hash.clone();
    session.runtime_state.style.active_preset =
        Some(riotbox_core::style::PerformancePresetId::FeralBreakAlphaV2);
    session.runtime_state.capture.length_intent =
        riotbox_core::action::CaptureLengthIntent::OneBar;
    session.runtime_state.source_timing.confirmed_bpm = Some(120.0);
    session.runtime_state.source_timing.confirmed_grid = Some(SourceTimingGridConfirmationState {
        source_id: graph.source.source_id.clone(),
        hypothesis_id: Some("w30-hook-grid".into()),
        confirmed_by_action: ActionId(1),
        confirmed_at: 1_250,
    });
    session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    session.captures[0].created_from_action = Some(ActionId(4));
    session.captures[0].source_window = Some(CaptureSourceWindow {
        source_id: graph.source.source_id.clone(),
        start_seconds: 0.0,
        end_seconds: 1.0,
        start_frame: 0,
        end_frame: 48_000,
        hook_selection: None,
    });
    let seed_action = session.action_log.actions[0].clone();
    let source_id = graph.source.source_id.clone();
    let capture_id = session.captures[0].capture_id.clone();
    let bank_id = BankId::from("bank-a");
    let pad_id = PadId::from("pad-01");
    session.action_log.actions = [
        (
            ActionCommand::SourceTimingConfirmGrid,
            ActionParams::SourceTimingGrid {
                source_id: Some(source_id),
                hypothesis_id: Some("w30-hook-grid".into()),
                confirmed_bpm: Some(120.0),
            },
            riotbox_core::action::ActionTarget {
                scope: Some(riotbox_core::action::TargetScope::Session),
                object_id: Some("w30-hook-grid".into()),
                ..Default::default()
            },
            riotbox_core::action::Quantization::Immediate,
        ),
        (
            ActionCommand::PresetActivate,
            ActionParams::Preset {
                preset_id: riotbox_core::style::PerformancePresetId::FeralBreakAlphaV2,
            },
            riotbox_core::action::ActionTarget {
                scope: Some(riotbox_core::action::TargetScope::Session),
                object_id: Some("feral_break_alpha_v2".into()),
                ..Default::default()
            },
            riotbox_core::action::Quantization::Immediate,
        ),
        (
            ActionCommand::CaptureSetLength,
            ActionParams::CaptureLength {
                intent: Some(riotbox_core::action::CaptureLengthIntent::OneBar),
            },
            riotbox_core::action::ActionTarget {
                scope: Some(riotbox_core::action::TargetScope::Session),
                object_id: Some("capture-length".into()),
                ..Default::default()
            },
            riotbox_core::action::Quantization::Immediate,
        ),
        (
            ActionCommand::CaptureBarGroup,
            ActionParams::Capture { bars: None },
            riotbox_core::action::ActionTarget {
                scope: Some(riotbox_core::action::TargetScope::LaneW30),
                ..Default::default()
            },
            riotbox_core::action::Quantization::NextBar,
        ),
        (
            ActionCommand::PromoteCaptureToPad,
            ActionParams::Promotion {
                capture_id: Some(capture_id.clone()),
                destination: Some("w30:bank-a/pad-01".into()),
            },
            riotbox_core::action::ActionTarget {
                scope: Some(riotbox_core::action::TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
            riotbox_core::action::Quantization::NextBar,
        ),
        (
            ActionCommand::W30TriggerPad,
            ActionParams::Mutation {
                intensity: 0.84,
                target_id: Some(capture_id.to_string()),
            },
            riotbox_core::action::ActionTarget {
                scope: Some(riotbox_core::action::TargetScope::LaneW30),
                bank_id: Some(bank_id),
                pad_id: Some(pad_id),
                ..Default::default()
            },
            riotbox_core::action::Quantization::NextBeat,
        ),
    ]
    .into_iter()
    .enumerate()
    .map(
        |(index, (command, params, target, quantization))| riotbox_core::action::Action {
            id: ActionId((index + 1) as u64),
            actor: riotbox_core::action::ActorType::User,
            command,
            params,
            target,
            quantization,
            status: ActionStatus::Committed,
            ..seed_action.clone()
        },
    )
    .collect();

    let frame_count = 48_000_usize;
    let mut samples = Vec::with_capacity(frame_count * 2);
    for frame in 0..frame_count {
        let sample = ((frame as f32 / 48_000.0) * 110.0 * std::f32::consts::TAU).sin() * 0.2;
        samples.extend([sample, sample]);
    }
    let capture_audio = SourceAudioCache::from_interleaved_samples(
        "capture-cap-01.wav",
        48_000,
        2,
        samples,
    )
    .expect("capture audio");
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state
        .capture_audio_cache
        .insert(CaptureId::from("cap-01"), capture_audio);
    state.refresh_view();
    state
}

pub(crate) fn write_source_matched_handoff_fixture(
    root: &Path,
    source_sha256: &str,
) -> PathBuf {
    let bundle = root.join("handoff");
    let stems = bundle.join("stems");
    fs::create_dir_all(&stems).expect("create handoff stems");
    let sample_rate = 1_000_u32;
    let channel_count = 2_u16;
    let frame_count = 2_000_usize;
    let mut drums = Vec::with_capacity(frame_count * usize::from(channel_count));
    let mut music = Vec::with_capacity(drums.capacity());
    let mut bass = Vec::with_capacity(drums.capacity());
    let mut full_mix = Vec::with_capacity(drums.capacity());
    for frame in 0..frame_count {
        let seconds = frame as f32 / sample_rate as f32;
        let drum = (seconds * 220.0 * std::f32::consts::TAU).sin() * 0.08;
        let musical = (seconds * 110.0 * std::f32::consts::TAU).sin() * 0.06;
        let low = (seconds * 55.0 * std::f32::consts::TAU).sin() * 0.07;
        for _ in 0..channel_count {
            drums.push(drum);
            music.push(musical);
            bass.push(low);
            full_mix.push(drum + musical + low);
        }
    }
    let paths = [
        ("stems/stem_drums.wav", &drums),
        ("stems/stem_music.wav", &music),
        ("stems/stem_bass.wav", &bass),
        ("full_grid_mix.wav", &full_mix),
    ];
    for (relative, samples) in paths {
        riotbox_audio::source_audio::write_interleaved_pcm16_wav(
            bundle.join(relative),
            sample_rate,
            channel_count,
            samples,
        )
        .expect("write source-matched fixture stem");
    }
    let drums_audio = SourceAudioCache::load_pcm_wav(stems.join("stem_drums.wav"))
        .expect("decode drums");
    let music_audio = SourceAudioCache::load_pcm_wav(stems.join("stem_music.wav"))
        .expect("decode music");
    let bass_audio = SourceAudioCache::load_pcm_wav(stems.join("stem_bass.wav"))
        .expect("decode bass");
    let full_audio = SourceAudioCache::load_pcm_wav(bundle.join("full_grid_mix.wav"))
        .expect("decode full mix");
    let mut max_abs_error = 0.0_f64;
    let mut squared_error_sum = 0.0_f64;
    for index in 0..full_audio.interleaved_samples().len() {
        let error = f64::from(drums_audio.interleaved_samples()[index])
            + f64::from(music_audio.interleaved_samples()[index])
            + f64::from(bass_audio.interleaved_samples()[index])
            - f64::from(full_audio.interleaved_samples()[index]);
        max_abs_error = max_abs_error.max(error.abs());
        squared_error_sum += error * error;
    }
    let rms_error = (squared_error_sum / full_audio.interleaved_samples().len() as f64).sqrt();
    let artifact = |role: &str, source_role: &str, path: &str, origin: &str| {
        serde_json::json!({
            "role": role,
            "source_role": source_role,
            "path": path,
            "media_type": "audio/wav",
            "sha256": sha256_bytes(&fs::read(bundle.join(path)).expect("read handoff artifact")),
            "origin": origin,
        })
    };
    let proof = serde_json::json!({
        "schema": "riotbox.product_stem_handoff.v2",
        "schema_version": 2,
        "boundary": "feral-grid generated-support product stems",
        "pack_id": "feral-grid-demo",
        "material_status": "development_only",
        "release_ready": false,
        "musician_export_action_ready": false,
        "source_sha256": source_sha256,
        "normalized_manifest_sha256": "c".repeat(64),
        "grid": {
            "sample_rate_hz": sample_rate,
            "channel_count": channel_count,
            "bpm": 120.0,
            "beats_per_bar": 4,
            "bars": 1,
            "total_beats": 4,
            "frame_count": frame_count,
            "duration_seconds": 2.0,
        },
        "artifacts": [
            artifact("stem_drums", "product_stem_drums", "stems/stem_drums.wav", "source_derived"),
            artifact("stem_music", "product_stem_music", "stems/stem_music.wav", "source_derived"),
            artifact("stem_bass", "product_stem_bass", "stems/stem_bass.wav", "source_derived"),
            artifact("full_grid_mix", "full_grid_mix", "full_grid_mix.wav", "composite"),
        ],
        "reconstruction": {
            "schema": "riotbox.product_stem_reconstruction.v1",
            "rule": "pcm_sum_v1",
            "passed": true,
            "sample_rate_hz": sample_rate,
            "channel_count": channel_count,
            "frame_count": frame_count,
            "max_abs_error": max_abs_error,
            "rms_error": rms_error,
            "max_allowed_abs_error": 3.0 / 32768.0,
            "max_allowed_rms_error": 1.5 / 32768.0,
        },
        "renderer_status": {
            "mc202_source_expression": {
                "schema": "riotbox.mc202_source_expression_origin.v1",
                "pattern_origin": "source_derived",
                "bass_pressure_applied": true,
                "bass_pressure_reason": "mc202_source_grid_proof_renderer",
                "source_expression_render_plan_applied": true,
                "source_expression_role": "bass_pressure",
                "source_failure_fallback": false,
                "source_contour_origin": "source_derived_contour",
                "source_contour_applied": true,
                "source_contour_delta_rms": 0.01,
                "source_contour_min_required_delta_rms": 0.00025,
                "source_grid_hit_ratio": 1.0,
                "source_grid_min_required_hit_ratio": 0.5,
            },
            "limitations": [],
        },
    });
    let proof_path = bundle.join("product_stem_handoff_proof.json");
    fs::write(
        &proof_path,
        serde_json::to_vec_pretty(&proof).expect("serialize source-matched proof"),
    )
    .expect("write source-matched proof");
    proof_path
}

fn source_matched_role_file_stem(role: ExportArtifactRole) -> &'static str {
    match role {
        ExportArtifactRole::StemDrums => "stem_drums",
        ExportArtifactRole::StemMusic => "stem_music",
        ExportArtifactRole::StemBass => "stem_bass",
        _ => panic!("unexpected source-matched stem role"),
    }
}

fn write_product_export_proof(path: &Path, export_artifact: &str, export_hash: &str) {
    fs::write(
        path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "riotbox.product_export_reproducibility.v1",
            "schema_version": 1,
            "boundary": "feral-grid generated-support export",
            "pack_id": "feral-grid-demo",
            "export_role": "full_grid_mix",
            "export_artifact": export_artifact,
            "source_sha256": "hash-1",
            "export_sha256": export_hash,
            "normalized_manifest_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "audio_artifact_sha256": {
                "full_grid_mix": export_hash,
                "generated_support_mix": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "source_first_mix": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            }
        }))
        .expect("serialize proof"),
    )
    .expect("write proof");
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}
