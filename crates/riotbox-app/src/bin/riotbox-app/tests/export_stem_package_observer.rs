use riotbox_core::{
    action::{
        Action, ActionParams, ActionResult, ActionStatus, ActorType, Quantization,
        StemPackageExportBoundary, StemPackageExportRole, StemPackageFallbackComparisonPolicy,
        StemPackageLineagePolicy, UndoPolicy,
    },
    export_readiness::{
        ExportReadinessStatus, ExportScope, ProductExportBoundary, ProductExportDestinationKind,
        ProductExportRole, STEM_PACKAGE_LOCAL_CI_PACK_ID,
    },
    ids::ExportReceiptId,
    session::{
        ExportArtifactAudioMetrics, ExportArtifactFallbackComparisonEvidence,
        ExportArtifactFallbackComparisonKind, ExportArtifactLocation, ExportArtifactMediaType,
        ExportArtifactRole, ExportArtifactSetEntry, ExportArtifactSourceGraphRef,
        ExportReceiptQaGateResult, ExportReceiptQaGateStatus, ExportReceiptState,
        STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID, STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID,
        STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID, STEM_PACKAGE_LINEAGE_QA_GATE_ID,
        STEM_PACKAGE_NON_SILENCE_QA_GATE_ID,
    },
};

#[test]
fn observer_snapshot_reports_completed_stem_package_lifecycle_from_session_receipt() {
    let action_id = ActionId(1129);
    let mut session = SessionFile::new(
        "observer-stem-package-completed",
        "0.1.0",
        "2026-06-03T00:00:00Z",
    );
    session
        .action_log
        .actions
        .push(committed_stem_package_action(action_id, 1_129));
    session
        .export_receipts
        .push(ready_stem_package_receipt(action_id, 1_129));
    let state = JamAppState::from_parts(session, None, ActionQueue::new());
    let shell = JamShellState::new(state, ShellLaunchMode::Load);

    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");
    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle[0]["stage"], "requested");
    assert_eq!(lifecycle[1]["stage"], "started");
    assert_eq!(lifecycle[2]["stage"], "completed");
    assert_eq!(lifecycle[2]["command"], "export.stem_package");
    assert_eq!(lifecycle[2]["receipt"]["export_scope"], "stem_package");
    assert_eq!(
        lifecycle[2]["receipt"]["stem_package_readiness"]["status"],
        "ready"
    );
    assert_eq!(lifecycle[2]["receipt"]["pack_id"], STEM_PACKAGE_LOCAL_CI_PACK_ID);
    assert_eq!(lifecycle[2]["receipt"]["export_role"], "package_manifest");
    assert_eq!(
        lifecycle[2]["receipt"]["export_boundary"],
        "stem_package_local_ci_package_v1"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["stem_package_readiness"]["ready"],
        true
    );
    assert_eq!(
        lifecycle[2]["receipt"]["stem_package_readiness"]["blockers"]
            .as_array()
            .expect("blockers")
            .len(),
        0
    );
    assert_eq!(
        lifecycle[2]["receipt"]["qa_gates"]
            .as_array()
            .expect("qa gates")
            .len(),
        5
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"]
            .as_array()
            .expect("artifact set")
            .len(),
        4
    );
    assert!(
        lifecycle[2]["receipt"]
            .get("arrangement_placement_readiness")
            .is_none()
    );
    assert!(
        lifecycle[2]["receipt"]
            .get("arrangement_placement_refs")
            .is_none()
    );
    assert!(
        lifecycle[2]["receipt"]
            .get("daw_tempo_map_readiness")
            .is_none()
    );
    assert!(lifecycle[2]["receipt"].get("daw_tempo_map_ref").is_none());
    assert_eq!(
        snapshot["export"]["stem_package_surface_gate"]["status"],
        "disabled"
    );
    assert_eq!(
        snapshot["export"]["stem_package_surface_gate"]["runnable"],
        false
    );
    assert_eq!(
        snapshot["export"]["stem_package_surface_gate"]["blockers"],
        serde_json::json!([
            "developer_proof_only",
            "daw_placement_workflow_missing",
            "structured_listening_review_missing"
        ])
    );
}

