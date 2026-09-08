//! Admits one pinned V2 receipt using only its exact WAV and proof files.
//! Hashing, decoding, metrics and semantic identity use the same opened bytes.

use crate::jam_app::{
    JamAppError, LiveMasterRecordingProof,
    live_master_recording::{
        LIVE_MASTER_RECORDING_DURATION_BEATS, LIVE_MASTER_RECORDING_PROOF_SCHEMA,
        decode_recorded_float32_wav, recorded_float32_sample_payload_sha256,
    },
};
use riotbox_audio::runtime::signal_metrics;
use riotbox_core::{
    action::{ActionCommand, ActionParams, ActionStatus, LiveRecordingExportBoundary},
    export_readiness::{ProductExportBoundary, ProductExportDestinationKind},
    ids::ExportReceiptId,
    session::{
        ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactRole,
        ExportArtifactSetEntry, ExportReceiptState, SessionFile,
    },
};
use sha2::{Digest, Sha256};
use std::{
    fs::OpenOptions,
    io::Read,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

pub(super) struct LiveMasterDawprojectInput {
    pub(super) source_receipt_id: ExportReceiptId,
    pub(super) source_proof_sha256: String,
    pub(super) source_wav_sha256: String,
    pub(super) source_wav_bytes: Vec<u8>,
    pub(super) proof: LiveMasterRecordingProof,
    pub(super) source_artifact: ExportArtifactSetEntry,
}

pub(super) fn latest_live_master_receipt(session: &SessionFile) -> Option<&ExportReceiptState> {
    session.export_receipts.iter().rev().find(|receipt| {
        receipt.export_boundary == ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2
    })
}

pub(super) fn validate_queue_source_receipt(session: &SessionFile) -> Result<(), JamAppError> {
    let receipt = latest_live_master_receipt(session).ok_or_else(|| {
        JamAppError::InvalidSession(
            "live-master DAWproject export requires a Session-owned V2 live-master receipt".into(),
        )
    })?;
    unique_pinned_receipt(session, &receipt.receipt_id)?;
    if !receipt.live_recording_runtime_master_ready() {
        return Err(JamAppError::InvalidSession(
            "latest V2 live-master receipt is not ready; no older take may be substituted".into(),
        ));
    }
    Ok(())
}

pub(super) fn prepare_input(
    session: &SessionFile,
    base_dir: Option<&Path>,
    source_receipt_id: &ExportReceiptId,
) -> Result<LiveMasterDawprojectInput, JamAppError> {
    let receipt = Some(unique_pinned_receipt(session, source_receipt_id)?)
        .filter(|receipt| {
            receipt.export_boundary == ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2
        })
        .ok_or_else(|| {
            JamAppError::InvalidSession(
                "pinned live-master DAWproject source receipt is absent or not V2".into(),
            )
        })?;
    if !receipt.live_recording_runtime_master_ready() {
        return Err(JamAppError::InvalidSession(
            "live-master source receipt is not ready".into(),
        ));
    }
    let (wav, proof_artifact) = unique_source_artifacts(receipt)?;
    let (_, wav_bytes, wav_sha256) = read_exact_regular_artifact(wav, base_dir, "live-master WAV")?;
    let (_, proof_bytes, proof_sha256) =
        read_exact_regular_artifact(proof_artifact, base_dir, "live-master proof")?;
    if wav_sha256 != wav.sha256
        || wav_sha256 != receipt.export_hash
        || proof_sha256 != proof_artifact.sha256
        || proof_sha256 != receipt.normalized_manifest_hash
    {
        return Err(JamAppError::InvalidSession(
            "live-master receipt artifact hash drift".into(),
        ));
    }
    let proof: LiveMasterRecordingProof = serde_json::from_slice(&proof_bytes)?;
    validate_source_identity(session, receipt, wav, proof_artifact, &proof, &wav_bytes)?;
    Ok(LiveMasterDawprojectInput {
        source_receipt_id: receipt.receipt_id.clone(),
        source_proof_sha256: proof_sha256,
        source_wav_sha256: wav_sha256,
        source_wav_bytes: wav_bytes,
        proof,
        source_artifact: wav.clone(),
    })
}

fn unique_pinned_receipt<'a>(
    session: &'a SessionFile,
    receipt_id: &ExportReceiptId,
) -> Result<&'a ExportReceiptState, JamAppError> {
    let mut matches = session
        .export_receipts
        .iter()
        .filter(|receipt| receipt.receipt_id == *receipt_id);
    match (matches.next(), matches.next()) {
        (Some(receipt), None) => Ok(receipt),
        _ => Err(JamAppError::InvalidSession(
            "live-master source receipt ID must resolve to exactly one stored receipt".into(),
        )),
    }
}

