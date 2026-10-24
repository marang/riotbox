//! Shared generated metadata only; helpers never read paths or run replay privately.
use crate::{
    action::{
        Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
        CaptureLengthIntent, CommitBoundary, Quantization, SourceMonitorMode, UndoPolicy,
    },
    ids::ActionId,
    replay::{
        SnapshotPayloadHydrationError, SnapshotRuntimeActionError as ActionError,
        SnapshotRuntimeField as Field, SnapshotRuntimeReference as Reference,
        SnapshotRuntimeReferenceError as ReferenceError, SnapshotRuntimeValidationError as Error,
        hydrate_replay_target_from_snapshot_payload as hydrate,
    },
    session::{
        ActionCommitRecord, ActionLog, CaptureRef, CaptureType, GraphStorageMode, Mc202RoleState,
        Mc202SourcePhraseNoteBudgetState, Mc202SourcePhrasePlanState, Mc202SourcePhraseSlotState,
        Mc202UndoSnapshotState, ReplayPolicy, RuntimeState, SceneMovementDirectionState,
        SceneMovementKindState, SceneMovementLaneIntentState, SceneMovementState,
        SceneMovementW30IntentState, SessionFile, Snapshot, SnapshotPayload, SourceGraphRef,
        SourceMonitorUndoSnapshotState, SourceRef, SourceTimingGridConfirmationState,
        Tr909FillUndoSnapshotState, W30HookArticulationProfileState, W30HookArticulationState,
    },
    source_graph::{
        DecodeProfile, EnergyClass, GraphProvenance, Section, SectionLabelHint, SourceDescriptor,
        SourceGraph, SourceGraphVersion,
    },
    transport::CommitBoundaryState,
};

pub(super) const ACTION_FIELDS: [Field; 8] = [
    Field::TimingConfirmation,
    Field::CaptureLength,
    Field::LastSceneMovement,
    Field::ActiveSceneProjection,
    Field::RestoreSceneProjection,
    Field::Mc202Undo,
    Field::SourceMonitorUndo,
    Field::Tr909FillUndo,
];

