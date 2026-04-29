use super::*;
use crate::{
    action::{ActionTarget, TargetScope},
    ids::{SceneId, SectionId, SourceId},
    replay::{apply_graph_aware_replay_plan_to_session, build_replay_target_plan},
    session::{
        SceneMovementDirectionState, SceneMovementKindState, SceneMovementLaneIntentState,
        SessionFile,
    },
    source_graph::{
        AnalysisSummary, DecodeProfile, EnergyClass, GraphProvenance, QualityClass, Section,
        SectionLabelHint, SourceDescriptor, SourceGraph,
    },
};

#[test]
fn snapshot_suffix_replay_converges_with_origin_for_scene_launch_restore() {
    let action_log = action_log(vec![
        scene_action(1, ActionCommand::SceneLaunch, "scene-b", 100),
        scene_action(2, ActionCommand::SceneRestore, "scene-a", 200),
    ]);
    let snapshots = vec![snapshot("snap-after-launch", 1)];
    let origin_plan = build_replay_target_plan(&action_log, &[], 2).expect("origin plan");
    let anchor_plan = build_replay_target_plan(&action_log, &[], 1).expect("anchor plan");
    let snapshot_plan =
        build_replay_target_plan(&action_log, &snapshots, 2).expect("snapshot plan");

    let mut origin_session = scene_session("origin-session", "scene-a");
    let mut snapshot_session = scene_session("snapshot-session", "scene-a");

    apply_replay_plan_to_session(&mut origin_session, &origin_plan.suffix)
        .expect("origin replay succeeds");
    apply_replay_plan_to_session(&mut snapshot_session, &anchor_plan.suffix)
        .expect("anchor replay succeeds");
    apply_replay_plan_to_session(&mut snapshot_session, &snapshot_plan.suffix)
        .expect("snapshot suffix replay succeeds");

    assert_eq!(
        snapshot_plan
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("snap-after-launch")
    );
    assert_eq!(
        snapshot_session.runtime_state.transport.current_scene,
        origin_session.runtime_state.transport.current_scene
    );
    assert_eq!(
        snapshot_session.runtime_state.scene_state.active_scene,
        origin_session.runtime_state.scene_state.active_scene
    );
    assert_eq!(
        snapshot_session.runtime_state.scene_state.restore_scene,
        origin_session.runtime_state.scene_state.restore_scene
    );
    assert_eq!(
        origin_session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-a"))
    );
    assert_eq!(
        origin_session.runtime_state.scene_state.restore_scene,
        Some(SceneId::from("scene-b"))
    );
    assert_eq!(origin_session.runtime_state.scene_state.last_movement, None);
}

#[test]
fn scene_replay_rejects_missing_target_without_mutating_session() {
    let action_log = action_log(vec![targeted_action(
        1,
        ActionCommand::SceneLaunch,
        ActionParams::Empty,
        ActionTarget {
            scope: Some(TargetScope::Scene),
            ..Default::default()
        },
        100,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = scene_session("session-1", "scene-a");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("scene replay requires explicit scene id");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::SceneLaunch,
            expected: "ActionTarget { scene_id: Some(_) } or ActionParams::Scene { scene_id: Some(_) }",
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn graph_aware_scene_replay_hydrates_last_movement_from_source_graph() {
    let action_log = action_log(vec![scene_action(
        1,
        ActionCommand::SceneLaunch,
        "scene-02-drop",
        100,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let graph = scene_energy_graph();
    let mut session = projected_scene_session("session-1", "scene-01-intro");

    let report = apply_graph_aware_replay_plan_to_session(&mut session, &plan.suffix, &graph)
        .expect("graph-aware scene replay succeeds");

    let movement = session
        .runtime_state
        .scene_state
        .last_movement
        .as_ref()
        .expect("graph-aware replay hydrates movement");
    assert_eq!(report.applied_action_ids, vec![ActionId(1)]);
    assert_eq!(
        session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-02-drop"))
    );
    assert_eq!(movement.action_id, ActionId(1));
    assert_eq!(movement.from_scene, Some(SceneId::from("scene-01-intro")));
    assert_eq!(movement.to_scene, SceneId::from("scene-02-drop"));
    assert_eq!(movement.kind, SceneMovementKindState::Launch);
    assert_eq!(movement.direction, SceneMovementDirectionState::Rise);
    assert_eq!(movement.tr909_intent, SceneMovementLaneIntentState::Drive);
    assert_eq!(movement.mc202_intent, SceneMovementLaneIntentState::Lift);
    assert_eq!(movement.intensity, 0.75);
    assert_eq!(movement.committed_bar_index, 1);
    assert_eq!(movement.committed_phrase_index, 0);
}

fn scene_session(session_id: &str, active_scene: &str) -> SessionFile {
    let mut session = SessionFile::new(session_id, "riotbox-test", "2026-04-29T21:45:00Z");
    let active_scene = SceneId::from(active_scene);
    session.runtime_state.scene_state.scenes =
        vec![SceneId::from("scene-a"), SceneId::from("scene-b")];
    session.runtime_state.scene_state.active_scene = Some(active_scene.clone());
    session.runtime_state.transport.current_scene = Some(active_scene);
    session
}

fn projected_scene_session(session_id: &str, active_scene: &str) -> SessionFile {
    let mut session = SessionFile::new(session_id, "riotbox-test", "2026-04-29T21:45:00Z");
    let active_scene = SceneId::from(active_scene);
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.scene_state.active_scene = Some(active_scene.clone());
    session.runtime_state.transport.current_scene = Some(active_scene);
    session
}

fn scene_action(
    id: u64,
    command: ActionCommand,
    scene_id: &str,
    committed_at: TimestampMs,
) -> Action {
    targeted_action(
        id,
        command,
        ActionParams::Scene {
            scene_id: Some(SceneId::from(scene_id)),
        },
        ActionTarget {
            scope: Some(TargetScope::Scene),
            scene_id: Some(SceneId::from(scene_id)),
            ..Default::default()
        },
        committed_at,
    )
}

fn scene_energy_graph() -> SourceGraph {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-1"),
            path: "source.wav".into(),
            content_hash: "hash-1".into(),
            duration_seconds: 32.0,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::NormalizedStereo,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["test".into()],
            generated_at: "2026-04-29T21:45:00Z".into(),
            source_hash: "hash-1".into(),
            analysis_seed: 1,
            run_notes: None,
        },
    );
    graph.sections = vec![
        Section {
            section_id: SectionId::from("section-01"),
            label_hint: SectionLabelHint::Intro,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: EnergyClass::Low,
            confidence: 0.9,
            tags: vec!["intro".into()],
        },
        Section {
            section_id: SectionId::from("section-02"),
            label_hint: SectionLabelHint::Drop,
            start_seconds: 16.0,
            end_seconds: 32.0,
            bar_start: 9,
            bar_end: 16,
            energy_class: EnergyClass::Peak,
            confidence: 0.9,
            tags: vec!["drop".into()],
        },
    ];
    graph.analysis_summary = AnalysisSummary {
        overall_confidence: 0.9,
        timing_quality: QualityClass::High,
        section_quality: QualityClass::High,
        loop_candidate_count: 0,
        hook_candidate_count: 0,
        break_rebuild_potential: QualityClass::Medium,
        warnings: Vec::new(),
    };
    graph
}
