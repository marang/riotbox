use super::{lifecycle::latest_commit_boundary_from_log, *};

mod graph_paths;
mod history_validation;
use history_validation::validate_mvp_session_restore_contracts;

#[cfg(test)]
mod graph_path_tests;
pub(super) mod graph_transaction;

impl JamAppState {
    pub fn from_json_files(
        session_path: impl AsRef<Path>,
        source_graph_path: Option<impl AsRef<Path>>,
    ) -> Result<Self, JamAppError> {
        Self::from_json_files_with_hydration_policy(
            session_path,
            source_graph_path,
            SessionHydrationPolicy::RuntimeFull,
        )
    }

    /// Restores only Session metadata for an export that consumes an already
    /// recorded artifact. It never resolves graph references or hydrates source
    /// or capture audio.
    pub fn from_json_files_for_export_metadata(
        session_path: impl AsRef<Path>,
    ) -> Result<Self, JamAppError> {
        Self::from_json_files_with_hydration_policy(
            session_path,
            None::<&Path>,
            SessionHydrationPolicy::ExportMetadataOnly,
        )
    }

    fn from_json_files_with_hydration_policy(
        session_path: impl AsRef<Path>,
        source_graph_path: Option<impl AsRef<Path>>,
        hydration_policy: SessionHydrationPolicy,
    ) -> Result<Self, JamAppError> {
        let session_path = graph_paths::anchored_file_path(session_path.as_ref())?;
        let mut session = load_session_json(&session_path)?;
        normalize_w30_preview_mode(&mut session);
        normalize_missing_typed_undo_policies(&mut session);
        validate_mvp_session_restore_contracts(&session)?;
        let (explicit_source_graph_path, source_graph) = match hydration_policy {
            SessionHydrationPolicy::RuntimeFull => {
                let explicit_source_graph_path = source_graph_path
                    .map(|path| graph_paths::anchored_file_path(path.as_ref()))
                    .transpose()?;
                let source_graph = resolve_source_graph(
                    &session,
                    &session_path,
                    explicit_source_graph_path.as_deref(),
                )?;
                (explicit_source_graph_path, source_graph)
            }
            SessionHydrationPolicy::ExportMetadataOnly => (None, None),
        };
        normalize_scene_candidates(&mut session, source_graph.as_ref());
        let mut queue = ActionQueue::new();
        queue.reserve_action_ids_after(max_action_id(&session));
        let transport = transport_clock_from_state(&session, source_graph.as_ref());
        let last_commit_boundary = latest_commit_boundary_from_log(&session);
        let jam_view = JamViewModel::build(&session, &queue, source_graph.as_ref());
        let runtime_view =
            JamRuntimeView::build(&AppRuntimeState::default(), &session, source_graph.as_ref());
        let (source_audio_cache, source_audio_status) = match hydration_policy {
            SessionHydrationPolicy::RuntimeFull => {
                load_source_audio_cache_for_graph(&session, source_graph.as_ref())
            }
            SessionHydrationPolicy::ExportMetadataOnly => (None, SourceAudioStatus::NotRequested),
        };
        let mut state = Self {
            files: Some(JamFileSet {
                session_path,
                source_graph_path: explicit_source_graph_path,
            }),
            session_hydration_policy: hydration_policy,
            session,
            source_graph,
            source_audio_cache,
            capture_audio_cache: Default::default(),
            queue,
            runtime: AppRuntimeState {
                transport,
                source_audio: SourceAudioRuntimeState {
                    status: source_audio_status,
                },
                last_commit_boundary,
                ..AppRuntimeState::default()
            },
            jam_view,
            runtime_view,
        };
        if hydration_policy == SessionHydrationPolicy::RuntimeFull {
            state.refresh_capture_audio_cache();
        }
        state.refresh_view();
        Ok(state)
    }

    pub fn analyze_source_file_to_json(
        source_path: impl AsRef<Path>,
        session_path: impl AsRef<Path>,
        source_graph_path: Option<PathBuf>,
        sidecar_script_path: impl AsRef<Path>,
        analysis_seed: u64,
    ) -> Result<Self, JamAppError> {
        Self::analyze_source_file_to_json_with_source_bpm_confirmation(
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path,
            analysis_seed,
            None,
        )
    }

