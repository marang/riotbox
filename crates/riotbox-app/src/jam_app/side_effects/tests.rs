use riotbox_core::{
    action::{
        Action, ActionCommand, ActionParams, ActionStatus, ActionTarget, ActorType,
        CaptureLengthIntent, CommitBoundary, GhostMode, Quantization, SourceMonitorMode,
        UndoPolicy,
    },
    ids::ActionId,
    session::{SessionFile, SourceTimingGridConfirmationState},
    source_graph::SourceGraph,
    style::PerformancePresetId,
    transport::CommitBoundaryState,
};

use super::{
    apply_capture_side_effects, apply_committed_side_effects, apply_ghost_side_effects,
    apply_mc202_side_effects, apply_preset_side_effects, apply_scene_side_effects,
    apply_source_monitor_side_effects, apply_source_timing_side_effects, apply_tr909_side_effects,
    apply_transport_side_effects, apply_w30_side_effects,
};
use crate::jam_app::tests::{sample_graph, sample_session};

// Frozen pre-refactor fan-out, used only as a differential oracle. Production
// dispatch has one exhaustive match on the canonical ActionCommand enum.
fn prior_broadcast(
    session: &mut SessionFile,
    action: &Action,
    boundary: &CommitBoundaryState,
    graph: Option<&SourceGraph>,
) {
    apply_w30_side_effects(session, action, Some(boundary));
    apply_mc202_side_effects(session, action, Some(boundary), graph);
    apply_tr909_side_effects(session, action, Some(boundary));
    apply_transport_side_effects(session, action);
    apply_capture_side_effects(session, action);
    apply_preset_side_effects(session, action);
    apply_source_monitor_side_effects(session, action);
    apply_source_timing_side_effects(session, action);
    apply_scene_side_effects(session, action, Some(boundary), graph);
    apply_ghost_side_effects(session, action);
}

#[test]
fn exhaustive_dispatch_preserves_prior_broadcast_state_and_results() {
    let graph = sample_graph();
    let params = [
        ActionParams::Empty,
        ActionParams::Preset {
            preset_id: PerformancePresetId::FeralBreakAlphaV1,
        },
        ActionParams::Preset {
            preset_id: PerformancePresetId::FeralBreakAlphaV2,
        },
        ActionParams::Transport {
            position_beats: Some(16),
        },
        ActionParams::Transport {
            position_beats: None,
        },
        ActionParams::Mutation {
            intensity: 0.9,
            target_id: Some("cap-01".into()),
        },
        ActionParams::Promotion {
            capture_id: Some("cap-01".into()),
            destination: Some("w30:resample".into()),
        },
        ActionParams::Scene {
            scene_id: Some("scene-2".into()),
        },
        ActionParams::CaptureLength {
            intent: Some(CaptureLengthIntent::OneBeat),
        },
        ActionParams::CaptureLength { intent: None },
        ActionParams::SourceMonitor {
            mode: Some(SourceMonitorMode::Riotbox),
        },
        ActionParams::Ghost {
            mode: Some(GhostMode::Assist),
            proposal_id: Some("gp-1".into()),
        },
        ActionParams::Ghost {
            mode: None,
            proposal_id: None,
        },
        ActionParams::SourceTimingGrid {
            source_id: Some(graph.source.source_id.clone()),
            hypothesis_id: None,
            confirmed_bpm: Some(126.0),
        },
        ActionParams::SourceTimingGrid {
            source_id: None,
            hypothesis_id: None,
            confirmed_bpm: None,
        },
    ];
    let targets = [
        ActionTarget::default(),
        ActionTarget {
            bank_id: Some("bank-a".into()),
            pad_id: Some("pad-01".into()),
            scene_id: Some("scene-2".into()),
            object_id: Some("follower".into()),
            ..ActionTarget::default()
        },
    ];
    let boundary = CommitBoundaryState {
        kind: CommitBoundary::Phrase,
        beat_index: 32,
        bar_index: 9,
        phrase_index: 1,
        scene_id: Some("scene-1".into()),
    };
    for &command in ActionCommand::ALL {
        for params in &params {
            for target in &targets {
                for graph_context in [None, Some(&graph)] {
                    let action = Action {
                        id: ActionId(500),
                        actor: ActorType::User,
                        command,
                        params: params.clone(),
                        target: target.clone(),
                        requested_at: 100,
                        quantization: Quantization::NextPhrase,
                        status: ActionStatus::Committed,
                        committed_at: Some(200),
                        result: None,
                        undo_policy: UndoPolicy::Undoable,
                        explanation: None,
                    };
                    let mut previous = sample_session(&graph);
                    previous.runtime_state.source_timing.confirmed_grid =
                        Some(SourceTimingGridConfirmationState {
                            source_id: graph.source.source_id.clone(),
                            hypothesis_id: None,
                            confirmed_by_action: ActionId(1),
                            confirmed_at: 10,
                        });
                    previous.runtime_state.source_timing.confirmed_bpm = Some(126.0);
                    previous.action_log.actions.push(action.clone());
                    let mut dispatched = previous.clone();
                    prior_broadcast(&mut previous, &action, &boundary, graph_context);
                    apply_committed_side_effects(
                        &mut dispatched,
                        &action,
                        &boundary,
                        graph_context,
                    );
                    assert_eq!(dispatched, previous, "{command:?}, {params:?}, {target:?}");
                }
            }
        }
    }
}
