use riotbox_core::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, ExportReadinessStatus, ExportScope,
        ProductExportBoundary, ProductExportRole,
    },
    ids::ExportReceiptId,
    session::{
        ExportArrangementPlacementRef, ExportArtifactRole, ExportArtifactSetEntry,
        ExportReceiptQaGateResult, ExportReceiptState,
    },
    session::ExportDawTempoMapRef,
};

#[test]
fn observer_snapshot_projects_blocked_arrangement_placement_from_receipt() {
    let mut state = JamAppState::from_parts(
        SessionFile::new(
            "observer-arrangement-placement",
            "0.1.0",
            "2026-06-03T00:00:00Z",
        ),
        None,
        ActionQueue::new(),
    );
    state.queue_product_mix_export(904, Some("exports/arrangement".into()));
    let action_id = state
        .queue
        .pending_actions()
        .into_iter()
        .find(|action| action.command == ActionCommand::ExportProductMix)
        .expect("pending export action")
        .id;
    assert!(state.queue.reject(
        action_id,
        "arrangement placement contract fixture only; no DAW files written"
    ));
    let mut receipt = product_mix_receipt_for_arrangement_observer(action_id, 904);
    receipt.export_scope = ExportScope::DawSession;
    receipt.pack_id = ARRANGEMENT_DAW_PLACEMENT_PACK_ID.into();
    receipt.export_role = ProductExportRole::ArrangementManifest;
    receipt.export_boundary = ProductExportBoundary::ArrangementDawPlacementContractV1;
    receipt.unsupported_scopes.clear();
    state.session.export_receipts.push(receipt);

    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");
    let receipt = &lifecycle[2]["receipt"];

    assert_eq!(receipt["export_scope"], "daw_session");
    assert_eq!(
        receipt["arrangement_placement_readiness"]["status"],
        "blocked"
    );
    assert_eq!(
        receipt["arrangement_placement_readiness"]["blockers"],
        serde_json::json!(["missing_placement_refs"])
    );
    assert_eq!(
        receipt["arrangement_placement_readiness"]["blocker_labels"][0],
        "arrangement scene placement evidence is missing"
    );
    assert_eq!(receipt["arrangement_placement_refs"], serde_json::json!([]));
    assert_eq!(receipt["daw_tempo_map_readiness"]["status"], "blocked");
    assert_eq!(
        receipt["daw_tempo_map_readiness"]["blockers"],
        serde_json::json!(["missing_tempo_map_ref"])
    );
    assert_eq!(
        receipt["daw_tempo_map_readiness"]["blocker_labels"][0],
        "DAW tempo-map evidence is missing"
    );
    assert_eq!(receipt["daw_tempo_map_ref"], serde_json::Value::Null);
}

#[test]
fn observer_snapshot_projects_ready_arrangement_placement_refs_from_receipt() {
    let mut state = JamAppState::from_parts(
        SessionFile::new(
            "observer-arrangement-placement-ready",
            "0.1.0",
            "2026-06-03T00:00:00Z",
        ),
        None,
        ActionQueue::new(),
    );
    state.queue_product_mix_export(905, Some("exports/arrangement".into()));
    let action_id = state
        .queue
        .pending_actions()
        .into_iter()
        .find(|action| action.command == ActionCommand::ExportProductMix)
        .expect("pending export action")
        .id;
    assert!(state.queue.reject(
        action_id,
        "arrangement placement contract fixture only; no DAW files written"
    ));
    let mut receipt = product_mix_receipt_for_arrangement_observer(action_id, 905);
    receipt.export_scope = ExportScope::DawSession;
    receipt.pack_id = ARRANGEMENT_DAW_PLACEMENT_PACK_ID.into();
    receipt.export_role = ProductExportRole::ArrangementManifest;
    receipt.export_boundary = ProductExportBoundary::ArrangementDawPlacementContractV1;
    receipt.unsupported_scopes.clear();
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
    receipt.artifact_set = vec![
        ExportArtifactSetEntry::export_manifest(
            "daw-out/daw_session/arrangement_manifest.json",
            "manifest-sha",
        ),
        ExportArtifactSetEntry::daw_session_tempo_map(
            "daw-out/daw_session/tempo_map.json",
            "tempo-map-sha",
        ),
        ExportArtifactSetEntry::product_export_proof(
            "daw-out/daw_session/daw_session_proof.json",
            "proof-sha",
        ),
    ];
    receipt.qa_gates = vec![ExportReceiptQaGateResult::daw_session_json_package_integrity(
        true,
        &[],
        vec![
            ExportArtifactRole::ExportManifest,
            ExportArtifactRole::DawSessionTempoMap,
            ExportArtifactRole::ProductExportProof,
        ],
    )];
    state.session.export_receipts.push(receipt);

    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");
    let receipt = &lifecycle[2]["receipt"];

    assert_eq!(
        receipt["arrangement_placement_readiness"]["status"],
        "ready"
    );
    assert_eq!(receipt["arrangement_placement_refs"][0]["scene_id"], "scene-a");
    assert_eq!(receipt["arrangement_placement_refs"][0]["start_bar"], 1);
    assert_eq!(receipt["arrangement_placement_refs"][0]["end_beat"], 16);
    assert_eq!(receipt["daw_tempo_map_readiness"]["status"], "ready");
    assert_eq!(receipt["daw_tempo_map_ref"]["source_id"], "src-1");
    assert_eq!(receipt["daw_tempo_map_ref"]["bpm_micros"], 128_000_000);
    assert_eq!(
        receipt["artifact_set"][1]["role"],
        "daw_session_tempo_map"
    );
    assert_eq!(
        receipt["qa_gates"][0]["gate_id"],
        "daw_session_json_package_integrity"
    );
    assert_eq!(receipt["qa_gates"][0]["status"], "passed");
    assert_eq!(
        snapshot["export"]["daw_session_surface_gate"]["status"],
        "disabled"
    );
    assert_eq!(
        snapshot["export"]["daw_session_surface_gate"]["blockers"],
        serde_json::json!([
            "developer_proof_only",
            "daw_writer_missing",
            "daw_host_import_proof_missing",
            "audible_output_proof_missing"
        ])
    );
    assert_eq!(
        snapshot["export"]["daw_session_receipt"]["receipt_id"],
        receipt["receipt_id"]
    );
    assert_eq!(
        snapshot["export"]["daw_session_receipt"]["qa_gates"][0]["gate_id"],
        "daw_session_json_package_integrity"
    );
}

