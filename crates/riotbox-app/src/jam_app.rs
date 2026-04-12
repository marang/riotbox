use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use riotbox_core::{
    persistence::{
        PersistenceError, load_session_json, load_source_graph_json, save_session_json,
        save_source_graph_json,
    },
    queue::ActionQueue,
    session::SessionFile,
    source_graph::SourceGraph,
    view::jam::JamViewModel,
};

#[derive(Debug)]
pub enum JamAppError {
    Persistence(PersistenceError),
}

impl Display for JamAppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Persistence(error) => write!(f, "persistence error: {error}"),
        }
    }
}

impl Error for JamAppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Persistence(error) => Some(error),
        }
    }
}

impl From<PersistenceError> for JamAppError {
    fn from(value: PersistenceError) -> Self {
        Self::Persistence(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JamFileSet {
    pub session_path: PathBuf,
    pub source_graph_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct JamAppState {
    pub files: Option<JamFileSet>,
    pub session: SessionFile,
    pub source_graph: Option<SourceGraph>,
    pub queue: ActionQueue,
    pub jam_view: JamViewModel,
}

impl JamAppState {
    #[must_use]
    pub fn from_parts(
        session: SessionFile,
        source_graph: Option<SourceGraph>,
        queue: ActionQueue,
    ) -> Self {
        let jam_view = JamViewModel::build(&session, &queue, source_graph.as_ref());

        Self {
            files: None,
            session,
            source_graph,
            queue,
            jam_view,
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
        let queue = ActionQueue::new();
        let jam_view = JamViewModel::build(&session, &queue, Some(&source_graph));

        Ok(Self {
            files: Some(JamFileSet {
                session_path,
                source_graph_path,
            }),
            session,
            source_graph: Some(source_graph),
            queue,
            jam_view,
        })
    }

    pub fn refresh_view(&mut self) {
        self.jam_view = JamViewModel::build(&self.session, &self.queue, self.source_graph.as_ref());
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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use riotbox_core::{
        action::{
            Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget,
            ActorType, GhostMode, Quantization, TargetScope, UndoPolicy,
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

    #[test]
    fn builds_jam_app_state_from_parts() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert!(state.jam_view.transport.is_playing);
        assert_eq!(state.jam_view.scene.scene_count, 1);
        assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("follower"));
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
}
