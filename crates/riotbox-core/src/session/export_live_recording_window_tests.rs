use super::ExportReceiptState;
use super::export_live_recording_contract_tests::runtime_master_fixture_receipt;
use crate::{
    action::{
        Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
        CommitBoundary, LiveRecordingDuration, LiveRecordingExportBoundary,
        LiveRecordingExportRole, Quantization, UndoPolicy,
    },
    export_readiness::{
        ExportScope, LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_V3_PACK_ID, ProductExportBoundary,
        ProductExportDestinationKind,
    },
    ids::SnapshotId,
    replay::{
        ReplayExecutionError, apply_replay_plan_to_session,
        hydrate_replay_target_from_snapshot_payload,
    },
    session::{
        ActionCommitRecord, ExportLiveRecordingTimingWindow, ExportReceiptQaGateResult,
        LiveRecordingDurationContractError, SessionFile, Snapshot, SnapshotPayload,
    },
    transport::CommitBoundaryState,
};

#[test]
fn recording_duration_enum_is_bounded_and_versioned_without_changing_legacy_defaults() {
    assert_eq!(
        LiveRecordingDuration::default(),
        LiveRecordingDuration::TwoBars
    );
    for (duration, bars, name) in [
        (LiveRecordingDuration::TwoBars, 2, "two_bars"),
        (LiveRecordingDuration::EightBars, 8, "eight_bars"),
        (LiveRecordingDuration::SixteenBars, 16, "sixteen_bars"),
    ] {
        assert_eq!(duration.bars(), bars);
        assert_eq!(duration.duration_beats(), u32::from(bars) * 4);
        assert_eq!(serde_json::to_value(duration).unwrap(), name);
        for boundary in [
            LiveRecordingExportBoundary::ReservedContractOnly,
            LiveRecordingExportBoundary::RuntimeMasterCaptureV1,
            LiveRecordingExportBoundary::RuntimeMasterBarWindowV2,
        ] {
            assert!(boundary.valid_duration(None));
            assert!(!boundary.valid_duration(Some(duration)));
        }
        assert_eq!(
            LiveRecordingExportBoundary::RuntimeMasterBarWindowV3.valid_duration(Some(duration)),
            bars != 2
        );
    }
    assert!(!LiveRecordingExportBoundary::RuntimeMasterBarWindowV3.valid_duration(None));
    for unsupported in ["four_bars", "thirty_two_bars", "arbitrary", "8"] {
        assert!(serde_json::from_value::<LiveRecordingDuration>(unsupported.into()).is_err());
    }
}

#[test]
fn frame_geometry_is_checked_and_rounded_once_for_each_supported_window() {
    for duration in [
        LiveRecordingDuration::EightBars,
        LiveRecordingDuration::SixteenBars,
    ] {
        assert_eq!(
            duration.target_frame_count(48_000, 120_000_000),
            Some(u64::from(duration.duration_beats()) * 24_000)
        );
        assert_eq!(duration.target_frame_count(0, 120_000_000), None);
        assert_eq!(duration.target_frame_count(48_000, 0), None);
        assert!(duration.target_frame_count(u32::MAX, 1).is_some());
        assert_eq!(duration.target_frame_count(1, u64::MAX), None);
    }
    assert_eq!(
        LiveRecordingDuration::TwoBars.target_frame_count(48_000, 130_000_000),
        Some(177_231)
    );
    assert_eq!(
        LiveRecordingDuration::EightBars.target_frame_count(44_100, 137_300_003),
        Some(616_693)
    );
    assert_eq!(
        LiveRecordingDuration::SixteenBars.target_frame_count(44_100, 137_300_003),
        Some(1_233_387)
    );
}