    pub fn analyze_source_file_to_json_with_source_bpm_confirmation(
        source_path: impl AsRef<Path>,
        session_path: impl AsRef<Path>,
        source_graph_path: Option<PathBuf>,
        sidecar_script_path: impl AsRef<Path>,
        analysis_seed: u64,
        explicit_source_bpm: Option<f32>,
    ) -> Result<Self, JamAppError> {
        Self::analyze_source_file_to_json_with_source_timing_confirmation(
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path,
            analysis_seed,
            explicit_source_bpm,
            None,
        )
    }

    pub fn analyze_source_file_to_json_with_source_timing_confirmation(
        source_path: impl AsRef<Path>,
        session_path: impl AsRef<Path>,
        source_graph_path: Option<PathBuf>,
        sidecar_script_path: impl AsRef<Path>,
        analysis_seed: u64,
        explicit_source_bpm: Option<f32>,
        explicit_source_downbeat_seconds: Option<f32>,
    ) -> Result<Self, JamAppError> {
        let source_path = source_path.as_ref().canonicalize()?;
        let session_path = graph_paths::anchored_file_path(session_path.as_ref())?;
        let source_graph_path = source_graph_path
            .as_deref()
            .map(graph_paths::anchored_file_path)
            .transpose()?;

        let mut client = StdioSidecarClient::spawn_python(sidecar_script_path)?;
        let pong = client.ping()?;
        let mut graph = client.analyze_source_file(&source_path, analysis_seed)?;
        drop(client);
        let source_audio = enrich_graph_with_rust_source_timing(&mut graph, &source_path)?;
        if let Some(explicit_source_downbeat_seconds) = explicit_source_downbeat_seconds {
            let explicit_source_bpm = explicit_source_bpm.ok_or_else(|| {
                JamAppError::InvalidSession(
                    "explicit source downbeat requires an explicit source BPM".into(),
                )
            })?;
            install_explicit_manual_source_grid(
                &mut graph,
                explicit_source_bpm,
                explicit_source_downbeat_seconds,
            )?;
        }
        if let Some(explicit_source_bpm) = explicit_source_bpm {
            validate_explicit_source_bpm(&graph, explicit_source_bpm)?;
        }
        attach_w30_hook_candidate_evidence(&mut graph, &source_audio);

        let session = session_from_ingested_graph(
            &graph,
            &source_path,
            &session_path,
            source_graph_path.as_deref(),
        )?;
        graph_transaction::save_graph_and_session(
            &session_path,
            &session,
            Some(&graph),
            source_graph_path.as_deref(),
        )?;

        let mut state = Self::from_json_files(&session_path, source_graph_path.as_deref())?;
        if let Some(explicit_source_bpm) = explicit_source_bpm {
            confirm_explicit_source_bpm(&mut state, explicit_source_bpm)?;
            state.save()?;
        }
        state.set_sidecar_state(SidecarState::Ready {
            version: Some(pong.sidecar_version),
            transport: "stdio-ndjson".into(),
        });
        Ok(state)
    }

    pub fn save(&self) -> Result<(), JamAppError> {
        if let Some(files) = &self.files {
            let session_to_save = self.session_prepared_for_save()?;
            if self.session_hydration_policy == SessionHydrationPolicy::ExportMetadataOnly {
                graph_transaction::validate_mutable_destination(&files.session_path)?;
                save_session_json(&files.session_path, &session_to_save)?;
                return Ok(());
            }
            let graph_path = resolve_external_graph_path(
                &session_to_save,
                &files.session_path,
                files.source_graph_path.as_deref(),
            );
            graph_transaction::save_graph_and_session(
                &files.session_path,
                &session_to_save,
                self.source_graph.as_ref(),
                graph_path.as_deref(),
            )?;
        }

        Ok(())
    }

    pub(super) fn save_session_without_source_graph_write(&self) -> Result<(), JamAppError> {
        if let Some(files) = &self.files {
            graph_transaction::validate_mutable_destination(&files.session_path)?;
            let session_to_save = self.session_prepared_for_save()?;
            // Recording rollback relies on failure occurring before Session
            // publication, including when an unsaved graph has changed its hash.
            if self.session_hydration_policy == SessionHydrationPolicy::RuntimeFull {
                resolve_source_graph(
                    &session_to_save,
                    &files.session_path,
                    files.source_graph_path.as_deref(),
                )?;
            }
            save_session_json(&files.session_path, &session_to_save)?;
        }
        Ok(())
    }