#[test]
fn observer_snapshot_reports_failed_reserved_stem_package_lifecycle_without_receipt() {
    let session = SessionFile::new(
        "observer-stem-package-failed",
        "0.1.0",
        "2026-06-03T00:00:00Z",
    );
    let mut state = JamAppState::from_parts(session, None, ActionQueue::new());

    state.queue_stem_package_export_reserved(
        950,
        Some("exports/stem-package".into()),
        vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
    );

    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");
    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle[0]["stage"], "requested");
    assert_eq!(lifecycle[1]["stage"], "started");
    assert_eq!(lifecycle[2]["stage"], "failed");
    assert_eq!(lifecycle[2]["command"], "export.stem_package");
    assert_eq!(lifecycle[2]["receipt"], serde_json::Value::Null);
    assert!(
        lifecycle[2]["failure_reason"]
            .as_str()
            .expect("failure reason")
            .contains("stem package export is disabled for musicians")
    );
    assert_eq!(
        snapshot["export"]["stem_package_surface_gate"]["blockers"],
        serde_json::json!([
            "ci_writer_proof_missing",
            "developer_proof_only",
            "daw_placement_workflow_missing",
            "structured_listening_review_missing"
        ])
    );
}

#[test]
fn observer_snapshot_reports_committed_local_ci_stem_package_writer_lifecycle() {
    let temp = tempfile::tempdir().expect("tempdir");
    let destination = temp.path().join("stem-export");
    let session = SessionFile::new(
        "observer-stem-package-writer",
        "0.1.0",
        "2026-06-03T00:00:00Z",
    );
    let mut state = JamAppState::from_parts(session, None, ActionQueue::new());

    let receipt = state
        .commit_stem_package_export_local_ci_package(
            &destination,
            1_131,
            vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        )
        .expect("commit local CI stem package export");

    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");
    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle[0]["stage"], "requested");
    assert_eq!(lifecycle[1]["stage"], "started");
    assert_eq!(lifecycle[2]["stage"], "completed");
    assert_eq!(lifecycle[2]["command"], "export.stem_package");
    assert_eq!(lifecycle[2]["action_id"], receipt.created_by_action.0);
    assert_eq!(
        lifecycle[2]["receipt"]["receipt_id"],
        receipt.receipt_id.to_string()
    );
    assert_eq!(
        lifecycle[2]["receipt"]["stem_package_readiness"]["status"],
        "ready"
    );
    assert_eq!(lifecycle[2]["receipt"]["pack_id"], STEM_PACKAGE_LOCAL_CI_PACK_ID);
    assert_eq!(lifecycle[2]["receipt"]["export_role"], "package_manifest");
    assert_eq!(
        lifecycle[2]["receipt"]["export_boundary"],
        "stem_package_local_ci_package_v1"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"]
            .as_array()
            .expect("artifact set")
            .len(),
        4
    );
    assert_eq!(
        snapshot["export"]["stem_package_surface_gate"]["blockers"],
        serde_json::json!([
            "developer_proof_only",
            "daw_placement_workflow_missing",
            "structured_listening_review_missing"
        ])
    );
}

fn committed_stem_package_action(action_id: ActionId, timestamp: u64) -> Action {
    Action {
        id: action_id,
        actor: ActorType::User,
        command: ActionCommand::ExportStemPackage,
        params: stem_package_action_params(),
        target: ActionTarget {
            scope: Some(TargetScope::Session),
            ..ActionTarget::default()
        },
        requested_at: timestamp,
        quantization: Quantization::Immediate,
        status: ActionStatus::Committed,
        committed_at: Some(timestamp),
        result: Some(ActionResult {
            accepted: true,
            summary: "exported stem package proof".into(),
        }),
        undo_policy: UndoPolicy::NotUndoable {
            reason: "stem package export writes files outside musical undo".into(),
        },
        explanation: Some("stem package writer proof".into()),
    }
}

fn stem_package_action_params() -> ActionParams {
    ActionParams::StemPackageExport {
        export_scope: ExportScope::StemPackage,
        export_role: StemPackageExportRole::PackageManifest,
        boundary: StemPackageExportBoundary::ReservedContractOnly,
        include_manifest: true,
        destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
        destination_path: Some("exports/stem-package".into()),
        claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        lineage_policy: StemPackageLineagePolicy::RequireAnyCoreLineage,
        fallback_comparison_policy: StemPackageFallbackComparisonPolicy::Required,
    }
}

