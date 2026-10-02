//! Generated metadata-only coverage for the extended live-master DAW contract.
use super::{
    ExportArtifactLocation, ExportReceiptState, export_live_recording_v4_tests::daw_receipt,
    export_live_recording_window_tests::window_session,
};
use crate::{
    action::{
        ActionCommand, ActionParams, ActionStatus, DawSessionExportBoundary, LiveRecordingDuration,
    },
    export_readiness::{
        ExportScope, LIVE_MASTER_DAWPROJECT_PACK_ID, LIVE_MASTER_DAWPROJECT_V2_PACK_ID,
        LIVE_MASTER_DAWPROJECT_V3_PACK_ID, ProductExportBoundary, ProductExportDestinationKind,
    },
    ids::{ActionId, ExportReceiptId, SnapshotId},
    replay::{apply_replay_plan_to_session, hydrate_replay_target_from_snapshot_payload},
    session::{
        ExportArrangementPlacementRef, ExportArtifactTimingGridRef, ExportDawTempoMapRef,
        SessionFile, Snapshot, SnapshotPayload,
    },
};

#[test]
fn daw_v3_typed_action_roundtrip_preserves_all_legacy_omitted_duration_bytes() {
    for boundary in [
        DawSessionExportBoundary::ReservedContractOnly,
        DawSessionExportBoundary::LocalProjectWriterV1,
        DawSessionExportBoundary::HostImportProofV1,
        DawSessionExportBoundary::AudibleOutputProofV1,
        DawSessionExportBoundary::W30HookDawprojectV1,
        DawSessionExportBoundary::LiveMasterDawprojectV1,
        DawSessionExportBoundary::LiveMasterDawprojectV2,
        DawSessionExportBoundary::LiveMasterDawprojectV3,
    ] {
        for duration in [
            None,
            Some(LiveRecordingDuration::TwoBars),
            Some(LiveRecordingDuration::EightBars),
            Some(LiveRecordingDuration::SixteenBars),
        ] {
            let expected = if boundary == DawSessionExportBoundary::LiveMasterDawprojectV3 {
                matches!(
                    duration,
                    Some(LiveRecordingDuration::EightBars | LiveRecordingDuration::SixteenBars)
                )
            } else {
                duration.is_none()
            };
            assert_eq!(boundary.valid_duration(duration), expected);
            let params = daw_action_params(boundary, duration, "source");
            let json = serde_json::to_value(&params).unwrap();
            assert_eq!(
                json["DawSessionExport"].get("duration").is_some(),
                duration.is_some()
            );
            let restored: ActionParams = serde_json::from_value(json.clone()).unwrap();
            assert_eq!(params, restored);
            assert_eq!(serde_json::to_value(restored).unwrap(), json);
        }
    }
    assert_eq!(
        serde_json::to_value(DawSessionExportBoundary::LiveMasterDawprojectV3).unwrap(),
        "live_master_dawproject_v3"
    );
    assert_eq!(
        ProductExportBoundary::DawSessionLiveMasterDawprojectV3.as_proof_str(),
        "daw_session.live_master_dawproject_v3"
    );
}

#[test]
fn daw_v3_archive_geometry_matches_both_durations_and_fractional_recorded_tempo() {
    for (duration, rate, bpm) in [
        (LiveRecordingDuration::EightBars, 44_100, 80.0082_f32),
        (LiveRecordingDuration::SixteenBars, 48_000, 90.049_f32),
        (LiveRecordingDuration::EightBars, 48_000, 121.5),
        (LiveRecordingDuration::SixteenBars, 96_000, 137.25),
    ] {
        let session = archive_session(duration, rate, bpm);
        let archive = &session.export_receipts[1];
        assert!(archive.is_live_master_dawproject_v3());
        assert!(archive.is_dawproject_archive_receipt());
        assert!(archive.live_master_dawproject_archive_ready());
        assert!(archive.live_master_dawproject_action_contract_matches(
            DawSessionExportBoundary::LiveMasterDawprojectV3,
            Some(duration)
        ));
        assert!(
            archive.live_master_dawproject_source_contract_matches(&session.export_receipts[0])
        );
        session
            .validate_live_recording_duration_contracts()
            .unwrap();
        let restored: SessionFile =
            serde_json::from_slice(&serde_json::to_vec(&session).unwrap()).unwrap();
        assert_eq!(restored, session);
        restored
            .validate_live_recording_duration_contracts()
            .unwrap();
    }
}

