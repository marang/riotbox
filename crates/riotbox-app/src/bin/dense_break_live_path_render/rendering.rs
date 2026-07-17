use riotbox_audio::runtime::{
    RuntimeMixRenderOutput, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
    SourceMonitorRenderState, render_mc202_offline,
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
};
use riotbox_core::action::SourceMonitorMode;

use crate::model::{
    CALLBACK_FRAME_COUNT, CHANNEL_COUNT, MONITOR_REVIEW_BARS, PreparedLivePath,
    RenderedGestureTransition, RenderedLivePath, SAMPLE_RATE,
};

pub fn render_live_path(
    prepared: &PreparedLivePath,
) -> Result<RenderedLivePath, Box<dyn std::error::Error>> {
    let bpm = prepared.source_timing.bpm;
    let monitor_review_frames = bar_frame_count(bpm).saturating_mul(MONITOR_REVIEW_BARS);
    let monitor_outputs = prepared
        .monitor_proofs
        .iter()
        .map(|proof| render(&proof.plan, monitor_review_frames))
        .collect::<Result<Vec<_>, _>>()?;
    let stage_steps = prepared
        .stages
        .iter()
        .map(|stage| {
            RuntimeMixRenderSequenceStep::new(
                &stage.plan,
                beat_frame_count(bpm).saturating_mul(stage.duration_beats as usize),
            )
        })
        .collect::<Vec<_>>();
    let stage_outputs = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &stage_steps,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        CALLBACK_FRAME_COUNT,
    );
    let transition_outputs = prepared
        .transitions
        .iter()
        .map(|transition| -> Result<_, Box<dyn std::error::Error>> {
            Ok(RenderedGestureTransition {
                before: render_transition_branch(&transition.prefix, &transition.before, bpm)?,
                after: render_transition_branch(&transition.prefix, &transition.after, bpm)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let normal = render_legacy(&prepared.normal_plan, bpm)?;
    let damaged = render_legacy(&prepared.damaged_plan, bpm)?;
    let w30 = render_legacy(&only_w30(&prepared.normal_plan), bpm)?;
    let tr909 = render_legacy(&only_tr909(&prepared.normal_plan), bpm)?;
    let mc202_selected_role = render_legacy(&only_mc202(&prepared.normal_plan), bpm)?;
    let mut direct_mc202_render = prepared.normal_plan.mc202_render;
    direct_mc202_render.is_transport_running = prepared.normal_plan.transport.is_transport_running;
    direct_mc202_render.tempo_bpm = prepared.normal_plan.transport.tempo_bpm;
    direct_mc202_render.position_beats = prepared.normal_plan.transport.position_beats;
    let direct_mc202 = render_mc202_offline(
        &direct_mc202_render,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        normal.samples.len() / usize::from(CHANNEL_COUNT),
    );

    Ok(RenderedLivePath {
        monitor_outputs,
        stage_outputs,
        transition_outputs,
        normal,
        damaged,
        w30,
        tr909,
        mc202_selected_role,
        direct_mc202,
    })
}

fn render_transition_branch(
    prefix: &[(RuntimeMixRenderPlan, u32)],
    target: &RuntimeMixRenderPlan,
    bpm: f32,
) -> Result<RuntimeMixRenderOutput, Box<dyn std::error::Error>> {
    let mut steps = prefix
        .iter()
        .map(|(plan, duration_beats)| {
            RuntimeMixRenderSequenceStep::new(
                plan,
                beat_frame_count(bpm).saturating_mul(*duration_beats as usize),
            )
        })
        .collect::<Vec<_>>();
    steps.push(RuntimeMixRenderSequenceStep::new(
        target,
        bar_frame_count(bpm),
    ));
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &steps,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        CALLBACK_FRAME_COUNT,
    )
    .pop()
    .ok_or_else(|| "transition render produced no exact-mix segment".into())
}

fn render(
    plan: &RuntimeMixRenderPlan,
    frame_count: usize,
) -> Result<RuntimeMixRenderOutput, Box<dyn std::error::Error>> {
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(plan, frame_count)],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        CALLBACK_FRAME_COUNT,
    )
    .pop()
    .ok_or_else(|| "single exact-mix render produced no segment".into())
}

fn render_legacy(
    plan: &RuntimeMixRenderPlan,
    bpm: f32,
) -> Result<RuntimeMixRenderOutput, Box<dyn std::error::Error>> {
    let frame_count = (8.0 * 4.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    render(plan, frame_count)
}

fn beat_frame_count(bpm: f32) -> usize {
    (60.0 / bpm * SAMPLE_RATE as f32).round() as usize
}

fn bar_frame_count(bpm: f32) -> usize {
    beat_frame_count(bpm).saturating_mul(4)
}

fn only_w30(plan: &RuntimeMixRenderPlan) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        tr909_render: Default::default(),
        mc202_render: Default::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
        ..plan.clone()
    }
}

fn only_tr909(plan: &RuntimeMixRenderPlan) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        mc202_render: Default::default(),
        w30_preview_render: Default::default(),
        w30_resample_tap: Default::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
        ..plan.clone()
    }
}

fn only_mc202(plan: &RuntimeMixRenderPlan) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        tr909_render: Default::default(),
        w30_preview_render: Default::default(),
        w30_resample_tap: Default::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
        ..plan.clone()
    }
}