#[test]
fn observer_snapshot_projects_daw_session_receipt_summary_without_fake_lifecycle() {
    let mut state = JamAppState::from_parts(
        SessionFile::new(
            "observer-daw-receipt-summary",
            "0.1.0",
            "2026-06-03T00:00:00Z",
        ),
        None,
        ActionQueue::new(),
    );
    let mut receipt = product_mix_receipt_for_arrangement_observer(ActionId(42), 906);
    receipt.export_scope = ExportScope::DawSession;
    receipt.pack_id = ARRANGEMENT_DAW_PLACEMENT_PACK_ID.into();
    receipt.export_role = ProductExportRole::ArrangementManifest;
    receipt.export_boundary = ProductExportBoundary::ArrangementDawPlacementContractV1;
    receipt.unsupported_scopes.clear();
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
    receipt.artifact_set = vec![
        ExportArtifactSetEntry::export_manifest(
            "daw-out/daw_session/arrangement_manifest.json",
            "manifest-sha",
        ),
        ExportArtifactSetEntry::daw_session_tempo_map(
            "daw-out/daw_session/tempo_map.json",
            "tempo-map-sha",
        ),
        ExportArtifactSetEntry::product_export_proof(
            "daw-out/daw_session/daw_session_proof.json",
            "proof-sha",
        ),
    ];
    receipt.qa_gates = vec![ExportReceiptQaGateResult::daw_session_json_package_integrity(
        true,
        &[],
        vec![
            ExportArtifactRole::ExportManifest,
            ExportArtifactRole::DawSessionTempoMap,
            ExportArtifactRole::ProductExportProof,
        ],
    )];
    state.session.export_receipts.push(receipt);

    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);

    assert_eq!(snapshot["export"]["present"], false);
    assert_eq!(snapshot["export"]["lifecycle"], serde_json::json!([]));
    assert_eq!(
        snapshot["export"]["daw_session_receipt"]["export_scope"],
        "daw_session"
    );
    assert_eq!(
        snapshot["export"]["daw_session_receipt"]["arrangement_placement_readiness"]["status"],
        "ready"
    );
    assert_eq!(
        snapshot["export"]["daw_session_receipt"]["daw_tempo_map_readiness"]["status"],
        "ready"
    );
    assert_eq!(
        snapshot["export"]["daw_session_receipt"]["artifact_set"][1]["role"],
        "daw_session_tempo_map"
    );
    assert_eq!(
        snapshot["export"]["daw_session_surface_gate"]["status"],
        "disabled"
    );
}

fn product_mix_receipt_for_arrangement_observer(
    action_id: ActionId,
    timestamp: u64,
) -> ExportReceiptState {
    ExportReceiptState {
        receipt_id: ExportReceiptId::new(format!("export-receipt-{action_id}")),
        created_by_action: action_id,
        created_at: timestamp,
        export_scope: ExportScope::ProductMix,
        pack_id: "feral-grid-demo".into(),
        export_role: ProductExportRole::FullGridMix,
        export_boundary: ProductExportBoundary::FeralGridGeneratedSupport,
        artifact_path: "exports/full_grid_mix.wav".into(),
        proof_path: "exports/product_export_proof.json".into(),
        manifest_path: None,
        export_hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        normalized_manifest_hash:
            "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into(),
        artifact_set: vec![ExportArtifactSetEntry::product_mix(
            "exports/full_grid_mix.wav",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            Some("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into()),
        )],
        qa_gates: Vec::new(),
        arrangement_placement_refs: Vec::new(),
        daw_tempo_map_ref: None,
        readiness_status: ExportReadinessStatus::Reproducible,
        unsupported_scopes: Vec::new(),
    }
}