fn unique_source_artifacts(
    receipt: &ExportReceiptState,
) -> Result<(&ExportArtifactSetEntry, &ExportArtifactSetEntry), JamAppError> {
    let wavs = receipt
        .artifact_set
        .iter()
        .filter(|entry| entry.role == ExportArtifactRole::LiveRecordingCapture)
        .collect::<Vec<_>>();
    let proofs = receipt
        .artifact_set
        .iter()
        .filter(|entry| entry.role == ExportArtifactRole::ProductExportProof)
        .collect::<Vec<_>>();
    if wavs.len() != 1
        || proofs.len() != 1
        || wavs[0].media_type != ExportArtifactMediaType::AudioWav
        || proofs[0].media_type != ExportArtifactMediaType::Json
    {
        return Err(JamAppError::InvalidSession(
            "live-master V2 receipt must declare exactly one WAV and one proof".into(),
        ));
    }
    Ok((wavs[0], proofs[0]))
}

fn validate_source_identity(
    session: &SessionFile,
    receipt: &ExportReceiptState,
    wav: &ExportArtifactSetEntry,
    proof_artifact: &ExportArtifactSetEntry,
    proof: &LiveMasterRecordingProof,
    wav_bytes: &[u8],
) -> Result<(), JamAppError> {
    if proof.schema != LIVE_MASTER_RECORDING_PROOF_SCHEMA
        || proof.receipt_id != receipt.receipt_id
        || proof.action_id != receipt.created_by_action
        || proof.session_id != session.session_id
        || proof.wav_sha256 != receipt.export_hash
        || proof.wav_sha256 != wav.sha256
        || proof.callback_count == 0
        || proof.armed_callback_count == 0
        || proof.active_sample_count == 0
        || proof.callback_scratch_overflow_count != 0
        || proof.stream_error_count != 0
        || proof.transport_mismatch_count != 0
        || proof.tempo_mismatch_count != 0
        || proof.timing_window_mismatch_count != 0
        || proof.clip_count != 0
        || proof.duration_beats != LIVE_MASTER_RECORDING_DURATION_BEATS
        || proof.beats_per_bar != 4
        || proof.wav_sample_format != "ieee_float32"
    {
        return Err(JamAppError::InvalidSession(
            "live-master proof does not match its V2 receipt identity".into(),
        ));
    }
    let source_action = session
        .action_log
        .actions
        .iter()
        .find(|action| action.id == proof.action_id)
        .ok_or_else(|| {
            JamAppError::InvalidSession(
                "live-master proof action is absent from the Session log".into(),
            )
        })?;
    let action_destination = match &source_action.params {
        ActionParams::LiveRecordingExport {
            boundary: LiveRecordingExportBoundary::RuntimeMasterBarWindowV2,
            destination_kind: ProductExportDestinationKind::LocalFilePath,
            destination_path: Some(path),
            ..
        } if source_action.command == ActionCommand::ExportLiveRecording
            && source_action.status == ActionStatus::Committed =>
        {
            path
        }
        _ => {
            return Err(JamAppError::InvalidSession(
                "live-master proof action is not the committed V2 recording action".into(),
            ));
        }
    };
    if source_action.target.scene_id.as_ref() != Some(&proof.scene_id)
        || !session
            .runtime_state
            .scene_state
            .scenes
            .contains(&proof.scene_id)
        || action_destination != &receipt.artifact_path
        || proof_artifact.location_identity() != receipt.proof_path
    {
        return Err(JamAppError::InvalidSession(
            "live-master action and receipt paths disagree".into(),
        ));
    }
    let frame_count = usize::try_from(proof.frame_count).map_err(|_| {
        JamAppError::InvalidSession("live-master proof frame count is too large".into())
    })?;
    let samples = decode_recorded_float32_wav(
        wav_bytes,
        proof.sample_rate_hz,
        proof.channel_count,
        frame_count,
    )?;
    if recorded_float32_sample_payload_sha256(&samples) != proof.sample_payload_sha256 {
        return Err(JamAppError::InvalidSession(
            "live-master WAV sample payload hash drift".into(),
        ));
    }
    let actual = signal_metrics(&samples);
    let peak = (f64::from(actual.peak_abs.max(0.0)) * 1_000_000.0).round() as u32;
    let rms = (f64::from(actual.rms.max(0.0)) * 1_000_000.0).round() as u32;
    if actual.active_samples as u64 != proof.active_sample_count
        || actual.clip_count as u64 != proof.clip_count
        || peak != proof.peak_amplitude_micros
        || rms != proof.rms_amplitude_micros
    {
        return Err(JamAppError::InvalidSession(
            "live-master WAV signal metrics disagree with proof".into(),
        ));
    }
    let metrics = wav
        .audio_metrics
        .as_ref()
        .ok_or_else(|| JamAppError::InvalidSession("live-master WAV is missing metrics".into()))?;
    if wav.sample_rate_hz != Some(proof.sample_rate_hz)
        || wav.channel_count != Some(proof.channel_count)
        || metrics.total_frame_count != Some(proof.frame_count)
        || metrics.peak_amplitude_micros != Some(proof.peak_amplitude_micros)
        || metrics.rms_amplitude_micros != Some(proof.rms_amplitude_micros)
    {
        return Err(JamAppError::InvalidSession(
            "live-master WAV format or metrics disagree with proof".into(),
        ));
    }
    let host = receipt.live_recording_host_audio_refs.as_slice();
    if host.len() != 1
        || host[0].host != proof.host
        || host[0].device != proof.device
        || host[0].callback_gap_summary.max_gap_ms
            != proof
                .max_callback_gap_micros
                .map(|value| value.div_ceil(1_000))
        || host[0].callback_gap_summary.over_threshold_count
            != u32::try_from(proof.callback_gap_over_threshold_count).unwrap_or(u32::MAX)
        || host[0].stream_error_summary.error_count
            != u32::try_from(proof.stream_error_count).unwrap_or(u32::MAX)
        || host[0].timing_window.as_ref().is_none_or(|window| {
            window.confirmed_bpm_micros != proof.confirmed_bpm_micros
                || window.bar_grid_anchor_position_microbeats
                    != proof.bar_grid_anchor_position_microbeats
                || window.beat_span_per_frame_nanobeats != proof.beat_span_per_frame_nanobeats
                || window.requested_start_position_microbeats
                    != proof.requested_start_position_microbeats
                || window.captured_start_position_microbeats
                    != proof.captured_start_position_microbeats
                || window.captured_end_position_microbeats != proof.captured_end_position_microbeats
                || window.start_alignment_error_frame_micros
                    != proof.start_alignment_error_frame_micros
                || window.duration_error_frame_micros != proof.duration_error_frame_micros
                || window.beats_per_bar != proof.beats_per_bar
                || window.duration_beats != proof.duration_beats
        })
    {
        return Err(JamAppError::InvalidSession(
            "live-master host or capture timing window disagrees with proof".into(),
        ));
    }
    validate_metadata_lineage(session, proof, wav)?;
    let expected_frames = (f64::from(proof.sample_rate_hz) * 60.0 * f64::from(proof.duration_beats)
        / (proof.confirmed_bpm_micros as f64 / 1_000_000.0))
        .round() as u64;
    if proof.confirmed_bpm_micros == 0 || proof.frame_count != expected_frames {
        return Err(JamAppError::InvalidSession(
            "live-master proof frame geometry does not match its recorded tempo".into(),
        ));
    }
    Ok(())
}

