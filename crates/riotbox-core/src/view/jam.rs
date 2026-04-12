use crate::{queue::ActionQueue, session::SessionFile, source_graph::SourceGraph};

#[derive(Clone, Debug, PartialEq)]
pub struct JamViewModel {
    pub transport: JamTransportView,
    pub source: SourceSummaryView,
    pub scene: SceneSummaryView,
    pub macros: MacroStripView,
    pub lanes: LaneSummaryView,
    pub pending_actions: Vec<PendingActionView>,
    pub recent_actions: Vec<RecentActionView>,
    pub ghost: GhostStatusView,
    pub warnings: Vec<String>,
}

impl JamViewModel {
    #[must_use]
    pub fn build(session: &SessionFile, queue: &ActionQueue, graph: Option<&SourceGraph>) -> Self {
        let pending_actions: Vec<PendingActionView> = queue
            .pending_actions()
            .into_iter()
            .map(|action| PendingActionView {
                id: action.id.to_string(),
                actor: action.actor.to_string(),
                command: action.command.to_string(),
                quantization: action.quantization.to_string(),
            })
            .collect();

        let recent_actions = session
            .action_log
            .actions
            .iter()
            .rev()
            .take(5)
            .map(|action| RecentActionView {
                id: action.id.to_string(),
                actor: action.actor.to_string(),
                command: action.command.to_string(),
                status: format!("{:?}", action.status).to_lowercase(),
            })
            .collect();

        let source = match graph {
            Some(graph) => SourceSummaryView {
                source_id: graph.source.source_id.to_string(),
                bpm_estimate: graph.timing.bpm_estimate,
                bpm_confidence: graph.timing.bpm_confidence,
                section_count: graph.sections.len(),
                loop_candidate_count: graph.loop_candidate_count(),
                hook_candidate_count: graph.hook_candidate_count(),
            },
            None => SourceSummaryView::default(),
        };

        let mut warnings = graph.map_or_else(Vec::new, SourceGraph::warnings);
        if pending_actions.is_empty() && !session.transport().is_playing {
            warnings.push("transport idle".into());
        }

        Self {
            transport: JamTransportView {
                is_playing: session.transport().is_playing,
                position_beats: session.transport().position_beats,
            },
            source,
            scene: SceneSummaryView {
                active_scene: session
                    .runtime_state
                    .scene_state
                    .active_scene
                    .as_ref()
                    .map(ToString::to_string),
                scene_count: session.runtime_state.scene_state.scenes.len(),
            },
            macros: MacroStripView {
                source_retain: session.runtime_state.macro_state.source_retain,
                chaos: session.runtime_state.macro_state.chaos,
                mc202_touch: session.runtime_state.macro_state.mc202_touch,
                w30_grit: session.runtime_state.macro_state.w30_grit,
                tr909_slam: session.runtime_state.macro_state.tr909_slam,
            },
            lanes: LaneSummaryView {
                mc202_role: session.runtime_state.lane_state.mc202.role.clone(),
                w30_active_bank: session
                    .runtime_state
                    .lane_state
                    .w30
                    .active_bank
                    .as_ref()
                    .map(ToString::to_string),
                tr909_slam_enabled: session.runtime_state.lane_state.tr909.slam_enabled,
            },
            pending_actions,
            recent_actions,
            ghost: GhostStatusView {
                mode: session.ghost_state.mode.to_string(),
                suggestion_count: session.ghost_state.suggestion_history.len(),
                is_blocked: session
                    .runtime_state
                    .lock_state
                    .locked_object_ids
                    .iter()
                    .any(|lock| lock.contains("ghost")),
            },
            warnings,
        }
    }
}

trait SessionAccessors {
    fn transport(&self) -> &crate::session::TransportRuntimeState;
}

