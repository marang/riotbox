use std::{
    fs,
    path::{Path, PathBuf},
};

use riotbox_audio::{
    runtime::{AudioOutputInfo, LIVE_MASTER_MAX_INTERLEAVED_SAMPLE_COUNT},
    w30::{
        W30PreviewRenderMode, W30PreviewRenderRouting, W30ResampleTapMode, W30ResampleTapRouting,
    },
};
use riotbox_core::{
    ids::{CaptureId, SceneId},
    session::{ExportArtifactSourceGraphRef, ExportArtifactTimingGridRef},
};
use sha2::{Digest, Sha256};

use super::{
    super::{JamAppError, JamAppState, persistence::source_graph_hash},
    LIVE_MASTER_RECORDING_DURATION_BEATS,
};

pub(super) struct LiveMasterSessionIdentity {
    pub(super) confirmed_bpm: f32,
    pub(super) scene_id: SceneId,
    pub(super) source_graph_ref: Option<ExportArtifactSourceGraphRef>,
    pub(super) timing_grid_ref: Option<ExportArtifactTimingGridRef>,
    pub(super) source_capture_refs: Vec<CaptureId>,
    pub(super) lineage_capture_refs: Vec<CaptureId>,
}
pub(super) fn prepare_recording_plan_input(
    state: &JamAppState,
    output: &AudioOutputInfo,
    destination_path: &Path,
) -> Result<(LiveMasterSessionIdentity, usize, PathBuf, String), JamAppError> {
    validate_destination(destination_path)?;
    validate_output(output)?;
    let identity = live_master_session_identity(state)?;
    let target_frame_count =
        live_master_target_frame_count(output.sample_rate, identity.confirmed_bpm)?;
    live_master_target_sample_count(target_frame_count, output.channel_count)?;
    let session_sha256 = sha256_bytes(&serde_json::to_vec(&state.session)?);
    Ok((
        identity,
        target_frame_count,
        proof_path_for(destination_path)?,
        session_sha256,
    ))
}

pub(super) fn live_master_session_identity(
    state: &JamAppState,
) -> Result<LiveMasterSessionIdentity, JamAppError> {
    let confirmed_bpm = state
        .session
        .runtime_state
        .source_timing
        .confirmed_bpm
        .filter(|bpm| bpm.is_finite() && *bpm > 0.0)
        .ok_or_else(|| {
            JamAppError::InvalidSession(
                "live master recording requires a positive confirmed Session BPM".into(),
            )
        })?;
    let scene_id = state
        .session
        .runtime_state
        .scene_state
        .active_scene
        .clone()
        .ok_or_else(|| {
            JamAppError::InvalidSession(
                "live master recording requires an active Session scene".into(),
            )
        })?;
    if state.session.runtime_state.transport.current_scene.as_ref() != Some(&scene_id)
        || state.runtime.transport.current_scene.as_ref() != Some(&scene_id)
    {
        return Err(JamAppError::InvalidSession(
            "live master recording active and transport scenes do not match".into(),
        ));
    }

    let (source_graph_ref, timing_grid_ref) = match state.source_graph.as_ref() {
        Some(graph) => {
            let graph_hash = source_graph_hash(graph)?;
            let graph_ref = state
                .session
                .source_graph_refs
                .iter()
                .find(|graph_ref| {
                    graph_ref.source_id == graph.source.source_id
                        && graph_ref.graph_version == graph.graph_version
                        && graph_ref.graph_hash == graph_hash
                })
                .ok_or_else(|| {
                    JamAppError::InvalidSession(
                        "live master recording requires exact active Source Graph Session lineage"
                            .into(),
                    )
                })?;
            let confirmed_grid = state
                .session
                .runtime_state
                .source_timing
                .confirmed_grid
                .as_ref()
                .filter(|grid| grid.source_id == graph.source.source_id)
                .ok_or_else(|| {
                    JamAppError::InvalidSession(
                        "live master recording requires the active source timing grid".into(),
                    )
                })?;
            (
                Some(ExportArtifactSourceGraphRef {
                    source_id: graph_ref.source_id.clone(),
                    graph_version: graph_ref.graph_version,
                    graph_hash: graph_ref.graph_hash.clone(),
                }),
                Some(ExportArtifactTimingGridRef {
                    source_id: confirmed_grid.source_id.clone(),
                    hypothesis_id: confirmed_grid.hypothesis_id.clone(),
                    confirmed_by_action: confirmed_grid.confirmed_by_action,
                    confirmed_at: confirmed_grid.confirmed_at,
                }),
            )
        }
        None if state.session.source_graph_refs.is_empty() => (None, None),
        None => {
            return Err(JamAppError::InvalidSession(
                "live master recording cannot resolve the Session Source Graph lineage".into(),
            ));
        }
    };

    let mut source_capture_refs = active_render_capture_refs(state)?;
    source_capture_refs.sort();
    source_capture_refs.dedup();
    let mut lineage_capture_refs = Vec::new();
    for capture_id in &source_capture_refs {
        let capture = state
            .session
            .captures
            .iter()
            .find(|capture| capture.capture_id == *capture_id)
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "live master recording active render capture {capture_id} is missing from Session"
                ))
            })?;
        for lineage_capture_id in &capture.lineage_capture_refs {
            if !state
                .session
                .captures
                .iter()
                .any(|candidate| candidate.capture_id == *lineage_capture_id)
            {
                return Err(JamAppError::InvalidSession(format!(
                    "live master recording lineage capture {lineage_capture_id} for active capture {capture_id} is missing from Session"
                )));
            }
            lineage_capture_refs.push(lineage_capture_id.clone());
        }
    }
    lineage_capture_refs.sort();
    lineage_capture_refs.dedup();

    Ok(LiveMasterSessionIdentity {
        confirmed_bpm,
        scene_id,
        source_graph_ref,
        timing_grid_ref,
        source_capture_refs,
        lineage_capture_refs,
    })
}

