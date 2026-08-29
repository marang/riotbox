use std::{fs, io::Write, path::Path};

use riotbox_audio::runtime::{
    AudioRuntimeHealth, AudioRuntimeLifecycle, LiveMasterCaptureOutcome, signal_metrics,
};
use riotbox_core::{
    TimestampMs,
    action::{ActionParams, LiveRecordingExportBoundary},
    export_readiness::{
        EXPORT_READINESS_CONTRACT_SCHEMA, ExportReadinessContract, ExportReadinessStatus,
        ExportScope, LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_PACK_ID, ProductExportBoundary,
        ProductExportDestinationKind, ProductExportRole,
    },
    ids::ExportReceiptId,
    session::{
        ExportArtifactAudioMetrics, ExportLiveRecordingCallbackGapSummary,
        ExportLiveRecordingHostAudioRef, ExportLiveRecordingStreamErrorSummary,
        ExportLiveRecordingTimingWindow, ExportReceiptQaGateResult, ExportReceiptState,
    },
};
use sha2::{Digest, Sha256};

use super::{
    super::{JamAppError, JamAppState},
    LIVE_MASTER_RECORDING_DURATION_BEATS, LIVE_MASTER_RECORDING_PROOF_SCHEMA,
    LiveMasterRecordingPlan, LiveMasterRecordingProof,
    session_identity::{
        live_master_session_identity, live_master_target_frame_count,
        live_master_target_sample_count, proof_path_for, validate_destination, validate_output,
    },
};

const WAV_HEADER_LEN: usize = 56;
const WAV_FORMAT_IEEE_FLOAT: u16 = 3;
const WAV_BITS_PER_SAMPLE: u16 = 32;
const ACTIVE_SAMPLE_THRESHOLD: f32 = 1.0e-5;

