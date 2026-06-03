use riotbox_core::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportRole,
    },
    session::{
        ExportArrangementPlacementRef, ExportArtifactRole, ExportArtifactSetEntry,
        ExportDawTempoMapRef, ExportReceiptQaGateResult, ExportReceiptState,
    },
};

#[test]
fn daw_export_report_surface_gate_tracks_host_import_proof_gate_without_enabling_export() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let tempo_map_path = temp.path().join("exports/tempo_map.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&tempo_map_path, "{}").expect("write tempo map");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-report-host-import-gated",
        "riotbox-test",
        "2026-06-03T18:25:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    attach_ready_artifacts(&mut receipt);
    receipt.qa_gates = vec![
        ready_json_package_gate(),
        ExportReceiptQaGateResult::daw_session_host_import_proof(
            false,
            &["host_import_runner_missing".into()],
        ),
    ];
    session.export_receipts.push(receipt.clone());
    save_session_json(&session_path, &session).expect("save failed host-import session");
    let report_launch = daw_report_launch(session_path.clone());

    let failed_summary =
        daw_export_readiness_report_summary(&report_launch).expect("failed report summary");

    assert_eq!(
        failed_summary["daw_session_surface_gate"]["blockers"],
        json!([
            "developer_proof_only",
            "daw_writer_missing",
            "daw_host_import_proof_missing",
            "audible_output_proof_missing"
        ])
    );

    let host_gate = receipt
        .qa_gates
        .iter_mut()
        .find(|gate| gate.gate_id == riotbox_core::session::DAW_SESSION_HOST_IMPORT_QA_GATE_ID)
        .expect("host import gate");
    *host_gate = ExportReceiptQaGateResult::daw_session_host_import_proof(true, &[]);
    session.export_receipts[0] = receipt;
    save_session_json(&session_path, &session).expect("save passed host-import session");

    let passed_summary =
        daw_export_readiness_report_summary(&report_launch).expect("passed report summary");

    assert_eq!(
        passed_summary["daw_session_surface_gate"]["blockers"],
        json!([
            "developer_proof_only",
            "daw_writer_missing",
            "audible_output_proof_missing"
        ])
    );
    assert_eq!(passed_summary["daw_session_surface_gate"]["runnable"], false);
}

#[test]
fn daw_export_report_surface_gate_tracks_writer_proof_without_enabling_export() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let tempo_map_path = temp.path().join("exports/tempo_map.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&tempo_map_path, "{}").expect("write tempo map");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-report-writer-gated",
        "riotbox-test",
        "2026-06-03T20:15:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    attach_ready_artifacts(&mut receipt);
    receipt.qa_gates = vec![ready_json_package_gate()];
    session.export_receipts.push(receipt.clone());
    save_session_json(&session_path, &session).expect("save missing writer session");
    let report_launch = daw_report_launch(session_path.clone());

    let missing_summary =
        daw_export_readiness_report_summary(&report_launch).expect("missing writer summary");

    assert_eq!(
        missing_summary["daw_session_surface_gate"]["blockers"],
        json!([
            "developer_proof_only",
            "daw_writer_missing",
            "daw_host_import_proof_missing",
            "audible_output_proof_missing"
        ])
    );

    receipt.qa_gates.push(ExportReceiptQaGateResult::daw_session_writer_proof(
        true,
        &[],
        vec![ExportArtifactRole::DawSessionWriterProof],
    ));
    session.export_receipts[0] = receipt;
    save_session_json(&session_path, &session).expect("save writer-proof session");

    let passed_summary =
        daw_export_readiness_report_summary(&report_launch).expect("passed writer summary");

    assert_eq!(
        passed_summary["daw_session_surface_gate"]["blockers"],
        json!([
            "developer_proof_only",
            "daw_host_import_proof_missing",
            "audible_output_proof_missing"
        ])
    );
    assert_eq!(passed_summary["daw_session_surface_gate"]["runnable"], false);
}

#[test]
fn daw_export_report_surface_gate_tracks_audible_output_proof_gate_without_enabling_export() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let tempo_map_path = temp.path().join("exports/tempo_map.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&tempo_map_path, "{}").expect("write tempo map");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-report-audible-output-gated",
        "riotbox-test",
        "2026-06-03T20:05:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    attach_ready_artifacts(&mut receipt);
    receipt.qa_gates = vec![
        ready_json_package_gate(),
        ExportReceiptQaGateResult::daw_session_host_import_proof(true, &[]),
        ExportReceiptQaGateResult::daw_session_audible_output_proof(
            false,
            &["host_audio_capture_missing".into()],
        ),
    ];
    session.export_receipts.push(receipt.clone());
    save_session_json(&session_path, &session).expect("save failed audible-output session");
    let report_launch = daw_report_launch(session_path.clone());

    let failed_summary =
        daw_export_readiness_report_summary(&report_launch).expect("failed report summary");

    assert_eq!(
        failed_summary["daw_session_surface_gate"]["blockers"],
        json!([
            "developer_proof_only",
            "daw_writer_missing",
            "audible_output_proof_missing"
        ])
    );

    let audible_gate = receipt
        .qa_gates
        .iter_mut()
        .find(|gate| gate.gate_id == riotbox_core::session::DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID)
        .expect("audible output gate");
    *audible_gate = ExportReceiptQaGateResult::daw_session_audible_output_proof(true, &[]);
    session.export_receipts[0] = receipt;
    save_session_json(&session_path, &session).expect("save passed audible-output session");

    let passed_summary =
        daw_export_readiness_report_summary(&report_launch).expect("passed report summary");

    assert_eq!(
        passed_summary["daw_session_surface_gate"]["blockers"],
        json!(["developer_proof_only", "daw_writer_missing"])
    );
    assert_eq!(passed_summary["daw_session_surface_gate"]["runnable"], false);
}

fn daw_report_launch(session_path: PathBuf) -> AppLaunch {
    AppLaunch {
        mode: LaunchMode::DawExportReadinessReport { session_path },
        observer_path: None,
    }
}

fn ready_json_package_gate() -> ExportReceiptQaGateResult {
    ExportReceiptQaGateResult::daw_session_json_package_integrity(
        true,
        &[],
        vec![
            ExportArtifactRole::ExportManifest,
            ExportArtifactRole::DawSessionTempoMap,
            ExportArtifactRole::ProductExportProof,
        ],
    )
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

fn attach_ready_artifacts(receipt: &mut ExportReceiptState) {
    receipt.artifact_set = vec![
        ExportArtifactSetEntry::export_manifest(
            "exports/arrangement_manifest.json",
            "manifest-sha",
        ),
        ExportArtifactSetEntry::daw_session_tempo_map(
            "exports/tempo_map.json",
            "tempo-map-sha",
        ),
        ExportArtifactSetEntry::product_export_proof("exports/proof.json", "proof-sha"),
    ];
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
