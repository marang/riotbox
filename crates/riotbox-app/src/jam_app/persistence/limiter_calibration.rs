//! In-memory construction from an existing graph and already admitted WAV bytes.

use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

use riotbox_audio::source_audio::SourceAudioCache;
use riotbox_core::{
    TimestampMs,
    action::{ActionCommand, ActionStatus, CommitBoundary},
    queue::ActionQueue,
    source_graph::{DecodeProfile, SourceGraph},
};
use sha2::{Digest, Sha256};

use super::{
    graph_paths, session_from_ingested_graph, source_audio_identity_ref,
    validate_source_audio_cache_identity,
};
use crate::jam_app::{
    JamAppError, JamAppState, JamFileSet, QueueControlResult,
    live_source_timing::validate_explicit_source_bpm,
};

impl JamAppState {
    /// Builds the ordinary fresh-ingest Session without opening the source path.
    ///
    /// The caller owns preregistered source admission and supplies its one read's
    /// bytes plus an already validated graph. This method hashes and decodes that
    /// same buffer once, so an unrelated cache cannot be paired with its digest.
    /// It neither analyzes nor changes the graph, confirms timing, or saves files.
    /// Session/Graph destinations must be fresh, distinct output files; subsequent
    /// ordinary capture commits retain artifact-required playback semantics.
    pub fn from_limiter_calibration_graph_bytes(
        graph: SourceGraph,
        source_wav_bytes: &[u8],
        output_files: JamFileSet,
    ) -> Result<Self, JamAppError> {
        if graph.source.decode_profile != DecodeProfile::Native {
            return Err(JamAppError::InvalidSession(
                "limiter calibration requires the native source decode profile".into(),
            ));
        }
        let output_files = checked_output_files(output_files, Path::new(&graph.source.path))?;
        let session = session_from_ingested_graph(
            &graph,
            Path::new(&graph.source.path),
            &output_files.session_path,
            output_files.source_graph_path.as_deref(),
        )?;
        let source_ref =
            source_audio_identity_ref(&session, &graph).map_err(JamAppError::InvalidSession)?;
        let actual_hash = format!("sha256:{:x}", Sha256::digest(source_wav_bytes));
        validate_source_audio_cache_identity(source_ref, &graph, &actual_hash)
            .map_err(JamAppError::InvalidSession)?;
        if graph.provenance.source_hash != actual_hash {
            return Err(JamAppError::InvalidSession(
                "limiter calibration graph provenance source hash mismatch".into(),
            ));
        }
        let cache = SourceAudioCache::from_pcm_wav_bytes(&graph.source.path, source_wav_bytes)
            .map_err(|error| JamAppError::InvalidSession(error.to_string()))?;
        validate_native_format(&graph, &cache)?;

        let mut state = Self::from_parts(session, Some(graph), ActionQueue::new());
        state.files = Some(output_files);
        state.source_audio_cache = Some(cache);
        state.refresh_view();
        Ok(state)
    }

    /// Commits the preregistered explicit BPM through the ordinary ingest action.
    ///
    /// The timestamp is caller-owned for repeatable diagnostics. An empty queue is
    /// required so this operation cannot commit unrelated pending performer actions.
    /// The existing source-BPM validation remains authoritative; the graph is unchanged.
    pub fn confirm_limiter_calibration_source_bpm(
        &mut self,
        explicit_source_bpm: f32,
        requested_at: TimestampMs,
    ) -> Result<(), JamAppError> {
        let graph = self.source_graph.as_ref().ok_or_else(|| {
            JamAppError::InvalidSession(
                "explicit source BPM cannot confirm timing without a Source Graph".into(),
            )
        })?;
        validate_explicit_source_bpm(graph, explicit_source_bpm)?;
        if !self.queue.pending_actions().is_empty() {
            return Err(JamAppError::InvalidSession(
                "limiter calibration BPM confirmation requires an empty action queue".into(),
            ));
        }
        if self
            .queue_source_timing_grid_confirmation_at_bpm(requested_at, Some(explicit_source_bpm))
            != QueueControlResult::Enqueued
        {
            return Err(JamAppError::InvalidSession(
                "explicit source BPM confirmation could not be queued".into(),
            ));
        }
        let boundary = self
            .runtime
            .transport
            .boundary_state(CommitBoundary::Immediate);
        let committed = self.commit_ready_actions(boundary, requested_at);
        let accepted_confirmation = committed.first().is_some_and(|committed| {
            self.session.action_log.actions.iter().any(|action| {
                action.id == committed.action_id
                    && action.command == ActionCommand::SourceTimingConfirmGrid
                    && action.status == ActionStatus::Committed
                    && action.result.as_ref().is_some_and(|result| result.accepted)
            })
        });
        if committed.len() != 1 || !accepted_confirmation {
            return Err(JamAppError::InvalidSession(
                "explicit source BPM confirmation did not commit exactly one accepted confirmation"
                    .into(),
            ));
        }
        Ok(())
    }
}