fn ready_stem_package_receipt(action_id: ActionId, timestamp: u64) -> ExportReceiptState {
    let claimed_roles = vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass];
    ExportReceiptState {
        receipt_id: ExportReceiptId::new(format!("export-receipt-{action_id}")),
        created_by_action: action_id,
        created_at: timestamp,
        export_scope: ExportScope::StemPackage,
        pack_id: STEM_PACKAGE_LOCAL_CI_PACK_ID.into(),
        export_role: ProductExportRole::PackageManifest,
        export_boundary: ProductExportBoundary::StemPackageLocalCiPackageV1,
        artifact_path: "exports/stem_package/stem_package_manifest.json".into(),
        proof_path: "exports/stem_package/stem_package_proof.json".into(),
        manifest_path: Some("exports/stem_package/stem_package_manifest.json".into()),
        export_hash: "manifest-sha".into(),
        normalized_manifest_hash: "manifest-sha".into(),
        artifact_set: vec![
            ready_stem_artifact(
                ExportArtifactRole::StemDrums,
                "exports/stem_package/stems/stem_drums.wav",
                "drums-sha",
            ),
            ready_stem_artifact(
                ExportArtifactRole::StemBass,
                "exports/stem_package/stems/stem_bass.wav",
                "bass-sha",
            ),
            ExportArtifactSetEntry::export_manifest(
                "exports/stem_package/stem_package_manifest.json",
                "manifest-sha",
            ),
            ExportArtifactSetEntry::stem_package_proof(
                "exports/stem_package/stem_package_proof.json",
                "proof-sha",
            ),
        ],
        qa_gates: vec![
            passed_stem_package_gate(
                STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
                &claimed_roles,
                "written stem package artifact-set accepted",
            ),
            passed_stem_package_gate(
                STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
                &claimed_roles,
                "written stem package hashes are stable",
            ),
            passed_stem_package_gate(
                STEM_PACKAGE_NON_SILENCE_QA_GATE_ID,
                &claimed_roles,
                "written stem package stems are non-silent",
            ),
            passed_stem_package_gate(
                STEM_PACKAGE_LINEAGE_QA_GATE_ID,
                &claimed_roles,
                "written stem package stems carry lineage",
            ),
            passed_stem_package_gate(
                STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID,
                &claimed_roles,
                "written stem package stems carry fallback comparison",
            ),
        ],
        arrangement_placement_refs: Vec::new(),
        daw_tempo_map_ref: None,
        readiness_status: ExportReadinessStatus::Reproducible,
        unsupported_scopes: Vec::new(),
    }
}

fn ready_stem_artifact(
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
        source_graph_ref: Some(ExportArtifactSourceGraphRef {
            source_id: SourceId::from("source-stem-observer"),
            graph_version: SourceGraphVersion::V1,
            graph_hash: "stem-observer-graph-sha".into(),
        }),
        timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: Some(ExportArtifactFallbackComparisonEvidence {
            comparison_kind: ExportArtifactFallbackComparisonKind::SourceVsFallback,
            reference_identity: "fallback://stem-observer".into(),
            rms_difference_micros: Some(180_000),
            normalized_correlation_micros: Some(260_000),
        }),
        audio_metrics: Some(ExportArtifactAudioMetrics {
            peak_milli_dbfs: Some(-120),
            rms_milli_dbfs: Some(-6_000),
            peak_amplitude_micros: Some(986_000),
            rms_amplitude_micros: Some(125_000),
            silent_frame_count: Some(0),
            total_frame_count: Some(48_000),
        }),
        sample_rate_hz: Some(48_000),
        channel_count: Some(2),
        duration_ms: Some(1_000),
    }
}

fn passed_stem_package_gate(
    gate_id: &str,
    claimed_roles: &[ExportArtifactRole],
    summary: &str,
) -> ExportReceiptQaGateResult {
    ExportReceiptQaGateResult {
        gate_id: gate_id.into(),
        status: ExportReceiptQaGateStatus::Passed,
        artifact_roles: claimed_roles.to_vec(),
        summary: Some(summary.into()),
    }
}
