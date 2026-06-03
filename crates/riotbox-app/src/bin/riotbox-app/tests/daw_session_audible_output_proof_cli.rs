use riotbox_core::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportRole,
    },
    persistence::load_session_json,
    session::{
        DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID, ExportArrangementPlacementRef,
        ExportArtifactRole, ExportArtifactSetEntry, ExportDawTempoMapRef,
        ExportReceiptQaGateResult, ExportReceiptQaGateStatus, ExportReceiptState,
    },
};

#[test]
fn daw_session_audible_output_proof_apply_attaches_gate_without_enabling_export() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let proof_path = temp.path().join("audible_output_proof.json");
    let mut session = SessionFile::new(
        "daw-audible-output-proof-apply",
        "riotbox-test",
        "2026-06-03T20:40:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save session");
    let launch = AppLaunch {
        mode: LaunchMode::DawSessionAudibleOutputProofApply {
            session_path: session_path.clone(),
            proof_path: proof_path.clone(),
        },
        observer_path: None,
    };

    let missing_summary =
        daw_session_audible_output_proof_apply_summary(&launch).expect("missing proof summary");

    assert_eq!(missing_summary["status"], "blocked");
    assert_eq!(missing_summary["mutates_session"], true);
    assert_eq!(
        missing_summary["readiness_blockers"],
        json!([
            "daw_host_import_proof_missing",
            "daw_writer_proof_missing",
            "missing_proof_file"
        ])
    );
    assert_eq!(
        missing_summary["daw_session_surface_gate"]["blockers"],
        json!([
            "json_package_evidence_missing",
            "developer_proof_only",
            "daw_writer_missing",
            "daw_host_import_proof_missing",
            "audible_output_proof_missing"
        ])
    );

    fs::write(
        &proof_path,
        json!({
            "schema_id": riotbox_app::jam_app::DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_ID,
            "schema_version": riotbox_app::jam_app::DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_VERSION,
            "package_dir": "exports/daw-package/daw_session",
            "audible": false,
            "blockers": ["host_audio_capture_missing"]
        })
        .to_string(),
    )
    .expect("write blocked proof");
    let failed_summary =
        daw_session_audible_output_proof_apply_summary(&launch).expect("failed proof summary");

    assert_eq!(failed_summary["status"], "blocked");
    assert_eq!(
        failed_summary["readiness_blockers"],
        json!([
            "audible_output_not_proven",
            "daw_host_import_proof_missing",
            "daw_writer_proof_missing",
            "host_audio_capture_missing",
            "proof_blockers_present",
        ])
    );
    assert_eq!(
        failed_summary["receipt"]["daw_audible_output_gate"]["status"],
        "failed"
    );

    fs::write(
        &proof_path,
        json!({
            "schema_id": riotbox_app::jam_app::DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_ID,
            "schema_version": riotbox_app::jam_app::DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_VERSION,
            "package_dir": "exports/daw-package/daw_session",
            "audible": true,
            "blockers": []
        })
        .to_string(),
    )
    .expect("write passed proof");
    let passed_summary =
        daw_session_audible_output_proof_apply_summary(&launch).expect("out-of-order proof summary");

    assert_eq!(passed_summary["mode"], "daw_session_audible_output_proof_apply");
    assert_eq!(passed_summary["status"], "blocked");
    assert_eq!(passed_summary["ready"], false);
    assert_eq!(passed_summary["writes_files"], false);
    assert_eq!(passed_summary["mutates_session"], true);
    assert_eq!(passed_summary["observer_events"], false);
    assert_eq!(
        passed_summary["readiness_blockers"],
        json!([
            "daw_host_import_proof_missing",
            "daw_writer_proof_missing"
        ])
    );
    assert_eq!(
        passed_summary["receipt"]["daw_audible_output_gate"]["gate_id"],
        DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID
    );
    assert_eq!(
        passed_summary["receipt"]["daw_audible_output_gate"]["status"],
        "failed"
    );
    assert_eq!(
        passed_summary["daw_session_surface_gate"]["blockers"],
        json!([
            "json_package_evidence_missing",
            "developer_proof_only",
            "daw_writer_missing",
            "daw_host_import_proof_missing",
            "audible_output_proof_missing"
        ])
    );
    assert_eq!(passed_summary["daw_session_surface_gate"]["runnable"], false);

    let mut saved = load_session_json(&session_path).expect("reload applied session");
    let saved_receipt = saved.export_receipts.last_mut().expect("saved receipt");
    saved_receipt
        .artifact_set
        .push(ExportArtifactSetEntry::daw_session_tempo_map(
            "exports/tempo_map.json",
            "tempo-map-sha",
        ));
    saved_receipt
        .artifact_set
        .push(ExportArtifactSetEntry::daw_session_writer_proof(
            "exports/daw_session_writer/writer_proof.json",
            "writer-proof-sha",
        ));
    saved_receipt
        .qa_gates
        .push(ExportReceiptQaGateResult::daw_session_json_package_integrity(
            true,
            &[],
            vec![
                ExportArtifactRole::ExportManifest,
                ExportArtifactRole::DawSessionTempoMap,
                ExportArtifactRole::ProductExportProof,
            ],
        ));
    saved_receipt
        .qa_gates
        .push(ExportReceiptQaGateResult::daw_session_writer_proof(
            true,
            &[],
            vec![ExportArtifactRole::DawSessionWriterProof],
        ));
    saved_receipt
        .qa_gates
        .push(ExportReceiptQaGateResult::daw_session_host_import_proof(
            true,
            &[],
        ));
    save_session_json(&session_path, &saved).expect("save ordered proof prerequisites");

    let ordered_summary =
        daw_session_audible_output_proof_apply_summary(&launch).expect("ordered proof summary");

    assert_eq!(ordered_summary["status"], "ready");
    assert_eq!(ordered_summary["ready"], true);
    assert_eq!(ordered_summary["readiness_blockers"], json!([]));
    assert_eq!(
        ordered_summary["receipt"]["daw_audible_output_gate"]["status"],
        "passed"
    );
    assert_eq!(
        ordered_summary["daw_session_surface_gate"]["blockers"],
        json!(["developer_proof_only"])
    );

    let saved = load_session_json(&session_path).expect("reload ordered session");
    let saved_receipt = saved.export_receipts.last().expect("saved receipt");
    let gate = saved_receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID)
        .expect("audible output gate");
    assert_eq!(gate.status, ExportReceiptQaGateStatus::Passed);
}