fn validate_native_format(
    graph: &SourceGraph,
    cache: &SourceAudioCache,
) -> Result<(), JamAppError> {
    if cache.path != Path::new(&graph.source.path)
        || cache.sample_rate != graph.source.sample_rate
        || cache.channel_count != graph.source.channel_count
        || cache.frame_count() == 0
    {
        return Err(JamAppError::InvalidSession(
            "limiter calibration graph path/native format does not match admitted WAV".into(),
        ));
    }
    // The existing Python ingest stores round(frame_count / sample_rate, 3).
    // Decimal formatting has the same ties-to-even rounding without rounding
    // a pre-scaled float a second time. Preserve that graph metadata verbatim.
    let duration = (cache.frame_count() as f64 / f64::from(cache.sample_rate)).max(0.001);
    let sidecar_duration: f32 = format!("{duration:.3}").parse().map_err(|_| {
        JamAppError::InvalidSession("limiter calibration source duration is invalid".into())
    })?;
    if graph.source.duration_seconds != cache.duration_seconds()
        && graph.source.duration_seconds != sidecar_duration
    {
        return Err(JamAppError::InvalidSession(
            "limiter calibration graph duration does not match admitted WAV".into(),
        ));
    }
    Ok(())
}

fn checked_output_files(files: JamFileSet, source_path: &Path) -> Result<JamFileSet, JamAppError> {
    let source_path = lexical_absolute(source_path)?;
    let paths = std::iter::once(&files.session_path).chain(files.source_graph_path.iter());
    for path in paths {
        if lexical_absolute(path)?.starts_with(&source_path) {
            return Err(JamAppError::InvalidSession(
                "limiter calibration output conflicts with source path".into(),
            ));
        }
    }
    let files = JamFileSet {
        session_path: graph_paths::anchored_file_path(&files.session_path)?,
        source_graph_path: files
            .source_graph_path
            .as_deref()
            .map(graph_paths::anchored_file_path)
            .transpose()?,
    };
    if files.source_graph_path.as_ref().is_some_and(|graph_path| {
        graph_path.starts_with(&files.session_path) || files.session_path.starts_with(graph_path)
    }) {
        return Err(JamAppError::InvalidSession(
            "limiter calibration Session and Graph outputs must be distinct".into(),
        ));
    }
    let paths = std::iter::once(&files.session_path).chain(files.source_graph_path.iter());
    for path in paths {
        if path.starts_with(&source_path) {
            return Err(JamAppError::InvalidSession(
                "limiter calibration output conflicts with source path".into(),
            ));
        }
        match fs::symlink_metadata(path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
            Ok(_) => {
                return Err(JamAppError::InvalidSession(
                    "limiter calibration requires fresh output paths".into(),
                ));
            }
        }
    }
    Ok(files)
}

// Only output paths may inspect filesystem metadata. Source identity is an
// opaque locator here, normalized lexically without canonicalizing or opening it.
fn lexical_absolute(path: &Path) -> io::Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in std::path::absolute(path)?.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            component => normalized.push(component.as_os_str()),
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests;