#[test]
fn fractional_runtime_tempos_preserve_half_frame_gate_instead_of_rounding_micro_bpm() {
    for (bpm, rate, duration, expected_frames) in [
        (
            80.0082_f32,
            44_100_u32,
            LiveRecordingDuration::EightBars,
            1_058_292,
        ),
        (
            90.049_f32,
            48_000_u32,
            LiveRecordingDuration::SixteenBars,
            2_046_886,
        ),
    ] {
        let micros = (f64::from(bpm) * 1_000_000.0).round() as u64;
        assert_eq!(
            LiveRecordingDuration::confirmed_runtime_bpm(micros)
                .unwrap()
                .to_bits(),
            bpm.to_bits()
        );
        assert_eq!(
            duration.target_frame_count(rate, micros),
            Some(expected_frames)
        );
        assert!(LiveRecordingDuration::confirmed_runtime_bpm(micros + 1).is_none());
        let exact_frames =
            f64::from(rate) * 60.0 * f64::from(duration.duration_beats()) / f64::from(bpm);
        let integer_micro_frames =
            (u128::from(rate) * 60 * u128::from(duration.duration_beats()) * 1_000_000
                + u128::from(micros) / 2)
                / u128::from(micros);
        assert_ne!(integer_micro_frames, u128::from(expected_frames));
        assert!((integer_micro_frames as f64 - exact_frames).abs() > 0.500_001);
        let frame_error = (expected_frames as f64 - exact_frames).abs();
        assert!(frame_error <= 0.500_001);

        let mut receipt = window_receipt(duration);
        let millis = (expected_frames * 1_000 + u64::from(rate) / 2) / u64::from(rate);
        let live = &mut receipt.artifact_set[0];
        live.sample_rate_hz = Some(rate);
        live.duration_ms = Some(millis);
        live.audio_metrics.as_mut().unwrap().total_frame_count = Some(expected_frames);
        receipt.live_recording_host_audio_refs[0].recording_duration_ms = millis;
        let window = receipt.live_recording_host_audio_refs[0]
            .timing_window
            .as_mut()
            .unwrap();
        let beats_per_frame = f64::from(bpm) / 60.0 / f64::from(rate);
        window.confirmed_bpm_micros = micros;
        window.beat_span_per_frame_nanobeats = (beats_per_frame * 1_000_000_000.0).round() as u64;
        window.captured_end_position_microbeats = window.captured_start_position_microbeats
            + (expected_frames as f64 * beats_per_frame * 1_000_000.0).round() as u64;
        window.duration_error_frame_micros = (frame_error * 1_000_000.0).round() as u64;
        assert!(receipt.live_recording_runtime_master_ready());
        receipt.artifact_set[0]
            .audio_metrics
            .as_mut()
            .unwrap()
            .total_frame_count = Some(integer_micro_frames as u64);
        assert!(!receipt.live_recording_runtime_master_ready());
    }
}

#[test]
fn generated_fractional_tempos_match_runtime_frame_geometry_across_output_rates() {
    for rate in [8_000, 44_100, 48_000, 96_000, 192_000] {
        for duration in [
            LiveRecordingDuration::EightBars,
            LiveRecordingDuration::SixteenBars,
        ] {
            for index in 0..2_000 {
                let bpm = (20.0 + f64::from(index) * 0.137_031) as f32;
                let micros = (f64::from(bpm) * 1_000_000.0).round() as u64;
                let exact =
                    f64::from(rate) * 60.0 * f64::from(duration.duration_beats()) / f64::from(bpm);
                let frames = duration.target_frame_count(rate, micros).unwrap();
                assert_eq!(frames, exact.round() as u64);
                assert!((frames as f64 - exact).abs() <= 0.500_001);
            }
        }
    }
}

#[test]
fn v3_roundoff_budget_admits_actual_clock_accumulation_but_not_frame_displacement() {
    let duration = LiveRecordingDuration::SixteenBars;
    let frames = 2_685_902;
    let span = 137.25 / 60.0 / 96_000.0;
    let start = 4.000_003_515_624_997;
    let end = 68.000_012_109_405_22;
    let expected_end = start + frames as f64 * span;
    let count = 20_985;
    let bound = duration
        .position_roundoff_bound(span, start, end, expected_end, count, frames)
        .unwrap();
    let residual = (end - expected_end).abs();
    assert!(residual > span * 1.0e-6);
    assert!(residual <= bound);
    assert!(bound <= span / 1_024.0);
    assert!(bound <= 1.0e-6);
    assert!(bound < span * 0.5);
    assert!(bound < (end + span - expected_end).abs());
    // Canonical V4 can use the same bounded arithmetic for TwoBars; historical
    // V2 callers keep their own unchanged representation check.
    assert_eq!(
        LiveRecordingDuration::TwoBars.position_roundoff_bound(
            span,
            start,
            end,
            expected_end,
            count,
            frames
        ),
        Some(bound)
    );
}