pub(super) fn owner(field: Field, id: u64) -> Action {
    let (command, params) = match field {
        Field::TimingConfirmation => (
            ActionCommand::SourceTimingConfirmGrid,
            ActionParams::SourceTimingGrid {
                source_id: Some("source-a".into()),
                hypothesis_id: None,
                confirmed_bpm: Some(126.0),
            },
        ),
        Field::CaptureLength => (
            ActionCommand::CaptureSetLength,
            ActionParams::CaptureLength {
                intent: Some(CaptureLengthIntent::OneBar),
            },
        ),
        Field::LastSceneMovement | Field::ActiveSceneProjection | Field::RestoreSceneProjection => {
            (
                ActionCommand::SceneLaunch,
                ActionParams::Scene {
                    scene_id: Some("custom-destination".into()),
                },
            )
        }
        Field::Mc202Undo => (ActionCommand::Mc202GenerateFollower, ActionParams::Empty),
        Field::SourceMonitorUndo => (
            ActionCommand::SourceMonitorSetMode,
            ActionParams::SourceMonitor {
                mode: Some(SourceMonitorMode::Blend),
            },
        ),
        Field::Tr909FillUndo => (ActionCommand::Tr909FillNext, ActionParams::Empty),
        _ => unreachable!("only explicit action fields construct owners"),
    };
    Action {
        id: ActionId(id),
        actor: ActorType::User,
        command,
        params,
        target: ActionTarget::default(),
        requested_at: 1,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(id + 100),
        result: Some(ActionResult {
            accepted: true,
            summary: "generated metadata".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: None,
    }
}

pub(super) fn movement(id: ActionId) -> SceneMovementState {
    SceneMovementState {
        action_id: id,
        from_scene: Some("custom-initial".into()),
        to_scene: "custom-destination".into(),
        kind: SceneMovementKindState::Launch,
        direction: SceneMovementDirectionState::Hold,
        tr909_intent: SceneMovementLaneIntentState::Anchor,
        mc202_intent: SceneMovementLaneIntentState::Anchor,
        w30_intent: SceneMovementW30IntentState::Pin,
        intensity: 0.35,
        committed_bar_index: 0,
        committed_phrase_index: 0,
    }
}

pub(super) fn set_action_reference(runtime: &mut RuntimeState, field: Field, id: ActionId) {
    match field {
        Field::TimingConfirmation => {
            runtime.source_timing.confirmed_bpm = Some(126.0);
            runtime.source_timing.confirmed_grid = Some(SourceTimingGridConfirmationState {
                source_id: "source-a".into(),
                hypothesis_id: None,
                confirmed_by_action: id,
                confirmed_at: id.0 + 100,
            });
        }
        Field::CaptureLength => {
            runtime.capture.length_intent = CaptureLengthIntent::OneBar;
            runtime.capture.length_set_by_action = Some(id);
        }
        Field::LastSceneMovement => runtime.scene_state.last_movement = Some(movement(id)),
        Field::ActiveSceneProjection => {
            runtime.scene_state.active_projection_movement = Some(movement(id))
        }
        Field::RestoreSceneProjection => {
            runtime.scene_state.restore_projection_movement = Some(movement(id))
        }
        Field::Mc202Undo => runtime
            .undo_state
            .mc202_snapshots
            .push(Mc202UndoSnapshotState {
                action_id: id,
                role: None,
                phrase_ref: Some("opaque initial phrase".into()),
                phrase_variant: None,
                source_phrase_plan: None,
                touch: 0.17,
            }),
        Field::SourceMonitorUndo => {
            runtime
                .undo_state
                .source_monitor_snapshots
                .push(SourceMonitorUndoSnapshotState {
                    action_id: id,
                    mode: SourceMonitorMode::Source,
                })
        }
        Field::Tr909FillUndo => {
            runtime
                .undo_state
                .tr909_fill_snapshots
                .push(Tr909FillUndoSnapshotState {
                    action_id: id,
                    last_fill_bar: Some(123),
                })
        }
        _ => unreachable!("only explicit action references are installed"),
    }
}

pub(super) fn snapshot(id: &str, cursor: usize, runtime: &RuntimeState) -> Snapshot {
    Snapshot {
        snapshot_id: id.into(),
        created_at: "generated".into(),
        label: "generated".into(),
        action_cursor: cursor,
        payload: Some(SnapshotPayload::from_runtime_state(
            &id.into(),
            cursor,
            runtime,
        )),
    }
}

pub(super) fn session(actions: Vec<Action>, cursor: usize, runtime: RuntimeState) -> SessionFile {
    let records = actions
        .iter()
        .enumerate()
        .filter(|(_, action)| {
            matches!(
                action.status,
                ActionStatus::Committed | ActionStatus::Undone
            )
        })
        .map(|(index, action)| ActionCommitRecord {
            action_id: action.id,
            boundary: CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: index as u64 * 4,
                bar_index: index as u64,
                phrase_index: index as u64 / 4,
                scene_id: None,
            },
            commit_sequence: 1,
            committed_at: action.committed_at.expect("committed fixture"),
            mc202_source_phrase_plan: None,
        })
        .collect();
    let mut session = SessionFile::new("generated", "test", "generated");
    session.source_refs.push(SourceRef {
        source_id: "source-a".into(),
        path_hint: "never-open-source.wav".into(),
        content_hash: "generated".into(),
        duration_seconds: 4.0,
        decode_profile: DecodeProfile::Native,
    });
    session.action_log = ActionLog {
        actions,
        commit_records: records,
        replay_policy: ReplayPolicy::DeterministicCommittedOrderV2,
    };
    session
        .snapshots
        .push(snapshot("selected", cursor, &runtime));
    // Refusal must not replace this current runtime with the selected payload.
    session.runtime_state.transport.position_beats = 987.0;
    session
}

pub(super) fn action_session(field: Field, cursor: usize, reference: ActionId) -> SessionFile {
    let mut runtime = RuntimeState::default();
    set_action_reference(&mut runtime, field, reference);
    session(vec![owner(field, 900)], cursor, runtime)
}

pub(super) fn payload(session: &mut SessionFile) -> &mut RuntimeState {
    &mut session
        .snapshots
        .last_mut()
        .unwrap()
        .payload
        .as_mut()
        .unwrap()
        .runtime_state
}

pub(super) fn action_error(field: Field, id: u64, reason: ActionError) -> Error {
    Error::Action {
        field,
        action_id: ActionId(id),
        reason,
    }
}

pub(super) fn reference_error(field: Field, reference: Reference, reason: ReferenceError) -> Error {
    Error::Reference {
        field,
        reference,
        reason,
    }
}

pub(super) fn assert_refused(
    session: &SessionFile,
    target: usize,
    graph: Option<&SourceGraph>,
    reason: Error,
) {
    let before = session.clone();
    assert_eq!(
        hydrate(session, target, graph).unwrap_err(),
        SnapshotPayloadHydrationError::RuntimeProvenance {
            snapshot_id: "selected".into(),
            reason
        }
    );
    assert_eq!(
        *session, before,
        "refusal must preserve all supplied Session metadata"
    );
}

pub(super) fn phrase(section: Option<&str>) -> Mc202SourcePhrasePlanState {
    Mc202SourcePhrasePlanState {
        source_id: "source-a".into(),
        source_section_id: section.map(Into::into),
        phrase_slot: Mc202SourcePhraseSlotState {
            phrase_index: 1,
            start_bar: 0,
            end_bar: 3,
        },
        source_expression: None,
        role: Mc202RoleState::Follower,
        rhythm_cells: [None; 16],
        note_budget: Mc202SourcePhraseNoteBudgetState::Sparse,
        touch: 0.4,
        confidence: 0.8,
        candidate_family: None,
        candidate_count: 0,
        rejected_candidate_count: 0,
        candidate_provenance_refs: vec!["opaque:do-not-parse".into()],
        candidate_scorecards: vec![],
        phrase_memory_distance: 0.0,
        fallback_reason: None,
    }
}

pub(super) fn phrase_session(nested: bool, section: Option<&str>) -> SessionFile {
    if nested {
        let mut candidate = action_session(Field::Mc202Undo, 1, ActionId(900));
        payload(&mut candidate).undo_state.mc202_snapshots[0].source_phrase_plan =
            Some(phrase(section));
        candidate
    } else {
        let mut runtime = RuntimeState::default();
        runtime.lane_state.mc202.source_phrase_plan = Some(phrase(section));
        session(vec![], 0, runtime)
    }
}

pub(super) fn graph() -> SourceGraph {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: "source-a".into(),
            path: "never-open-source.wav".into(),
            content_hash: "generated".into(),
            duration_seconds: 4.0,
            sample_rate: 48_000,
            channel_count: 1,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "generated".into(),
            provider_set: vec![],
            generated_at: "generated".into(),
            source_hash: "generated".into(),
            analysis_seed: 0,
            run_notes: None,
        },
    );
    graph.sections.push(Section {
        section_id: "section-a".into(),
        label_hint: SectionLabelHint::Unknown,
        start_seconds: 0.0,
        end_seconds: 4.0,
        bar_start: 1,
        bar_end: 2,
        energy_class: EnergyClass::Low,
        confidence: 0.8,
        tags: vec![],
    });
    graph
}

