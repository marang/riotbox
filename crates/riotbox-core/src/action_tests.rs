use std::collections::BTreeSet;

use super::*;

#[test]
fn source_monitor_mode_cycles_through_all_reachable_modes() {
    assert_eq!(SourceMonitorMode::Source.next(), SourceMonitorMode::Blend);
    assert_eq!(SourceMonitorMode::Blend.next(), SourceMonitorMode::Riotbox);
    assert_eq!(SourceMonitorMode::Riotbox.next(), SourceMonitorMode::Source);
}

#[test]
fn action_command_lexicon_labels_are_unique_and_complete() {
    assert_eq!(ActionCommand::all().len(), 61);

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

    assert_eq!(supported, 43);
    assert_eq!(unsupported, 18);
}

#[test]
fn typed_undo_semantics_are_limited_to_restorable_runtime_state() {
    for command in [
        ActionCommand::Mc202SetRole,
        ActionCommand::Mc202GenerateFollower,
        ActionCommand::Mc202GenerateAnswer,
        ActionCommand::Mc202GeneratePressure,
        ActionCommand::Mc202GenerateInstigator,
        ActionCommand::Mc202MutatePhrase,
        ActionCommand::Tr909FillNext,
        ActionCommand::SourceMonitorSetMode,
    ] {
        assert!(command.has_typed_undo_semantics(), "{command}");
    }

    for command in [
        ActionCommand::SceneLaunch,
        ActionCommand::W30TriggerPad,
        ActionCommand::CaptureNow,
    ] {
        assert!(!command.has_typed_undo_semantics(), "{command}");
    }
}

#[test]
fn action_draft_only_advertises_undo_when_typed_restoration_exists() {
    let typed = ActionDraft::new(
        ActorType::User,
        ActionCommand::Tr909FillNext,
        Quantization::NextBar,
        ActionTarget::default(),
    );
    assert_eq!(typed.undo_policy, UndoPolicy::Undoable);

    let unsupported = ActionDraft::new(
        ActorType::User,
        ActionCommand::SceneLaunch,
        Quantization::NextBar,
        ActionTarget::default(),
    );
    assert!(matches!(
        unsupported.undo_policy,
        UndoPolicy::NotUndoable { .. }
    ));
}

#[test]
fn w30_damage_profile_intent_roundtrips_in_the_action_contract() {
    let params = ActionParams::W30DamageProfile {
        intensity: 0.82,
        target_id: CaptureId::from("cap-hard-01"),
        intent: crate::w30::W30HardIntent::Texture,
        base_grit_level: 0.68,
    };

    let json = serde_json::to_value(&params).expect("serialize W-30 hard intent");
    assert_eq!(
        json["W30DamageProfile"]["target_id"],
        serde_json::json!("cap-hard-01")
    );
    assert_eq!(
        json["W30DamageProfile"]["intent"],
        serde_json::json!("texture")
    );
    assert!(
        (json["W30DamageProfile"]["base_grit_level"]
            .as_f64()
            .expect("serialized base grit")
            - 0.68)
            .abs()
            < 1.0e-6
    );
    assert_eq!(
        serde_json::from_value::<ActionParams>(json).expect("deserialize W-30 hard intent"),
        params
    );
}

#[test]
fn older_w30_damage_profile_defaults_missing_base_grit_to_clean() {
    let params: ActionParams = serde_json::from_value(serde_json::json!({
        "W30DamageProfile": {
            "intensity": 0.82,
            "target_id": "cap-hard-legacy",
            "intent": "impact"
        }
    }))
    .expect("deserialize pre-V7 W-30 damage params");

    assert!(matches!(
        params,
        ActionParams::W30DamageProfile {
            base_grit_level,
            ..
        } if base_grit_level == 0.0
    ));
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
