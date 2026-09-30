use crate::jam_app::state::JamAppState;
use crate::jam_app::tests::fixtures::regression_models::SceneRegressionFixture;
use crate::test_support::scene_energy_for_label;
use crate::test_support::scene_label_hint;
use riotbox_core::action::Action;
use riotbox_core::action::ActionCommand;
use riotbox_core::action::ActionParams;
use riotbox_core::action::ActionResult;
use riotbox_core::action::ActionStatus;
use riotbox_core::action::ActionTarget;
use riotbox_core::action::ActorType;
use riotbox_core::action::GhostMode;
use riotbox_core::action::Quantization;
use riotbox_core::action::TargetScope;
use riotbox_core::action::UndoPolicy;
use riotbox_core::ids::ActionId;
use riotbox_core::ids::AssetId;
use riotbox_core::ids::BankId;
use riotbox_core::ids::CaptureId;
use riotbox_core::ids::PadId;
use riotbox_core::ids::SceneId;
use riotbox_core::ids::SectionId;
use riotbox_core::ids::SnapshotId;
use riotbox_core::ids::SourceId;
use riotbox_core::queue::ActionQueue;
use riotbox_core::session::CaptureRef;
use riotbox_core::session::CaptureType;
use riotbox_core::session::GhostBudgetState;
use riotbox_core::session::GhostState;
use riotbox_core::session::GhostSuggestionRecord;
use riotbox_core::session::GraphStorageMode;
use riotbox_core::session::Mc202RoleState;
use riotbox_core::session::SessionFile;
use riotbox_core::session::Snapshot;
use riotbox_core::session::SourceGraphRef;
use riotbox_core::session::SourceRef;
use riotbox_core::session::W30PreviewModeState;
use riotbox_core::source_graph::AnalysisSummary;
use riotbox_core::source_graph::AnalysisWarning;
use riotbox_core::source_graph::Asset;
use riotbox_core::source_graph::AssetType;
use riotbox_core::source_graph::Candidate;
use riotbox_core::source_graph::CandidateType;
use riotbox_core::source_graph::DecodeProfile;
use riotbox_core::source_graph::EnergyClass;
use riotbox_core::source_graph::GraphProvenance;
use riotbox_core::source_graph::QualityClass;
use riotbox_core::source_graph::Relationship;
use riotbox_core::source_graph::RelationshipType;
use riotbox_core::source_graph::Section;
use riotbox_core::source_graph::SectionLabelHint;
use riotbox_core::source_graph::SourceDescriptor;
use riotbox_core::source_graph::SourceGraph;
use riotbox_core::source_graph::SourceGraphVersion;

pub(crate) fn sample_graph() -> SourceGraph {
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
        from_id: riotbox_core::source_graph::GraphNodeRef::Asset("asset-a".into()),
        to_id: riotbox_core::source_graph::GraphNodeRef::Section("section-a".into()),
        weight: 1.0,
        notes: Some("primary loop".into()),
    });
    graph.timing.bpm_estimate = Some(126.0);
    graph.timing.bpm_confidence = 0.81;
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

pub(in crate::jam_app::tests) fn scene_regression_graph(section_labels: &[String]) -> SourceGraph {
    let mut graph = sample_graph();
    graph.sections.clear();
    // This fixture replaces the section catalog, not just its presentation.
    // The sample graph's asset -> section-a relation no longer has a target.
    graph.relationships.clear();

    for (index, label) in section_labels.iter().enumerate() {
        let bar_start = (index as u32 * 8) + 1;
        graph.sections.push(Section {
            section_id: SectionId::from(format!("section-{index}")),
            label_hint: scene_label_hint(label),
            start_seconds: index as f32 * 16.0,
            end_seconds: (index + 1) as f32 * 16.0,
            bar_start,
            bar_end: bar_start + 7,
            energy_class: scene_energy_for_label(label),
            confidence: 0.9,
            tags: vec![label.clone()],
        });
    }

    graph
}

pub(in crate::jam_app::tests) fn seed_scene_fixture_state(
    state: &mut JamAppState,
    fixture: &SceneRegressionFixture,
) {
    if let Some(current_scene) = fixture.initial_current_scene.as_deref() {
        state.session.runtime_state.transport.current_scene = Some(SceneId::from(current_scene));
    }
    if let Some(active_scene) = fixture.initial_active_scene.as_deref() {
        state.session.runtime_state.scene_state.active_scene = Some(SceneId::from(active_scene));
    }
    if let Some(restore_scene) = fixture.initial_restore_scene.as_deref() {
        state.session.runtime_state.scene_state.restore_scene = Some(SceneId::from(restore_scene));
    }
    if let Some(reinforcement_mode) = fixture.tr909_reinforcement_mode {
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_enabled = false;
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_profile = None;
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode = Some(reinforcement_mode);
    }
    if let Some(pattern_ref) = fixture.tr909_pattern_ref.as_deref() {
        state.session.runtime_state.lane_state.tr909.pattern_ref = Some(pattern_ref.into());
    }
    state.refresh_view();
}

pub(crate) fn sample_session(graph: &SourceGraph) -> SessionFile {
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
    session.source_refs.push(SourceRef {
        source_id: graph.source.source_id.clone(),
        path_hint: graph.source.path.clone(),
        content_hash: graph.source.content_hash.clone(),
        duration_seconds: graph.source.duration_seconds,
        decode_profile: graph.source.decode_profile.clone(),
    });
    session.source_graph_refs.push(SourceGraphRef {
        source_id: SourceId::from("src-1"),
        graph_version: SourceGraphVersion::V1,
        graph_hash: crate::jam_app::persistence::source_graph_hash(graph)
            .expect("hash sample graph"),
        storage_mode: GraphStorageMode::Embedded,
        embedded_graph: Some(graph.clone()),
        external_path: None,
        provenance: graph.provenance.clone(),
    });
    session.runtime_state.transport.is_playing = true;
    // Cursor 31 is the final beat of one-based bar 8; cursor 32 starts bar 9.
    session.runtime_state.transport.position_beats = 31.0;
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-1"));
    session.runtime_state.macro_state.scene_aggression = 0.75;
    session.runtime_state.macro_state.tr909_slam = 0.55;
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Follower);
    session.runtime_state.lane_state.w30.preview_mode = Some(W30PreviewModeState::LiveRecall);
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    session.runtime_state.mixer_state.drum_level = 0.72;
    session.runtime_state.mixer_state.music_level = 0.64;
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
        audio_identity: None,
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
fn derives_scene_candidates_from_source_sections_when_session_is_empty() {
    let mut graph = sample_graph();
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Break,
        start_seconds: 16.0,
        end_seconds: 24.0,
        bar_start: 9,
        bar_end: 12,
        energy_class: EnergyClass::Medium,
        confidence: 0.84,
        tags: vec!["contrast".into()],
    });

    let mut session = sample_session(&graph);
    session.runtime_state.transport.current_scene = None;
    session.runtime_state.scene_state.active_scene = None;
    session.runtime_state.scene_state.scenes.clear();

    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state
            .session
            .runtime_state
            .scene_state
            .scenes
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["scene-01-drop".to_string(), "scene-02-break".to_string()]
    );
    assert_eq!(
        state.session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(
        state.session.runtime_state.transport.current_scene,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(state.jam_view.scene.scene_count, 2);
    assert_eq!(
        state.jam_view.scene.active_scene.as_deref(),
        Some("scene-01-drop")
    );
}
