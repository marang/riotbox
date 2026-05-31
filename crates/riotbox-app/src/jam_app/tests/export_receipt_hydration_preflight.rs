use super::product_export::{
    ExportReceiptArtifactPreflightError, preflight_export_receipt_artifacts,
};

fn state_with_export_receipt_path(
    dir: &Path,
    artifact_path: &str,
    proof_path: &str,
) -> (JamAppState, ExportReceiptState, PathBuf, PathBuf) {
    let session_path = dir.join("session.json");
    let graph_path = dir.join("source_graph.json");
    let mut graph = sample_graph();
    graph.source.path = dir.join("source.wav").to_string_lossy().into_owned();

    let mut session = sample_session(&graph);
    session.export_receipts.push(export_receipt(artifact_path, proof_path));
    let receipt = session.export_receipts[0].clone();

    save_source_graph_json(&graph_path, &graph).expect("save graph");
    save_session_json(&session_path, &session).expect("save session");

    let state =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
    let expected_artifact_path = expected_receipt_path(dir, artifact_path);
    let expected_proof_path = expected_receipt_path(dir, proof_path);

    (state, receipt, expected_artifact_path, expected_proof_path)
}

#[test]
fn export_receipt_hydration_preflight_accepts_existing_artifact_and_proof() {
    let dir = tempdir().expect("create temp dir");
    let export_dir = dir.path().join("exports");
    fs::create_dir(&export_dir).expect("create export dir");
    fs::write(export_dir.join("full_grid_mix.wav"), b"mix").expect("write export artifact");
    fs::write(export_dir.join("product_export_proof.json"), b"{}").expect("write proof");
    let (state, receipt, expected_artifact, expected_proof) = state_with_export_receipt_path(
        dir.path(),
        "exports/full_grid_mix.wav",
        "exports/product_export_proof.json",
    );
    assert_eq!(state.session.export_receipts.len(), 1);

    let (artifact, proof) = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect("export receipt artifacts exist");

    assert_eq!(artifact, expected_artifact);
    assert_eq!(proof, expected_proof);
}

#[test]
fn export_receipt_hydration_preflight_reports_missing_export_artifact() {
    let dir = tempdir().expect("create temp dir");
    let export_dir = dir.path().join("exports");
    fs::create_dir(&export_dir).expect("create export dir");
    fs::write(export_dir.join("product_export_proof.json"), b"{}").expect("write proof");
    let (state, receipt, expected_artifact, _) = state_with_export_receipt_path(
        dir.path(),
        "exports/missing_full_grid_mix.wav",
        "exports/product_export_proof.json",
    );
    assert_eq!(state.session.export_receipts.len(), 1);

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing export artifact should reject");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingExportArtifact {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            path: expected_artifact,
        }
    );
}

#[test]
fn export_receipt_hydration_preflight_reports_missing_proof_path_identity() {
    let dir = tempdir().expect("create temp dir");
    let export_dir = dir.path().join("exports");
    fs::create_dir(&export_dir).expect("create export dir");
    fs::write(export_dir.join("full_grid_mix.wav"), b"mix").expect("write export artifact");
    let (state, receipt, _, _) =
        state_with_export_receipt_path(dir.path(), "exports/full_grid_mix.wav", " ");
    assert_eq!(state.session.export_receipts.len(), 1);

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing proof path should reject");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingProofPath {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
        }
    );
}

fn export_receipt(artifact_path: &str, proof_path: &str) -> ExportReceiptState {
    ExportReceiptState {
        receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
        created_by_action: ActionId(4),
        created_at: 900,
        export_role: ProductExportRole::FullGridMix,
        export_boundary: ProductExportBoundary::FeralGridGeneratedSupport,
        artifact_path: artifact_path.into(),
        proof_path: proof_path.into(),
        manifest_path: None,
        export_hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        normalized_manifest_hash: "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            .into(),
        artifact_set: vec![ExportArtifactSetEntry::product_mix(
            artifact_path,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )],
        readiness_status: ExportReadinessStatus::Reproducible,
        unsupported_scopes: vec![
            UnsupportedExportScope::StemPackage,
            UnsupportedExportScope::LiveRecording,
            UnsupportedExportScope::DawExport,
            UnsupportedExportScope::HostAudioSoak,
        ],
    }
}

fn expected_receipt_path(dir: &Path, receipt_path: &str) -> PathBuf {
    let receipt_path = Path::new(receipt_path);
    if receipt_path.is_absolute() {
        receipt_path.to_path_buf()
    } else {
        dir.join(receipt_path)
    }
}
