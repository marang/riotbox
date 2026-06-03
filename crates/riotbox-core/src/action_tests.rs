use std::collections::BTreeSet;

use super::*;

#[test]
fn action_command_lexicon_labels_are_unique_and_complete() {
    assert_eq!(ActionCommand::all().len(), 60);

    let labels = ActionCommand::all()
        .iter()
        .map(|command| command.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(labels.len(), ActionCommand::all().len());
    assert!(!labels.contains(""));
}

#[test]
fn action_command_replay_coverage_is_declared_for_every_command() {
    let supported = ActionCommand::all()
        .iter()
        .filter(|command| command.replay_coverage() == ActionReplayCoverage::Supported)
        .count();
    let unsupported = ActionCommand::all().len() - supported;

    assert_eq!(supported, 42);
    assert_eq!(unsupported, 18);
}

#[test]
fn product_export_action_params_default_scope_for_older_logs() {
    let params: ActionParams = serde_json::from_value(serde_json::json!({
        "ProductExport": {
            "export_role": "full_grid_mix",
            "boundary": "feral_grid_generated_support",
            "include_manifest": true,
            "destination_kind": "local_artifact_directory",
            "destination_path": "exports"
        }
    }))
    .expect("older product export params deserialize");

    assert_eq!(
        params,
        ActionParams::ProductExport {
            export_scope: ExportScope::ProductMix,
            export_role: ProductExportRole::FullGridMix,
            boundary: ProductExportBoundary::FeralGridGeneratedSupport,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path: Some("exports".into()),
        }
    );
}

#[test]
fn stem_package_export_action_contract_roundtrips_as_reserved_scope() {
    let action = Action {
        id: ActionId(1),
        actor: ActorType::User,
        command: ActionCommand::ExportStemPackage,
        params: ActionParams::StemPackageExport {
            export_scope: ExportScope::StemPackage,
            export_role: StemPackageExportRole::PackageManifest,
            boundary: StemPackageExportBoundary::ReservedContractOnly,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path: Some("exports/stem-package".into()),
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
            lineage_policy: StemPackageLineagePolicy::RequireAnyCoreLineage,
            fallback_comparison_policy: StemPackageFallbackComparisonPolicy::Required,
        },
        target: ActionTarget {
            scope: Some(TargetScope::Session),
            ..ActionTarget::default()
        },
        requested_at: 100,
        quantization: Quantization::Immediate,
        status: ActionStatus::Requested,
        committed_at: None,
        result: None,
        undo_policy: UndoPolicy::NotUndoable {
            reason: "reserved stem-package export writes files outside musical undo".into(),
        },
        explanation: Some("reserved contract only; not runnable yet".into()),
    };

    let json = serde_json::to_value(&action).expect("serialize reserved stem action");
    assert_eq!(json["command"], "ExportStemPackage");
    assert_eq!(
        json["params"]["StemPackageExport"]["export_scope"],
        "stem_package"
    );
    assert_eq!(
        json["params"]["StemPackageExport"]["claimed_stem_roles"],
        serde_json::json!(["stem_drums", "stem_bass"])
    );
    assert_eq!(
        json["params"]["StemPackageExport"]["lineage_policy"],
        "require_any_core_lineage"
    );
    assert_eq!(
        json["params"]["StemPackageExport"]["fallback_comparison_policy"],
        "required"
    );

    let roundtrip: Action = serde_json::from_value(json).expect("deserialize reserved stem action");
    assert_eq!(roundtrip, action);
    assert_eq!(
        roundtrip.command.replay_coverage(),
        ActionReplayCoverage::Unsupported
    );
    let local_ci_json = serde_json::to_value(StemPackageExportBoundary::LocalCiPackageV1)
        .expect("serialize local CI boundary");
    assert_eq!(local_ci_json, "local_ci_package_v1");
    let local_ci_boundary: StemPackageExportBoundary =
        serde_json::from_value(local_ci_json).expect("deserialize local CI boundary");
    assert_eq!(
        local_ci_boundary,
        StemPackageExportBoundary::LocalCiPackageV1
    );
}

#[test]
fn live_recording_export_action_contract_roundtrips_as_reserved_scope() {
    let action = Action {
        id: ActionId(3),
        actor: ActorType::User,
        command: ActionCommand::ExportLiveRecording,
        params: ActionParams::LiveRecordingExport {
            export_scope: ExportScope::LiveRecording,
            export_role: LiveRecordingExportRole::LiveRecordingCapture,
            boundary: LiveRecordingExportBoundary::ReservedContractOnly,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path: Some("exports/live-recording".into()),
            receipt_id: Some("export-receipt-live-42".into()),
        },
        target: ActionTarget {
            scope: Some(TargetScope::Session),
            ..ActionTarget::default()
        },
        requested_at: 110,
        quantization: Quantization::Immediate,
        status: ActionStatus::Requested,
        committed_at: None,
        result: None,
        undo_policy: UndoPolicy::NotUndoable {
            reason: "reserved live recording export writes files outside musical undo".into(),
        },
        explanation: Some("reserved live recording export contract; not runnable yet".into()),
    };

    let json = serde_json::to_value(&action).expect("serialize reserved live action");
    assert_eq!(json["command"], "ExportLiveRecording");
    assert_eq!(
        json["params"]["LiveRecordingExport"]["export_scope"],
        "live_recording"
    );
    assert_eq!(
        json["params"]["LiveRecordingExport"]["export_role"],
        "live_recording_capture"
    );
    assert_eq!(
        json["params"]["LiveRecordingExport"]["boundary"],
        "reserved_contract_only"
    );
    assert_eq!(
        json["params"]["LiveRecordingExport"]["receipt_id"],
        "export-receipt-live-42"
    );

    let roundtrip: Action = serde_json::from_value(json).expect("deserialize reserved live action");
    assert_eq!(roundtrip, action);
    assert_eq!(
        roundtrip.command.replay_coverage(),
        ActionReplayCoverage::Unsupported
    );
    assert_eq!(
        ActionCommand::ExportLiveRecording.as_str(),
        "export.live_recording"
    );

    let older_params: ActionParams = serde_json::from_value(serde_json::json!({
        "LiveRecordingExport": {
            "export_role": "live_recording_capture",
            "boundary": "reserved_contract_only",
            "include_manifest": true,
            "destination_kind": "local_artifact_directory",
            "destination_path": "exports/live-recording",
            "receipt_id": null
        }
    }))
    .expect("older live recording export params deserialize");
    assert_eq!(
        older_params,
        ActionParams::LiveRecordingExport {
            export_scope: ExportScope::LiveRecording,
            export_role: LiveRecordingExportRole::LiveRecordingCapture,
            boundary: LiveRecordingExportBoundary::ReservedContractOnly,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path: Some("exports/live-recording".into()),
            receipt_id: None,
        }
    );
}

#[test]
fn daw_session_export_action_contract_roundtrips_as_reserved_scope() {
    let action = Action {
        id: ActionId(2),
        actor: ActorType::User,
        command: ActionCommand::ExportDawSession,
        params: ActionParams::DawSessionExport {
            export_scope: ExportScope::DawSession,
            boundary: DawSessionExportBoundary::ReservedContractOnly,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path: Some("exports/daw-session".into()),
            receipt_id: Some("export-receipt-42".into()),
        },
        target: ActionTarget {
            scope: Some(TargetScope::Session),
            ..ActionTarget::default()
        },
        requested_at: 120,
        quantization: Quantization::Immediate,
        status: ActionStatus::Requested,
        committed_at: None,
        result: None,
        undo_policy: UndoPolicy::NotUndoable {
            reason: "reserved DAW session export writes files outside musical undo".into(),
        },
        explanation: Some("reserved DAW session export contract; not runnable yet".into()),
    };

    let json = serde_json::to_value(&action).expect("serialize reserved DAW action");
    assert_eq!(json["command"], "ExportDawSession");
    assert_eq!(
        json["params"]["DawSessionExport"]["export_scope"],
        "daw_session"
    );
    assert_eq!(
        json["params"]["DawSessionExport"]["boundary"],
        "reserved_contract_only"
    );
    assert_eq!(
        json["params"]["DawSessionExport"]["receipt_id"],
        "export-receipt-42"
    );

    let roundtrip: Action = serde_json::from_value(json).expect("deserialize reserved DAW action");
    assert_eq!(roundtrip, action);
    assert_eq!(
        roundtrip.command.replay_coverage(),
        ActionReplayCoverage::Unsupported
    );
    let writer_json = serde_json::to_value(DawSessionExportBoundary::LocalProjectWriterV1)
        .expect("serialize local DAW writer boundary");
    assert_eq!(writer_json, "local_project_writer_v1");
    let writer_boundary: DawSessionExportBoundary =
        serde_json::from_value(writer_json).expect("deserialize local DAW writer boundary");
    assert_eq!(
        writer_boundary,
        DawSessionExportBoundary::LocalProjectWriterV1
    );
    let host_import_json = serde_json::to_value(DawSessionExportBoundary::HostImportProofV1)
        .expect("serialize DAW host import proof boundary");
    assert_eq!(host_import_json, "host_import_proof_v1");
    let host_import_boundary: DawSessionExportBoundary = serde_json::from_value(host_import_json)
        .expect("deserialize DAW host import proof boundary");
    assert_eq!(
        host_import_boundary,
        DawSessionExportBoundary::HostImportProofV1
    );
    let audible_output_json = serde_json::to_value(DawSessionExportBoundary::AudibleOutputProofV1)
        .expect("serialize DAW audible output proof boundary");
    assert_eq!(audible_output_json, "audible_output_proof_v1");
    let audible_output_boundary: DawSessionExportBoundary =
        serde_json::from_value(audible_output_json)
            .expect("deserialize DAW audible output proof boundary");
    assert_eq!(
        audible_output_boundary,
        DawSessionExportBoundary::AudibleOutputProofV1
    );
}