    fn session_prepared_for_save(&self) -> Result<SessionFile, JamAppError> {
        let mut session_to_save = self.session.clone();
        sync_latest_snapshot_payloads(&mut session_to_save);
        if self.session_hydration_policy == SessionHydrationPolicy::RuntimeFull {
            sync_graph_refs_with_state(
                &mut session_to_save,
                self.source_graph.as_ref(),
                self.files
                    .as_ref()
                    .map(|files| files.session_path.as_path()),
                self.files
                    .as_ref()
                    .and_then(|files| files.source_graph_path.as_deref()),
            )?;
        }
        Ok(session_to_save)
    }
}

fn load_source_audio_cache_for_graph(
    session: &SessionFile,
    source_graph: Option<&SourceGraph>,
) -> (Option<SourceAudioCache>, SourceAudioStatus) {
    let Some(graph) = source_graph else {
        return (None, SourceAudioStatus::NotRequested);
    };

    let bytes = match std::fs::read(&graph.source.path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return (
                None,
                SourceAudioStatus::unavailable(
                    graph.source.path.clone(),
                    format!("source audio I/O failed: {error}"),
                ),
            );
        }
    };

    match SourceAudioCache::from_pcm_wav_bytes(&graph.source.path, &bytes) {
        Ok(cache) => {
            let actual_hash = format!("sha256:{:x}", Sha256::digest(&bytes));
            if let Err(reason) = validate_source_audio_cache_identity(session, graph, &actual_hash)
            {
                return (
                    None,
                    SourceAudioStatus::unavailable(graph.source.path.clone(), reason),
                );
            }
            let status = SourceAudioStatus::loaded(&cache);
            (Some(cache), status)
        }
        Err(error) => (
            None,
            SourceAudioStatus::unavailable(graph.source.path.clone(), error.to_string()),
        ),
    }
}

fn validate_source_audio_cache_identity(
    session: &SessionFile,
    graph: &SourceGraph,
    actual_hash: &str,
) -> Result<(), String> {
    if graph.source.content_hash != actual_hash {
        return Err(format!(
            "source audio hash mismatch: graph has {}, loaded WAV has {actual_hash}",
            graph.source.content_hash
        ));
    }

    let Some(source_ref) = session
        .source_refs
        .iter()
        .find(|source_ref| source_ref.source_id == graph.source.source_id)
    else {
        return Err(format!(
            "source audio identity unavailable: session has no source ref for {}",
            graph.source.source_id
        ));
    };

    if source_ref.content_hash != actual_hash {
        return Err(format!(
            "source audio hash mismatch: session source ref {} has {}, loaded WAV has {actual_hash}",
            source_ref.source_id, source_ref.content_hash
        ));
    }

    Ok(())
}

fn resolve_source_graph(
    session: &SessionFile,
    session_path: &Path,
    explicit_source_graph_path: Option<&Path>,
) -> Result<Option<SourceGraph>, JamAppError> {
    if let Some(path) = explicit_source_graph_path {
        let graph = match session.source_graph_refs.first() {
            Some(graph_ref) => graph_transaction::load_graph_for_ref(path, graph_ref)?,
            None => load_source_graph_json(path)?,
        };
        return Ok(Some(graph));
    }

    let Some(graph_ref) = session.source_graph_refs.first() else {
        return Ok(None);
    };

    let graph = match graph_ref.storage_mode {
        GraphStorageMode::Embedded => graph_ref.embedded_graph.clone().ok_or_else(|| {
            JamAppError::InvalidSession(
                "source graph ref is embedded but embedded_graph is missing".into(),
            )
        }),
        GraphStorageMode::External => match graph_ref.external_path.as_deref() {
            Some(path) => graph_transaction::load_graph_for_ref(
                &resolve_session_relative_path(session_path, path),
                graph_ref,
            ),
            None => Err(JamAppError::InvalidSession(
                "source graph ref is external but external_path is missing".into(),
            )),
        },
    }?;
    validate_source_graph_hash(graph_ref, &graph)?;
    Ok(Some(graph))
}

fn validate_source_graph_hash(
    graph_ref: &SourceGraphRef,
    graph: &SourceGraph,
) -> Result<(), JamAppError> {
    let actual_hash = source_graph_hash(graph)?;
    if graph_ref.graph_hash != actual_hash {
        return Err(JamAppError::InvalidSession(format!(
            "source graph ref {} hash mismatch: session has {}, loaded graph has {}",
            graph_ref.source_id, graph_ref.graph_hash, actual_hash
        )));
    }
    Ok(())
}