fn validate_metadata_lineage(
    session: &SessionFile,
    proof: &LiveMasterRecordingProof,
    wav: &ExportArtifactSetEntry,
) -> Result<(), JamAppError> {
    if wav.source_graph_ref != proof.source_graph_ref
        || wav.timing_grid_ref != proof.timing_grid_ref
        || wav.source_capture_refs != proof.source_capture_refs
        || wav.lineage_capture_refs != proof.lineage_capture_refs
    {
        return Err(JamAppError::InvalidSession(
            "live-master artifact lineage disagrees with proof".into(),
        ));
    }
    let graph = proof.source_graph_ref.as_ref().ok_or_else(|| {
        JamAppError::InvalidSession("live-master proof is missing Source Graph lineage".into())
    })?;
    if !session.source_graph_refs.iter().any(|stored| {
        stored.source_id == graph.source_id
            && stored.graph_version == graph.graph_version
            && stored.graph_hash == graph.graph_hash
    }) {
        return Err(JamAppError::InvalidSession(
            "live-master Source Graph lineage is absent from stored Session metadata".into(),
        ));
    }
    let timing = proof.timing_grid_ref.as_ref().ok_or_else(|| {
        JamAppError::InvalidSession("live-master proof is missing timing-grid lineage".into())
    })?;
    if graph.source_id != timing.source_id {
        return Err(JamAppError::InvalidSession(
            "live-master graph and timing source lineage disagree".into(),
        ));
    }
    if session.runtime_state.source_timing.confirmed_grid.as_ref()
        != Some(&riotbox_core::session::SourceTimingGridConfirmationState {
            source_id: timing.source_id.clone(),
            hypothesis_id: timing.hypothesis_id.clone(),
            confirmed_by_action: timing.confirmed_by_action,
            confirmed_at: timing.confirmed_at,
        })
    {
        return Err(JamAppError::InvalidSession(
            "live-master timing-grid lineage is absent from stored Session metadata".into(),
        ));
    }
    if proof
        .source_capture_refs
        .iter()
        .chain(&proof.lineage_capture_refs)
        .any(|id| {
            !session
                .captures
                .iter()
                .any(|capture| capture.capture_id == *id)
        })
    {
        return Err(JamAppError::InvalidSession(
            "live-master capture lineage is absent from stored Session metadata".into(),
        ));
    }
    Ok(())
}

/// Opens one receipt-declared regular file exactly once and returns the bytes
/// used for every subsequent validation and archive embedding operation.
pub(super) fn read_exact_regular_artifact(
    entry: &ExportArtifactSetEntry,
    base_dir: Option<&Path>,
    label: &str,
) -> Result<(PathBuf, Vec<u8>, String), JamAppError> {
    let ExportArtifactLocation::LocalPath { path } = &entry.location else {
        return Err(JamAppError::InvalidSession(format!(
            "{label} must be an exact local path"
        )));
    };
    let stored = Path::new(path);
    let path = if stored.is_absolute() {
        stored.to_owned()
    } else {
        base_dir.map(|base| base.join(stored)).ok_or_else(|| {
            JamAppError::InvalidSession(format!(
                "relative {label} requires the Session base directory"
            ))
        })?
    };
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(&path)?;
    let metadata = file.metadata()?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(JamAppError::InvalidSession(format!(
            "{label} must be a regular non-symlink file"
        )));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    Ok((path, bytes, sha256))
}
