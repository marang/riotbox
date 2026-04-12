use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use riotbox_audio::runtime::{AudioRuntimeHealth, AudioRuntimeLifecycle};
use riotbox_core::{
    TimestampMs,
    ids::SourceId,
    persistence::{
        PersistenceError, load_session_json, load_source_graph_json, save_session_json,
        save_source_graph_json,
    },
    queue::{ActionQueue, CommittedActionRef},
    session::{GraphStorageMode, SessionFile, SourceGraphRef, SourceRef},
    source_graph::{DecodeProfile, SourceGraph},
    transport::{CommitBoundaryState, TransportClockState},
    view::jam::JamViewModel,
};
use riotbox_sidecar::client::{ClientError as SidecarClientError, StdioSidecarClient};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub enum JamAppError {
    Io(io::Error),
    Persistence(PersistenceError),
    Serialization(serde_json::Error),
    Sidecar(SidecarClientError),
}

impl Display for JamAppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Persistence(error) => write!(f, "persistence error: {error}"),
            Self::Serialization(error) => write!(f, "serialization error: {error}"),
            Self::Sidecar(error) => write!(f, "sidecar error: {error}"),
        }
    }
}

impl Error for JamAppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Persistence(error) => Some(error),
            Self::Serialization(error) => Some(error),
            Self::Sidecar(error) => Some(error),
        }
    }
}

impl From<io::Error> for JamAppError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<PersistenceError> for JamAppError {
    fn from(value: PersistenceError) -> Self {
        Self::Persistence(value)
    }
}

impl From<serde_json::Error> for JamAppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