impl SessionAccessors for SessionFile {
    fn transport(&self) -> &crate::session::TransportRuntimeState {
        &self.runtime_state.transport
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct JamTransportView {
    pub is_playing: bool,
    pub position_beats: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SourceSummaryView {
    pub source_id: String,
    pub bpm_estimate: Option<f32>,
    pub bpm_confidence: f32,
    pub section_count: usize,
    pub loop_candidate_count: usize,
    pub hook_candidate_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneSummaryView {
    pub active_scene: Option<String>,
    pub scene_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MacroStripView {
    pub source_retain: f32,
    pub chaos: f32,
    pub mc202_touch: f32,
    pub w30_grit: f32,
    pub tr909_slam: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneSummaryView {
    pub mc202_role: Option<String>,
    pub w30_active_bank: Option<String>,
    pub tr909_slam_enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingActionView {
    pub id: String,
    pub actor: String,
    pub command: String,
    pub quantization: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentActionView {
    pub id: String,
    pub actor: String,
    pub command: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GhostStatusView {
    pub mode: String,
    pub suggestion_count: usize,
    pub is_blocked: bool,
}

#[cfg(test)]
mod tests {
    use crate::{
        action::{
            ActionCommand, ActionDraft, ActionTarget, ActorType, GhostMode, Quantization,
            TargetScope, UndoPolicy,
        },
        ids::{BankId, SceneId, SourceId},
        queue::ActionQueue,
        session::{ActionLog, GhostSuggestionRecord, RuntimeState, SessionFile, SourceGraphRef},
        source_graph::{
            AnalysisSummary, Asset, AssetType, Candidate, CandidateType, DecodeProfile,
            GraphProvenance, QualityClass, SourceDescriptor, SourceGraph,
        },
    };

    use super::*;

    #[test]
    fn builds_minimal_jam_view_model() {
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
                run_notes: None,
            },
        );
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-a".into(),
            label_hint: crate::source_graph::SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: crate::source_graph::EnergyClass::High,
            confidence: 0.9,
            tags: vec![],
        });
        graph.assets.push(Asset {
            asset_id: "asset-a".into(),
            asset_type: AssetType::LoopWindow,
            start_seconds: 0.0,
            end_seconds: 4.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.8,
            tags: vec![],
            source_refs: vec![],
        });
        graph.candidates.push(Candidate {
            candidate_id: "cand-loop".into(),
            candidate_type: CandidateType::LoopCandidate,
            asset_ref: "asset-a".into(),
            score: 0.9,
            confidence: 0.9,
            tags: vec![],
            constraints: vec![],
            provenance_refs: vec![],
        });
        graph.candidates.push(Candidate {
            candidate_id: "cand-hook".into(),
            candidate_type: CandidateType::HookCandidate,
            asset_ref: "asset-a".into(),
            score: 0.7,
            confidence: 0.8,
            tags: vec![],
            constraints: vec![],
            provenance_refs: vec![],
        });
        graph.analysis_summary = AnalysisSummary {
            overall_confidence: 0.85,
            timing_quality: QualityClass::High,
            section_quality: QualityClass::Medium,
            loop_candidate_count: 1,
            hook_candidate_count: 1,
            break_rebuild_potential: QualityClass::High,
            warnings: vec![],
        };

        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
        session.runtime_state.transport.is_playing = true;
        session.runtime_state.transport.position_beats = 16.0;
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-1"));
        session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-1")];
        session.runtime_state.lane_state.mc202.role = Some("follower".into());
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        session.runtime_state.lane_state.tr909.slam_enabled = true;
        session.ghost_state.mode = GhostMode::Assist;
        session.ghost_state.suggestion_history = vec![GhostSuggestionRecord {
            proposal_id: "gp-1".into(),
            summary: "capture next bar".into(),
            accepted: false,
        }];
        session.action_log = ActionLog {
            actions: vec![],
            replay_policy: crate::session::ReplayPolicy::DeterministicPreferred,
        };
        session.source_graph_refs = vec![SourceGraphRef {
            source_id: SourceId::from("src-1"),
            graph_version: crate::source_graph::SourceGraphVersion::V1,
            graph_hash: "graph-1".into(),
            storage_mode: crate::session::GraphStorageMode::Embedded,
            embedded_graph: Some(graph.clone()),
            external_path: None,
            provenance: graph.provenance.clone(),
        }];
        session.runtime_state = RuntimeState {
            transport: session.runtime_state.transport.clone(),
            macro_state: session.runtime_state.macro_state.clone(),
            lane_state: session.runtime_state.lane_state.clone(),
            mixer_state: session.runtime_state.mixer_state.clone(),
            scene_state: session.runtime_state.scene_state.clone(),
            lock_state: session.runtime_state.lock_state.clone(),
            pending_policy: session.runtime_state.pending_policy.clone(),
        };

        let mut queue = ActionQueue::new();
        let mut draft = ActionDraft::new(
            ActorType::Ghost,
            ActionCommand::CaptureNow,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
        );
        draft.undo_policy = UndoPolicy::Undoable;
        draft.explanation = Some("capture current break".into());
        queue.enqueue(draft, 100);

        let vm = JamViewModel::build(&session, &queue, Some(&graph));

        assert!(vm.transport.is_playing);
        assert_eq!(vm.source.loop_candidate_count, 1);
        assert_eq!(vm.source.hook_candidate_count, 1);
        assert_eq!(vm.scene.scene_count, 1);
        assert_eq!(vm.pending_actions.len(), 1);
        assert_eq!(vm.ghost.mode, "assist");
    }
}
