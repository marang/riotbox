use sha2::{Digest, Sha256};

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
            "source_sha256": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
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
