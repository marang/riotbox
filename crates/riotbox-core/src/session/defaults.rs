impl Default for GhostState {
    fn default() -> Self {
        Self {
            mode: GhostMode::Watch,
            budgets: GhostBudgetState::default(),
            suggestion_history: Vec::new(),
            lock_awareness_enabled: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GhostBudgetState {
    pub max_actions_per_phrase: u8,
    pub max_destructive_actions_per_scene: u8,
    pub max_pending_actions: u8,
}

impl Default for GhostBudgetState {
    fn default() -> Self {
        Self {
            max_actions_per_phrase: 2,
            max_destructive_actions_per_scene: 1,
            max_pending_actions: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GhostSuggestionRecord {
    pub proposal_id: String,
    pub summary: String,
    pub accepted: bool,
}

#[cfg(test)]
mod tests {
    use crate::{
        action::{
            Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget,
            ActorType, GhostMode, Quantization, TargetScope, UndoPolicy,
        },
        ids::{ActionId, BankId, CaptureId, PadId, SceneId, SnapshotId, SourceId},
        source_graph::{
            DecodeProfile, GraphProvenance, SourceDescriptor, SourceGraph, SourceGraphVersion,
        },
    };

    use super::*;

    #[test]
    fn session_file_roundtrips_via_json() {
        let graph = SourceGraph::new(
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
                run_notes: Some("roundtrip".into()),
            },
        );

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
        session
            .runtime_state
            .undo_state
            .mc202_snapshots
            .push(Mc202UndoSnapshotState {
                action_id: ActionId(2),
                role: Some("follower".into()),
                phrase_ref: Some("follower-scene-1".into()),
                phrase_variant: Some(Mc202PhraseVariantState::MutatedDrive),
                touch: 0.78,
            });
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
            }],
            lock_awareness_enabled: true,
        };
        session.notes = Some("keeper session".into());

        let json = serde_json::to_string_pretty(&session).expect("serialize session");
        let decoded: SessionFile = serde_json::from_str(&json).expect("deserialize session");

        assert_eq!(decoded, session);
    }

    #[test]
    fn legacy_capture_refs_without_source_window_still_load() {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
        session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-01"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-a".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: Some(ActionId(1)),
            storage_path: "captures/cap-01.wav".into(),
            assigned_target: None,
            is_pinned: false,
            notes: None,
        });

        let mut value = serde_json::to_value(&session).expect("serialize session");
        value["captures"][0]
            .as_object_mut()
            .expect("capture object")
            .remove("source_window");

        let decoded: SessionFile =
            serde_json::from_value(value).expect("deserialize legacy session");

        assert_eq!(decoded.captures[0].source_window, None);
    }
}
