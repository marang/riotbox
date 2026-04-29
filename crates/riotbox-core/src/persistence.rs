use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{session::SessionFile, source_graph::SourceGraph};

#[derive(Debug)]
pub enum PersistenceError {
    Io(io::Error),
    Json(serde_json::Error),
}

impl Display for PersistenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Json(error) => write!(f, "JSON error: {error}"),
        }
    }
}

impl Error for PersistenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
        }
    }
}

impl From<io::Error> for PersistenceError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub fn save_source_graph_json(
    path: impl AsRef<Path>,
    graph: &SourceGraph,
) -> Result<(), PersistenceError> {
    save_json(path, graph)
}

pub fn load_source_graph_json(path: impl AsRef<Path>) -> Result<SourceGraph, PersistenceError> {
    load_json(path)
}

pub fn save_session_json(
    path: impl AsRef<Path>,
    session: &SessionFile,
) -> Result<(), PersistenceError> {
    save_json(path, session)
}

pub fn load_session_json(path: impl AsRef<Path>) -> Result<SessionFile, PersistenceError> {
    load_json(path)
}

fn save_json<T>(path: impl AsRef<Path>, value: &T) -> Result<(), PersistenceError>
where
    T: serde::Serialize,
{
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(value)?;

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let temp_path = atomic_save_temp_path(path);
    if temp_path.exists() {
        fs::remove_file(&temp_path)?;
    }

    fs::write(&temp_path, json)?;
    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error.into());
    }

    Ok(())
}

fn atomic_save_temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("riotbox.json");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    path.with_file_name(format!(".{file_name}.tmp-{}-{nonce}", std::process::id()))
}

fn load_json<T>(path: impl AsRef<Path>) -> Result<T, PersistenceError>
where
    T: serde::de::DeserializeOwned,
{
    let json = fs::read_to_string(path)?;
    let value = serde_json::from_str(&json)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        action::{
            Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget,
            ActorType, GhostMode, Quantization, TargetScope, UndoPolicy,
        },
        ids::{
            ActionId, AssetId, BankId, CaptureId, PadId, SceneId, SectionId, SnapshotId, SourceId,
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
    };

    use super::*;

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
                run_notes: Some("io-test".into()),
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
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: Some(ActionId(1)),
            storage_path: "captures/cap-01.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-a"),
                pad_id: PadId::from("pad-01"),
            }),
            is_pinned: false,
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
                rejected: false,
            }],
            lock_awareness_enabled: true,
        };
        session.notes = Some("keeper session".into());
        session
    }

    #[test]
    fn saves_and_loads_source_graph_json_file() {
        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("graphs").join("source-graph.json");
        let graph = sample_graph();

        save_source_graph_json(&path, &graph).expect("save source graph");
        let loaded = load_source_graph_json(&path).expect("load source graph");

        assert_eq!(loaded, graph);

        let json = fs::read_to_string(path).expect("read graph file");
        assert!(json.contains("\"graph_version\""));
    }

    #[test]
    fn saves_and_loads_session_json_file() {
        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("sessions").join("session.json");
        let graph = sample_graph();
        let session = sample_session(&graph);

        save_session_json(&path, &session).expect("save session");
        let loaded = load_session_json(&path).expect("load session");

        assert_eq!(loaded, session);

        let json = fs::read_to_string(path).expect("read session file");
        assert!(json.contains("\"session_version\""));
    }

    #[test]
    fn save_json_replaces_existing_file_after_successful_serialization() {
        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("session.json");
        let graph = sample_graph();
        let old_session = sample_session(&graph);
        let mut new_session = sample_session(&graph);
        new_session.notes = Some("replacement session".into());

        save_session_json(&path, &old_session).expect("save old session");
        save_session_json(&path, &new_session).expect("replace session");

        let loaded = load_session_json(&path).expect("load replaced session");
        assert_eq!(loaded.notes.as_deref(), Some("replacement session"));
    }

    #[test]
    fn save_json_serialization_failure_does_not_clobber_existing_file() {
        struct FailingSerialize;

        impl serde::Serialize for FailingSerialize {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                Err(<S::Error as serde::ser::Error>::custom(
                    "intentional serialization failure",
                ))
            }
        }

        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("session.json");
        fs::write(&path, "existing session").expect("write existing session");

        let error = save_json(&path, &FailingSerialize).expect_err("save should fail");

        assert!(matches!(error, PersistenceError::Json(_)));
        assert_eq!(
            fs::read_to_string(&path).expect("read existing session"),
            "existing session"
        );
    }

    #[test]
    fn truncated_session_json_load_fails_without_replacing_adjacent_valid_session() {
        let dir = tempdir().expect("create temp dir");
        let graph = sample_graph();
        let session = sample_session(&graph);
        let valid_path = dir.path().join("session-valid.json");
        let truncated_path = dir.path().join("session.json");

        save_session_json(&valid_path, &session).expect("save adjacent valid session");
        let valid_json = fs::read_to_string(&valid_path).expect("read valid session");
        let truncated_json = &valid_json[..valid_json.len() / 2];
        fs::write(&truncated_path, truncated_json).expect("write truncated session");

        let error = load_session_json(&truncated_path).expect_err("truncated load should fail");

        assert!(matches!(&error, PersistenceError::Json(json_error) if json_error.is_eof()));
        assert_eq!(
            fs::read_to_string(&truncated_path).expect("read truncated session"),
            truncated_json
        );
        let adjacent_valid =
            load_session_json(&valid_path).expect("adjacent valid session stays loadable");
        assert_eq!(adjacent_valid, session);
    }
}
