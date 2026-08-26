use sha2::{Digest, Sha256};

#[test]
fn parse_args_accepts_complete_interactive_product_export_handoff() {
    let launch = parse_args([
        "--session".into(),
        "session.json".into(),
        "--product-export-proof".into(),
        "proof/product_export_proof.json".into(),
        "--product-export-destination".into(),
        "exports/mix".into(),
    ])
    .expect("parse product export handoff");

    let handoff = launch
        .mode
        .product_mix_export_handoff()
        .expect("product export handoff");
    assert_eq!(
        handoff.proof_path,
        PathBuf::from("proof/product_export_proof.json")
    );
    assert_eq!(handoff.destination_path, PathBuf::from("exports/mix"));
}

#[test]
fn parse_args_rejects_partial_or_noninteractive_product_export_handoff() {
    let missing_destination = parse_args([
        "--session".into(),
        "session.json".into(),
        "--product-export-proof".into(),
        "proof.json".into(),
    ])
    .expect_err("proof requires destination");
    assert!(missing_destination.contains("requires --product-export-destination"));

    let missing_proof = parse_args([
        "--session".into(),
        "session.json".into(),
        "--product-export-destination".into(),
        "exports".into(),
    ])
    .expect_err("destination requires proof");
    assert!(missing_proof.contains("requires --product-export-proof"));

    let noninteractive = parse_args([
        "--live-recording-readiness-report".into(),
        "--session".into(),
        "session.json".into(),
        "--product-export-proof".into(),
        "proof.json".into(),
        "--product-export-destination".into(),
        "exports".into(),
    ])
    .expect_err("handoff belongs only to interactive launch");
    assert!(noninteractive.contains("only for interactive source/session launches"));
}

#[test]
fn product_export_control_fails_closed_without_handoff_and_clears_queue() {
    let mut shell = product_export_shell("hash-1");

    execute_product_mix_export(&mut shell, None, 1_000);

    assert!(shell.status_message.contains("product mix export unavailable"));
    assert!(shell.app.queue.pending_actions().is_empty());
    assert!(shell.app.session.export_receipts.is_empty());
    assert_eq!(shell.app.queue.history().len(), 1);
    assert_eq!(
        shell.app.queue.history()[0].status,
        riotbox_core::action::ActionStatus::Rejected
    );
}

#[test]
fn product_export_control_commits_hash_identical_active_source_export() {
    let temp = tempfile::tempdir().expect("tempdir");
    let proof_dir = temp.path().join("proof");
    let destination = temp.path().join("export");
    let proof_path = write_product_export_handoff_fixture(&proof_dir, "hash-1");
    let source_artifact = proof_dir.join("full_grid_mix.wav");
    let handoff = ProductMixExportHandoff {
        proof_path,
        destination_path: destination.clone(),
    };
    let mut shell = product_export_shell("sha256:hash-1");

    execute_product_mix_export(&mut shell, Some(&handoff), 1_001);

    assert!(shell.status_message.contains("exported full_grid_mix"));
    assert!(shell.app.queue.pending_actions().is_empty());
    assert_eq!(shell.app.session.export_receipts.len(), 1);
    assert_eq!(
        sha256(&fs::read(source_artifact).expect("read source artifact")),
        sha256(
            &fs::read(destination.join("full_grid_mix.wav")).expect("read exported artifact")
        )
    );
    let action = shell
        .app
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportProductMix)
        .expect("committed export action");
    assert_eq!(action.status, riotbox_core::action::ActionStatus::Committed);

    execute_product_mix_export(&mut shell, Some(&handoff), 1_002);
    assert!(shell.app.queue.pending_actions().is_empty());
    assert_eq!(shell.app.session.export_receipts.len(), 2);
}