pub(super) struct ValidatedLiveMasterRecording {
    wav_bytes: Vec<u8>,
    wav_sha256: String,
    proof_bytes: Vec<u8>,
    proof_sha256: String,
    proof: LiveMasterRecordingProof,
}
pub(super) fn prepare_validated_recording(
    state: &JamAppState,
    plan: &LiveMasterRecordingPlan,
    outcome: &LiveMasterCaptureOutcome,
    health: &AudioRuntimeHealth,
) -> Result<ValidatedLiveMasterRecording, JamAppError> {
    validate_destination(&plan.destination_path)?;
    validate_output(&plan.output)?;
    let identity = live_master_session_identity(state)?;
    let expected_frame_count =
        live_master_target_frame_count(plan.output.sample_rate, plan.confirmed_bpm)?;
    let requested_start_beat_cursor = plan.requested_start_position_beats as u64;
    if identity.confirmed_bpm.to_bits() != plan.confirmed_bpm.to_bits()
        || identity.scene_id != plan.scene_id
        || state.session.session_id != plan.session_id
        || identity.source_graph_ref != plan.source_graph_ref
        || identity.timing_grid_ref != plan.timing_grid_ref
        || identity.beats_per_bar != plan.beats_per_bar
        || identity.bar_grid_anchor_beat_cursor != plan.bar_grid_anchor_beat_cursor
        || identity.source_capture_refs != plan.source_capture_refs
        || identity.lineage_capture_refs != plan.lineage_capture_refs
        || plan.request.target_frame_count != expected_frame_count
        || plan.request.channel_count != plan.output.channel_count
        || plan.request.expected_tempo_bpm.to_bits() != plan.confirmed_bpm.to_bits()
        || plan.request.start_position_beats.map(f64::to_bits)
            != Some(plan.requested_start_position_beats.to_bits())
        || plan.beats_per_bar != 4
        || !plan.requested_start_position_beats.is_finite()
        || plan.requested_start_position_beats < 0.0
        || plan.requested_start_position_beats != requested_start_beat_cursor as f64
        || requested_start_beat_cursor < plan.bar_grid_anchor_beat_cursor
        || !(requested_start_beat_cursor - plan.bar_grid_anchor_beat_cursor)
            .is_multiple_of(u64::from(plan.beats_per_bar))
        || plan.proof_path != proof_path_for(&plan.destination_path)?
    {
        return Err(JamAppError::InvalidSession(
            "live master recording Session or frozen capture identity changed during capture"
                .into(),
        ));
    }
    let pending = state
        .queue
        .pending_actions()
        .into_iter()
        .find(|action| action.id == plan.action_id)
        .ok_or_else(|| {
            JamAppError::InvalidSession("live master recording action is not pending".into())
        })?;
    if pending.target.scene_id.as_ref() != Some(&plan.scene_id)
        || !matches!(
            &pending.params,
            ActionParams::LiveRecordingExport {
                boundary: LiveRecordingExportBoundary::RuntimeMasterBarWindowV2,
                destination_kind: ProductExportDestinationKind::LocalFilePath,
                destination_path: Some(destination),
                ..
            } if destination == &plan.destination_path.to_string_lossy()
        )
    {
        return Err(JamAppError::InvalidSession(
            "live master recording pending action identity changed".into(),
        ));
    }
    if health.lifecycle != AudioRuntimeLifecycle::Running
        || health.output.as_ref() != Some(&plan.output)
    {
        return Err(JamAppError::InvalidSession(
            "live master recording output runtime is not the planned running device".into(),
        ));
    }
    if health.stream_error_count > 0 || health.callback_scratch_overflow_count > 0 {
        return Err(JamAppError::InvalidSession(
            "live master recording runtime health reports a stream or scratch fault".into(),
        ));
    }
    let expected_sample_count = live_master_target_sample_count(
        plan.request.target_frame_count,
        plan.output.channel_count,
    )?;
    if !outcome.progress.complete
        || outcome.progress.target_sample_count != expected_sample_count
        || outcome.progress.written_sample_count != expected_sample_count
        || outcome.progress.fault_count() > 0
        || !outcome.progress.capture_started
        || outcome.progress.armed_callback_count == 0
    {
        return Err(JamAppError::InvalidSession(format!(
            "live master recording callback capture is incomplete or faulted: {:?}",
            outcome.progress
        )));
    }
    let captured_start = outcome.captured_start_position_beats.ok_or_else(|| {
        JamAppError::InvalidSession(
            "bar-aligned live master recording has no captured start position".into(),
        )
    })?;
    let captured_end = outcome.captured_end_position_beats.ok_or_else(|| {
        JamAppError::InvalidSession(
            "bar-aligned live master recording has no captured end position".into(),
        )
    })?;
    let beats_per_frame = f64::from(plan.confirmed_bpm) / 60.0 / f64::from(plan.output.sample_rate);
    let beat_span_per_frame_nanobeats = (beats_per_frame * 1_000_000_000.0).round() as u64;
    let start_error_frames =
        (captured_start - plan.requested_start_position_beats) / beats_per_frame;
    let captured_duration_beats = captured_end - captured_start;
    let duration_error_frames =
        (captured_duration_beats - f64::from(LIVE_MASTER_RECORDING_DURATION_BEATS)).abs()
            / beats_per_frame;
    let expected_end = captured_start + beats_per_frame * plan.request.target_frame_count as f64;
    let position_tolerance = beats_per_frame * 1.0e-6;
    if !captured_start.is_finite()
        || !captured_end.is_finite()
        || !start_error_frames.is_finite()
        || !duration_error_frames.is_finite()
        || beat_span_per_frame_nanobeats == 0
        || start_error_frames < -1.0e-6
        || start_error_frames > 1.0 + 1.0e-6
        || duration_error_frames > 0.500_001
        || (captured_end - expected_end).abs() > position_tolerance
    {
        return Err(JamAppError::InvalidSession(
            "bar-aligned live master recording timing window is not exact".into(),
        ));
    }
    if outcome.samples.len() != expected_sample_count
        || outcome.samples.iter().any(|sample| !sample.is_finite())
    {
        return Err(JamAppError::InvalidSession(
            "live master recording sample payload is incomplete or non-finite".into(),
        ));
    }
    let metrics = signal_metrics(&outcome.samples);
    if metrics.active_samples == 0 || metrics.peak_abs <= ACTIVE_SAMPLE_THRESHOLD {
        return Err(JamAppError::InvalidSession(
            "live master recording is silent".into(),
        ));
    }
    if metrics.clip_count > 0 || metrics.peak_abs >= 1.0 {
        return Err(JamAppError::InvalidSession(
            "live master recording clips at or above full scale".into(),
        ));
    }

    let wav_bytes = encode_float32_wav(
        plan.output.sample_rate,
        plan.output.channel_count,
        &outcome.samples,
    )?;
    validate_float32_wav(
        &wav_bytes,
        plan.output.sample_rate,
        plan.output.channel_count,
        plan.request.target_frame_count,
        &outcome.samples,
    )?;
    let wav_sha256 = sha256_bytes(&wav_bytes);
    let sample_payload_sha256 = sha256_float_samples(&outcome.samples);
    let receipt_id = ExportReceiptId::new(format!("export-receipt-{}", plan.action_id));
    let proof = LiveMasterRecordingProof {
        schema: LIVE_MASTER_RECORDING_PROOF_SCHEMA.into(),
        receipt_id,
        action_id: plan.action_id,
        session_id: plan.session_id.clone(),
        session_pre_capture_sha256: plan.session_pre_capture_sha256.clone(),
        scene_id: plan.scene_id.clone(),
        confirmed_bpm_micros: (f64::from(plan.confirmed_bpm) * 1_000_000.0).round() as u64,
        duration_beats: LIVE_MASTER_RECORDING_DURATION_BEATS,
        beats_per_bar: plan.beats_per_bar,
        bar_grid_anchor_position_microbeats: beat_cursor_microbeats(
            plan.bar_grid_anchor_beat_cursor,
        )?,
        beat_span_per_frame_nanobeats,
        requested_start_position_microbeats: beat_position_microbeats(
            plan.requested_start_position_beats,
        )?,
        captured_start_position_microbeats: beat_position_microbeats(captured_start)?,
        captured_end_position_microbeats: beat_position_microbeats(captured_end)?,
        start_alignment_error_frame_micros: (start_error_frames.max(0.0) * 1_000_000.0).round()
            as u64,
        duration_error_frame_micros: (duration_error_frames * 1_000_000.0).round() as u64,
        host: plan.output.host_name.clone(),
        device: plan.output.device_name.clone(),
        device_sample_format: plan.output.sample_format.clone(),
        wav_sample_format: "ieee_float32".into(),
        sample_rate_hz: plan.output.sample_rate,
        channel_count: plan.output.channel_count,
        frame_count: plan.request.target_frame_count as u64,
        callback_count: outcome.progress.callback_count,
        max_callback_gap_micros: outcome.progress.max_callback_gap_micros,
        callback_gap_over_threshold_count: outcome.progress.callback_gap_over_threshold_count,
        callback_scratch_overflow_count: outcome.progress.callback_scratch_overflow_count,
        stream_error_count: outcome.progress.stream_error_count,
        transport_mismatch_count: outcome.progress.transport_mismatch_count,
        tempo_mismatch_count: outcome.progress.tempo_mismatch_count,
        timing_window_mismatch_count: outcome.progress.timing_window_mismatch_count,
        armed_callback_count: outcome.progress.armed_callback_count,
        active_sample_count: metrics.active_samples as u64,
        peak_amplitude_micros: amplitude_micros(metrics.peak_abs),
        rms_amplitude_micros: amplitude_micros(metrics.rms),
        clip_count: metrics.clip_count as u64,
        sample_payload_sha256,
        wav_sha256: wav_sha256.clone(),
        source_graph_ref: plan.source_graph_ref.clone(),
        timing_grid_ref: plan.timing_grid_ref.clone(),
        source_capture_refs: plan.source_capture_refs.clone(),
        lineage_capture_refs: plan.lineage_capture_refs.clone(),
    };
    let mut proof_bytes = serde_json::to_vec_pretty(&proof)?;
    proof_bytes.push(b'\n');
    let proof_sha256 = sha256_bytes(&proof_bytes);
    Ok(ValidatedLiveMasterRecording {
        wav_bytes,
        wav_sha256,
        proof_bytes,
        proof_sha256,
        proof,
    })
}

