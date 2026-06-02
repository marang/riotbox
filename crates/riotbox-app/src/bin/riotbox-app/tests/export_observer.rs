use sha2::{Digest, Sha256};

use riotbox_core::action::ActionStatus;

#[test]
fn observer_snapshot_reports_completed_product_export_lifecycle() {
    let temp = tempfile::tempdir().expect("tempdir");
    let proof_dir = temp.path().join("proof");
    let destination = temp.path().join("export");
    fs::create_dir_all(&proof_dir).expect("create proof dir");
    let artifact_path = proof_dir.join("full_grid_mix.wav");
    riotbox_audio::source_audio::write_interleaved_pcm16_wav(
        &artifact_path,
        1_000,
        1,
        &[0.0, 0.25, -0.25, 0.0],
    )
    .expect("write product artifact");
    let artifact_bytes = fs::read(&artifact_path).expect("read product artifact");
    let artifact_hash = sha256_bytes(&artifact_bytes);
    let proof_path = proof_dir.join("product_export_proof.json");
    write_product_export_proof(&proof_path, "full_grid_mix.wav", &artifact_hash);

    let mut session = SessionFile::new("observer-export", "0.1.0", "2026-05-31T00:00:00Z");
    session.source_graph_refs.push(SourceGraphRef {
        source_id: SourceId::from("src-1"),
        graph_version: SourceGraphVersion::V1,
        graph_hash: "graph-hash-1".into(),
        storage_mode: GraphStorageMode::External,
        embedded_graph: None,
        external_path: Some("source_graph.json".into()),
        provenance: GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["beat".into(), "section".into()],
            generated_at: "2026-06-02T18:00:00Z".into(),
            source_hash: "source-hash-1".into(),
            analysis_seed: 7,
            run_notes: Some("observer export lineage".into()),
        },
    });
    session.runtime_state.source_timing.confirmed_grid =
        Some(SourceTimingGridConfirmationState {
            source_id: SourceId::from("src-1"),
            hypothesis_id: Some("primary-grid".into()),
            confirmed_by_action: ActionId(8),
            confirmed_at: 880,
        });
    let mut state = JamAppState::from_parts(session, None, ActionQueue::new());
    let receipt = state
        .commit_product_mix_export_from_proof(&proof_path, &destination, 900)
        .expect("commit product export");
    let shell = JamShellState::new(state, ShellLaunchMode::Load);

    let snapshot = observer_snapshot(&shell);
    let export = &snapshot["export"];
    assert_eq!(export["present"], true);
    assert_eq!(export["receipt_count"], 1);
    let lifecycle = export["lifecycle"].as_array().expect("lifecycle array");
    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle[0]["stage"], "requested");
    assert_eq!(lifecycle[1]["stage"], "started");
    assert_eq!(lifecycle[2]["stage"], "completed");
    assert_eq!(lifecycle[2]["action_id"], receipt.created_by_action.0);
    assert_eq!(lifecycle[2]["receipt"]["receipt_id"], receipt.receipt_id.to_string());
    assert_eq!(lifecycle[2]["receipt"]["export_scope"], "product_mix");
    assert_eq!(lifecycle[2]["receipt"]["pack_id"], "feral-grid-demo");
    assert_eq!(lifecycle[2]["receipt"]["export_role"], "full_grid_mix");
    assert_eq!(
        lifecycle[2]["receipt"]["export_boundary"],
        "feral_grid_generated_support"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_path"],
        destination
            .join("full_grid_mix.wav")
            .to_string_lossy()
            .into_owned()
    );
    assert_eq!(lifecycle[2]["receipt"]["export_hash"], artifact_hash);
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["role"],
        "full_grid_mix"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["location"]["kind"],
        "local_path"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["location"]["path"],
        destination
            .join("full_grid_mix.wav")
            .to_string_lossy()
            .into_owned()
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["media_type"],
        "audio_wav"
    );
    assert_eq!(lifecycle[2]["receipt"]["artifact_set"][0]["sha256"], artifact_hash);
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["audio_metrics"]["total_frame_count"],
        4
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["audio_metrics"]["silent_frame_count"],
        2
    );
    assert!(
        lifecycle[2]["receipt"]["artifact_set"][0]["audio_metrics"]["peak_amplitude_micros"]
            .as_u64()
            .expect("peak amplitude")
            > 200_000
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["sample_rate_hz"],
        1_000
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["channel_count"],
        1
    );
    assert_eq!(lifecycle[2]["receipt"]["artifact_set"][0]["duration_ms"], 4);
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["normalized_manifest_hash"],
        "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["source_graph_ref"]["source_id"],
        "src-1"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["source_graph_ref"]["graph_version"],
        "V1"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["source_graph_ref"]["graph_hash"],
        "graph-hash-1"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["timing_grid_ref"]["source_id"],
        "src-1"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["timing_grid_ref"]["hypothesis_id"],
        "primary-grid"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["timing_grid_ref"]["confirmed_by_action"],
        8
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][0]["timing_grid_ref"]["confirmed_at"],
        880
    );
    assert_eq!(
        lifecycle[2]["receipt"]["qa_gates"][0]["gate_id"],
        "product_export_reproducibility_smoke"
    );
    assert_eq!(lifecycle[2]["receipt"]["qa_gates"][0]["status"], "passed");
    assert_eq!(
        lifecycle[2]["receipt"]["qa_gates"][0]["artifact_roles"][0],
        "full_grid_mix"
    );
    assert_eq!(lifecycle[2]["receipt"]["readiness_status"], "reproducible");
    assert_eq!(
        lifecycle[2]["receipt"]["unsupported_scopes"]
            .as_array()
            .expect("unsupported scopes")
            .len(),
        4
    );
    assert_eq!(
        lifecycle[2]["receipt"]["unsupported_scopes"][0],
        "stem_package"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["unsupported_scope_labels"][0],
        "stem package export"
    );
}

#[test]
fn observer_snapshot_reports_failed_product_export_lifecycle() {
    let temp = tempfile::tempdir().expect("tempdir");
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

    let mut state = JamAppState::from_parts(
        SessionFile::new("observer-export-failed", "0.1.0", "2026-05-31T00:00:00Z"),
        None,
        ActionQueue::new(),
    );
    state
        .commit_product_mix_export_from_proof(&proof_path, &destination, 901)
        .expect_err("hash mismatch rejects export");
    let rejected = state
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportProductMix)
        .expect("rejected export action");
    assert_eq!(rejected.status, ActionStatus::Rejected);

    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");
    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle[0]["stage"], "requested");
    assert_eq!(lifecycle[1]["stage"], "started");
    assert_eq!(lifecycle[2]["stage"], "failed");
    assert_eq!(lifecycle[2]["receipt"], serde_json::Value::Null);
    assert!(
        lifecycle[2]["failure_reason"]
            .as_str()
            .expect("failure reason")
            .contains("export artifact hash mismatch")
    );
}

#[test]
fn observer_snapshot_reports_requested_product_export_lifecycle() {
    let mut state = JamAppState::from_parts(
        SessionFile::new("observer-export-requested", "0.1.0", "2026-05-31T00:00:00Z"),
        None,
        ActionQueue::new(),
    );
    state.queue_product_mix_export(902, Some("exports".into()));
    let shell = JamShellState::new(state, ShellLaunchMode::Load);

    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");
    assert_eq!(lifecycle.len(), 1);
    assert_eq!(lifecycle[0]["stage"], "requested");
    assert_eq!(lifecycle[0]["timestamp_ms"], 902);
    assert_eq!(lifecycle[0]["command"], "export.product_mix");
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
