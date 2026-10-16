//! Generated metadata-only replay regression; never opens source or capture audio.
use std::{fs, path::Path};

use riotbox_app::jam_app::{JamAppState, JamFileSet, QueueControlResult};
use riotbox_core::{
    action::{
        Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
        CommitBoundary, Quantization, UndoPolicy,
    },
    ids::ActionId,
    persistence::load_session_json,
    queue::ActionQueue,
    replay::{
        apply_replay_plan_to_session, apply_replay_target_suffix_to_session,
        build_committed_replay_plan,
    },
    session::{
        ActionCommitRecord, Mc202RoleState, Mc202SourcePhraseNoteBudgetState,
        Mc202SourcePhrasePlanState, Mc202SourcePhraseSlotState, ReplayPolicy, SessionFile,
    },
    source_graph::{
        DecodeProfile, GraphProvenance, ManualSourceTimingGrid, SourceDescriptor, SourceGraph,
        install_manual_source_timing_grid,
    },
    transport::CommitBoundaryState,
};

#[path = "mc202_plan_invalidation/construction.rs"]
mod construction;
#[path = "mc202_plan_invalidation/output.rs"]
mod output;
#[path = "mc202_plan_invalidation/payload.rs"]
mod payload;
#[path = "mc202_plan_invalidation/targets.rs"]
mod targets;
#[path = "mc202_plan_invalidation/timing.rs"]
mod timing;
#[path = "mc202_plan_invalidation/touch.rs"]
mod touch;

fn generated_plan() -> Mc202SourcePhrasePlanState {
    Mc202SourcePhrasePlanState {
        source_id: "generated-metadata-source".into(),
        source_section_id: None,
        phrase_slot: Mc202SourcePhraseSlotState {
            phrase_index: 1,
            start_bar: 0,
            end_bar: 3,
        },
        source_expression: None,
        role: Mc202RoleState::Follower,
        rhythm_cells: [Some(0); 16],
        note_budget: Mc202SourcePhraseNoteBudgetState::Sparse,
        touch: 0.5,
        confidence: 0.8,
        candidate_family: None,
        candidate_count: 0,
        rejected_candidate_count: 0,
        candidate_provenance_refs: vec!["generated metadata, not source evidence".into()],
        candidate_scorecards: vec![],
        phrase_memory_distance: 0.5,
        fallback_reason: None,
    }
}

fn confirmed_app_with_persisted_plan(nonexistent_source: &Path) -> JamAppState {
    assert!(!nonexistent_source.exists());
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: "generated-metadata-source".into(),
            path: nonexistent_source.to_str().unwrap().into(),
            content_hash: "generated-only".into(),
            duration_seconds: 16.0,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "generated".into(),
            provider_set: vec![],
            generated_at: "2026-10-16".into(),
            source_hash: "generated-only".into(),
            analysis_seed: 0,
            run_notes: None,
        },
    );
    install_manual_source_timing_grid(
        &mut graph,
        ManualSourceTimingGrid {
            bpm: 120.0,
            downbeat_seconds: 0.0,
        },
    )
    .unwrap();
    let mut session = SessionFile::new("generated-plan-invalidation", "test", "2026-10-16");
    for (id, command, params, plan) in [
        (
            1,
            ActionCommand::SourceTimingConfirmGrid,
            ActionParams::SourceTimingGrid {
                source_id: Some(graph.source.source_id.clone()),
                hypothesis_id: graph.timing.primary_hypothesis_id.clone(),
                confirmed_bpm: Some(120.0),
            },
            None,
        ),
        (
            2,
            ActionCommand::Mc202GenerateFollower,
            ActionParams::Empty,
            Some(generated_plan()),
        ),
    ] {
        session.action_log.actions.push(Action {
            id: ActionId(id),
            actor: ActorType::User,
            command,
            params,
            target: ActionTarget::default(),
            requested_at: id * 10,
            quantization: Quantization::Immediate,
            status: ActionStatus::Committed,
            committed_at: Some(id * 10),
            result: Some(ActionResult {
                accepted: true,
                summary: "generated metadata".into(),
            }),
            undo_policy: UndoPolicy::NotUndoable {
                reason: "generated metadata fixture".into(),
            },
            explanation: None,
        });
        session.action_log.commit_records.push(ActionCommitRecord {
            action_id: ActionId(id),
            boundary: CommitBoundaryState {
                kind: CommitBoundary::Immediate,
                beat_index: 0,
                bar_index: 1,
                phrase_index: 1,
                scene_id: None,
            },
            commit_sequence: id as u32,
            committed_at: id * 10,
            mc202_source_phrase_plan: plan,
        });
    }
    let log = session.action_log.clone();
    let plan = build_committed_replay_plan(&log).unwrap();
    apply_replay_plan_to_session(&mut session, &plan).unwrap();
    let app = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    assert_eq!(
        app.session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan,
        Some(generated_plan())
    );
    assert_no_hydration(&app);
    app
}