pub(super) fn publish_recording(
    plan: &LiveMasterRecordingPlan,
    written: &ValidatedLiveMasterRecording,
) -> Result<(), JamAppError> {
    validate_destination(&plan.destination_path)?;
    let parent = plan.destination_path.parent().ok_or_else(|| {
        JamAppError::InvalidSession("live recording destination has no parent".into())
    })?;
    let mut staged_wav = tempfile::Builder::new()
        .prefix(".riotbox-live-wav-staging-")
        .tempfile_in(parent)?;
    staged_wav.write_all(&written.wav_bytes)?;
    staged_wav.as_file().sync_all()?;
    let mut staged_proof = tempfile::Builder::new()
        .prefix(".riotbox-live-proof-staging-")
        .tempfile_in(parent)?;
    staged_proof.write_all(&written.proof_bytes)?;
    staged_proof.as_file().sync_all()?;

    staged_wav
        .persist_noclobber(&plan.destination_path)
        .map_err(|error| error.error)?;
    if let Err(error) = staged_proof.persist_noclobber(&plan.proof_path) {
        remove_file_if_hash(&plan.destination_path, &written.wav_sha256);
        return Err(error.error.into());
    }
    if let Err(error) = validate_published_recording(plan, written) {
        remove_owned_recording(plan, written);
        return Err(error);
    }
    Ok(())
}