pub(super) fn graph_ref(embedded: Option<SourceGraph>) -> SourceGraphRef {
    let external_path = embedded
        .is_none()
        .then(|| "must-not-open-graph.json".into());
    SourceGraphRef {
        source_id: "source-a".into(),
        graph_version: SourceGraphVersion::V1,
        graph_hash: "generated".into(),
        storage_mode: if embedded.is_some() {
            GraphStorageMode::Embedded
        } else {
            GraphStorageMode::External
        },
        embedded_graph: embedded,
        external_path,
        provenance: graph().provenance,
    }
}

pub(super) fn capture(id: &str) -> CaptureRef {
    CaptureRef {
        audio_identity: None,
        capture_id: id.into(),
        capture_type: CaptureType::Loop,
        source_origin_refs: vec!["opaque:never-resolve-as-path".into()],
        source_window: None,
        lineage_capture_refs: vec![],
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "never-open-capture.wav".into(),
        assigned_target: None,
        is_pinned: false,
        notes: None,
    }
}

pub(super) fn capture_session(hook: bool) -> SessionFile {
    let mut runtime = RuntimeState::default();
    if hook {
        runtime.lane_state.w30.hook_articulation = Some(W30HookArticulationState {
            profile: W30HookArticulationProfileState::PitchDiveV1,
            capture_id: "capture-a".into(),
            started_at_beat: 999_999,
        });
    } else {
        runtime.lane_state.w30.last_capture = Some("capture-a".into());
    }
    session(vec![], 0, runtime)
}