fn active_render_capture_refs(state: &JamAppState) -> Result<Vec<CaptureId>, JamAppError> {
    let mut capture_refs = Vec::new();
    let preview = &state.runtime.w30_preview;
    let preview_active = !matches!(preview.mode, W30PreviewRenderMode::Idle)
        && matches!(preview.routing, W30PreviewRenderRouting::MusicBusPreview)
        && preview.music_bus_level > 0.0;
    if preview_active {
        let capture_id = preview.capture_id.as_deref().ok_or_else(|| {
            JamAppError::InvalidSession(
                "live master recording active W-30 preview has no capture owner".into(),
            )
        })?;
        capture_refs.push(CaptureId::from(capture_id));
    }

    let resample = &state.runtime.w30_resample_tap;
    let resample_active = !matches!(resample.mode, W30ResampleTapMode::Idle)
        && matches!(resample.routing, W30ResampleTapRouting::InternalCaptureTap)
        && resample
            .source_audio
            .as_ref()
            .is_some_and(|audio| audio.sample_count > 0)
        && resample.music_bus_level > 0.0;
    if resample_active {
        let capture_id = resample.source_capture_id.as_deref().ok_or_else(|| {
            JamAppError::InvalidSession(
                "live master recording active W-30 resample tap has no capture owner".into(),
            )
        })?;
        capture_refs.push(CaptureId::from(capture_id));
    }

    Ok(capture_refs)
}

pub(super) fn live_master_target_frame_count(
    sample_rate: u32,
    bpm: f32,
) -> Result<usize, JamAppError> {
    let frames = (f64::from(sample_rate) * 60.0 * f64::from(LIVE_MASTER_RECORDING_DURATION_BEATS)
        / f64::from(bpm))
    .round();
    if !frames.is_finite() || frames <= 0.0 || frames > usize::MAX as f64 {
        return Err(JamAppError::InvalidSession(
            "live master recording frame count is invalid".into(),
        ));
    }
    Ok(frames as usize)
}

pub(super) fn live_master_target_sample_count(
    target_frame_count: usize,
    channel_count: u16,
) -> Result<usize, JamAppError> {
    target_frame_count
        .checked_mul(usize::from(channel_count))
        .filter(|sample_count| {
            *sample_count > 0
                && *sample_count <= LIVE_MASTER_MAX_INTERLEAVED_SAMPLE_COUNT
        })
        .ok_or_else(|| {
            JamAppError::InvalidSession(format!(
                "live master recording exceeds the V1 allocation limit of {LIVE_MASTER_MAX_INTERLEAVED_SAMPLE_COUNT} interleaved samples"
            ))
        })
}

pub(super) fn validate_output(output: &AudioOutputInfo) -> Result<(), JamAppError> {
    if output.host_name.trim().is_empty()
        || output.device_name.trim().is_empty()
        || output.sample_format.trim().is_empty()
        || output.sample_rate == 0
        || output.channel_count == 0
    {
        return Err(JamAppError::InvalidSession(
            "live master recording requires complete host output identity".into(),
        ));
    }
    Ok(())
}

pub(super) fn validate_destination(destination: &Path) -> Result<(), JamAppError> {
    if destination.extension().and_then(|value| value.to_str()) != Some("wav") {
        return Err(JamAppError::InvalidSession(
            "live master recording destination must end in .wav".into(),
        ));
    }
    let parent = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| {
            JamAppError::InvalidSession(
                "live master recording destination requires an explicit parent".into(),
            )
        })?;
    if !parent.is_dir() {
        return Err(JamAppError::InvalidSession(format!(
            "live master recording destination parent does not exist: {}",
            parent.display()
        )));
    }
    let proof_path = proof_path_for(destination)?;
    if destination.exists()
        || fs::symlink_metadata(destination).is_ok()
        || proof_path.exists()
        || fs::symlink_metadata(&proof_path).is_ok()
    {
        return Err(JamAppError::InvalidSession(
            "live master recording destination or proof already exists".into(),
        ));
    }
    Ok(())
}

pub(super) fn proof_path_for(destination: &Path) -> Result<PathBuf, JamAppError> {
    let file_name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            JamAppError::InvalidSession(
                "live master recording destination has no UTF-8 file name".into(),
            )
        })?;
    Ok(destination.with_file_name(format!("{file_name}.riotbox.json")))
}
fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_count_rounds_the_frozen_eight_beat_window() {
        assert_eq!(
            live_master_target_frame_count(48_000, 130.0).expect("frame count"),
            177_231
        );
    }

    #[test]
    fn interleaved_sample_count_rejects_the_first_oversized_payload() {
        let error =
            live_master_target_sample_count(LIVE_MASTER_MAX_INTERLEAVED_SAMPLE_COUNT + 1, 1)
                .expect_err("oversized payload rejected");

        assert!(error.to_string().contains("allocation limit"));
    }
}
