#[test]
fn jam_inspect_surfaces_export_readiness_without_export_action() {
    let mut shell = sample_shell_state();
    shell.jam_mode = JamViewMode::Inspect;

    let inspect = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(inspect.contains("export full_grid_mix | feral-grid"), "{inspect}");
    assert!(
        inspect.contains("reproducible | no stem/live/DAW/host"),
        "{inspect}"
    );
    assert!(!inspect.contains("queue export"), "{inspect}");
}

#[test]
fn jam_inspect_surfaces_latest_export_receipt_without_adding_perform_control() {
    let mut shell = sample_shell_state();
    shell.app.session.export_receipts.push(ExportReceiptState {
        receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
        created_by_action: ActionId(4),
        created_at: 900,
        export_scope: ExportScope::ProductMix,
        export_role: ProductExportRole::FullGridMix,
        export_boundary: ProductExportBoundary::FeralGridGeneratedSupport,
        artifact_path: "exports/full_grid_mix.wav".into(),
        proof_path: "exports/product_export_proof.json".into(),
        manifest_path: None,
        export_hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        normalized_manifest_hash: "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            .into(),
        artifact_set: vec![ExportArtifactSetEntry::product_mix(
            "exports/full_grid_mix.wav",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )],
        readiness_status: ExportReadinessStatus::Reproducible,
        unsupported_scopes: vec![
            UnsupportedExportScope::StemPackage,
            UnsupportedExportScope::DawExport,
        ],
    });
    shell.app.refresh_view();
    shell.jam_mode = JamViewMode::Inspect;

    let inspect = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(inspect.contains("export full_grid_mix | feral-grid"), "{inspect}");
    assert!(
        inspect.contains("a-0004 ok | wav+proof | no stem/DAW"),
        "{inspect}"
    );
    assert!(!inspect.contains("queue export"), "{inspect}");

    shell.jam_mode = JamViewMode::Perform;
    let perform = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(!perform.contains("receipt export-receipt-a-0004"), "{perform}");
    assert!(!perform.contains("queue export"), "{perform}");
}

#[test]
fn jam_inspect_surfaces_export_failure_feedback() {
    let mut shell = sample_shell_state();
    shell.app.queue_product_mix_export(900, None);
    let action_id = shell
        .app
        .queue
        .pending_actions()
        .into_iter()
        .find(|action| action.command == ActionCommand::ExportProductMix)
        .expect("pending export action")
        .id;
    assert!(
        shell
            .app
            .queue
            .reject(action_id, "export artifact hash mismatch for full_grid_mix")
    );
    shell.jam_mode = JamViewMode::Inspect;

    let inspect = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(inspect.contains("export full_grid_mix | failed"), "{inspect}");
    assert!(
        inspect.contains("a-0004 | export artifact hash"),
        "{inspect}"
    );
}

#[test]
fn jam_perform_does_not_claim_export_readiness_as_a_play_control() {
    let shell = sample_shell_state();

    let perform = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(!perform.contains("export full_grid_mix"), "{perform}");
    assert!(!perform.contains("queue export"), "{perform}");
}
