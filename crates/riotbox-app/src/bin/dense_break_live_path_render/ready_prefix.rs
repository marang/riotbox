//! Existing committed performer preparation shared by legacy packs and generated fixtures.
//! Core/App retain every action, policy and capture identity; this owner starts no runtime.
use std::error::Error;

use riotbox_app::jam_app::{JamAppState, QueueControlResult};
use riotbox_audio::runtime::{AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan};
use riotbox_core::{
    action::{ActionCommand, CaptureLengthIntent, CommitBoundary, SourceMonitorMode},
    ids::SceneId,
    live_performance_policy::{LivePerformancePolicy, derive_live_performance_policy},
    queue::CommittedActionRef,
    style::PerformancePresetId,
    transport::{
        CommitBoundaryState, DEFAULT_BARS_PER_PHRASE, DEFAULT_BEATS_PER_BAR, TransportClockState,
        TransportGridPosition,
    },
};

use crate::model::{CaptureJourneyProof, ConfirmedSourceTiming};

pub(super) struct ReadyPrefix {
    pub(super) capture_journey_proof: CaptureJourneyProof,
    pub(super) preset_id: PerformancePresetId,
    pub(super) preset_action_id: u64,
    pub(super) live_policy: LivePerformancePolicy,
    pub(super) source_plan: Box<RuntimeMixRenderPlan>,
    pub(super) phrase_cursor: u64,
}

/// Commit the existing prefix on a freshly ingested Source-monitor Session.
/// The facade stays borrowed; the source-only snapshot predates preset activation.
/// Logging, family-specific admission, saving and rendering belong to the caller.
pub(super) fn prepare(
    state: &mut JamAppState,
    source_timing: &ConfirmedSourceTiming,
) -> Result<ReadyPrefix, Box<dyn Error>> {
    if source_timing.beats_per_bar != DEFAULT_BEATS_PER_BAR {
        return Err(format!(
            "Feral Break Alpha v2 requires 4/4 source timing, got {} beats per bar",
            source_timing.beats_per_bar
        )
        .into());
    }
    let bpm = source_timing.bpm;
    let capture_cursor = source_timing
        .bar_start_beat_cursor(1)
        .ok_or("dense-break source timing cannot resolve bar 1")?;
    let audition_cursor = source_timing
        .bar_start_beat_cursor(2)
        .ok_or("dense-break source timing cannot resolve audition bar 2")?;
    let promote_cursor = source_timing
        .bar_start_beat_cursor(3)
        .ok_or("dense-break source timing cannot resolve promotion bar 3")?;
    let phrase_cursor = source_timing
        .bar_start_beat_cursor(5)
        .ok_or("dense-break source timing cannot resolve phrase-2 bar 5")?;
    state.set_transport_playing(true);
    let initial_scene = current_scene(state);

    if state.session.runtime_state.source_monitor.mode != SourceMonitorMode::Source {
        return Err("dense-break live path must begin in Source monitor mode".into());
    }
    state.queue_capture_length_intent(CaptureLengthIntent::OneBar, 90)?;
    commit(
        state,
        CommitBoundary::Immediate,
        0,
        initial_scene.clone(),
        95,
        1,
    )?;
    state.queue_capture_bar(100)?;
    let capture_commit = one_commit(commit(
        state,
        CommitBoundary::Bar,
        capture_cursor,
        initial_scene.clone(),
        200,
        1,
    )?)?;
    require_committed_command(state, &capture_commit, ActionCommand::CaptureBarGroup)?;
    if state.queue_w30_raw_capture_audition(210)? != Some(QueueControlResult::Enqueued) {
        return Err("raw W-30 capture audition was unavailable".into());
    }
    let audition_commit = one_commit(commit(
        state,
        CommitBoundary::Bar,
        audition_cursor,
        initial_scene.clone(),
        220,
        1,
    )?)?;
    require_committed_command(
        state,
        &audition_commit,
        ActionCommand::W30AuditionRawCapture,
    )?;
    if !state.queue_promote_last_capture(230)? {
        return Err("capture promotion was unavailable".into());
    }
    let promotion_commit = one_commit(commit(
        state,
        CommitBoundary::Bar,
        promote_cursor,
        initial_scene.clone(),
        300,
        1,
    )?)?;
    require_committed_command(state, &promotion_commit, ActionCommand::PromoteCaptureToPad)?;
    // Source monitor EOF clamps by product contract. Keep the raw reference proof
    // at source start instead of asking a short loop to play from performance bar 5.
    let source_plan = Box::new(render_plan(state, bpm, 0.0));
    let preset_id = PerformancePresetId::FeralBreakAlphaV2;
    if state.queue_performance_preset(preset_id, 305)? != QueueControlResult::Enqueued {
        return Err("Feral Break Alpha preset was unavailable".into());
    }
    let scene = current_scene(state);
    let preset_commit = one_commit(commit(
        state,
        CommitBoundary::Immediate,
        promote_cursor,
        scene,
        306,
        1,
    )?)?;
    require_committed_command(state, &preset_commit, ActionCommand::PresetActivate)?;
    if state.queue_mc202_generate_instigator(311)? != QueueControlResult::Enqueued {
        return Err("MC-202 instigator generation was unavailable".into());
    }
    commit(
        state,
        CommitBoundary::Phrase,
        phrase_cursor,
        initial_scene,
        400,
        1,
    )?;

    let live_policy = state
        .source_graph
        .as_ref()
        .and_then(|graph| derive_live_performance_policy(&state.session, graph))
        .ok_or_else(|| {
            format!(
                "dense-break live performance policy was unavailable: confirmed_grid={:?}, mc202_source_phrase_plan={:?}, latest_action_result={:?}",
                state.session.runtime_state.source_timing.confirmed_grid,
                state
                    .session
                    .runtime_state
                    .lane_state
                    .mc202
                    .source_phrase_plan,
                state
                    .session
                    .action_log
                    .actions
                    .last()
                    .and_then(|action| action.result.as_ref())
            )
        })?;
    Ok(ReadyPrefix {
        capture_journey_proof: CaptureJourneyProof {
            capture_action_id: capture_commit.action_id.0,
            raw_audition_action_id: audition_commit.action_id.0,
            promotion_action_id: promotion_commit.action_id.0,
        },
        preset_id,
        preset_action_id: preset_commit.action_id.0,
        live_policy,
        source_plan,
        phrase_cursor,
    })
}