fn commit(app: &mut JamAppState, at: u64) {
    let boundary = app
        .runtime
        .transport
        .boundary_state(CommitBoundary::Immediate);
    assert_eq!(app.commit_ready_actions(boundary, at).unwrap().len(), 1);
    assert!(
        app.session
            .action_log
            .actions
            .last()
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .accepted
    );
}

fn invalidated_history(path: &Path, policy: ReplayPolicy) -> JamAppState {
    let mut app = confirmed_app_with_persisted_plan(path);
    app.session.action_log.replay_policy = policy;
    app.queue_source_timing_grid_revert(30).unwrap();
    commit(&mut app, 31);
    app.queue_source_timing_grid_confirmation(40).unwrap();
    commit(&mut app, 41);
    app
}

fn assert_no_hydration(app: &JamAppState) {
    assert!(app.session.source_refs.is_empty());
    assert!(app.session.source_graph_refs.is_empty());
    assert!(app.session.captures.is_empty());
    assert!(app.session.export_receipts.is_empty());
    assert!(app.source_audio_cache.is_none());
    assert!(app.capture_audio_cache.is_empty());
}

#[test]
fn matching_revert_then_reconfirm_does_not_resurrect_invalidated_plan_on_app_target_restore() {
    let owner = tempfile::tempdir().unwrap();
    let nonexistent_source = owner.path().join("never-created-source.wav");
    let mut app = confirmed_app_with_persisted_plan(&nonexistent_source);
    let confirmed_grid = app
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .clone();
    assert!(confirmed_grid.is_some());
    assert_eq!(
        app.queue_source_timing_grid_revert(30).unwrap(),
        QueueControlResult::Enqueued
    );
    commit(&mut app, 31);
    assert!(
        app.session
            .runtime_state
            .source_timing
            .confirmed_grid
            .is_none()
    );
    assert!(
        app.session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
    assert_eq!(
        app.queue_source_timing_grid_confirmation(40).unwrap(),
        QueueControlResult::Enqueued
    );
    commit(&mut app, 41);
    let reconfirmed = app
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .unwrap();
    let original = confirmed_grid.as_ref().unwrap();
    assert_eq!(reconfirmed.source_id, original.source_id);
    assert_eq!(reconfirmed.hypothesis_id, original.hypothesis_id);
    assert_ne!(
        reconfirmed.confirmed_by_action,
        original.confirmed_by_action
    );
    assert_eq!(
        app.session.runtime_state.source_timing.confirmed_bpm,
        Some(120.0)
    );
    assert!(
        app.session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
    assert_no_hydration(&app);

    // The public App save has no graph refs or graph output path to publish.
    // Only generated Session metadata is written; the in-memory graph and its
    // nonexistent source descriptor are not resolved or serialized.
    let path = owner.path().join("session.json");
    app.files = Some(JamFileSet {
        session_path: path.clone(),
        source_graph_path: None,
    });
    app.save().unwrap();
    let saved = fs::read(&path).unwrap();
    let cursor = app.session.action_log.actions.len();
    let mut core_restored = load_session_json(&path).unwrap();
    apply_replay_target_suffix_to_session(&mut core_restored, cursor, None).unwrap();
    assert!(
        core_restored
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );

    let mut loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert!(loaded.source_graph.is_none());
    assert_no_hydration(&loaded);
    assert!(
        loaded
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
    loaded.apply_restore_target_suffix(cursor).unwrap();
    assert_no_hydration(&loaded);
    assert_eq!(fs::read(&path).unwrap(), saved);
    assert!(!nonexistent_source.exists());
    assert_eq!(fs::read_dir(owner.path()).unwrap().count(), 1);
    assert!(
        loaded
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none(),
        "App target restore must preserve Core's Revert invalidation even after matching Reconfirm"
    );
}