#[test]
fn daw_session_audible_output_proof_apply_blocks_without_daw_session_receipt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let proof_path = temp.path().join("audible_output_proof.json");
    save_session_json(
        &session_path,
        &SessionFile::new(
            "daw-audible-output-proof-apply-no-receipt",
            "riotbox-test",
            "2026-06-03T20:45:00Z",
        ),
    )
    .expect("save session");
    fs::write(
        &proof_path,
        json!({
            "schema_id": riotbox_app::jam_app::DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_ID,
            "schema_version": riotbox_app::jam_app::DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_VERSION,
            "package_dir": "exports/daw-package/daw_session",
            "audible": true,
            "blockers": []
        })
        .to_string(),
    )
    .expect("write proof");
    let launch = AppLaunch {
        mode: LaunchMode::DawSessionAudibleOutputProofApply {
            session_path,
            proof_path,
        },
        observer_path: None,
    };

    let summary =
        daw_session_audible_output_proof_apply_summary(&launch).expect("no receipt summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["mutates_session"], false);
    assert_eq!(summary["readiness_blockers"], json!(["no_daw_session_receipt"]));
    assert_eq!(summary["receipt"], serde_json::Value::Null);
}

fn daw_receipt(artifact_path: &str, proof_path: &str) -> ExportReceiptState {
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::DawSession,
        boundary: ProductExportBoundary::ArrangementDawPlacementContractV1,
        pack_id: ARRANGEMENT_DAW_PLACEMENT_PACK_ID.into(),
        export_role: ProductExportRole::ArrangementManifest,
        export_artifact: artifact_path.into(),
        source_sha256: "source-sha".into(),
        export_sha256: "manifest-sha".into(),
        normalized_manifest_sha256: "normalized-manifest-sha".into(),
        unsupported_scopes: Vec::new(),
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        ActionId(42),
        91_000,
        &contract,
        artifact_path,
        proof_path,
        Some(artifact_path.into()),
    );
    receipt.artifact_set = vec![
        ExportArtifactSetEntry::export_manifest(artifact_path, "manifest-sha"),
        ExportArtifactSetEntry::product_export_proof(proof_path, "proof-sha"),
    ];
    receipt
}

fn attach_ready_daw_refs(receipt: &mut ExportReceiptState) {
    receipt
        .arrangement_placement_refs
        .push(ExportArrangementPlacementRef::scene_range(
            "scene-a",
            Some(SourceId::from("src-1")),
            1,
            4,
            0,
            16,
        ));
    receipt.daw_tempo_map_ref = Some(ExportDawTempoMapRef::confirmed_grid(
        "src-1",
        Some("primary-grid".into()),
        ActionId(8),
        880,
        0,
        16,
        128_000_000,
    ));
}
