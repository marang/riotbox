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
    assert!(
        inspect.contains("stem_package surface | disabled"),
        "{inspect}"
    );
    assert!(
        inspect.contains("needs ci-proof/dev-only/DAW/listening"),
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
        pack_id: riotbox_core::export_readiness::PRODUCT_EXPORT_PACK_ID.into(),
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
            Some("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into()),
        )],
        qa_gates: vec![ExportReceiptQaGateResult::product_export_reproducibility()],
        arrangement_placement_refs: Vec::new(),
        daw_tempo_map_ref: None,
        live_recording_host_audio_refs: Vec::new(),
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
    assert!(
        inspect.contains("stem_package surface | disabled"),
        "{inspect}"
    );
    assert!(!inspect.contains("queue export"), "{inspect}");

    shell.jam_mode = JamViewMode::Perform;
    let perform = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(!perform.contains("receipt export-receipt-a-0004"), "{perform}");
    assert!(!perform.contains("queue export"), "{perform}");
}

#[test]
fn jam_inspect_surfaces_ready_stem_package_receipt_without_adding_perform_control() {
    let mut shell = sample_shell_state();
    shell
        .app
        .session
        .export_receipts
        .push(stem_package_receipt(
            ActionId(1_132),
            vec![
                passed_stem_package_gate(STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID),
                passed_stem_package_gate(STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID),
                passed_stem_package_gate(STEM_PACKAGE_NON_SILENCE_QA_GATE_ID),
                passed_stem_package_gate(STEM_PACKAGE_LINEAGE_QA_GATE_ID),
                passed_stem_package_gate(STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID),
            ],
            Vec::new(),
        ));
    shell.app.refresh_view();
    shell.jam_mode = JamViewMode::Inspect;

    let inspect = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(inspect.contains("export stem_package | stem-pkg"), "{inspect}");
    assert!(
        inspect.contains("1132 ready | bass/drums | art4"),
        "{inspect}"
    );
    assert!(
        inspect.contains("gates art/hsh/aud/lin/cmp pass"),
        "{inspect}"
    );
    assert!(inspect.contains("blockers none"), "{inspect}");
    assert!(
        inspect.contains("stem_package surface | disabled"),
        "{inspect}"
    );
    assert!(
        inspect.contains("needs dev-only/DAW/listening"),
        "{inspect}"
    );
    assert!(!inspect.contains("queue export"), "{inspect}");

    shell.jam_mode = JamViewMode::Perform;
    let perform = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(!perform.contains("export stem_package"), "{perform}");
    assert!(!perform.contains("queue export"), "{perform}");
}

#[test]
fn jam_inspect_surfaces_blocked_stem_package_receipt() {
    let mut shell = sample_shell_state();
    shell
        .app
        .session
        .export_receipts
        .push(stem_package_receipt(
            ActionId(1_133),
            vec![passed_stem_package_gate(
                STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
            )],
            vec![UnsupportedExportScope::StemPackage],
        ));
    shell.app.refresh_view();
    shell.jam_mode = JamViewMode::Inspect;

    let inspect = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(inspect.contains("export stem_package | stem-pkg"), "{inspect}");
    assert!(
        inspect.contains("1133 blocked | bass/drums | art4"),
        "{inspect}"
    );
    assert!(
        inspect.contains("gates pass art | miss hsh/aud/lin/cmp"),
        "{inspect}"
    );
    assert!(
        inspect.contains("blockers unsup | miss hsh/aud/lin/cmp"),
        "{inspect}"
    );
    assert!(
        inspect.contains("needs qa/dev-only/DAW/listening"),
        "{inspect}"
    );
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

fn stem_package_receipt(
    action_id: ActionId,
    qa_gates: Vec<ExportReceiptQaGateResult>,
    unsupported_scopes: Vec<UnsupportedExportScope>,
) -> ExportReceiptState {
    ExportReceiptState {
        receipt_id: ExportReceiptId::new(format!("export-receipt-{}", action_id.0)),
        created_by_action: action_id,
        created_at: 900,
        export_scope: ExportScope::StemPackage,
        pack_id: STEM_PACKAGE_LOCAL_CI_PACK_ID.into(),
        export_role: ProductExportRole::PackageManifest,
        export_boundary: ProductExportBoundary::StemPackageLocalCiPackageV1,
        artifact_path: "exports/stem_package/stem_package_manifest.json".into(),
        proof_path: "exports/stem_package/stem_package_proof.json".into(),
        manifest_path: Some("exports/stem_package/stem_package_manifest.json".into()),
        export_hash: "drums-sha".into(),
        normalized_manifest_hash: "manifest-sha".into(),
        artifact_set: vec![
            stem_artifact(ExportArtifactRole::StemDrums, "stem_drums.wav", "drums-sha"),
            stem_artifact(ExportArtifactRole::StemBass, "stem_bass.wav", "bass-sha"),
            ExportArtifactSetEntry::export_manifest(
                "exports/stem_package/stem_package_manifest.json",
                "manifest-sha",
            ),
            ExportArtifactSetEntry::stem_package_proof(
                "exports/stem_package/stem_package_proof.json",
                "proof-sha",
            ),
        ],
        qa_gates,
        arrangement_placement_refs: Vec::new(),
        daw_tempo_map_ref: None,
        live_recording_host_audio_refs: Vec::new(),
        readiness_status: ExportReadinessStatus::Reproducible,
        unsupported_scopes,
    }
}

fn stem_artifact(
    role: ExportArtifactRole,
    file_name: &str,
    sha256: &str,
) -> ExportArtifactSetEntry {
    ExportArtifactSetEntry {
        role,
        location: ExportArtifactLocation::LocalPath {
            path: format!("exports/stem_package/stems/{file_name}"),
        },
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

fn passed_stem_package_gate(gate_id: &str) -> ExportReceiptQaGateResult {
    ExportReceiptQaGateResult {
        gate_id: gate_id.into(),
        status: ExportReceiptQaGateStatus::Passed,
        artifact_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        summary: Some("test stem package gate passed".into()),
    }
}
