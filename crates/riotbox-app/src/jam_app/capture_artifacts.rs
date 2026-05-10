use std::{
    io,
    path::{Path, PathBuf},
};

use riotbox_audio::{
    runtime::render_w30_resample_tap_offline,
    source_audio::{SourceAudioCache, SourceAudioWindow, write_interleaved_pcm16_wav},
    w30::{
        W30ResampleTapMode, W30ResampleTapRouting, W30ResampleTapSourceProfile, W30ResampleTapState,
    },
};
use riotbox_core::{ids::CaptureId, session::CaptureRef};

use super::JamAppState;
use super::helpers::append_capture_note;
use super::state::W30BusPrintInput;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::jam_app) enum CaptureArtifactHydrationPreflightError {
    MissingStoragePath {
        capture_id: CaptureId,
    },
    MissingSessionFileSet {
        capture_id: CaptureId,
    },
    MissingArtifact {
        capture_id: CaptureId,
        path: PathBuf,
    },
    UnreadableArtifact {
        capture_id: CaptureId,
        path: PathBuf,
        reason: String,
    },
    NotFile {
        capture_id: CaptureId,
        path: PathBuf,
    },
}

impl JamAppState {
    pub(in crate::jam_app) fn require_capture_artifact_for_hydration(
        &self,
        capture: &CaptureRef,
    ) -> Result<PathBuf, CaptureArtifactHydrationPreflightError> {
        let base_dir = self
            .files
            .as_ref()
            .and_then(|files| files.session_path.parent());
        preflight_capture_artifact_hydration(capture, base_dir)
    }

    pub(in crate::jam_app) fn persist_capture_audio_artifact(&mut self, capture: &mut CaptureRef) {
        match self.write_capture_audio_artifact(capture) {
            Ok(Some(path)) => {
                if let Ok(cache) = SourceAudioCache::load_pcm_wav(&path) {
                    self.capture_audio_cache
                        .insert(capture.capture_id.clone(), cache);
                }
                append_capture_note(
                    capture,
                    &format!("audio artifact written {}", capture.storage_path),
                );
            }
            Ok(None) => {}
            Err(reason) => {
                append_capture_note(capture, &format!("audio artifact pending: {reason}"))
            }
        }
    }

    pub(in crate::jam_app) fn persist_w30_bus_print_artifact(&mut self, capture: &mut CaptureRef) {
        match self.write_w30_bus_print_artifact(capture) {
            Ok(Some(path)) => {
                if let Ok(cache) = SourceAudioCache::load_pcm_wav(&path) {
                    self.capture_audio_cache
                        .insert(capture.capture_id.clone(), cache);
                }
                append_capture_note(
                    capture,
                    &format!("bus print artifact written {}", capture.storage_path),
                );
            }
            Ok(None) => {}
            Err(reason) => append_capture_note(capture, &format!("bus print pending: {reason}")),
        }
    }

    fn write_w30_bus_print_artifact(
        &self,
        capture: &CaptureRef,
    ) -> Result<Option<PathBuf>, String> {
        if capture.capture_type != riotbox_core::session::CaptureType::Resample {
            return Ok(None);
        }

        let Some(source_capture_id) = capture.lineage_capture_refs.last() else {
            return Err("resample capture has no source capture lineage".into());
        };
        let source_capture = self
            .session
            .captures
            .iter()
            .find(|candidate| candidate.capture_id == *source_capture_id)
            .ok_or_else(|| format!("source capture {source_capture_id} not found"))?;
        let Some(path) = self.capture_audio_artifact_path(capture) else {
            return Ok(None);
        };
        let input = self
            .w30_bus_print_input(source_capture)?
            .ok_or_else(|| format!("source capture {source_capture_id} has no printable audio"))?;
        let channel_count = usize::from(input.channel_count);
        let input_frames = input.samples.len() / channel_count.max(1);
        if input_frames == 0 {
            return Err("source capture audio is empty".into());
        }
        let max_frames = usize::try_from(input.sample_rate)
            .unwrap_or(usize::MAX)
            .saturating_mul(8);
        let frame_count = input_frames.min(max_frames).max(1);
        let sample_count = frame_count.saturating_mul(channel_count);
        let dry = &input.samples[..sample_count.min(input.samples.len())];
        let render_state = self.w30_bus_print_render_state(capture, source_capture);
        let wet = render_w30_resample_tap_offline(
            &render_state,
            input.sample_rate,
            input.channel_count,
            frame_count,
        );
        let printed: Vec<f32> = dry
            .iter()
            .zip(wet.iter())
            .map(|(dry, wet)| (dry * 0.68 + wet * 1.45).clamp(-1.0, 1.0))
            .collect();

        write_interleaved_pcm16_wav(&path, input.sample_rate, input.channel_count, &printed)
            .map_err(|error| error.to_string())?;
        Ok(Some(path))
    }

    fn w30_bus_print_input(
        &self,
        capture: &CaptureRef,
    ) -> Result<Option<W30BusPrintInput>, String> {
        if let Some(cache) = self.capture_audio_cache.get(&capture.capture_id) {
            return Ok(Some(W30BusPrintInput {
                sample_rate: cache.sample_rate,
                channel_count: cache.channel_count,
                samples: cache.interleaved_samples().to_vec(),
            }));
        }

        let Some(source_window) = capture.source_window.as_ref() else {
            return Ok(None);
        };
        let Some(source_audio_cache) = self.source_audio_cache.as_ref() else {
            return Ok(None);
        };
        let frame_count = source_window
            .end_frame
            .saturating_sub(source_window.start_frame);
        if frame_count == 0 {
            return Ok(None);
        }
        let samples = source_audio_cache
            .window_samples(SourceAudioWindow {
                start_frame: usize::try_from(source_window.start_frame)
                    .map_err(|_| "source window start frame exceeds usize".to_string())?,
                frame_count: usize::try_from(frame_count)
                    .map_err(|_| "source window frame count exceeds usize".to_string())?,
            })
            .to_vec();

        Ok(Some(W30BusPrintInput {
            sample_rate: source_audio_cache.sample_rate,
            channel_count: source_audio_cache.channel_count,
            samples,
        }))
    }

