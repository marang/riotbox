use sha2::{Digest, Sha256};

#[test]
fn product_mix_export_writes_artifact_and_receipt_after_proof_success() {
    let temp = tempdir().expect("tempdir");
    let proof_dir = temp.path().join("proof");
    let destination = temp.path().join("export");
    fs::create_dir_all(&proof_dir).expect("create proof dir");
    let artifact_path = proof_dir.join("full_grid_mix.wav");
    let artifact_bytes = b"riotbox full grid product mix";
    fs::write(&artifact_path, artifact_bytes).expect("write product artifact");
    let artifact_hash = sha256_bytes(artifact_bytes);
    let proof_path = proof_dir.join("product_export_proof.json");
    write_product_export_proof(&proof_path, "full_grid_mix.wav", &artifact_hash);

    let graph = sample_graph();
    let session = sample_session(&graph);
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
    assert_eq!(
        receipt.artifact_set,
        vec![ExportArtifactSetEntry::product_mix(
            destination.join("full_grid_mix.wav").to_string_lossy().into_owned(),
            artifact_hash.clone(),
            Some("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into()),
        )]
    );
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
