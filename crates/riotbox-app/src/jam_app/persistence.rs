use super::{lifecycle::latest_commit_boundary_from_log, *};

impl JamAppState {
    pub fn from_json_files(
        session_path: impl AsRef<Path>,
        source_graph_path: Option<impl AsRef<Path>>,
    ) -> Result<Self, JamAppError> {
        let session_path = session_path.as_ref().to_path_buf();
        let mut session = load_session_json(&session_path)?;
        normalize_w30_preview_mode(&mut session);
        validate_mvp_session_restore_contracts(&session)?;
        let explicit_source_graph_path = source_graph_path.map(|path| path.as_ref().to_path_buf());
        let source_graph = resolve_source_graph(&session, explicit_source_graph_path.as_deref())?;
        normalize_scene_candidates(&mut session, source_graph.as_ref());
        let mut queue = ActionQueue::new();
        queue.reserve_action_ids_after(max_action_id(&session));
        let transport = transport_clock_from_state(&session, source_graph.as_ref());
        let last_commit_boundary = latest_commit_boundary_from_log(&session);
        let jam_view = JamViewModel::build(&session, &queue, source_graph.as_ref());
        let runtime_view =
            JamRuntimeView::build(&AppRuntimeState::default(), &session, source_graph.as_ref());
        let source_audio_cache = source_graph
            .as_ref()
            .and_then(|graph| SourceAudioCache::load_pcm_wav(&graph.source.path).ok());
        let mut state = Self {
            files: Some(JamFileSet {
                session_path,
                source_graph_path: explicit_source_graph_path,
            }),
            session,
            source_graph,
            source_audio_cache,
            capture_audio_cache: Default::default(),
            queue,
            runtime: AppRuntimeState {
                transport,
                last_commit_boundary,
                ..AppRuntimeState::default()
            },
            jam_view,
            runtime_view,
        };
        state.refresh_capture_audio_cache();
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
        let source_path = source_path.as_ref().canonicalize()?;
        let session_path = session_path.as_ref().to_path_buf();

        let mut client = StdioSidecarClient::spawn_python(sidecar_script_path)?;
        let pong = client.ping()?;
        let graph = client.analyze_source_file(&source_path, analysis_seed)?;

        let session =
            session_from_ingested_graph(&graph, &source_path, source_graph_path.as_deref())?;
        if let Some(source_graph_path) = source_graph_path.as_deref() {
            save_source_graph_json(source_graph_path, &graph)?;
        }
        save_session_json(&session_path, &session)?;

        let mut state = Self::from_json_files(&session_path, source_graph_path.as_deref())?;
        state.set_sidecar_state(SidecarState::Ready {
            version: Some(pong.sidecar_version),
            transport: "stdio-ndjson".into(),
        });
        Ok(state)
    }

    pub fn save(&self) -> Result<(), JamAppError> {
        if let Some(files) = &self.files {
            let mut session_to_save = self.session.clone();
            sync_graph_refs_with_state(
                &mut session_to_save,
                self.source_graph.as_ref(),
                files.source_graph_path.as_deref(),
            );
            save_session_json(&files.session_path, &session_to_save)?;

            if let Some(source_graph) = &self.source_graph
                && let Some(source_graph_path) = resolve_external_graph_path(
                    &session_to_save,
                    files.source_graph_path.as_deref(),
                )
            {
                save_source_graph_json(source_graph_path, source_graph)?;
            }
        }

        Ok(())
    }
}

fn resolve_source_graph(
    session: &SessionFile,
    explicit_source_graph_path: Option<&Path>,
) -> Result<Option<SourceGraph>, JamAppError> {
    if let Some(path) = explicit_source_graph_path {
        return Ok(Some(load_source_graph_json(path)?));
    }

    let Some(graph_ref) = session.source_graph_refs.first() else {
        return Ok(None);
    };

    match graph_ref.storage_mode {
        GraphStorageMode::Embedded => graph_ref.embedded_graph.clone().map(Some).ok_or_else(|| {
            JamAppError::InvalidSession(
                "source graph ref is embedded but embedded_graph is missing".into(),
            )
        }),
        GraphStorageMode::External => match graph_ref.external_path.as_deref() {
            Some(path) => Ok(Some(load_source_graph_json(path)?)),
            None => Err(JamAppError::InvalidSession(
                "source graph ref is external but external_path is missing".into(),
            )),
        },
    }
}