fn validate_published_recording(
    plan: &LiveMasterRecordingPlan,
    written: &ValidatedLiveMasterRecording,
) -> Result<(), JamAppError> {
    let wav_bytes = fs::read(&plan.destination_path)?;
    let proof_bytes = fs::read(&plan.proof_path)?;
    if wav_bytes != written.wav_bytes
        || proof_bytes != written.proof_bytes
        || sha256_bytes(&wav_bytes) != written.wav_sha256
        || sha256_bytes(&proof_bytes) != written.proof_sha256
    {
        return Err(JamAppError::InvalidSession(
            "published live master recording differs from validated staging bytes".into(),
        ));
    }
    let proof: LiveMasterRecordingProof = serde_json::from_slice(&proof_bytes)?;
    if proof != written.proof {
        return Err(JamAppError::InvalidSession(
            "published live master recording proof failed exact read-back".into(),
        ));
    }
    let samples = decode_float32_wav(
        &wav_bytes,
        plan.output.sample_rate,
        plan.output.channel_count,
        plan.request.target_frame_count,
    )?;
    if sha256_float_samples(&samples) != written.proof.sample_payload_sha256 {
        return Err(JamAppError::InvalidSession(
            "published live master recording sample payload hash mismatch".into(),
        ));
    }
    Ok(())
}

