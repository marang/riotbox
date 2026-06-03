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

#[test]
fn export_receipt_hydration_preflight_accepts_extra_local_artifact_set_entry() {
    let dir = tempdir().expect("create temp dir");
    let export_dir = dir.path().join("exports");
    fs::create_dir(&export_dir).expect("create export dir");
    fs::write(export_dir.join("full_grid_mix.wav"), b"mix").expect("write export artifact");
    fs::write(export_dir.join("drums.wav"), b"drums").expect("write stem artifact");
    fs::write(export_dir.join("product_export_proof.json"), b"{}").expect("write proof");
    let (_, mut receipt, _, _) = state_with_export_receipt_path(
        dir.path(),
        "exports/full_grid_mix.wav",
        "exports/product_export_proof.json",
    );
    receipt.artifact_set.push(ExportArtifactSetEntry {
        role: ExportArtifactRole::StemDrums,
        location: ExportArtifactLocation::LocalPath {
            path: "exports/drums.wav".into(),
        },
        media_type: ExportArtifactMediaType::AudioWav,
        sha256: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
        normalized_manifest_hash: None,
        source_graph_ref: None,
            timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: None,
        audio_metrics: None,
        sample_rate_hz: None,
        channel_count: None,
        duration_ms: None,
    });

    preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect("all local artifact-set entries exist");
}

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
fn export_receipt_hydration_preflight_reports_missing_artifact_set_entry() {
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
    receipt.artifact_set.push(ExportArtifactSetEntry {
        role: ExportArtifactRole::StemDrums,
        location: ExportArtifactLocation::LocalPath {
            path: "exports/missing-drums.wav".into(),
        },
        media_type: ExportArtifactMediaType::AudioWav,
        sha256: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
        normalized_manifest_hash: None,
        source_graph_ref: None,
            timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: None,
        audio_metrics: None,
        sample_rate_hz: None,
        channel_count: None,
        duration_ms: None,
    });

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing artifact-set entry should reject");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingArtifactSetArtifact {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            role: ExportArtifactRole::StemDrums,
            path: dir.path().join("exports/missing-drums.wav"),
        }
    );
}

#[test]
fn export_receipt_hydration_preflight_treats_uri_artifact_set_entry_as_identity_only() {
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
    receipt.artifact_set.push(ExportArtifactSetEntry {
        role: ExportArtifactRole::StemDrums,
        location: ExportArtifactLocation::Uri {
            uri: "s3://riotbox/stems/drums.wav".into(),
        },
        media_type: ExportArtifactMediaType::AudioWav,
        sha256: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
        normalized_manifest_hash: None,
        source_graph_ref: None,
            timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: None,
        audio_metrics: None,
        sample_rate_hz: None,
        channel_count: None,
        duration_ms: None,
    });

    preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect("URI artifact-set entry is identity-only until fetch contract exists");
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
fn export_receipt_hydration_preflight_reports_missing_arrangement_file_after_placement_ready() {
    let dir = tempdir().expect("create temp dir");
    let mut receipt = arrangement_receipt(
        "exports/missing_arrangement_manifest.json",
        "exports/missing_arrangement_proof.json",
    );
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

    let error = preflight_export_receipt_artifacts(&receipt, Some(dir.path()))
        .expect_err("missing local arrangement file should reject after placement is ready");

    assert_eq!(
        error,
        ExportReceiptArtifactPreflightError::MissingExportArtifact {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            path: dir.path().join("exports/missing_arrangement_manifest.json"),
        }
    );
}

fn export_receipt(artifact_path: &str, proof_path: &str) -> ExportReceiptState {
    ExportReceiptState {
        receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
        created_by_action: ActionId(4),
        created_at: 900,
        export_scope: ExportScope::ProductMix,
        pack_id: riotbox_core::export_readiness::PRODUCT_EXPORT_PACK_ID.into(),
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
            Some("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into()),
        )],
        qa_gates: vec![ExportReceiptQaGateResult::product_export_reproducibility()],
        arrangement_placement_refs: Vec::new(),
        readiness_status: ExportReadinessStatus::Reproducible,
        unsupported_scopes: vec![
            UnsupportedExportScope::StemPackage,
            UnsupportedExportScope::LiveRecording,
            UnsupportedExportScope::DawExport,
            UnsupportedExportScope::HostAudioSoak,
        ],
    }
}

