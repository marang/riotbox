//! Generated metadata proof of version-selected canonical two-bar identity.
use super::export_live_recording_contract_tests::runtime_master_fixture_receipt;
use super::export_live_recording_window_tests::window_session;
use super::{
    ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactRole, ExportArtifactSetEntry,
    ExportReceiptState,
};
use crate::{
    action::{
        ActionParams, DawSessionExportBoundary, LiveRecordingDuration, LiveRecordingExportBoundary,
    },
    export_readiness::{
        ExportScope, LIVE_MASTER_DAWPROJECT_PACK_ID, LIVE_MASTER_DAWPROJECT_V2_PACK_ID,
        LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_PACK_ID,
        LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_V3_PACK_ID,
        LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_V4_PACK_ID, ProductExportBoundary,
        ProductExportRole,
    },
    ids::SnapshotId,
    replay::{apply_replay_plan_to_session, hydrate_replay_target_from_snapshot_payload},
    session::{ExportReceiptQaGateResult, SessionFile, Snapshot, SnapshotPayload},
};

#[test]
fn v4_two_bar_action_identity_is_distinct_from_v2_and_v3() {
    let boundary = LiveRecordingExportBoundary::RuntimeMasterBarWindowV4;
    assert!(boundary.valid_duration(Some(LiveRecordingDuration::TwoBars)));
    for duration in [
        None,
        Some(LiveRecordingDuration::EightBars),
        Some(LiveRecordingDuration::SixteenBars),
    ] {
        assert!(!boundary.valid_duration(duration));
    }
    assert_eq!(
        serde_json::to_value(boundary).unwrap(),
        "runtime_master_bar_window_v4"
    );
    assert_eq!(
        serde_json::from_value::<LiveRecordingExportBoundary>(serde_json::json!(
            "runtime_master_bar_window_v4"
        ))
        .unwrap(),
        boundary
    );
    assert_eq!(
        ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4.as_proof_str(),
        "live_recording.runtime_master_bar_window_v4"
    );
    assert_eq!(
        serde_json::to_value(DawSessionExportBoundary::LiveMasterDawprojectV2).unwrap(),
        "live_master_dawproject_v2"
    );
    assert_eq!(
        serde_json::from_value::<DawSessionExportBoundary>(serde_json::json!(
            "live_master_dawproject_v2"
        ))
        .unwrap(),
        DawSessionExportBoundary::LiveMasterDawprojectV2
    );
    assert_eq!(
        ProductExportBoundary::DawSessionLiveMasterDawprojectV2.as_proof_str(),
        "daw_session.live_master_dawproject_v2"
    );
}

#[test]
fn canonical_v4_two_bar_ties_are_ready_without_reinterpreting_legacy_v2() {
    for (bpm, expected_runtime_span) in [(121.5_f32, 42_187), (166.5_f32, 57_812)] {
        let session = v4_session(bpm, 48_000);
        session
            .validate_live_recording_duration_contracts()
            .unwrap();
        let receipt = &session.export_receipts[0];
        let live = &receipt.artifact_set[0];
        let frames = live
            .audio_metrics
            .as_ref()
            .unwrap()
            .total_frame_count
            .unwrap();
        let window = receipt.live_recording_host_audio_refs[0]
            .timing_window
            .as_ref()
            .unwrap();
        assert_eq!(window.beat_span_per_frame_nanobeats, expected_runtime_span);
        assert!(window.canonical_runtime_bar_window_ready(
            48_000,
            frames,
            LiveRecordingDuration::TwoBars
        ));
        assert!(!window.bar_aligned_two_bar_window_ready(48_000, frames));
        assert!(!window.bar_aligned_window_ready(48_000, frames, LiveRecordingDuration::TwoBars));
        assert!(receipt.live_recording_runtime_master_ready());
        let mut legacy = receipt.clone();
        legacy.export_boundary = ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2;
        legacy.pack_id = LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_PACK_ID.into();
        legacy.live_recording_duration = None;
        assert!(legacy.live_recording_host_audio_readiness_report().ready());
        assert!(!legacy.live_recording_runtime_master_ready());
    }
    let mut legacy_ready = v4_session(120.0, 48_000).export_receipts.remove(0);
    legacy_ready.export_boundary = ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2;
    legacy_ready.pack_id = LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_PACK_ID.into();
    legacy_ready.live_recording_duration = None;
    assert!(legacy_ready.live_recording_runtime_master_ready());
}

#[test]
fn canonical_v4_fractional_frame_rounding_differs_from_frozen_integer_micro_bpm() {
    for (bpm, expected_frames, old_frames) in [
        (80.6632_f32, 262_424, 262_425),
        (80.6838_f32, 262_358, 262_357),
    ] {
        let session = v4_session(bpm, 44_100);
        let receipt = &session.export_receipts[0];
        assert!(receipt.live_recording_runtime_master_ready());
        let frames = receipt.artifact_set[0]
            .audio_metrics
            .as_ref()
            .unwrap()
            .total_frame_count
            .unwrap();
        assert_eq!(frames, expected_frames);
        let window = receipt.live_recording_host_audio_refs[0]
            .timing_window
            .as_ref()
            .unwrap();
        let numerator = 44_100_u128 * 60 * 8 * 1_000_000;
        let micros = u128::from(window.confirmed_bpm_micros);
        assert_eq!((numerator + micros / 2) / micros, old_frames);
        assert!(!window.bar_aligned_two_bar_window_ready(44_100, frames));
        let mut wrong_frames = receipt.clone();
        wrong_frames.artifact_set[0]
            .audio_metrics
            .as_mut()
            .unwrap()
            .total_frame_count = Some(old_frames as u64);
        assert!(!wrong_frames.live_recording_runtime_master_ready());
    }
}