#[test]
fn daw_v3_archive_rejects_duration_placement_tempo_and_audio_geometry_drift() {
    let session = archive_session(LiveRecordingDuration::EightBars, 48_000, 120.0);
    let mutations: &[fn(&mut ExportReceiptState)] = &[
        |receipt| receipt.live_recording_duration = None,
        |receipt| receipt.live_recording_duration = Some(LiveRecordingDuration::TwoBars),
        |receipt| receipt.live_recording_duration = Some(LiveRecordingDuration::SixteenBars),
        |receipt| receipt.arrangement_placement_refs.clear(),
        |receipt| {
            receipt
                .arrangement_placement_refs
                .push(receipt.arrangement_placement_refs[0].clone())
        },
        |receipt| receipt.arrangement_placement_refs[0].end_bar = 2,
        |receipt| receipt.arrangement_placement_refs[0].start_beat = 1,
        |receipt| receipt.arrangement_placement_refs[0].end_beat = 8,
        |receipt| receipt.arrangement_placement_refs[0].source_id = None,
        |receipt| receipt.daw_tempo_map_ref = None,
        |receipt| receipt.daw_tempo_map_ref.as_mut().unwrap().end_beat = 8,
        |receipt| receipt.daw_tempo_map_ref.as_mut().unwrap().bpm_micros += 1_000_000,
        |receipt| {
            receipt.artifact_set[2]
                .audio_metrics
                .as_mut()
                .unwrap()
                .total_frame_count = Some(1)
        },
        |receipt| receipt.artifact_set[2].duration_ms = Some(1),
        |receipt| receipt.artifact_set[2].sample_rate_hz = Some(0),
        |receipt| receipt.artifact_set[2].channel_count = Some(0),
        |receipt| receipt.artifact_set[2].timing_grid_ref = None,
        |receipt| receipt.pack_id = LIVE_MASTER_DAWPROJECT_V2_PACK_ID.into(),
    ];
    for (index, mutate) in mutations.iter().enumerate() {
        let mut receipt = session.export_receipts[1].clone();
        mutate(&mut receipt);
        assert!(
            !receipt.live_master_dawproject_archive_ready(),
            "mutation {index}"
        );
    }
}

#[test]
fn daw_v3_source_pin_and_action_identity_fail_closed_on_restore() {
    let original = archive_session(LiveRecordingDuration::EightBars, 48_000, 120.0);
    let mutations: &[fn(&mut SessionFile)] = &[
        |session| {
            if let ActionParams::DawSessionExport { receipt_id, .. } =
                &mut session.action_log.actions[1].params
            {
                *receipt_id = None;
            }
        },
        |session| {
            if let ActionParams::DawSessionExport { receipt_id, .. } =
                &mut session.action_log.actions[1].params
            {
                *receipt_id = Some("missing-source".into());
            }
        },
        |session| {
            if let ActionParams::DawSessionExport { duration, .. } =
                &mut session.action_log.actions[1].params
            {
                *duration = None;
            }
        },
        |session| {
            if let ActionParams::DawSessionExport { duration, .. } =
                &mut session.action_log.actions[1].params
            {
                *duration = Some(LiveRecordingDuration::SixteenBars);
            }
        },
        |session| {
            if let ActionParams::DawSessionExport { boundary, .. } =
                &mut session.action_log.actions[1].params
            {
                *boundary = DawSessionExportBoundary::LiveMasterDawprojectV2;
            }
        },
        |session| {
            if let ActionParams::DawSessionExport {
                destination_path, ..
            } = &mut session.action_log.actions[1].params
            {
                *destination_path = Some("replacement.dawproject".into());
            }
        },
        |session| {
            session
                .export_receipts
                .push(session.export_receipts[0].clone())
        },
        |session| {
            session.export_receipts.remove(0);
        },
        |session| {
            session.export_receipts[0].live_recording_host_audio_refs[0]
                .stream_error_summary
                .error_count = 1
        },
        |session| session.action_log.actions[0].status = ActionStatus::Rejected,
        |session| session.export_receipts[1].artifact_set[2].sha256 = "9".repeat(64),
        |session| {
            session.export_receipts[1].artifact_set[2]
                .lineage_capture_refs
                .clear()
        },
        |session| session.export_receipts[1].live_recording_duration = None,
        |session| {
            session.action_log.actions.remove(1);
        },
    ];
    for (index, mutate) in mutations.iter().enumerate() {
        let mut session = original.clone();
        mutate(&mut session);
        assert!(
            session
                .validate_live_recording_duration_contracts()
                .is_err(),
            "mutation {index}"
        );
    }
}