impl From<SidecarClientError> for JamAppError {
    fn from(value: SidecarClientError) -> Self {
        Self::Sidecar(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JamFileSet {
    pub session_path: PathBuf,
    pub source_graph_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppRuntimeState {
    pub audio: Option<AudioRuntimeHealth>,
    pub sidecar: SidecarState,
    pub transport: TransportClockState,
    pub last_commit_boundary: Option<CommitBoundaryState>,
}

impl Default for AppRuntimeState {
    fn default() -> Self {
        Self {
            audio: None,
            sidecar: SidecarState::Unknown,
            transport: TransportClockState::default(),
            last_commit_boundary: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SidecarState {
    Unknown,
    Ready {
        version: Option<String>,
        transport: String,
    },
    Unavailable {
        reason: String,
    },
    Degraded {
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JamRuntimeView {
    pub audio_status: String,
    pub audio_callback_count: u64,
    pub audio_last_error: Option<String>,
    pub sidecar_status: String,
    pub sidecar_version: Option<String>,
    pub runtime_warnings: Vec<String>,
}

impl JamRuntimeView {
    #[must_use]
    pub fn build(runtime: &AppRuntimeState) -> Self {
        let (audio_status, audio_callback_count, audio_last_error) = match &runtime.audio {
            Some(health) => (
                match health.lifecycle {
                    AudioRuntimeLifecycle::Idle => "idle".into(),
                    AudioRuntimeLifecycle::Running => "running".into(),
                    AudioRuntimeLifecycle::Stopped => "stopped".into(),
                    AudioRuntimeLifecycle::Faulted => "faulted".into(),
                },
                health.callback_count,
                health.last_stream_error.clone(),
            ),
            None => ("unknown".into(), 0, None),
        };

        let (sidecar_status, sidecar_version) = match &runtime.sidecar {
            SidecarState::Unknown => ("unknown".into(), None),
            SidecarState::Ready { version, .. } => ("ready".into(), version.clone()),
            SidecarState::Unavailable { .. } => ("unavailable".into(), None),
            SidecarState::Degraded { .. } => ("degraded".into(), None),
        };

        let mut runtime_warnings = Vec::new();
        if matches!(
            runtime.audio.as_ref().map(|health| health.lifecycle),
            Some(AudioRuntimeLifecycle::Faulted)
        ) {
            runtime_warnings.push("audio runtime faulted".into());
        }
        match &runtime.sidecar {
            SidecarState::Unavailable { reason } => {
                runtime_warnings.push(format!("sidecar unavailable: {reason}"));
            }
            SidecarState::Degraded { reason } => {
                runtime_warnings.push(format!("sidecar degraded: {reason}"));
            }
            SidecarState::Unknown | SidecarState::Ready { .. } => {}
        }

        Self {
            audio_status,
            audio_callback_count,
            audio_last_error,
            sidecar_status,
            sidecar_version,
            runtime_warnings,
        }
    }
}

#[derive(Clone, Debug)]
pub struct JamAppState {
    pub files: Option<JamFileSet>,
    pub session: SessionFile,
    pub source_graph: Option<SourceGraph>,
    pub queue: ActionQueue,
    pub runtime: AppRuntimeState,
    pub jam_view: JamViewModel,
    pub runtime_view: JamRuntimeView,
}

impl JamAppState {
    #[must_use]
    pub fn from_parts(
        session: SessionFile,
        source_graph: Option<SourceGraph>,
        mut queue: ActionQueue,
    ) -> Self {
        queue.reserve_action_ids_after(max_action_id(&session));
        let jam_view = JamViewModel::build(&session, &queue, source_graph.as_ref());
        let runtime = AppRuntimeState {
            transport: transport_clock_from_session(&session),
            ..AppRuntimeState::default()
        };
        let runtime_view = JamRuntimeView::build(&runtime);

        Self {
            files: None,
            session,
            source_graph,
            queue,
            runtime,
            jam_view,
            runtime_view,
        }
    }

    pub fn from_json_files(
        session_path: impl AsRef<Path>,
        source_graph_path: impl AsRef<Path>,
    ) -> Result<Self, JamAppError> {
        let session_path = session_path.as_ref().to_path_buf();
        let source_graph_path = source_graph_path.as_ref().to_path_buf();
        let session = load_session_json(&session_path)?;
        let source_graph = load_source_graph_json(&source_graph_path)?;
        let mut queue = ActionQueue::new();
        queue.reserve_action_ids_after(max_action_id(&session));
        let jam_view = JamViewModel::build(&session, &queue, Some(&source_graph));
        let runtime = AppRuntimeState {
            transport: transport_clock_from_session(&session),
            ..AppRuntimeState::default()
        };
        let runtime_view = JamRuntimeView::build(&runtime);

        Ok(Self {
            files: Some(JamFileSet {
                session_path,
                source_graph_path,
            }),
            session,
            source_graph: Some(source_graph),
            queue,
            runtime,
            jam_view,
            runtime_view,
        })
    }

    pub fn analyze_source_file_to_json(
        source_path: impl AsRef<Path>,
        session_path: impl AsRef<Path>,
        source_graph_path: impl AsRef<Path>,
        sidecar_script_path: impl AsRef<Path>,
        analysis_seed: u64,
    ) -> Result<Self, JamAppError> {
        let source_path = source_path.as_ref().canonicalize()?;
        let session_path = session_path.as_ref().to_path_buf();
        let source_graph_path = source_graph_path.as_ref().to_path_buf();

        let mut client = StdioSidecarClient::spawn_python(sidecar_script_path)?;
        let pong = client.ping()?;
        let graph = client.analyze_source_file(&source_path, analysis_seed)?;

        let session = session_from_ingested_graph(&graph, &source_path, &source_graph_path)?;
        save_source_graph_json(&source_graph_path, &graph)?;
        save_session_json(&session_path, &session)?;

        let mut state = Self::from_json_files(&session_path, &source_graph_path)?;
        state.set_sidecar_state(SidecarState::Ready {
            version: Some(pong.sidecar_version),
            transport: "stdio-ndjson".into(),
        });
        Ok(state)
    }

    pub fn refresh_view(&mut self) {
        self.jam_view = JamViewModel::build(&self.session, &self.queue, self.source_graph.as_ref());
        self.runtime_view = JamRuntimeView::build(&self.runtime);
    }

    pub fn set_audio_health(&mut self, health: AudioRuntimeHealth) {
        self.runtime.audio = Some(health);
        self.runtime_view = JamRuntimeView::build(&self.runtime);
    }

    pub fn set_sidecar_state(&mut self, state: SidecarState) {
        self.runtime.sidecar = state;
        self.runtime_view = JamRuntimeView::build(&self.runtime);
    }

    pub fn update_transport_clock(&mut self, clock: TransportClockState) {
        self.runtime.transport = clock.clone();
        self.session.runtime_state.transport.is_playing = clock.is_playing;
        self.session.runtime_state.transport.position_beats = clock.position_beats;
        self.session.runtime_state.transport.current_scene = clock.current_scene.clone();
        self.session.runtime_state.scene_state.active_scene = clock.current_scene;
        self.refresh_view();
    }

    pub fn commit_ready_actions(
        &mut self,
        boundary: CommitBoundaryState,
        committed_at: TimestampMs,
    ) -> Vec<CommittedActionRef> {
        let committed = self
            .queue
            .commit_ready_for_transport(boundary.clone(), committed_at);

        for committed_ref in &committed {
            if let Some(action) = self.queue.history_action(committed_ref.action_id) {
                self.session.action_log.actions.push(action.clone());
            }
        }

        self.runtime.last_commit_boundary = Some(boundary);
        self.refresh_view();
        committed
    }

    pub fn save(&self) -> Result<(), JamAppError> {
        if let Some(files) = &self.files {
            save_session_json(&files.session_path, &self.session)?;

            if let Some(source_graph) = &self.source_graph {
                save_source_graph_json(&files.source_graph_path, source_graph)?;
            }
        }

        Ok(())
    }
}

fn session_from_ingested_graph(
    graph: &SourceGraph,
    source_path: &Path,
    source_graph_path: &Path,
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
        storage_mode: GraphStorageMode::External,
        embedded_graph: None,
        external_path: Some(source_graph_path.to_string_lossy().into_owned()),
        provenance: graph.provenance.clone(),
    });
    session.notes = Some("session created from analysis ingest slice".into());

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

fn transport_clock_from_session(session: &SessionFile) -> TransportClockState {
    TransportClockState {
        is_playing: session.runtime_state.transport.is_playing,
        position_beats: session.runtime_state.transport.position_beats,
        beat_index: session.runtime_state.transport.position_beats.floor() as u64,
        bar_index: 0,
        phrase_index: 0,
        current_scene: session.runtime_state.transport.current_scene.clone(),
    }
}

fn max_action_id(session: &SessionFile) -> Option<riotbox_core::ids::ActionId> {
    session
        .action_log
        .actions
        .iter()
        .map(|action| action.id)
        .max()
}

#[cfg(test)]
mod tests {
    use std::{f32::consts::PI, fs, io, path::Path, path::PathBuf};

    use tempfile::tempdir;

    use riotbox_audio::runtime::{AudioOutputInfo, AudioRuntimeHealth, AudioRuntimeLifecycle};
    use riotbox_core::{
        action::{
            Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus,
            ActionTarget, ActorType, CommitBoundary, GhostMode, Quantization, TargetScope,
            UndoPolicy,
        },
        ids::{
            ActionId, AssetId, BankId, CaptureId, PadId, SceneId, SectionId, SnapshotId, SourceId,
        },
        persistence::{
            load_session_json, load_source_graph_json, save_session_json, save_source_graph_json,
        },
        session::{
            CaptureRef, CaptureTarget, CaptureType, GhostBudgetState, GhostState,
            GhostSuggestionRecord, GraphStorageMode, SessionFile, Snapshot, SourceGraphRef,
            SourceRef,
        },
        source_graph::{
            AnalysisSummary, AnalysisWarning, Asset, AssetType, Candidate, CandidateType,
            DecodeProfile, EnergyClass, GraphProvenance, QualityClass, Relationship,
            RelationshipType, Section, SectionLabelHint, SourceDescriptor, SourceGraph,
            SourceGraphVersion,
        },
        transport::TransportClockState,
    };
    use riotbox_sidecar::client::ClientError as SidecarClientError;

    use super::*;

    fn sample_audio_health(lifecycle: AudioRuntimeLifecycle) -> AudioRuntimeHealth {
        AudioRuntimeHealth {
            lifecycle,
            output: Some(AudioOutputInfo {
                host_name: "Alsa".into(),
                device_name: "default".into(),
                sample_format: "F32".into(),
                sample_rate: 44_100,
                channel_count: 2,
                buffer_size: "Default".into(),
                supported_output_config_count: Some(160),
            }),
            callback_count: 18,
            max_callback_gap_micros: Some(21_330),
            stream_error_count: u64::from(matches!(lifecycle, AudioRuntimeLifecycle::Faulted)),
            last_stream_error: matches!(lifecycle, AudioRuntimeLifecycle::Faulted)
                .then(|| "stream stalled".into()),
        }
    }

    fn sample_graph() -> SourceGraph {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "input.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 120.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into(), "section".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 7,
                run_notes: Some("app-test".into()),
            },
        );
        graph.sections.push(Section {
            section_id: SectionId::from("section-a"),
            label_hint: SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: EnergyClass::High,
            confidence: 0.9,
            tags: vec!["main".into()],
        });
        graph.assets.push(Asset {
            asset_id: AssetId::from("asset-a"),
            asset_type: AssetType::LoopWindow,
            start_seconds: 0.0,
            end_seconds: 4.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.8,
            tags: vec!["loop".into()],
            source_refs: vec!["src-1".into()],
        });
        graph.candidates.push(Candidate {
            candidate_id: "candidate-a".into(),
            candidate_type: CandidateType::LoopCandidate,
            asset_ref: "asset-a".into(),
            score: 0.88,
            confidence: 0.91,
            tags: vec!["useful".into()],
            constraints: vec!["bar_aligned".into()],
            provenance_refs: vec!["provider:beats".into()],
        });
        graph.relationships.push(Relationship {
            relation_type: RelationshipType::BelongsToSection,
            from_id: "asset-a".into(),
            to_id: "section-a".into(),
            weight: 1.0,
            notes: Some("primary loop".into()),
        });
        graph.analysis_summary = AnalysisSummary {
            overall_confidence: 0.87,
            timing_quality: QualityClass::High,
            section_quality: QualityClass::Medium,
            loop_candidate_count: 1,
            hook_candidate_count: 0,
            break_rebuild_potential: QualityClass::High,
            warnings: vec![AnalysisWarning {
                code: "low_hook_density".into(),
                message: "few hook fragments".into(),
            }],
        };
        graph
    }

    fn sample_session(graph: &SourceGraph) -> SessionFile {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
        session.source_refs.push(SourceRef {
            source_id: SourceId::from("src-1"),
            path_hint: "input.wav".into(),
            content_hash: "hash-1".into(),
            duration_seconds: 120.0,
            decode_profile: "normalized_stereo".into(),
        });
        session.source_graph_refs.push(SourceGraphRef {
            source_id: SourceId::from("src-1"),
            graph_version: SourceGraphVersion::V1,
            graph_hash: "graph-hash-1".into(),
            storage_mode: GraphStorageMode::Embedded,
            embedded_graph: Some(graph.clone()),
            external_path: None,
            provenance: graph.provenance.clone(),
        });
        session.runtime_state.transport.is_playing = true;
        session.runtime_state.transport.position_beats = 32.0;
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-1"));
        session.runtime_state.macro_state.scene_aggression = 0.75;
        session.runtime_state.lane_state.mc202.role = Some("follower".into());
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
        session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-1"));
        session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-1")];
        session.runtime_state.lock_state.locked_object_ids = vec!["ghost.main".into()];
        session.action_log.actions.push(Action {
            id: ActionId(1),
            actor: ActorType::User,
            command: ActionCommand::CaptureNow,
            params: ActionParams::Capture { bars: Some(2) },
            target: ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(BankId::from("bank-a")),
                pad_id: Some(PadId::from("pad-01")),
                ..Default::default()
            },
            requested_at: 100,
            quantization: Quantization::NextBar,
            status: ActionStatus::Committed,
            committed_at: Some(200),
            result: Some(ActionResult {
                accepted: true,
                summary: "captured".into(),
            }),
            undo_policy: UndoPolicy::Undoable,
            explanation: Some("capture current break".into()),
        });
        session.snapshots.push(Snapshot {
            snapshot_id: SnapshotId::from("snap-1"),
            created_at: "2026-04-12T18:05:00Z".into(),
            label: "first jam".into(),
            action_cursor: 1,
        });
        session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-01"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-a".into()],
            created_from_action: Some(ActionId(1)),
            storage_path: "captures/cap-01.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-a"),
                pad_id: PadId::from("pad-01"),
            }),
            notes: Some("keeper".into()),
        });
        session.ghost_state = GhostState {
            mode: GhostMode::Assist,
            budgets: GhostBudgetState {
                max_actions_per_phrase: 2,
                max_destructive_actions_per_scene: 1,
                max_pending_actions: 2,
            },
            suggestion_history: vec![GhostSuggestionRecord {
                proposal_id: "gp-1".into(),
                summary: "capture next bar".into(),
                accepted: false,
            }],
            lock_awareness_enabled: true,
        };
        session.notes = Some("keeper session".into());
        session
    }

    fn sidecar_script_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../python/sidecar/json_stdio_sidecar.py")
            .canonicalize()
            .expect("resolve sidecar script path")
    }

    fn write_pcm16_wave(
        path: impl AsRef<Path>,
        sample_rate: u32,
        channel_count: u16,
        duration_seconds: f32,
    ) {
        let path = path.as_ref();
        let frame_count = (sample_rate as f32 * duration_seconds) as u32;
        let bits_per_sample = 16_u16;
        let bytes_per_sample = (bits_per_sample / 8) as u32;
        let byte_rate = sample_rate * channel_count as u32 * bytes_per_sample;
        let block_align = channel_count * (bits_per_sample / 8);
        let data_len = frame_count * channel_count as u32 * bytes_per_sample;

        let mut bytes = Vec::with_capacity((44 + data_len) as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&channel_count.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&block_align.to_le_bytes());
        bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());

        for frame_index in 0..frame_count {
            let phase = (frame_index as f32 / sample_rate as f32) * 220.0 * 2.0 * PI;
            let sample = (phase.sin() * i16::MAX as f32 * 0.25) as i16;
            for _ in 0..channel_count {
                bytes.extend_from_slice(&sample.to_le_bytes());
            }
        }

        fs::write(path, bytes).expect("write PCM wave fixture");
    }

    #[test]
    fn builds_jam_app_state_from_parts() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert!(state.jam_view.transport.is_playing);
        assert_eq!(state.jam_view.scene.scene_count, 1);
        assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("follower"));
        assert_eq!(state.runtime_view.audio_status, "unknown");
        assert_eq!(state.runtime_view.sidecar_status, "unknown");
    }

    #[test]
    fn loads_and_saves_jam_app_state_from_files() {
        let dir = tempdir().expect("create temp dir");
        let session_path = dir.path().join("sessions").join("session.json");
        let graph_path = dir.path().join("graphs").join("source-graph.json");

        let graph = sample_graph();
        let session = sample_session(&graph);
        save_session_json(&session_path, &session).expect("save session fixture");
        save_source_graph_json(&graph_path, &graph).expect("save graph fixture");

        let mut state =
            JamAppState::from_json_files(&session_path, &graph_path).expect("load app state");
        assert!(state.jam_view.transport.is_playing);
        assert_eq!(state.jam_view.source.section_count, 1);

        state.session.notes = Some("updated".into());
        state.refresh_view();
        state.save().expect("save app state");

        let persisted_session = load_session_json(&session_path).expect("reload session");
        let persisted_graph = load_source_graph_json(&graph_path).expect("reload graph");

        assert_eq!(persisted_session.notes.as_deref(), Some("updated"));
        assert_eq!(persisted_graph, graph);
    }

    #[test]
    fn runtime_view_updates_from_audio_and_sidecar_state() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.set_audio_health(sample_audio_health(AudioRuntimeLifecycle::Running));
        state.set_sidecar_state(SidecarState::Ready {
            version: Some("0.1.0".into()),
            transport: "stdio-ndjson".into(),
        });

        assert_eq!(state.runtime_view.audio_status, "running");
        assert_eq!(state.runtime_view.audio_callback_count, 18);
        assert_eq!(state.runtime_view.sidecar_status, "ready");
        assert_eq!(state.runtime_view.sidecar_version.as_deref(), Some("0.1.0"));
        assert!(state.runtime_view.runtime_warnings.is_empty());
    }

    #[test]
    fn runtime_view_surfaces_faulted_and_degraded_states() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.set_audio_health(sample_audio_health(AudioRuntimeLifecycle::Faulted));
        state.set_sidecar_state(SidecarState::Degraded {
            reason: "worker restart pending".into(),
        });

        assert_eq!(state.runtime_view.audio_status, "faulted");
        assert_eq!(
            state.runtime_view.audio_last_error.as_deref(),
            Some("stream stalled")
        );
        assert_eq!(state.runtime_view.sidecar_status, "degraded");
        assert!(
            state
                .runtime_view
                .runtime_warnings
                .iter()
                .any(|warning| warning == "audio runtime faulted")
        );
        assert!(
            state
                .runtime_view
                .runtime_warnings
                .iter()
                .any(|warning| warning.contains("sidecar degraded"))
        );
    }

    #[test]
    fn updates_transport_clock_and_refreshes_jam_state() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        let clock = TransportClockState {
            is_playing: false,
            position_beats: 48.5,
            beat_index: 48,
            bar_index: 13,
            phrase_index: 4,
            current_scene: Some(SceneId::from("scene-2")),
        };

        state.update_transport_clock(clock.clone());

        assert_eq!(state.runtime.transport, clock);
        assert!(!state.session.runtime_state.transport.is_playing);
        assert_eq!(state.session.runtime_state.transport.position_beats, 48.5);
        assert_eq!(
            state
                .session
                .runtime_state
                .transport
                .current_scene
                .as_ref()
                .map(ToString::to_string),
            Some("scene-2".into())
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .scene_state
                .active_scene
                .as_ref()
                .map(ToString::to_string),
            Some("scene-2".into())
        );
        assert!(!state.jam_view.transport.is_playing);
        assert_eq!(state.jam_view.transport.position_beats, 48.5);
        assert_eq!(
            state.jam_view.scene.active_scene.as_deref(),
            Some("scene-2")
        );
    }

    #[test]
    fn commits_ready_actions_into_session_log_in_stable_order() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        let first = state.queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::CaptureNow,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    ..Default::default()
                },
            ),
            300,
        );
        let second = state.queue.enqueue(
            ActionDraft::new(
                ActorType::Ghost,
                ActionCommand::MutateScene,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::Scene),
                    ..Default::default()
                },
            ),
            301,
        );

        let boundary = CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 64,
            bar_index: 17,
            phrase_index: 4,
            scene_id: Some(SceneId::from("scene-1")),
        };

        let committed = state.commit_ready_actions(boundary.clone(), 400);

        assert_eq!(committed.len(), 2);
        assert_eq!(committed[0].action_id, first);
        assert_eq!(committed[0].commit_sequence, 1);
        assert_eq!(committed[1].action_id, second);
        assert_eq!(committed[1].commit_sequence, 2);
        assert_eq!(state.runtime.last_commit_boundary, Some(boundary));
        assert_eq!(state.queue.pending_actions().len(), 0);
        assert_eq!(state.session.action_log.actions.len(), 3);
        assert_eq!(state.session.action_log.actions[1].id, first);
        assert_eq!(state.session.action_log.actions[2].id, second);
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            vec![ActionId(1), first, second]
        );
        assert_eq!(state.jam_view.pending_actions.len(), 0);
        assert_eq!(state.jam_view.recent_actions[0].id, second.to_string());
        assert_eq!(state.jam_view.recent_actions[1].id, first.to_string());
    }

    #[test]
    fn ingests_source_file_through_sidecar_and_persists_state() {
        let dir = tempdir().expect("create temp dir");
        let source_path = dir.path().join("input.wav");
        let session_path = dir.path().join("sessions").join("session.json");
        let graph_path = dir.path().join("graphs").join("source-graph.json");

        write_pcm16_wave(&source_path, 44_100, 2, 2.0);

        let state = JamAppState::analyze_source_file_to_json(
            &source_path,
            &session_path,
            &graph_path,
            sidecar_script_path(),
            29,
        )
        .expect("ingest source file");

        assert_eq!(state.runtime_view.sidecar_status, "ready");
        assert_eq!(state.runtime_view.sidecar_version.as_deref(), Some("0.1.0"));
        assert_eq!(
            state
                .source_graph
                .as_ref()
                .map(|graph| graph.source.path.clone()),
            Some(source_path.to_string_lossy().into_owned())
        );
        assert_eq!(state.session.source_refs.len(), 1);
        assert_eq!(state.session.source_graph_refs.len(), 1);
        assert_eq!(
            state.session.source_graph_refs[0].storage_mode,
            GraphStorageMode::External
        );
        assert_eq!(
            state.session.source_graph_refs[0].external_path.as_deref(),
            Some(graph_path.to_string_lossy().as_ref())
        );
        assert!(session_path.exists());
        assert!(graph_path.exists());

        let persisted_graph = load_source_graph_json(&graph_path).expect("reload graph");
        assert_eq!(
            persisted_graph.provenance.provider_set,
            vec!["decoded.wav_baseline"]
        );
        assert_eq!(persisted_graph.provenance.analysis_seed, 29);
        assert_eq!(persisted_graph.source.sample_rate, 44_100);
        assert_eq!(persisted_graph.source.channel_count, 2);
        assert!(persisted_graph.source.duration_seconds >= 1.9);
        assert!(persisted_graph.timing.bpm_estimate.is_some());
    }

    #[test]
    fn ingest_surfaces_missing_source_file_as_sidecar_error() {
        let dir = tempdir().expect("create temp dir");
        let source_path = dir.path().join("missing.wav");
        let session_path = dir.path().join("sessions").join("session.json");
        let graph_path = dir.path().join("graphs").join("source-graph.json");

        let error = JamAppState::analyze_source_file_to_json(
            &source_path,
            &session_path,
            &graph_path,
            sidecar_script_path(),
            29,
        )
        .expect_err("missing source should fail");

        match error {
            JamAppError::Io(io_error) => {
                assert_eq!(io_error.kind(), io::ErrorKind::NotFound);
            }
            JamAppError::Sidecar(SidecarClientError::Sidecar(payload)) => {
                assert_eq!(payload.code, "source_missing");
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
