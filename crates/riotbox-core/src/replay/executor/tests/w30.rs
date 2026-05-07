use super::*;
use crate::{
    action::{ActionTarget, TargetScope},
    ids::{BankId, CaptureId, PadId, SceneId},
    replay::{W30ArtifactReplayHydrationError, build_replay_target_plan},
    session::{CaptureRef, CaptureSourceWindow, CaptureTarget, CaptureType, W30PreviewModeState},
};

mod artifact_hydration;
mod capture_replay;
mod cue_moves;
mod promotion_replay;

fn w30_action(
    id: u64,
    command: ActionCommand,
    params: ActionParams,
    committed_at: TimestampMs,
) -> Action {
    targeted_action(
        id,
        command,
        params,
        ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-a")),
            pad_id: Some(PadId::from("pad-01")),
            ..Default::default()
        },
        committed_at,
    )
}

fn w30_capture_params(capture_id: &str, intensity: f32) -> ActionParams {
    ActionParams::Mutation {
        intensity,
        target_id: Some(CaptureId::from(capture_id).to_string()),
    }
}

fn w30_promotion_params(capture_id: &str, destination: &str) -> ActionParams {
    ActionParams::Promotion {
        capture_id: Some(CaptureId::from(capture_id)),
        destination: Some(destination.into()),
    }
}

fn source_capture(capture_id: &str) -> CaptureRef {
    CaptureRef {
        capture_id: CaptureId::from(capture_id),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["source-1".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: format!("captures/{capture_id}.wav"),
        assigned_target: None,
        is_pinned: false,
        notes: None,
    }
}

fn resample_capture_for_action(action_id: u64) -> CaptureRef {
    CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Resample,
        source_origin_refs: vec!["source-1".into()],
        source_window: None,
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 1,
        created_from_action: Some(ActionId(action_id)),
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: None,
        is_pinned: false,
        notes: None,
    }
}

fn w30_capture_to_pad_capture_for_action(action_id: u64) -> CaptureRef {
    CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["source-1".into()],
        source_window: Some(CaptureSourceWindow {
            source_id: "source-1".into(),
            start_seconds: 1.0,
            end_seconds: 3.0,
            start_frame: 48_000,
            end_frame: 144_000,
        }),
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: Some(ActionId(action_id)),
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: false,
        notes: None,
    }
}

fn capture_bar_group_capture_for_action(action_id: u64) -> CaptureRef {
    let mut capture = w30_capture_to_pad_capture_for_action(action_id);
    capture.assigned_target = None;
    capture.notes = Some("captured bar group".into());
    capture
}

fn loop_capture_for_action(action_id: u64) -> CaptureRef {
    let mut capture = w30_capture_to_pad_capture_for_action(action_id);
    capture.capture_type = CaptureType::Loop;
    capture.assigned_target = None;
    capture.notes = Some("captured loop".into());
    capture
}

fn loop_freeze_capture_for_action(action_id: u64, capture_id: &str) -> CaptureRef {
    CaptureRef {
        capture_id: CaptureId::from(capture_id),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["source-1".into()],
        source_window: None,
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 0,
        created_from_action: Some(ActionId(action_id)),
        storage_path: format!("captures/{capture_id}.wav"),
        assigned_target: None,
        is_pinned: true,
        notes: None,
    }
}