pub(super) fn build_recording_receipt(
    state: &JamAppState,
    plan: &LiveMasterRecordingPlan,
    written: &ValidatedLiveMasterRecording,
    created_at: TimestampMs,
) -> Result<ExportReceiptState, JamAppError> {
    let wav_path = plan.destination_path.to_string_lossy().into_owned();
    let proof_path = plan.proof_path.to_string_lossy().into_owned();
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: LIVE_MASTER_RECORDING_PROOF_SCHEMA.into(),
        export_scope: ExportScope::LiveRecording,
        boundary: ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2,
        pack_id: LIVE_RECORDING_RUNTIME_MASTER_BAR_WINDOW_PACK_ID.into(),
        export_role: ProductExportRole::LiveRecordingCapture,
        export_artifact: wav_path.clone(),
        source_sha256: written.proof.session_pre_capture_sha256.clone(),
        export_sha256: written.wav_sha256.clone(),
        normalized_manifest_sha256: written.proof_sha256.clone(),
        unsupported_scopes: Vec::new(),
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        plan.action_id,
        created_at,
        &contract,
        wav_path.clone(),
        proof_path.clone(),
        Some(proof_path.clone()),
    );
    let duration_ms = duration_ms(plan.request.target_frame_count, plan.output.sample_rate)?;
    let mut audio_artifact = riotbox_core::session::ExportArtifactSetEntry::live_recording_capture(
        wav_path,
        written.wav_sha256.clone(),
    );
    audio_artifact.source_graph_ref = written.proof.source_graph_ref.clone();
    audio_artifact.timing_grid_ref = written.proof.timing_grid_ref.clone();
    audio_artifact.source_capture_refs = written.proof.source_capture_refs.clone();
    audio_artifact.lineage_capture_refs = written.proof.lineage_capture_refs.clone();
    audio_artifact.audio_metrics = Some(ExportArtifactAudioMetrics {
        peak_milli_dbfs: None,
        rms_milli_dbfs: None,
        peak_amplitude_micros: Some(written.proof.peak_amplitude_micros),
        rms_amplitude_micros: Some(written.proof.rms_amplitude_micros),
        silent_frame_count: None,
        total_frame_count: Some(plan.request.target_frame_count as u64),
    });
    audio_artifact.sample_rate_hz = Some(plan.output.sample_rate);
    audio_artifact.channel_count = Some(plan.output.channel_count);
    audio_artifact.duration_ms = Some(duration_ms);
    receipt.artifact_set = vec![
        audio_artifact,
        riotbox_core::session::ExportArtifactSetEntry::product_export_proof(
            proof_path,
            written.proof_sha256.clone(),
        ),
    ];
    receipt.qa_gates = vec![
        ExportReceiptQaGateResult::live_recording_runtime_master_capture(),
        ExportReceiptQaGateResult::live_recording_wav_readback(),
        ExportReceiptQaGateResult::live_recording_bar_window_alignment(),
    ];
    receipt.live_recording_host_audio_refs = vec![ExportLiveRecordingHostAudioRef {
        host: plan.output.host_name.clone(),
        device: plan.output.device_name.clone(),
        recording_duration_ms: duration_ms,
        callback_gap_summary: ExportLiveRecordingCallbackGapSummary {
            max_gap_ms: written
                .proof
                .max_callback_gap_micros
                .map(|value| value.div_ceil(1_000)),
            over_threshold_count: u32::try_from(written.proof.callback_gap_over_threshold_count)
                .unwrap_or(u32::MAX),
        },
        stream_error_summary: ExportLiveRecordingStreamErrorSummary {
            error_count: u32::try_from(written.proof.stream_error_count).unwrap_or(u32::MAX),
            last_error: state
                .runtime
                .audio
                .as_ref()
                .and_then(|health| health.last_stream_error.clone()),
        },
        timing_window: Some(ExportLiveRecordingTimingWindow {
            confirmed_bpm_micros: written.proof.confirmed_bpm_micros,
            bar_grid_anchor_position_microbeats: written.proof.bar_grid_anchor_position_microbeats,
            beat_span_per_frame_nanobeats: written.proof.beat_span_per_frame_nanobeats,
            requested_start_position_microbeats: written.proof.requested_start_position_microbeats,
            captured_start_position_microbeats: written.proof.captured_start_position_microbeats,
            captured_end_position_microbeats: written.proof.captured_end_position_microbeats,
            start_alignment_error_frame_micros: written.proof.start_alignment_error_frame_micros,
            duration_error_frame_micros: written.proof.duration_error_frame_micros,
            beats_per_bar: written.proof.beats_per_bar,
            duration_beats: written.proof.duration_beats,
        }),
    }];
    Ok(receipt)
}

