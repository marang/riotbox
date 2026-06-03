#[test]
fn export_receipt_hydration_preflight_reports_missing_arrangement_placement_before_files() {
    let dir = tempdir().expect("create temp dir");
    let receipt = arrangement_receipt(
        "exports/missing_arrangement_manifest.json",
        "exports/missing_arrangement_proof.json",
    );

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing placement evidence should reject before file availability");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::ArrangementPlacementBlocked {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            blockers: vec![ArrangementExportPlacementReadinessBlocker::MissingPlacementRefs],
        }
    );
}

#[test]
fn export_receipt_hydration_preflight_reports_missing_daw_tempo_map_after_placement_ready() {
    let dir = tempdir().expect("create temp dir");
    let mut receipt = arrangement_receipt(
        "exports/missing_arrangement_manifest.json",
        "exports/missing_arrangement_proof.json",
    );
    attach_ready_arrangement_placement(&mut receipt);

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing tempo-map evidence should reject before file availability");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::DawTempoMapBlocked {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            blockers: vec![DawTempoMapReadinessBlocker::MissingTempoMapRef],
        }
    );
}

#[test]
fn export_receipt_hydration_preflight_reports_missing_arrangement_file_after_daw_contract_ready() {
    let dir = tempdir().expect("create temp dir");
    let mut receipt = arrangement_receipt(
        "exports/missing_arrangement_manifest.json",
        "exports/missing_arrangement_proof.json",
    );
    attach_ready_arrangement_placement(&mut receipt);
    attach_ready_daw_tempo_map(&mut receipt);

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing local arrangement file should reject after DAW contract is ready");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingExportArtifact {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            path: dir.path().join("exports/missing_arrangement_manifest.json"),
        }
    );
}

fn attach_ready_arrangement_placement(receipt: &mut ExportReceiptState) {
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
}

fn attach_ready_daw_tempo_map(receipt: &mut ExportReceiptState) {
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
