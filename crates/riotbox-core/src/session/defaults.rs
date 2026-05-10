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
    #[serde(default)]
    pub rejected: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GhostSuggestionStatus {
    Suggested,
    Accepted,
    Rejected,
}

impl GhostSuggestionStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }
}

impl GhostSuggestionRecord {
    #[must_use]
    pub const fn status(&self) -> GhostSuggestionStatus {
        if self.rejected {
            GhostSuggestionStatus::Rejected
        } else if self.accepted {
            GhostSuggestionStatus::Accepted
        } else {
            GhostSuggestionStatus::Suggested
        }
    }

    pub fn mark_accepted(&mut self) {
        self.accepted = true;
        self.rejected = false;
    }

    pub fn mark_rejected(&mut self) {
        self.accepted = false;
        self.rejected = true;
    }
}

impl GhostState {
    pub fn accept_suggestion(&mut self, proposal_id: &str) -> bool {
        if !matches!(self.mode, GhostMode::Assist) {
            return false;
        }

        let Some(record) = self
            .suggestion_history
            .iter_mut()
            .rev()
            .find(|record| record.proposal_id == proposal_id)
        else {
            return false;
        };

        if record.rejected {
            return false;
        }

        record.mark_accepted();
        true
    }

    pub fn reject_suggestion(&mut self, proposal_id: &str) -> bool {
        let Some(record) = self
            .suggestion_history
            .iter_mut()
            .rev()
            .find(|record| record.proposal_id == proposal_id)
        else {
            return false;
        };

        record.mark_rejected();
        true
    }
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
    fn mc202_role_labels_roundtrip_through_typed_contract() {
        let cases = [
            (Mc202RoleState::Leader, "leader", 0.85),
            (Mc202RoleState::Follower, "follower", 0.78),
            (Mc202RoleState::Answer, "answer", 0.82),
            (Mc202RoleState::Pressure, "pressure", 0.84),
            (Mc202RoleState::Instigator, "instigator", 0.90),
        ];

        for (role, label, default_touch) in cases {
            assert_eq!(role.label(), label);
            assert_eq!(Mc202RoleState::from_label(label), Some(role));
            assert_eq!(serde_json::to_string(&role).unwrap(), format!("\"{label}\""));
            assert_eq!(serde_json::from_str::<Mc202RoleState>(&format!("\"{label}\"")).unwrap(), role);
            assert!((role.default_touch() - default_touch).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn mc202_role_labels_reject_unknown_behavior_labels() {
        for label in ["", "follow", "mutated_drive", "scene_lock"] {
            assert_eq!(Mc202RoleState::from_label(label), None);
        }
    }

    #[test]
    fn mc202_phrase_intent_preserves_existing_mutation_variant_label() {
        assert_eq!(Mc202PhraseIntentState::Base.label(), "base");
        assert_eq!(Mc202PhraseIntentState::Base.phrase_variant(), None);
        assert_eq!(Mc202PhraseIntentState::from_label("base"), Some(Mc202PhraseIntentState::Base));
        assert_eq!(
            Mc202PhraseIntentState::from_phrase_variant(None),
            Mc202PhraseIntentState::Base
        );

        assert_eq!(Mc202PhraseIntentState::MutatedDrive.label(), "mutated_drive");
        assert_eq!(
            Mc202PhraseIntentState::MutatedDrive.phrase_variant(),
            Some(Mc202PhraseVariantState::MutatedDrive)
        );
        assert_eq!(
            Mc202PhraseIntentState::from_label("mutated_drive"),
            Some(Mc202PhraseIntentState::MutatedDrive)
        );
        assert_eq!(
            Mc202PhraseIntentState::from_phrase_variant(Some(Mc202PhraseVariantState::MutatedDrive)),
            Mc202PhraseIntentState::MutatedDrive
        );
        assert_eq!(
            serde_json::to_string(&Mc202PhraseIntentState::MutatedDrive).unwrap(),
            "\"mutated_drive\""
        );
    }

    #[test]
    fn mc202_phrase_intent_rejects_unknown_behavior_labels() {
        for label in ["", "leader", "mutated", "answer_space"] {
            assert_eq!(Mc202PhraseIntentState::from_label(label), None);
        }
    }

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
            payload: None,
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

        let json = serde_json::to_string_pretty(&session).expect("serialize session");
        let decoded: SessionFile = serde_json::from_str(&json).expect("deserialize session");

        assert_eq!(decoded, session);
    }

    #[test]
    fn assist_accepts_suggestion_without_implicit_musical_state_change() {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-29T16:45:00Z");
        session.ghost_state.mode = GhostMode::Assist;
        session
            .ghost_state
            .suggestion_history
            .push(GhostSuggestionRecord {
                proposal_id: "ghost-proposal-1".into(),
                summary: "fill next bar".into(),
                accepted: false,
                rejected: false,
            });
        let before_log = session.action_log.actions.len();
        let before_scene = session.runtime_state.scene_state.clone();

        assert!(session
            .ghost_state
            .accept_suggestion("ghost-proposal-1"));

        let record = &session.ghost_state.suggestion_history[0];
        assert_eq!(record.status(), GhostSuggestionStatus::Accepted);
        assert!(record.accepted);
        assert!(!record.rejected);
        assert_eq!(session.action_log.actions.len(), before_log);
        assert_eq!(session.runtime_state.scene_state, before_scene);
    }

    #[test]
    fn watch_accept_does_not_mutate_suggestion_state() {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-29T16:46:00Z");
        session.ghost_state.mode = GhostMode::Watch;
        session
            .ghost_state
            .suggestion_history
            .push(GhostSuggestionRecord {
                proposal_id: "ghost-proposal-1".into(),
                summary: "fill next bar".into(),
                accepted: false,
                rejected: false,
            });

        assert!(!session
            .ghost_state
            .accept_suggestion("ghost-proposal-1"));

        let record = &session.ghost_state.suggestion_history[0];
        assert_eq!(record.status(), GhostSuggestionStatus::Suggested);
        assert!(!record.accepted);
        assert!(!record.rejected);
    }

    #[test]
    fn rejected_suggestion_is_distinct_from_unaccepted_suggestion() {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-29T16:47:00Z");
        session
            .ghost_state
            .suggestion_history
            .push(GhostSuggestionRecord {
                proposal_id: "ghost-proposal-1".into(),
                summary: "fill next bar".into(),
                accepted: false,
                rejected: false,
            });

        assert!(session
            .ghost_state
            .reject_suggestion("ghost-proposal-1"));

        let record = &session.ghost_state.suggestion_history[0];
        assert_eq!(record.status(), GhostSuggestionStatus::Rejected);
        assert!(!record.accepted);
        assert!(record.rejected);
    }

    #[test]
    fn rejected_suggestion_cannot_be_accepted_later_without_new_proposal() {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-29T16:48:00Z");
        session.ghost_state.mode = GhostMode::Assist;
        session
            .ghost_state
            .suggestion_history
            .push(GhostSuggestionRecord {
                proposal_id: "ghost-proposal-1".into(),
                summary: "fill next bar".into(),
                accepted: false,
                rejected: false,
            });

        assert!(session
            .ghost_state
            .reject_suggestion("ghost-proposal-1"));
        assert!(!session
            .ghost_state
            .accept_suggestion("ghost-proposal-1"));

        let record = &session.ghost_state.suggestion_history[0];
        assert_eq!(record.status(), GhostSuggestionStatus::Rejected);
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