#[test]
fn v3_roundoff_budget_rejects_invalid_counts_positions_or_excessive_precision_loss() {
    let duration = LiveRecordingDuration::EightBars;
    let span = 120.0 / 60.0 / 48_000.0;
    let frames = 768_000;
    for (span, start, end, expected_end, callbacks, frames) in [
        (span, 4.0, 36.0, 36.0, 0, frames),
        (span, 4.0, 36.0, 36.0, frames + 1, frames),
        (span, 4.0, 36.0, 36.0, 1, 0),
        (span, -1.0, 36.0, 36.0, 1, frames),
        (span, f64::NAN, 36.0, 36.0, 1, frames),
        (span, 4.0, f64::INFINITY, 36.0, 1, frames),
        (span, 4.0, 36.0, f64::NAN, 1, frames),
        (span, 4.0, 4.0, 36.0, 1, frames),
        (span, 4.0, 36.0, 4.0, 1, frames),
        (0.0, 4.0, 36.0, 36.0, 1, frames),
        (f64::NAN, 4.0, 36.0, 36.0, 1, frames),
        (span, 1.0e12, 1.0e12 + 32.0, 1.0e12 + 32.0, 1, frames),
        (span, 4.0, 36.0, 36.0, u64::MAX, u64::MAX),
    ] {
        assert!(
            duration
                .position_roundoff_bound(span, start, end, expected_end, callbacks, frames)
                .is_none()
        );
    }
    let bound = duration
        .position_roundoff_bound(span, 4.0, 36.0, 36.0, 1, frames)
        .unwrap();
    assert_eq!(bound, span * 1.0e-6);
    // A large beat coordinate with a large frame span hits the serialization
    // ceiling even when it could satisfy the sub-frame representation ceiling.
    assert!(
        duration
            .position_roundoff_bound(1.0, 1.0e9, 1.0e9 + 32.0, 1.0e9 + 32.0, 1, 32)
            .is_none()
    );
}

#[test]
fn v3_frame_span_uses_the_exact_callback_division_order_at_half_nanobeats() {
    for bpm in [121.5_f32, 166.5_f32] {
        let duration = LiveRecordingDuration::EightBars;
        let rate = 48_000;
        let micros = (f64::from(bpm) * 1_000_000.0).round() as u64;
        let frames = duration.target_frame_count(rate, micros).unwrap();
        let exact_frames =
            f64::from(rate) * 60.0 * f64::from(duration.duration_beats()) / f64::from(bpm);
        let span = f64::from(bpm) / 60.0 / f64::from(rate);
        let runtime_nanobeats = (span * 1_000_000_000.0).round() as u64;
        let reordered_nanobeats =
            (f64::from(bpm) / (60.0 * f64::from(rate)) * 1_000_000_000.0).round() as u64;
        assert_ne!(runtime_nanobeats, reordered_nanobeats);
        let mut window = ExportLiveRecordingTimingWindow {
            confirmed_bpm_micros: micros,
            bar_grid_anchor_position_microbeats: 3_000_000,
            beat_span_per_frame_nanobeats: runtime_nanobeats,
            requested_start_position_microbeats: 7_000_000,
            captured_start_position_microbeats: 7_000_000,
            captured_end_position_microbeats: 7_000_000
                + (frames as f64 * span * 1_000_000.0).round() as u64,
            start_alignment_error_frame_micros: 0,
            duration_error_frame_micros: ((frames as f64 - exact_frames).abs() * 1_000_000.0)
                .round() as u64,
            beats_per_bar: 4,
            duration_beats: duration.duration_beats(),
        };
        assert!(window.bar_aligned_window_ready(rate, frames, duration));
        window.beat_span_per_frame_nanobeats = reordered_nanobeats;
        assert!(!window.bar_aligned_window_ready(rate, frames, duration));
    }
}

#[test]
fn v3_windows_roundtrip_and_crossmatch_action_receipt_timing_identity() {
    for duration in [
        LiveRecordingDuration::EightBars,
        LiveRecordingDuration::SixteenBars,
    ] {
        let session = window_session(duration);
        let receipt = &session.export_receipts[0];
        assert!(receipt.is_live_recording_runtime_master_bar_window_v3());
        assert!(!receipt.is_live_recording_runtime_master_bar_window_v2());
        assert!(receipt.live_recording_runtime_master_ready());
        assert!(receipt.live_recording_action_contract_matches(
            LiveRecordingExportBoundary::RuntimeMasterBarWindowV3,
            Some(duration)
        ));
        session
            .validate_live_recording_duration_contracts()
            .unwrap();
        let json = serde_json::to_value(&session).unwrap();
        assert_eq!(
            json["action_log"]["actions"][0]["params"]["LiveRecordingExport"]["duration"],
            serde_json::to_value(duration).unwrap()
        );
        assert_eq!(
            json["export_receipts"][0]["live_recording_duration"],
            serde_json::to_value(duration).unwrap()
        );
        let restored: SessionFile = serde_json::from_value(json).unwrap();
        assert_eq!(restored, session);
        restored
            .validate_live_recording_duration_contracts()
            .unwrap();
    }
}