pub(super) fn render_plan(
    state: &JamAppState,
    bpm: f32,
    position_beats: f64,
) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: bpm,
            position_beats,
        },
        tr909_render: state.runtime.tr909_render.clone(),
        mc202_render: state.runtime.mc202_render,
        w30_preview_render: state.runtime.w30_preview.clone(),
        w30_resample_tap: state.runtime.w30_resample_tap.clone(),
        source_monitor_render: state.source_monitor_render_state(),
    }
}

pub(super) fn current_scene(state: &JamAppState) -> Option<SceneId> {
    state
        .session
        .runtime_state
        .scene_state
        .active_scene
        .clone()
        .or_else(|| state.runtime.transport.current_scene.clone())
}

pub(super) fn one_commit(
    mut committed: Vec<CommittedActionRef>,
) -> Result<CommittedActionRef, Box<dyn Error>> {
    committed
        .pop()
        .ok_or_else(|| "expected one committed action".into())
}

pub(super) fn require_committed_command(
    state: &JamAppState,
    committed: &CommittedActionRef,
    expected: ActionCommand,
) -> Result<(), Box<dyn Error>> {
    let actual = state
        .queue
        .history_action(committed.action_id)
        .map(|action| action.command)
        .ok_or_else(|| {
            format!(
                "committed action {} missing from queue history",
                committed.action_id.0
            )
        })?;
    if actual != expected {
        return Err(format!(
            "committed action {} was {}, expected {}",
            committed.action_id.0,
            actual.as_str(),
            expected.as_str()
        )
        .into());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn commit(
    state: &mut JamAppState,
    kind: CommitBoundary,
    position_beats: u64,
    scene_id: Option<SceneId>,
    timestamp: u64,
    expected_count: usize,
) -> Result<Vec<CommittedActionRef>, Box<dyn Error>> {
    let grid_position = grid_position(state, position_beats);
    state.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: position_beats as f64,
        beat_index: grid_position.beat_cursor,
        bar_index: grid_position.bar_index,
        phrase_index: grid_position.phrase_index,
        current_scene: scene_id.clone(),
    });
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind,
            // Session V1 persists the integral zero-based cursor here.
            beat_index: position_beats,
            bar_index: grid_position.bar_index,
            phrase_index: grid_position.phrase_index,
            scene_id,
        },
        timestamp,
    )?;
    if committed.len() != expected_count {
        return Err(format!(
            "expected {expected_count} {kind:?} commits, got {}",
            committed.len()
        )
        .into());
    }
    Ok(committed)
}

pub(super) fn grid_position(state: &JamAppState, position_beats: u64) -> TransportGridPosition {
    state
        .source_graph
        .as_ref()
        .and_then(|graph| graph.timing.primary_hypothesis())
        .and_then(|hypothesis| hypothesis.transport_grid_position(position_beats as f64))
        .unwrap_or_else(|| {
            TransportGridPosition::from_zero_based_position_beats(
                position_beats as f64,
                DEFAULT_BEATS_PER_BAR,
                DEFAULT_BARS_PER_PHRASE,
            )
        })
}