fn validate_mvp_session_restore_contracts(session: &SessionFile) -> Result<(), JamAppError> {
    if session.source_refs.len() > 1 {
        return Err(JamAppError::InvalidSession(
            "Riotbox MVP currently supports exactly one source reference per session".into(),
        ));
    }

    if session.source_graph_refs.len() > 1 {
        return Err(JamAppError::InvalidSession(
            "Riotbox MVP currently supports exactly one source graph reference per session".into(),
        ));
    }

    if let (Some(source_ref), Some(graph_ref)) = (
        session.source_refs.first(),
        session.source_graph_refs.first(),
    ) && source_ref.source_id != graph_ref.source_id
    {
        return Err(JamAppError::InvalidSession(format!(
            "source ref {} does not match source graph ref {}",
            source_ref.source_id, graph_ref.source_id
        )));
    }

    let action_count = session.action_log.actions.len();
    for snapshot in &session.snapshots {
        if snapshot.action_cursor > action_count {
            return Err(JamAppError::InvalidSession(format!(
                "snapshot {} action cursor {} exceeds action log length {}",
                snapshot.snapshot_id, snapshot.action_cursor, action_count
            )));
        }
    }

    for commit_record in &session.action_log.commit_records {
        let Some(action) = session
            .action_log
            .actions
            .iter()
            .find(|action| action.id == commit_record.action_id)
        else {
            return Err(JamAppError::InvalidSession(format!(
                "commit record references missing action {}",
                commit_record.action_id
            )));
        };

        if action.status != ActionStatus::Committed {
            return Err(JamAppError::InvalidSession(format!(
                "commit record references action {} with non-committed status {:?}",
                commit_record.action_id, action.status
            )));
        }

        let Some(action_committed_at) = action.committed_at else {
            return Err(JamAppError::InvalidSession(format!(
                "commit record references action {} without committed_at timestamp",
                commit_record.action_id
            )));
        };

        if commit_record.committed_at != action_committed_at {
            return Err(JamAppError::InvalidSession(format!(
                "commit record for action {} has committed_at {} but action has committed_at {}",
                commit_record.action_id, commit_record.committed_at, action_committed_at
            )));
        }

        if commit_record.commit_sequence == 0 {
            return Err(JamAppError::InvalidSession(format!(
                "commit record for action {} has invalid sequence 0",
                commit_record.action_id
            )));
        }
    }

    for (index, commit_record) in session.action_log.commit_records.iter().enumerate() {
        for previous in &session.action_log.commit_records[..index] {
            if previous.action_id == commit_record.action_id {
                return Err(JamAppError::InvalidSession(format!(
                    "commit record is duplicated for action {}",
                    commit_record.action_id
                )));
            }

            if previous.boundary == commit_record.boundary
                && previous.commit_sequence == commit_record.commit_sequence
            {
                return Err(JamAppError::InvalidSession(format!(
                    "commit record sequence {} is duplicated within boundary {:?} beat {} bar {} phrase {}",
                    commit_record.commit_sequence,
                    commit_record.boundary.kind,
                    commit_record.boundary.beat_index,
                    commit_record.boundary.bar_index,
                    commit_record.boundary.phrase_index
                )));
            }
        }
    }

    Ok(())
}

fn sync_graph_refs_with_state(
    session: &mut SessionFile,
    source_graph: Option<&SourceGraph>,
    explicit_source_graph_path: Option<&Path>,
) {
    for graph_ref in &mut session.source_graph_refs {
        match graph_ref.storage_mode {
            GraphStorageMode::Embedded => {
                graph_ref.embedded_graph = source_graph.cloned();
            }
            GraphStorageMode::External => {
                if let Some(path) = explicit_source_graph_path {
                    graph_ref.external_path = Some(path.to_string_lossy().into_owned());
                }
            }
        }
    }
}

fn resolve_external_graph_path<'a>(
    session: &'a SessionFile,
    explicit_source_graph_path: Option<&'a Path>,
) -> Option<&'a Path> {
    if let Some(path) = explicit_source_graph_path {
        return Some(path);
    }

    session
        .source_graph_refs
        .iter()
        .find(|graph_ref| graph_ref.storage_mode == GraphStorageMode::External)
        .and_then(|graph_ref| graph_ref.external_path.as_deref())
        .map(Path::new)
}

fn session_from_ingested_graph(
    graph: &SourceGraph,
    source_path: &Path,
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
        external_path: source_graph_path.map(|path| path.to_string_lossy().into_owned()),
        provenance: graph.provenance.clone(),
    });
    // Keep the music bus open enough that W-30 preview work is audible in fresh ingest sessions.
    session.runtime_state.mixer_state.music_level = 0.64;
    session.notes = Some("session created from analysis ingest slice".into());
    normalize_scene_candidates(&mut session, Some(graph));

    Ok(session)
}

fn source_graph_hash(graph: &SourceGraph) -> Result<String, JamAppError> {
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