fn stem_package_receipt() -> ExportReceiptState {
    let mut receipt = export_receipt(
        "exports/stems/drums.wav",
        "exports/stem_package_proof.json",
    );
    receipt.export_scope = ExportScope::StemPackage;
    receipt.unsupported_scopes.clear();
    receipt.artifact_set = vec![
        stem_audio_artifact(
            ExportArtifactRole::StemDrums,
            "exports/stems/drums.wav",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ),
        stem_audio_artifact(
            ExportArtifactRole::StemBass,
            "exports/stems/bass.wav",
            "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        ),
        ExportArtifactSetEntry::export_manifest(
            "exports/stem_package_manifest.json",
            "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        ),
        ExportArtifactSetEntry::stem_package_proof(
            "exports/stem_package_proof.json",
            "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        ),
    ];
    let claimed_roles = vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass];
    receipt.qa_gates = all_required_stem_package_gates(&claimed_roles);
    receipt
}

fn arrangement_receipt(artifact_path: &str, proof_path: &str) -> ExportReceiptState {
    let mut receipt = export_receipt(artifact_path, proof_path);
    receipt.export_scope = ExportScope::DawSession;
    receipt.pack_id =
        riotbox_core::export_readiness::ARRANGEMENT_DAW_PLACEMENT_PACK_ID.into();
    receipt.export_role = ProductExportRole::ArrangementManifest;
    receipt.export_boundary = ProductExportBoundary::ArrangementDawPlacementContractV1;
    receipt.unsupported_scopes.clear();
    receipt.arrangement_placement_refs.clear();
    receipt
}

fn write_ready_stem_package_files(dir: &Path) {
    let stems_dir = dir.join("exports/stems");
    fs::create_dir_all(&stems_dir).expect("create stems dir");
    fs::write(stems_dir.join("drums.wav"), b"drums").expect("write drums stem");
    fs::write(stems_dir.join("bass.wav"), b"bass").expect("write bass stem");
    fs::write(dir.join("exports/stem_package_manifest.json"), b"{}").expect("write manifest");
    fs::write(dir.join("exports/stem_package_proof.json"), b"{}").expect("write proof");
}

fn stem_audio_artifact(
    role: ExportArtifactRole,
    path: impl Into<String>,
    sha256: impl Into<String>,
) -> ExportArtifactSetEntry {
    ExportArtifactSetEntry {
        role,
        location: ExportArtifactLocation::LocalPath { path: path.into() },
        media_type: ExportArtifactMediaType::AudioWav,
        sha256: sha256.into(),
        normalized_manifest_hash: None,
        source_graph_ref: None,
        timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: None,
        audio_metrics: None,
        sample_rate_hz: None,
        channel_count: None,
        duration_ms: None,
    }
}

fn all_required_stem_package_gates(
    claimed_roles: &[ExportArtifactRole],
) -> Vec<ExportReceiptQaGateResult> {
    [
        STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
        STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
        STEM_PACKAGE_NON_SILENCE_QA_GATE_ID,
        STEM_PACKAGE_LINEAGE_QA_GATE_ID,
        STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID,
    ]
    .into_iter()
    .map(|gate_id| ExportReceiptQaGateResult {
        gate_id: gate_id.into(),
        status: ExportReceiptQaGateStatus::Passed,
        artifact_roles: claimed_roles.to_vec(),
        summary: Some("stem package recovery fixture gate passed".into()),
    })
    .collect()
}

fn push_stem_package_json_entries(receipt: &mut ExportReceiptState) {
    receipt.artifact_set.push(ExportArtifactSetEntry::export_manifest(
        "exports/stem_package_manifest.json",
        "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    ));
    receipt.artifact_set.push(ExportArtifactSetEntry::stem_package_proof(
        "exports/stem_package_proof.json",
        "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
    ));
}

fn push_stem_package_json_uri_entries(receipt: &mut ExportReceiptState) {
    receipt.artifact_set.push(ExportArtifactSetEntry {
        role: ExportArtifactRole::ExportManifest,
        location: ExportArtifactLocation::Uri {
            uri: "s3://riotbox/export/stem_package_manifest.json".into(),
        },
        media_type: ExportArtifactMediaType::Json,
        sha256: "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into(),
        normalized_manifest_hash: None,
        source_graph_ref: None,
        timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: None,
        audio_metrics: None,
        sample_rate_hz: None,
        channel_count: None,
        duration_ms: None,
    });
    receipt.artifact_set.push(ExportArtifactSetEntry {
        role: ExportArtifactRole::ProductExportProof,
        location: ExportArtifactLocation::Uri {
            uri: "s3://riotbox/export/stem_package_proof.json".into(),
        },
        media_type: ExportArtifactMediaType::Json,
        sha256: "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into(),
        normalized_manifest_hash: None,
        source_graph_ref: None,
        timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: None,
        audio_metrics: None,
        sample_rate_hz: None,
        channel_count: None,
        duration_ms: None,
    });
}

fn expected_receipt_path(dir: &Path, receipt_path: &str) -> PathBuf {
    let receipt_path = Path::new(receipt_path);
    if receipt_path.is_absolute() {
        receipt_path.to_path_buf()
    } else {
        dir.join(receipt_path)
    }
}