#[test]
fn daw_v3_replay_validates_pins_without_opening_or_regenerating_archives() {
    let mut session = archive_session(LiveRecordingDuration::SixteenBars, 48_000, 120.0);
    let original = session.clone();
    assert!(
        apply_replay_plan_to_session(&mut session, &[])
            .unwrap()
            .applied_action_ids
            .is_empty()
    );
    assert_eq!(session, original);
    let id = SnapshotId::from("extended-daw");
    session.snapshots.push(Snapshot {
        snapshot_id: id.clone(),
        created_at: "generated".into(),
        label: "generated".into(),
        action_cursor: 2,
        payload: Some(SnapshotPayload::from_runtime_state(
            &id,
            2,
            &session.runtime_state,
        )),
    });
    let restored = hydrate_replay_target_from_snapshot_payload(&session, 2, None).unwrap();
    assert_eq!(restored.session.export_receipts, session.export_receipts);
    if let ActionParams::DawSessionExport { receipt_id, .. } =
        &mut session.action_log.actions[1].params
    {
        *receipt_id = Some("absent".into());
    }
    assert!(hydrate_replay_target_from_snapshot_payload(&session, 2, None).is_err());
    assert!(apply_replay_plan_to_session(&mut session, &[]).is_err());
}

#[test]
fn historical_daw_receipts_keep_omitted_duration_and_existing_archive_readiness() {
    for (boundary, action_boundary, pack) in [
        (
            ProductExportBoundary::DawSessionLiveMasterDawprojectV1,
            DawSessionExportBoundary::LiveMasterDawprojectV1,
            LIVE_MASTER_DAWPROJECT_PACK_ID,
        ),
        (
            ProductExportBoundary::DawSessionLiveMasterDawprojectV2,
            DawSessionExportBoundary::LiveMasterDawprojectV2,
            LIVE_MASTER_DAWPROJECT_V2_PACK_ID,
        ),
    ] {
        let receipt = daw_receipt(boundary, pack);
        assert!(receipt.live_master_dawproject_archive_ready());
        assert!(receipt.live_master_dawproject_action_contract_matches(action_boundary, None));
        assert!(!receipt.live_master_dawproject_action_contract_matches(
            action_boundary,
            Some(LiveRecordingDuration::EightBars)
        ));
        let bytes = serde_json::to_vec(&receipt).unwrap();
        let json = serde_json::to_value(&receipt).unwrap();
        assert!(json.get("live_recording_duration").is_none());
        let restored: ExportReceiptState = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
        let mut session = SessionFile::new("legacy", "generated", "generated");
        session.export_receipts.push(restored);
        session
            .validate_live_recording_duration_contracts()
            .unwrap();
    }
}

