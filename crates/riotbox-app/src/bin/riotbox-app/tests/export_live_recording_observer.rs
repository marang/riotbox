use riotbox_core::{
    action::{
        Action, ActionParams, ActionResult, ActionStatus, ActorType,
        LiveRecordingExportBoundary, LiveRecordingExportRole, UndoPolicy,
    },
    export_readiness::{
        EXPORT_READINESS_CONTRACT_SCHEMA, ExportReadinessContract, ExportReadinessStatus,
        ExportScope, LIVE_RECORDING_RECEIPT_PACK_ID, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportDestinationKind, ProductExportRole,
        UnsupportedExportScope,
    },
    session::{
        ExportLiveRecordingCallbackGapSummary, ExportLiveRecordingHostAudioRef,
        ExportLiveRecordingStreamErrorSummary, ExportReceiptState,
    },
};

#[test]
fn observer_snapshot_projects_live_recording_host_audio_refs_from_real_action_receipt() {
    let action_id = ActionId(1_174);
    let mut session = SessionFile::new(
        "observer-live-recording-evidence",
        "0.1.0",
        "2026-06-03T00:00:00Z",
    );
    session.action_log.actions.push(Action {
        id: action_id,
        actor: ActorType::User,
        command: ActionCommand::ExportLiveRecording,
        params: ActionParams::LiveRecordingExport {
            export_scope: ExportScope::LiveRecording,
            export_role: LiveRecordingExportRole::LiveRecordingCapture,
            boundary: LiveRecordingExportBoundary::ReservedContractOnly,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path: Some("exports/live".into()),
            receipt_id: Some("export-receipt-1174".into()),
        },
        target: ActionTarget {
            scope: Some(TargetScope::Session),
            ..Default::default()
        },
        requested_at: 1_174,
        quantization: Quantization::Immediate,
        status: ActionStatus::Committed,
        committed_at: Some(1_180),
        result: Some(ActionResult {
            accepted: true,
            summary: "recorded live host-audio evidence fixture".into(),
        }),
        undo_policy: UndoPolicy::NotUndoable {
            reason: "live recording export writes files outside musical undo".into(),
        },
        explanation: Some("live recording evidence fixture".into()),
    });

    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::LiveRecording,
        boundary: ProductExportBoundary::LiveRecordingReceiptContractV1,
        pack_id: LIVE_RECORDING_RECEIPT_PACK_ID.into(),
        export_role: ProductExportRole::LiveRecordingCapture,
        export_artifact: "exports/live/recording.wav".into(),
        source_sha256: "source-sha".into(),
        export_sha256: "1212121212121212121212121212121212121212121212121212121212121212"
            .into(),
        normalized_manifest_sha256:
            "3434343434343434343434343434343434343434343434343434343434343434".into(),
        unsupported_scopes: vec![],
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        action_id,
        1_180,
        &contract,
        "exports/live/recording.wav",
        "exports/live/proof.json",
        Some("exports/live/manifest.json".into()),
    );
    receipt.live_recording_host_audio_refs = vec![ExportLiveRecordingHostAudioRef {
        host: "Alsa".into(),
        device: "pipewire-default".into(),
        recording_duration_ms: 16_000,
        callback_gap_summary: ExportLiveRecordingCallbackGapSummary {
            max_gap_ms: Some(3),
            over_threshold_count: 0,
        },
        stream_error_summary: ExportLiveRecordingStreamErrorSummary {
            error_count: 0,
            last_error: None,
        },
    }];
    session.export_receipts.push(receipt);

    let state = JamAppState::from_parts(session, None, ActionQueue::new());
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");

    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle[2]["stage"], "completed");
    assert_eq!(lifecycle[2]["command"], "export.live_recording");
    assert_eq!(
        lifecycle[2]["receipt"]["live_recording_host_audio_refs"][0]["host"],
        "Alsa"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["live_recording_host_audio_refs"][0]["device"],
        "pipewire-default"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["live_recording_host_audio_refs"][0]["recording_duration_ms"],
        16_000
    );
    assert_eq!(
        lifecycle[2]["receipt"]["live_recording_host_audio_refs"][0]["callback_gap_summary"]
            ["over_threshold_count"],
        0
    );
    assert_eq!(
        lifecycle[2]["receipt"]["live_recording_host_audio_refs"][0]["stream_error_summary"]
            ["error_count"],
        0
    );
    assert_eq!(
        lifecycle[2]["receipt"]["live_recording_host_audio_readiness"]["status"],
        "ready"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["live_recording_host_audio_readiness"]["ready"],
        true
    );
    assert_eq!(
        lifecycle[2]["receipt"]["live_recording_host_audio_readiness"]["blockers"],
        serde_json::json!([])
    );
}