#[test]
fn v3_readiness_rejects_missing_unsupported_or_contradictory_window_identity() {
    let receipt = window_receipt(LiveRecordingDuration::EightBars);
    for duration in [
        None,
        Some(LiveRecordingDuration::TwoBars),
        Some(LiveRecordingDuration::SixteenBars),
    ] {
        let mut invalid = receipt.clone();
        invalid.live_recording_duration = duration;
        assert!(!invalid.live_recording_runtime_master_ready());
    }
    let mut invalid = receipt.clone();
    invalid.live_recording_host_audio_refs[0].timing_window = None;
    assert!(!invalid.live_recording_runtime_master_ready());
    let mut invalid = receipt.clone();
    invalid.artifact_set[0]
        .audio_metrics
        .as_mut()
        .unwrap()
        .total_frame_count = Some(768_001);
    assert!(!invalid.live_recording_runtime_master_ready());
    let mut invalid = receipt.clone();
    invalid.live_recording_host_audio_refs[0].recording_duration_ms += 1;
    invalid.artifact_set[0].duration_ms =
        Some(invalid.live_recording_host_audio_refs[0].recording_duration_ms);
    assert!(!invalid.live_recording_runtime_master_ready());
    let mut invalid = receipt.clone();
    invalid.live_recording_host_audio_refs[0]
        .timing_window
        .as_mut()
        .unwrap()
        .captured_end_position_microbeats += 1_000;
    assert!(!invalid.live_recording_runtime_master_ready());
    let mut invalid = receipt.clone();
    invalid
        .qa_gates
        .push(ExportReceiptQaGateResult::live_recording_bar_window_alignment());
    assert!(!invalid.live_recording_runtime_master_ready());
    let mut invalid = receipt;
    invalid.export_boundary = ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2;
    invalid.pack_id =
        crate::export_readiness::LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_PACK_ID.into();
    assert!(!invalid.live_recording_runtime_master_ready());
}

#[test]
fn legacy_receipts_omit_duration_and_cannot_be_relabelled_as_longer_windows() {
    let receipt = runtime_master_fixture_receipt();
    assert!(
        serde_json::to_value(&receipt)
            .unwrap()
            .get("live_recording_duration")
            .is_none()
    );
    let mut explicit = receipt.clone();
    explicit.live_recording_duration = Some(LiveRecordingDuration::EightBars);
    assert!(!explicit.live_recording_runtime_master_ready());
    assert!(!receipt.live_recording_action_contract_matches(
        LiveRecordingExportBoundary::RuntimeMasterBarWindowV3,
        Some(LiveRecordingDuration::EightBars)
    ));
    let mut legacy_session = SessionFile::new("legacy", "test", "test");
    legacy_session.export_receipts.push(receipt);
    legacy_session
        .validate_live_recording_duration_contracts()
        .unwrap();
}

#[test]
fn session_contract_rejects_missing_duplicate_or_mismatched_action_receipt_pairs() {
    let session = window_session(LiveRecordingDuration::EightBars);
    let mut invalid = session.clone();
    invalid.export_receipts.clear();
    assert!(matches!(
        invalid.validate_live_recording_duration_contracts(),
        Err(LiveRecordingDurationContractError::MissingReceipt { .. })
    ));
    let mut invalid = session.clone();
    invalid.action_log.actions.clear();
    assert!(matches!(
        invalid.validate_live_recording_duration_contracts(),
        Err(LiveRecordingDurationContractError::MissingAction { .. })
    ));
    let mut invalid = session.clone();
    invalid
        .export_receipts
        .push(invalid.export_receipts[0].clone());
    assert!(matches!(
        invalid.validate_live_recording_duration_contracts(),
        Err(LiveRecordingDurationContractError::AmbiguousReceipt { .. })
    ));
    let mut invalid = session.clone();
    invalid
        .action_log
        .actions
        .push(invalid.action_log.actions[0].clone());
    assert!(matches!(
        invalid.validate_live_recording_duration_contracts(),
        Err(LiveRecordingDurationContractError::AmbiguousAction { .. })
    ));
    let mut invalid = session.clone();
    let ActionParams::LiveRecordingExport { duration, .. } =
        &mut invalid.action_log.actions[0].params
    else {
        unreachable!()
    };
    *duration = Some(LiveRecordingDuration::SixteenBars);
    assert!(matches!(
        invalid.validate_live_recording_duration_contracts(),
        Err(LiveRecordingDurationContractError::ActionReceiptMismatch { .. })
    ));
    let mut invalid = session;
    invalid.action_log.actions[0].status = ActionStatus::Rejected;
    assert!(matches!(
        invalid.validate_live_recording_duration_contracts(),
        Err(LiveRecordingDurationContractError::InvalidAction { .. })
    ));
    invalid.export_receipts.clear();
    invalid
        .validate_live_recording_duration_contracts()
        .unwrap();
}