fn daw_action_params(
    boundary: DawSessionExportBoundary,
    duration: Option<LiveRecordingDuration>,
    source: &str,
) -> ActionParams {
    ActionParams::DawSessionExport {
        export_scope: ExportScope::DawSession,
        boundary,
        duration,
        include_manifest: true,
        destination_kind: ProductExportDestinationKind::LocalFilePath,
        destination_path: Some("generated.dawproject".into()),
        receipt_id: Some(source.into()),
    }
}

fn archive_session(duration: LiveRecordingDuration, rate: u32, bpm: f32) -> SessionFile {
    let mut session = window_session(duration);
    let source = &mut session.export_receipts[0];
    let micros = (f64::from(bpm) * 1_000_000.0).round() as u64;
    let frames = duration.target_frame_count(rate, micros).unwrap();
    let millis = (frames * 1_000 + u64::from(rate) / 2) / u64::from(rate);
    let audio = &mut source.artifact_set[0];
    let grid = ExportArtifactTimingGridRef {
        source_id: "generated-source".into(),
        hypothesis_id: None,
        confirmed_by_action: ActionId(1),
        confirmed_at: 1,
    };
    audio.timing_grid_ref = Some(grid.clone());
    audio.lineage_capture_refs = vec!["generated-capture".into()];
    audio.sample_rate_hz = Some(rate);
    audio.duration_ms = Some(millis);
    audio.audio_metrics.as_mut().unwrap().total_frame_count = Some(frames);
    source.live_recording_host_audio_refs[0].recording_duration_ms = millis;
    let window = source.live_recording_host_audio_refs[0]
        .timing_window
        .as_mut()
        .unwrap();
    let span = f64::from(bpm) / 60.0 / f64::from(rate);
    window.confirmed_bpm_micros = micros;
    window.beat_span_per_frame_nanobeats = (span * 1_000_000_000.0).round() as u64;
    window.captured_end_position_microbeats =
        (7_000_000.0 + frames as f64 * span * 1_000_000.0).round() as u64;
    window.duration_error_frame_micros =
        ((frames as f64 * span - f64::from(duration.duration_beats())).abs() / span * 1_000_000.0)
            .round() as u64;
    let mut archive = daw_receipt(
        ProductExportBoundary::DawSessionLiveMasterDawprojectV3,
        LIVE_MASTER_DAWPROJECT_V3_PACK_ID,
    );
    archive.receipt_id = ExportReceiptId::new("extended-daw-receipt");
    archive.created_by_action = ActionId(1486);
    archive.live_recording_duration = Some(duration);
    archive.artifact_set[2] = source.artifact_set[0].clone();
    archive.artifact_set[2].location = ExportArtifactLocation::Uri {
        uri: "generated.dawproject#audio/live_master.wav".into(),
    };
    archive.arrangement_placement_refs = vec![ExportArrangementPlacementRef::scene_range(
        "generated-scene",
        Some(grid.source_id.clone()),
        1,
        u32::from(duration.bars()),
        0,
        u64::from(duration.duration_beats()),
    )];
    archive.daw_tempo_map_ref = Some(ExportDawTempoMapRef::confirmed_grid(
        grid.source_id,
        grid.hypothesis_id,
        grid.confirmed_by_action,
        grid.confirmed_at,
        0,
        u64::from(duration.duration_beats()),
        u32::try_from(micros).unwrap(),
    ));
    let mut action = session.action_log.actions[0].clone();
    action.id = archive.created_by_action;
    action.command = ActionCommand::ExportDawSession;
    action.params = daw_action_params(
        DawSessionExportBoundary::LiveMasterDawprojectV3,
        Some(duration),
        source.receipt_id.as_str(),
    );
    session.action_log.actions.push(action);
    let mut commit = session.action_log.commit_records[0].clone();
    commit.action_id = archive.created_by_action;
    commit.commit_sequence = 2;
    session.action_log.commit_records.push(commit);
    session.export_receipts.push(archive);
    session
}