#[test]
fn v4_identity_roundtrips_and_rejects_missing_or_cross_version_duration() {
    let session = v4_session(121.5, 48_000);
    let serialized = serde_json::to_value(&session).unwrap();
    assert_eq!(
        serialized["export_receipts"][0]["live_recording_duration"],
        "two_bars"
    );
    assert_eq!(
        serialized["action_log"]["actions"][0]["params"]["LiveRecordingExport"]["duration"],
        "two_bars"
    );
    let restored: SessionFile = serde_json::from_value(serialized).unwrap();
    restored
        .validate_live_recording_duration_contracts()
        .unwrap();
    assert_eq!(session, restored);
    for duration in [
        None,
        Some(LiveRecordingDuration::EightBars),
        Some(LiveRecordingDuration::SixteenBars),
    ] {
        let mut invalid = session.clone();
        invalid.export_receipts[0].live_recording_duration = duration;
        assert!(!invalid.export_receipts[0].live_recording_runtime_master_ready());
        assert!(
            invalid
                .validate_live_recording_duration_contracts()
                .is_err()
        );
        let mut invalid = session.clone();
        let ActionParams::LiveRecordingExport {
            duration: selected, ..
        } = &mut invalid.action_log.actions[0].params
        else {
            unreachable!()
        };
        *selected = duration;
        assert!(
            invalid
                .validate_live_recording_duration_contracts()
                .is_err()
        );
    }
    for (boundary, pack) in [
        (
            ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2,
            LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_PACK_ID,
        ),
        (
            ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV3,
            LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_V3_PACK_ID,
        ),
    ] {
        let mut invalid = session.clone();
        invalid.export_receipts[0].export_boundary = boundary;
        invalid.export_receipts[0].pack_id = pack.into();
        assert!(!invalid.export_receipts[0].live_recording_runtime_master_ready());
        assert!(
            invalid
                .validate_live_recording_duration_contracts()
                .is_err()
        );
    }
    let mut missing = session.clone();
    missing.export_receipts.clear();
    assert!(
        missing
            .validate_live_recording_duration_contracts()
            .is_err()
    );
    let mut orphan = session;
    orphan.action_log.actions.clear();
    assert!(orphan.validate_live_recording_duration_contracts().is_err());
}

#[test]
fn v4_restore_and_replay_validate_canonical_identity_without_artifact_io() {
    let mut session = v4_session(80.6632, 44_100);
    let original = session.clone();
    assert!(
        apply_replay_plan_to_session(&mut session, &[])
            .unwrap()
            .applied_action_ids
            .is_empty()
    );
    assert_eq!(session, original);
    let id = SnapshotId::from("canonical-two-bar");
    session.snapshots.push(Snapshot {
        snapshot_id: id.clone(),
        created_at: "generated".into(),
        label: "generated".into(),
        action_cursor: 1,
        payload: Some(SnapshotPayload::from_runtime_state(
            &id,
            1,
            &session.runtime_state,
        )),
    });
    let restored = hydrate_replay_target_from_snapshot_payload(&session, 1, None).unwrap();
    assert_eq!(restored.session.export_receipts, session.export_receipts);
    session.export_receipts[0].live_recording_duration = None;
    assert!(hydrate_replay_target_from_snapshot_payload(&session, 1, None).is_err());
    assert!(apply_replay_plan_to_session(&mut session, &[]).is_err());
}

#[test]
fn daw_v2_archive_receipt_identity_and_gates_remain_version_specific() {
    for (boundary, pack) in [
        (
            ProductExportBoundary::DawSessionLiveMasterDawprojectV1,
            LIVE_MASTER_DAWPROJECT_PACK_ID,
        ),
        (
            ProductExportBoundary::DawSessionLiveMasterDawprojectV2,
            LIVE_MASTER_DAWPROJECT_V2_PACK_ID,
        ),
    ] {
        let receipt = daw_receipt(boundary, pack);
        assert_eq!(
            receipt.is_live_master_dawproject_v1(),
            pack == LIVE_MASTER_DAWPROJECT_PACK_ID
        );
        assert_eq!(
            receipt.is_live_master_dawproject_v2(),
            pack == LIVE_MASTER_DAWPROJECT_V2_PACK_ID
        );
        assert!(receipt.is_dawproject_archive_receipt());
        assert!(receipt.live_master_dawproject_archive_ready());
        let restored: ExportReceiptState =
            serde_json::from_value(serde_json::to_value(&receipt).unwrap()).unwrap();
        assert_eq!(restored, receipt);
        let mut wrong_pack = receipt.clone();
        wrong_pack.pack_id = if pack == LIVE_MASTER_DAWPROJECT_PACK_ID {
            LIVE_MASTER_DAWPROJECT_V2_PACK_ID
        } else {
            LIVE_MASTER_DAWPROJECT_PACK_ID
        }
        .into();
        assert!(!wrong_pack.live_master_dawproject_archive_ready());
        let mut duplicate = receipt.clone();
        duplicate
            .qa_gates
            .push(ExportReceiptQaGateResult::live_master_dawproject_archive_readback());
        assert!(!duplicate.live_master_dawproject_archive_ready());
        let mut wrong_member = receipt;
        wrong_member.artifact_set[2].location = ExportArtifactLocation::Uri {
            uri: "generated.dawproject#audio/replacement.wav".into(),
        };
        assert!(!wrong_member.live_master_dawproject_archive_ready());
    }
}