fn sync_latest_snapshot_payloads(session: &mut SessionFile) {
    let latest_action_cursor = session.action_log.actions.len();
    let runtime_state = session.runtime_state.clone();

    for snapshot in &mut session.snapshots {
        if snapshot.action_cursor != latest_action_cursor || snapshot.payload.is_some() {
            continue;
        }

        snapshot.payload = Some(riotbox_core::session::SnapshotPayload::from_runtime_state(
            &snapshot.snapshot_id,
            snapshot.action_cursor,
            &runtime_state,
        ));
    }
}

fn sync_graph_refs_with_state(
    session: &mut SessionFile,
    source_graph: Option<&SourceGraph>,
    session_path: Option<&Path>,
    explicit_source_graph_path: Option<&Path>,
) -> Result<(), JamAppError> {
    for graph_ref in &mut session.source_graph_refs {
        if let Some(source_graph) = source_graph {
            graph_ref.graph_hash = source_graph_hash(source_graph)?;
        }
        match graph_ref.storage_mode {
            GraphStorageMode::Embedded => {
                graph_ref.embedded_graph = source_graph.cloned();
            }
            GraphStorageMode::External => {
                if let Some(path) = explicit_source_graph_path {
                    let session_path = session_path.ok_or_else(|| {
                        JamAppError::InvalidSession(
                            "external Graph save requires a Session path".into(),
                        )
                    })?;
                    graph_ref.external_path =
                        Some(graph_paths::stored_graph_path(session_path, path)?);
                }
            }
        }
    }
    Ok(())
}

fn resolve_external_graph_path<'a>(
    session: &'a SessionFile,
    session_path: &Path,
    explicit_source_graph_path: Option<&'a Path>,
) -> Option<PathBuf> {
    if let Some(path) = explicit_source_graph_path {
        return Some(path.to_path_buf());
    }

    session
        .source_graph_refs
        .iter()
        .find(|graph_ref| graph_ref.storage_mode == GraphStorageMode::External)
        .and_then(|graph_ref| graph_ref.external_path.as_deref())
        .map(|path| resolve_session_relative_path(session_path, path))
}

fn resolve_session_relative_path(session_path: &Path, stored_path: &str) -> PathBuf {
    let path = Path::new(stored_path);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    session_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(path)
}

fn session_from_ingested_graph(
    graph: &SourceGraph,
    source_path: &Path,
    session_path: &Path,
    source_graph_path: Option<&Path>,
) -> Result<SessionFile, JamAppError> {
    let timestamp = timestamp_now();
    let source_id = SourceId::from(graph.source.source_id.as_str());
    let graph_hash = source_graph_hash(graph)?;

    let mut session = SessionFile::new(
        format!("session-{}", graph.source.source_id.as_str()),
        env!("CARGO_PKG_VERSION"),
        timestamp.clone(),
    );
    session.updated_at = timestamp;
    session.source_refs.push(SourceRef {
        source_id: source_id.clone(),
        path_hint: source_path.to_string_lossy().into_owned(),
        content_hash: graph.source.content_hash.clone(),
        duration_seconds: graph.source.duration_seconds,
        decode_profile: decode_profile_label(&graph.source.decode_profile),
    });
    session.source_graph_refs.push(SourceGraphRef {
        source_id,
        graph_version: graph.graph_version,
        graph_hash,
        storage_mode: if source_graph_path.is_some() {
            GraphStorageMode::External
        } else {
            GraphStorageMode::Embedded
        },
        embedded_graph: source_graph_path.is_none().then(|| graph.clone()),
        external_path: source_graph_path
            .map(|path| graph_paths::stored_graph_path(session_path, path))
            .transpose()?,
        provenance: graph.provenance.clone(),
    });
    // Keep the music bus open enough that W-30 preview work is audible in fresh ingest sessions.
    session.runtime_state.mixer_state.music_level = 0.64;
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    session.notes = Some("session created from analysis ingest slice".into());
    normalize_scene_candidates(&mut session, Some(graph));

    Ok(session)
}

pub(in crate::jam_app) fn source_graph_hash(graph: &SourceGraph) -> Result<String, JamAppError> {
    let encoded = serde_json::to_vec(graph)?;
    Ok(format!("sha256:{:x}", Sha256::digest(encoded)))
}

fn decode_profile_label(profile: &DecodeProfile) -> String {
    match profile {
        DecodeProfile::Native => "native".into(),
        DecodeProfile::NormalizedStereo => "normalized_stereo".into(),
        DecodeProfile::NormalizedMono => "normalized_mono".into(),
        DecodeProfile::Custom(value) => value.clone(),
    }
}

fn timestamp_now() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("unix_ms:{millis}")
}