#[test]
fn product_export_control_rejects_stale_source_proof_without_files_or_receipt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let proof_dir = temp.path().join("proof");
    let destination = temp.path().join("export");
    let handoff = ProductMixExportHandoff {
        proof_path: write_product_export_handoff_fixture(&proof_dir, "old-source"),
        destination_path: destination.clone(),
    };
    let mut shell = product_export_shell("current-source");

    execute_product_mix_export(&mut shell, Some(&handoff), 1_003);

    assert!(shell.status_message.contains("source mismatch"));
    assert!(!destination.exists());
    assert!(shell.app.queue.pending_actions().is_empty());
    assert!(shell.app.session.export_receipts.is_empty());
    assert_eq!(
        shell.app.queue.history()[0].status,
        riotbox_core::action::ActionStatus::Rejected
    );
}

#[test]
fn product_export_control_rejects_destination_failure_without_receipt_or_pending_action() {
    let temp = tempfile::tempdir().expect("tempdir");
    let proof_dir = temp.path().join("proof");
    let destination = temp.path().join("destination-is-a-file");
    fs::write(&destination, b"occupied").expect("write destination blocker");
    let handoff = ProductMixExportHandoff {
        proof_path: write_product_export_handoff_fixture(&proof_dir, "hash-1"),
        destination_path: destination.clone(),
    };
    let mut shell = product_export_shell("hash-1");

    execute_product_mix_export(&mut shell, Some(&handoff), 1_004);

    assert!(shell.status_message.contains("product mix export failed"));
    assert_eq!(fs::read(destination).expect("destination blocker survives"), b"occupied");
    assert!(shell.app.queue.pending_actions().is_empty());
    assert!(shell.app.session.export_receipts.is_empty());
    assert_eq!(
        shell.app.queue.history()[0].status,
        riotbox_core::action::ActionStatus::Rejected
    );
}

#[test]
fn product_export_control_preserves_incomplete_existing_destination() {
    let temp = tempfile::tempdir().expect("tempdir");
    let proof_dir = temp.path().join("proof");
    let destination = temp.path().join("export");
    fs::create_dir_all(&destination).expect("create destination");
    let existing_artifact = destination.join("full_grid_mix.wav");
    fs::write(&existing_artifact, b"musician-owned-existing-file")
        .expect("write existing artifact");
    let handoff = ProductMixExportHandoff {
        proof_path: write_product_export_handoff_fixture(&proof_dir, "hash-1"),
        destination_path: destination,
    };
    let mut shell = product_export_shell("hash-1");

    execute_product_mix_export(&mut shell, Some(&handoff), 1_005);

    assert!(shell.status_message.contains("incomplete existing bundle"));
    assert_eq!(
        fs::read(existing_artifact).expect("existing artifact survives"),
        b"musician-owned-existing-file"
    );
    assert!(shell.app.queue.pending_actions().is_empty());
    assert!(shell.app.session.export_receipts.is_empty());
}

fn product_export_shell(source_hash: &str) -> JamShellState {
    let mut graph = ghost_capture_candidate_graph();
    graph.source.content_hash = source_hash.into();
    JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("product-export", "0.1.0", "2026-08-26T00:00:00Z"),
            Some(graph),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    )
}

fn write_product_export_handoff_fixture(proof_dir: &Path, source_hash: &str) -> PathBuf {
    fs::create_dir_all(proof_dir).expect("create proof dir");
    let artifact_path = proof_dir.join("full_grid_mix.wav");
    riotbox_audio::source_audio::write_interleaved_pcm16_wav(
        &artifact_path,
        1_000,
        1,
        &[0.0, 0.3, -0.3, 0.0],
    )
    .expect("write product mix");
    let artifact_hash = sha256(&fs::read(&artifact_path).expect("read product mix"));
    let proof_path = proof_dir.join("product_export_proof.json");
    fs::write(
        &proof_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema": "riotbox.product_export_reproducibility.v1",
            "schema_version": 1,
            "boundary": "feral-grid generated-support export",
            "pack_id": "feral-grid-demo",
            "export_role": "full_grid_mix",
            "export_artifact": "full_grid_mix.wav",
            "source_sha256": source_hash,
            "export_sha256": artifact_hash,
            "normalized_manifest_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "audio_artifact_sha256": {
                "full_grid_mix": artifact_hash,
            }
        }))
        .expect("serialize proof"),
    )
    .expect("write proof");
    proof_path
}

fn sha256(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}