fn beat_cursor_microbeats(beat_cursor: u64) -> Result<u64, JamAppError> {
    beat_cursor.checked_mul(1_000_000).ok_or_else(|| {
        JamAppError::InvalidSession(
            "live recording bar-grid anchor cannot be represented in proof evidence".into(),
        )
    })
}

fn beat_position_microbeats(position_beats: f64) -> Result<u64, JamAppError> {
    let microbeats = (position_beats * 1_000_000.0).round();
    if !microbeats.is_finite() || microbeats < 0.0 || microbeats > u64::MAX as f64 {
        return Err(JamAppError::InvalidSession(
            "live recording beat position cannot be represented in proof evidence".into(),
        ));
    }
    Ok(microbeats as u64)
}

fn encode_float32_wav(
    sample_rate: u32,
    channel_count: u16,
    samples: &[f32],
) -> Result<Vec<u8>, JamAppError> {
    if channel_count == 0 || !samples.len().is_multiple_of(usize::from(channel_count)) {
        return Err(JamAppError::InvalidSession(
            "live WAV samples are not aligned to complete output frames".into(),
        ));
    }
    let data_len = samples
        .len()
        .checked_mul(4)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| JamAppError::InvalidSession("live WAV payload is too large".into()))?;
    let byte_rate = sample_rate
        .checked_mul(u32::from(channel_count))
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| JamAppError::InvalidSession("live WAV byte rate overflow".into()))?;
    let block_align = channel_count
        .checked_mul(4)
        .ok_or_else(|| JamAppError::InvalidSession("live WAV block align overflow".into()))?;
    let frame_count = u32::try_from(samples.len() / usize::from(channel_count))
        .map_err(|_| JamAppError::InvalidSession("live WAV frame count is too large".into()))?;
    let riff_len = 48_u32
        .checked_add(data_len)
        .ok_or_else(|| JamAppError::InvalidSession("live WAV RIFF size overflow".into()))?;
    let mut bytes = Vec::with_capacity(WAV_HEADER_LEN + data_len as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_len.to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&WAV_FORMAT_IEEE_FLOAT.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&WAV_BITS_PER_SAMPLE.to_le_bytes());
    bytes.extend_from_slice(b"fact");
    bytes.extend_from_slice(&4_u32.to_le_bytes());
    bytes.extend_from_slice(&frame_count.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    Ok(bytes)
}

fn validate_float32_wav(
    bytes: &[u8],
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
    expected_samples: &[f32],
) -> Result<(), JamAppError> {
    let samples = decode_float32_wav(bytes, sample_rate, channel_count, frame_count)?;
    if samples.len() != expected_samples.len()
        || samples
            .iter()
            .zip(expected_samples)
            .any(|(actual, expected)| actual.to_bits() != expected.to_bits())
    {
        return Err(JamAppError::InvalidSession(
            "live float32 WAV samples failed exact read-back".into(),
        ));
    }
    Ok(())
}

