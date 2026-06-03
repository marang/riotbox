use riotbox_core::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, ExportReadinessStatus, ExportScope,
        ProductExportBoundary, ProductExportRole,
    },
    ids::ExportReceiptId,
    session::{ExportArrangementPlacementRef, ExportArtifactSetEntry, ExportReceiptState},
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
        readiness_status: ExportReadinessStatus::Reproducible,
        unsupported_scopes: Vec::new(),
    }
}