#[test]
fn observer_snapshot_projects_blocked_live_recording_host_audio_readiness() {
    let action_id = ActionId(1_176);
    let mut session = SessionFile::new(
        "observer-live-recording-readiness-blocked",
        "0.1.0",
        "2026-06-03T00:00:00Z",
    );
    session.action_log.actions.push(live_recording_action(
        action_id,
        "missing live host-audio evidence fixture",
    ));

    let mut receipt = live_recording_receipt(action_id);
    receipt
        .unsupported_scopes
        .push(UnsupportedExportScope::LiveRecording);
    session.export_receipts.push(receipt);

    let state = JamAppState::from_parts(session, None, ActionQueue::new());
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");
    let readiness = &lifecycle[2]["receipt"]["live_recording_host_audio_readiness"];

    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle[2]["stage"], "completed");
    assert_eq!(lifecycle[2]["command"], "export.live_recording");
    assert_eq!(readiness["status"], "blocked");
    assert_eq!(readiness["ready"], false);
    assert_eq!(
        readiness["blockers"],
        serde_json::json!(["unsupported_scope_flag_present", "missing_host_audio_evidence"])
    );
    assert_eq!(
        readiness["blocker_labels"],
        serde_json::json!([
            "live recording export is still marked unsupported",
            "live recording host-audio evidence is missing"
        ])
    );
}

fn live_recording_action(action_id: ActionId, summary: &str) -> Action {
    Action {
        id: action_id,
        actor: ActorType::User,
        command: ActionCommand::ExportLiveRecording,
        params: ActionParams::LiveRecordingExport {
            export_scope: ExportScope::LiveRecording,
            export_role: LiveRecordingExportRole::LiveRecordingCapture,
            boundary: LiveRecordingExportBoundary::ReservedContractOnly,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path: Some("exports/live".into()),
            receipt_id: Some(format!("export-receipt-{action_id}", action_id = action_id.0)),
        },
        target: ActionTarget {
            scope: Some(TargetScope::Session),
            ..Default::default()
        },
        requested_at: action_id.0,
        quantization: Quantization::Immediate,
        status: ActionStatus::Committed,
        committed_at: Some(action_id.0 + 6),
        result: Some(ActionResult {
            accepted: true,
            summary: summary.into(),
        }),
        undo_policy: UndoPolicy::NotUndoable {
            reason: "live recording export writes files outside musical undo".into(),
        },
        explanation: Some("live recording evidence fixture".into()),
    }
}

fn live_recording_receipt(action_id: ActionId) -> ExportReceiptState {
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::LiveRecording,
        boundary: ProductExportBoundary::LiveRecordingReceiptContractV1,
        pack_id: LIVE_RECORDING_RECEIPT_PACK_ID.into(),
        export_role: ProductExportRole::LiveRecordingCapture,
        export_artifact: "exports/live/recording.wav".into(),
        source_sha256: "source-sha".into(),
        export_sha256: "1212121212121212121212121212121212121212121212121212121212121212"
            .into(),
        normalized_manifest_sha256:
            "3434343434343434343434343434343434343434343434343434343434343434".into(),
        unsupported_scopes: vec![],
    };
    ExportReceiptState::from_readiness_contract(
        action_id,
        action_id.0 + 6,
        &contract,
        "exports/live/recording.wav",
        "exports/live/proof.json",
        Some("exports/live/manifest.json".into()),
    )
}