fn v4_session(bpm: f32, rate: u32) -> SessionFile {
    let mut session = window_session(LiveRecordingDuration::TwoBars);
    let ActionParams::LiveRecordingExport { boundary, .. } =
        &mut session.action_log.actions[0].params
    else {
        unreachable!()
    };
    *boundary = LiveRecordingExportBoundary::RuntimeMasterBarWindowV4;
    let receipt = &mut session.export_receipts[0];
    receipt.export_boundary = ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4;
    receipt.pack_id = LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_V4_PACK_ID.into();
    let micros = (f64::from(bpm) * 1_000_000.0).round() as u64;
    let frames = LiveRecordingDuration::TwoBars
        .target_frame_count(rate, micros)
        .unwrap();
    let millis = (frames * 1_000 + u64::from(rate) / 2) / u64::from(rate);
    receipt.artifact_set[0].sample_rate_hz = Some(rate);
    receipt.artifact_set[0].duration_ms = Some(millis);
    receipt.artifact_set[0]
        .audio_metrics
        .as_mut()
        .unwrap()
        .total_frame_count = Some(frames);
    receipt.live_recording_host_audio_refs[0].recording_duration_ms = millis;
    let window = receipt.live_recording_host_audio_refs[0]
        .timing_window
        .as_mut()
        .unwrap();
    let span = f64::from(bpm) / 60.0 / f64::from(rate);
    window.confirmed_bpm_micros = micros;
    window.beat_span_per_frame_nanobeats = (span * 1_000_000_000.0).round() as u64;
    window.captured_end_position_microbeats =
        (7.0 * 1_000_000.0 + frames as f64 * span * 1_000_000.0).round() as u64;
    window.duration_error_frame_micros =
        ((frames as f64 * span - 8.0).abs() / span * 1_000_000.0).round() as u64;
    session
}

pub(super) fn daw_receipt(boundary: ProductExportBoundary, pack: &str) -> ExportReceiptState {
    let mut receipt = runtime_master_fixture_receipt();
    let path = "generated.dawproject";
    let archive_hash = "a".repeat(64);
    let xml_hash = "b".repeat(64);
    receipt.export_scope = ExportScope::DawSession;
    receipt.export_boundary = boundary;
    receipt.pack_id = pack.into();
    receipt.export_role = ProductExportRole::ArrangementManifest;
    receipt.artifact_path = path.into();
    receipt.proof_path = path.into();
    receipt.manifest_path = Some(path.into());
    receipt.export_hash = archive_hash.clone();
    receipt.normalized_manifest_hash = xml_hash.clone();
    receipt.live_recording_host_audio_refs.clear();
    receipt.qa_gates = vec![
        ExportReceiptQaGateResult::dawproject_xml_document(),
        ExportReceiptQaGateResult::live_master_dawproject_archive_readback(),
    ];
    receipt.artifact_set = [
        (
            ExportArtifactRole::DawProjectFile,
            ExportArtifactMediaType::DawProjectZip,
            None,
        ),
        (
            ExportArtifactRole::ExportManifest,
            ExportArtifactMediaType::Xml,
            Some("project.xml"),
        ),
        (
            ExportArtifactRole::LiveRecordingCapture,
            ExportArtifactMediaType::AudioWav,
            Some("audio/live_master.wav"),
        ),
        (
            ExportArtifactRole::DawProjectProof,
            ExportArtifactMediaType::Json,
            Some("riotbox-proof.json"),
        ),
    ]
    .into_iter()
    .map(|(role, media, member)| {
        let mut entry = ExportArtifactSetEntry::product_export_proof(
            path,
            if role == ExportArtifactRole::DawProjectFile {
                archive_hash.clone()
            } else {
                xml_hash.clone()
            },
        );
        entry.role = role;
        entry.media_type = media;
        entry.location = member.map_or_else(
            || ExportArtifactLocation::LocalPath { path: path.into() },
            |member| ExportArtifactLocation::Uri {
                uri: format!("{path}#{member}"),
            },
        );
        entry.normalized_manifest_hash =
            (role == ExportArtifactRole::DawProjectFile).then(|| xml_hash.clone());
        entry
    })
    .collect();
    receipt
}