#[test]
fn replay_and_snapshot_restore_validate_receipts_without_reproducing_export_side_effects() {
    let mut session = window_session(LiveRecordingDuration::SixteenBars);
    let before = session.clone();
    assert!(
        apply_replay_plan_to_session(&mut session, &[])
            .unwrap()
            .applied_action_ids
            .is_empty()
    );
    assert_eq!(session, before);
    let id = SnapshotId::from("after-recording");
    session.snapshots.push(Snapshot {
        snapshot_id: id.clone(),
        created_at: "test".into(),
        label: "test".into(),
        action_cursor: 1,
        payload: Some(SnapshotPayload::from_runtime_state(
            &id,
            1,
            &session.runtime_state,
        )),
    });
    let restored = hydrate_replay_target_from_snapshot_payload(&session, 1, None).unwrap();
    assert_eq!(restored.session.export_receipts, session.export_receipts);
    session.export_receipts[0].live_recording_duration = Some(LiveRecordingDuration::EightBars);
    let corrupted = session.clone();
    assert!(matches!(
        apply_replay_plan_to_session(&mut session, &[]),
        Err(ReplayExecutionError::InvalidLiveRecordingDuration(_))
    ));
    assert_eq!(session, corrupted);
    assert!(hydrate_replay_target_from_snapshot_payload(&session, 1, None).is_err());
}

fn window_receipt(duration: LiveRecordingDuration) -> ExportReceiptState {
    let mut receipt = runtime_master_fixture_receipt();
    receipt.pack_id = LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_V3_PACK_ID.into();
    receipt.export_boundary = ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV3;
    receipt.live_recording_duration = Some(duration);
    receipt
        .qa_gates
        .push(ExportReceiptQaGateResult::live_recording_bar_window_alignment());
    let duration_ms = u64::from(duration.duration_beats()) * 500;
    receipt.artifact_set[0].duration_ms = Some(duration_ms);
    receipt.artifact_set[0]
        .audio_metrics
        .as_mut()
        .unwrap()
        .total_frame_count = duration.target_frame_count(48_000, 120_000_000);
    receipt.live_recording_host_audio_refs[0].recording_duration_ms = duration_ms;
    receipt.live_recording_host_audio_refs[0].timing_window =
        Some(ExportLiveRecordingTimingWindow {
            confirmed_bpm_micros: 120_000_000,
            bar_grid_anchor_position_microbeats: 3_000_000,
            beat_span_per_frame_nanobeats: 41_667,
            requested_start_position_microbeats: 7_000_000,
            captured_start_position_microbeats: 7_000_000,
            captured_end_position_microbeats: 7_000_000
                + u64::from(duration.duration_beats()) * 1_000_000,
            start_alignment_error_frame_micros: 0,
            duration_error_frame_micros: 0,
            beats_per_bar: 4,
            duration_beats: duration.duration_beats(),
        });
    receipt
}

pub(super) fn window_session(duration: LiveRecordingDuration) -> SessionFile {
    let mut session = SessionFile::new("bounded-window-test", "test", "test");
    let receipt = window_receipt(duration);
    session.action_log.actions.push(Action {
        id: receipt.created_by_action,
        actor: ActorType::User,
        command: ActionCommand::ExportLiveRecording,
        params: ActionParams::LiveRecordingExport {
            export_scope: ExportScope::LiveRecording,
            export_role: LiveRecordingExportRole::LiveRecordingCapture,
            boundary: LiveRecordingExportBoundary::RuntimeMasterBarWindowV3,
            duration: Some(duration),
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalFilePath,
            destination_path: Some(receipt.artifact_path.clone()),
            receipt_id: None,
        },
        target: ActionTarget::default(),
        requested_at: 1,
        quantization: Quantization::Immediate,
        status: ActionStatus::Committed,
        committed_at: Some(receipt.created_at),
        result: Some(ActionResult {
            accepted: true,
            summary: "generated metadata".into(),
        }),
        undo_policy: UndoPolicy::NotUndoable {
            reason: "export".into(),
        },
        explanation: None,
    });
    session.action_log.commit_records.push(ActionCommitRecord {
        action_id: receipt.created_by_action,
        boundary: CommitBoundaryState {
            kind: CommitBoundary::Immediate,
            beat_index: 0,
            bar_index: 0,
            phrase_index: 0,
            scene_id: None,
        },
        commit_sequence: 1,
        committed_at: receipt.created_at,
        mc202_source_phrase_plan: None,
    });
    session.export_receipts.push(receipt);
    session
}