    fn w30_bus_print_render_state(
        &self,
        capture: &CaptureRef,
        source_capture: &CaptureRef,
    ) -> W30ResampleTapState {
        let source_profile = if source_capture.is_pinned {
            Some(W30ResampleTapSourceProfile::PinnedCapture)
        } else if source_capture.assigned_target.is_some() {
            Some(W30ResampleTapSourceProfile::PromotedCapture)
        } else {
            Some(W30ResampleTapSourceProfile::RawCapture)
        };

        W30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            source_profile,
            source_capture_id: Some(source_capture.capture_id.to_string()),
            lineage_capture_count: capture
                .lineage_capture_refs
                .len()
                .try_into()
                .unwrap_or(u8::MAX),
            generation_depth: capture.resample_generation_depth,
            music_bus_level: self
                .session
                .runtime_state
                .mixer_state
                .music_level
                .clamp(0.0, 1.0),
            grit_level: self
                .session
                .runtime_state
                .macro_state
                .w30_grit
                .clamp(0.0, 1.0),
            is_transport_running: self.runtime.transport.is_playing,
        }
    }

    pub(in crate::jam_app) fn refresh_capture_audio_cache(&mut self) {
        self.capture_audio_cache.clear();
        for capture in &self.session.captures {
            let Ok(path) = self.require_capture_artifact_for_hydration(capture) else {
                continue;
            };
            let Ok(cache) = SourceAudioCache::load_pcm_wav(path) else {
                continue;
            };
            self.capture_audio_cache
                .insert(capture.capture_id.clone(), cache);
        }
    }

    fn write_capture_audio_artifact(
        &self,
        capture: &CaptureRef,
    ) -> Result<Option<PathBuf>, String> {
        let Some(source_window) = capture.source_window.as_ref() else {
            return Ok(None);
        };
        let Some(source_audio_cache) = self.source_audio_cache.as_ref() else {
            return Ok(None);
        };
        if let Some(source_graph) = self.source_graph.as_ref()
            && source_graph.source.source_id != source_window.source_id
        {
            return Err(format!(
                "capture source {} does not match loaded source {}",
                source_window.source_id, source_graph.source.source_id
            ));
        }
        if let Some(source_graph) = self.source_graph.as_ref()
            && (source_graph.source.sample_rate != source_audio_cache.sample_rate
                || source_graph.source.channel_count != source_audio_cache.channel_count)
        {
            return Err(format!(
                "source graph audio shape {} Hz/{} ch does not match decoded source {} Hz/{} ch",
                source_graph.source.sample_rate,
                source_graph.source.channel_count,
                source_audio_cache.sample_rate,
                source_audio_cache.channel_count
            ));
        }
        let Some(path) = self.capture_audio_artifact_path(capture) else {
            return Ok(None);
        };

        let frame_count = source_window
            .end_frame
            .saturating_sub(source_window.start_frame);
        if frame_count == 0 {
            return Err("source window is empty".into());
        }

        source_audio_cache
            .write_window_pcm16_wav(
                &path,
                SourceAudioWindow {
                    start_frame: usize::try_from(source_window.start_frame)
                        .map_err(|_| "source window start frame exceeds usize".to_string())?,
                    frame_count: usize::try_from(frame_count)
                        .map_err(|_| "source window frame count exceeds usize".to_string())?,
                },
            )
            .map_err(|error| error.to_string())?;

        Ok(Some(path))
    }

    fn capture_audio_artifact_path(&self, capture: &CaptureRef) -> Option<PathBuf> {
        let storage_path = Path::new(&capture.storage_path);
        if storage_path.is_absolute() {
            return Some(storage_path.to_path_buf());
        }

        let files = self.files.as_ref()?;
        let session_dir = files
            .session_path
            .parent()
            .unwrap_or_else(|| Path::new("."));
        Some(session_dir.join(storage_path))
    }
}

pub(in crate::jam_app) fn preflight_capture_artifact_hydration(
    capture: &CaptureRef,
    base_dir: Option<&Path>,
) -> Result<PathBuf, CaptureArtifactHydrationPreflightError> {
    let storage_path = capture.storage_path.trim();
    if storage_path.is_empty() {
        return Err(CaptureArtifactHydrationPreflightError::MissingStoragePath {
            capture_id: capture.capture_id.clone(),
        });
    }

    let storage_path = Path::new(storage_path);
    let path = if storage_path.is_absolute() {
        storage_path.to_path_buf()
    } else {
        let base_dir = base_dir.ok_or_else(|| {
            CaptureArtifactHydrationPreflightError::MissingSessionFileSet {
                capture_id: capture.capture_id.clone(),
            }
        })?;
        base_dir.join(storage_path)
    };

    match std::fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => Ok(path),
        Ok(_) => Err(CaptureArtifactHydrationPreflightError::NotFile {
            capture_id: capture.capture_id.clone(),
            path,
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Err(CaptureArtifactHydrationPreflightError::MissingArtifact {
                capture_id: capture.capture_id.clone(),
                path,
            })
        }
        Err(error) => Err(CaptureArtifactHydrationPreflightError::UnreadableArtifact {
            capture_id: capture.capture_id.clone(),
            path,
            reason: error.to_string(),
        }),
    }
}