fn decode_float32_wav(
    bytes: &[u8],
    expected_sample_rate: u32,
    expected_channel_count: u16,
    expected_frame_count: usize,
) -> Result<Vec<f32>, JamAppError> {
    let expected_block_align = expected_channel_count
        .checked_mul(4)
        .ok_or_else(|| JamAppError::InvalidSession("live WAV block align overflow".into()))?;
    let expected_byte_rate = expected_sample_rate
        .checked_mul(u32::from(expected_block_align))
        .ok_or_else(|| JamAppError::InvalidSession("live WAV byte rate overflow".into()))?;
    if bytes.len() < WAV_HEADER_LEN
        || &bytes[0..4] != b"RIFF"
        || &bytes[8..16] != b"WAVEfmt "
        || read_u32(bytes, 16)? != 16
        || read_u16(bytes, 20)? != WAV_FORMAT_IEEE_FLOAT
        || read_u16(bytes, 22)? != expected_channel_count
        || read_u32(bytes, 24)? != expected_sample_rate
        || read_u32(bytes, 28)? != expected_byte_rate
        || read_u16(bytes, 32)? != expected_block_align
        || read_u16(bytes, 34)? != WAV_BITS_PER_SAMPLE
        || &bytes[36..40] != b"fact"
        || read_u32(bytes, 40)? != 4
        || read_u32(bytes, 44)? as usize != expected_frame_count
        || &bytes[48..52] != b"data"
    {
        return Err(JamAppError::InvalidSession(
            "live WAV format is not the frozen IEEE-float32 contract".into(),
        ));
    }
    let data_len = read_u32(bytes, 52)? as usize;
    let expected_sample_count = expected_frame_count
        .checked_mul(usize::from(expected_channel_count))
        .ok_or_else(|| JamAppError::InvalidSession("live WAV sample count overflow".into()))?;
    if data_len != expected_sample_count.saturating_mul(4)
        || bytes.len() != WAV_HEADER_LEN.saturating_add(data_len)
        || read_u32(bytes, 4)? as usize != bytes.len().saturating_sub(8)
    {
        return Err(JamAppError::InvalidSession(
            "live WAV frame or RIFF length differs from the capture contract".into(),
        ));
    }
    Ok(bytes[WAV_HEADER_LEN..]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|sample| f32::from_le_bytes(*sample))
        .collect())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, JamAppError> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| JamAppError::InvalidSession("live WAV header ended unexpectedly".into()))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, JamAppError> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| JamAppError::InvalidSession("live WAV header ended unexpectedly".into()))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn duration_ms(frame_count: usize, sample_rate: u32) -> Result<u64, JamAppError> {
    let frames = u64::try_from(frame_count)
        .map_err(|_| JamAppError::InvalidSession("live duration frame overflow".into()))?;
    Ok(frames
        .saturating_mul(1_000)
        .saturating_add(u64::from(sample_rate) / 2)
        / u64::from(sample_rate))
}

fn amplitude_micros(value: f32) -> u32 {
    (f64::from(value.max(0.0)) * 1_000_000.0)
        .round()
        .clamp(0.0, f64::from(u32::MAX)) as u32
}

fn sha256_float_samples(samples: &[f32]) -> String {
    let mut digest = Sha256::new();
    for sample in samples {
        digest.update(sample.to_le_bytes());
    }
    format!("{:x}", digest.finalize())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn remove_owned_recording(
    plan: &LiveMasterRecordingPlan,
    written: &ValidatedLiveMasterRecording,
) {
    remove_file_if_hash(&plan.proof_path, &written.proof_sha256);
    remove_file_if_hash(&plan.destination_path, &written.wav_sha256);
}

pub(super) fn remove_file_if_hash(path: &Path, expected_sha256: &str) {
    if super::super::product_export::sha256_file(path)
        .is_ok_and(|actual_sha256| actual_sha256 == expected_sha256)
    {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float32_wav_roundtrip_preserves_exact_callback_sample_bits() {
        let samples = [0.0, 0.125, -0.5, 0.75];
        let bytes = encode_float32_wav(48_000, 2, &samples).expect("encode float WAV");
        validate_float32_wav(&bytes, 48_000, 2, 2, &samples).expect("validate float WAV");
        assert_eq!(read_u16(&bytes, 20).expect("format tag"), 3);
        assert_eq!(read_u16(&bytes, 34).expect("bits"), 32);
        assert_eq!(&bytes[36..40], b"fact");
        assert_eq!(read_u32(&bytes, 44).expect("fact frame count"), 2);
    }
}
