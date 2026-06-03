#[test]
fn export_receipt_hydration_preflight_accepts_stem_package_manifest_and_proof_entries() {
    let dir = tempdir().expect("create temp dir");
    let export_dir = dir.path().join("exports");
    fs::create_dir(&export_dir).expect("create export dir");
    fs::write(export_dir.join("full_grid_mix.wav"), b"mix").expect("write export artifact");
    fs::write(export_dir.join("product_export_proof.json"), b"{}").expect("write proof");
    fs::write(export_dir.join("stem_package_manifest.json"), b"{}").expect("write manifest");
    fs::write(export_dir.join("stem_package_proof.json"), b"{}")
        .expect("write stem package proof");
    let (_, mut receipt, _, _) = state_with_export_receipt_path(
        dir.path(),
        "exports/full_grid_mix.wav",
        "exports/product_export_proof.json",
    );
    push_stem_package_json_entries(&mut receipt);

    preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect("manifest and proof artifact-set entries exist");
}

#[test]
fn export_receipt_hydration_preflight_reports_missing_stem_package_manifest_entry() {
    let dir = tempdir().expect("create temp dir");
    let export_dir = dir.path().join("exports");
    fs::create_dir(&export_dir).expect("create export dir");
    fs::write(export_dir.join("full_grid_mix.wav"), b"mix").expect("write export artifact");
    fs::write(export_dir.join("product_export_proof.json"), b"{}").expect("write proof");
    fs::write(export_dir.join("stem_package_proof.json"), b"{}")
        .expect("write stem package proof");
    let (_, mut receipt, _, _) = state_with_export_receipt_path(
        dir.path(),
        "exports/full_grid_mix.wav",
        "exports/product_export_proof.json",
    );
    push_stem_package_json_entries(&mut receipt);

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing manifest artifact-set entry should reject");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingArtifactSetArtifact {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            role: ExportArtifactRole::ExportManifest,
            path: dir.path().join("exports/stem_package_manifest.json"),
        }
    );
}

#[test]
fn export_receipt_hydration_preflight_reports_missing_stem_package_proof_entry() {
    let dir = tempdir().expect("create temp dir");
    let export_dir = dir.path().join("exports");
    fs::create_dir(&export_dir).expect("create export dir");
    fs::write(export_dir.join("full_grid_mix.wav"), b"mix").expect("write export artifact");
    fs::write(export_dir.join("product_export_proof.json"), b"{}").expect("write proof");
    fs::write(export_dir.join("stem_package_manifest.json"), b"{}").expect("write manifest");
    let (_, mut receipt, _, _) = state_with_export_receipt_path(
        dir.path(),
        "exports/full_grid_mix.wav",
        "exports/product_export_proof.json",
    );
    push_stem_package_json_entries(&mut receipt);

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing proof artifact-set entry should reject");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingArtifactSetArtifact {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            role: ExportArtifactRole::ProductExportProof,
            path: dir.path().join("exports/stem_package_proof.json"),
        }
    );
}

#[test]
fn export_receipt_hydration_preflight_treats_stem_package_manifest_and_proof_uri_entries_as_identity_only() {
    let dir = tempdir().expect("create temp dir");
    let export_dir = dir.path().join("exports");
    fs::create_dir(&export_dir).expect("create export dir");
    fs::write(export_dir.join("full_grid_mix.wav"), b"mix").expect("write export artifact");
    fs::write(export_dir.join("product_export_proof.json"), b"{}").expect("write proof");
    let (_, mut receipt, _, _) = state_with_export_receipt_path(
        dir.path(),
        "exports/full_grid_mix.wav",
        "exports/product_export_proof.json",
    );
    push_stem_package_json_uri_entries(&mut receipt);

    preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect("URI manifest/proof entries are identity-only until fetch contract exists");
}

#[test]
fn export_receipt_hydration_preflight_accepts_ready_stem_package_contract() {
    let dir = tempdir().expect("create temp dir");
    write_ready_stem_package_files(dir.path());
    let receipt = stem_package_receipt();

    preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect("ready stem package receipt local files exist");
}

#[test]
fn export_receipt_hydration_preflight_reports_missing_claimed_stem_artifact() {
    let dir = tempdir().expect("create temp dir");
    write_ready_stem_package_files(dir.path());
    fs::remove_file(dir.path().join("exports/stems/bass.wav")).expect("remove bass stem");
    let receipt = stem_package_receipt();

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing claimed stem local file should reject");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingArtifactSetArtifact {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            role: ExportArtifactRole::StemBass,
            path: dir.path().join("exports/stems/bass.wav"),
        }
    );
}

#[test]
fn export_receipt_hydration_preflight_reports_missing_stem_package_manifest_file() {
    let dir = tempdir().expect("create temp dir");
    write_ready_stem_package_files(dir.path());
    fs::remove_file(dir.path().join("exports/stem_package_manifest.json"))
        .expect("remove manifest");
    let receipt = stem_package_receipt();

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing stem package manifest local file should reject");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingArtifactSetArtifact {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            role: ExportArtifactRole::ExportManifest,
            path: dir.path().join("exports/stem_package_manifest.json"),
        }
    );
}

#[test]
fn export_receipt_hydration_preflight_reports_missing_stem_package_proof_identity() {
    let dir = tempdir().expect("create temp dir");
    write_ready_stem_package_files(dir.path());
    let mut receipt = stem_package_receipt();
    receipt
        .artifact_set
        .retain(|artifact| artifact.role != ExportArtifactRole::ProductExportProof);

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing stem package proof artifact-set identity should reject");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            role: ExportArtifactRole::ProductExportProof,
        }
    );
}

#[test]
fn export_receipt_hydration_preflight_reports_blank_claimed_stem_identity() {
    let dir = tempdir().expect("create temp dir");
    write_ready_stem_package_files(dir.path());
    let mut receipt = stem_package_receipt();
    let bass = receipt
        .artifact_set
        .iter_mut()
        .find(|artifact| artifact.role == ExportArtifactRole::StemBass)
        .expect("bass artifact");
    bass.location = ExportArtifactLocation::LocalPath { path: " ".into() };

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("blank claimed stem local identity should reject");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            role: ExportArtifactRole::StemBass,
        }
    );
}

#[test]
fn export_receipt_hydration_preflight_reports_legacy_stem_package_shape_as_missing_identity() {
    let dir = tempdir().expect("create temp dir");
    let export_dir = dir.path().join("exports");
    fs::create_dir(&export_dir).expect("create export dir");
    fs::write(export_dir.join("full_grid_mix.wav"), b"mix").expect("write export artifact");
    fs::write(export_dir.join("product_export_proof.json"), b"{}").expect("write proof");
    let (_, mut receipt, _, _) = state_with_export_receipt_path(
        dir.path(),
        "exports/full_grid_mix.wav",
        "exports/product_export_proof.json",
    );
    receipt.export_scope = ExportScope::StemPackage;
    receipt.unsupported_scopes.clear();
    receipt.qa_gates.clear();

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("legacy stem package shape without claimed roles should reject");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            role: ExportArtifactRole::ExportManifest,
        }
    );
}
