use crate::{
    mc202::Mc202RenderState,
    tr909::Tr909RenderState,
    w30::{W30PreviewRenderState, W30ResampleTapState},
};

use super::{
    FillFocusRenderState,
    public_api_shell::{
        AudioRuntimeTimingSnapshot, MasterBusLimiterReport, apply_master_bus_soft_limiter,
        master_bus_limiter_ceiling, master_bus_limiter_threshold, signal_metrics,
    },
    shared_mc202_w30_preview::{SharedMc202RenderState, SharedW30PreviewRenderState},
    shared_transport_tr909::{SharedTr909RenderState, SharedTransportTimingState},
    shared_w30_resample_callback::{
        SharedW30ResampleTapState, Tr909CallbackState, TransportTimingCallbackState,
        W30MixRenderState, W30PreviewCallbackState, W30ResampleTapCallbackState,
        advance_transport_timing, render_mix_buffer,
    },
    source_monitor::{
        SharedSourceMonitorRenderState, SourceMonitorCallbackState, SourceMonitorRenderState,
        apply_source_monitor_policy_with_state_and_fill_focus,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMixRenderPlan {
    pub transport: AudioRuntimeTimingSnapshot,
    pub tr909_render: Tr909RenderState,
    pub mc202_render: Mc202RenderState,
    pub w30_preview_render: W30PreviewRenderState,
    pub w30_resample_tap: W30ResampleTapState,
    pub source_monitor_render: SourceMonitorRenderState,
}

impl Default for RuntimeMixRenderPlan {
    fn default() -> Self {
        Self {
            transport: AudioRuntimeTimingSnapshot::default(),
            tr909_render: Tr909RenderState::default(),
            mc202_render: Mc202RenderState::default(),
            w30_preview_render: W30PreviewRenderState::default(),
            w30_resample_tap: W30ResampleTapState::default(),
            source_monitor_render: SourceMonitorRenderState::control_only(
                riotbox_core::action::SourceMonitorMode::Riotbox,
            ),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RuntimeMixRenderSequenceStep<'a> {
    pub plan: &'a RuntimeMixRenderPlan,
    pub frame_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMixRenderOutput {
    pub samples: Vec<f32>,
    pub limiter: MasterBusLimiterReport,
}

impl<'a> RuntimeMixRenderSequenceStep<'a> {
    #[must_use]
    pub const fn new(plan: &'a RuntimeMixRenderPlan, frame_count: usize) -> Self {
        Self { plan, frame_count }
    }
}

#[must_use]
pub fn render_runtime_mix_offline(
    plan: &RuntimeMixRenderPlan,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
) -> Vec<f32> {
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(plan, frame_count)],
        sample_rate,
        channel_count,
        frame_count.max(1),
    )
    .pop()
    .map(|output| output.samples)
    .unwrap_or_default()
}

#[must_use]
pub fn render_runtime_mix_realtime_simulation_offline(
    plan: &RuntimeMixRenderPlan,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
    callback_frame_count: usize,
) -> Vec<f32> {
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(plan, frame_count)],
        sample_rate,
        channel_count,
        callback_frame_count.max(1),
    )
    .pop()
    .map(|output| output.samples)
    .unwrap_or_default()
}

/// Renders plan steps through the exact product mix seam while retaining callback state.
///
/// Each returned buffer corresponds to the step at the same index. Plan updates occur only
/// between simulated callbacks. The first plan owns the source-audio allocation for the whole
/// sequence, matching the live runtime; later plans may update its monitor mode and anchor.
#[must_use]
pub fn render_runtime_mix_plan_sequence_realtime_simulation_offline(
    steps: &[RuntimeMixRenderSequenceStep<'_>],
    sample_rate: u32,
    channel_count: u16,
    callback_frame_count: usize,
) -> Vec<Vec<f32>> {
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        steps,
        sample_rate,
        channel_count,
        callback_frame_count,
    )
    .into_iter()
    .map(|output| output.samples)
    .collect()
}

/// Renders the exact product mix seam and preserves honest pre/post-limiter evidence.
///
/// This is an offline diagnostic API. It retains callback state exactly like
/// [`render_runtime_mix_plan_sequence_realtime_simulation_offline`] while recording the
/// signal immediately before the master limiter, the audible post-limiter signal, and how
/// many samples required limiting. The live callback remains allocation-free.
#[must_use]
pub fn render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
    steps: &[RuntimeMixRenderSequenceStep<'_>],
    sample_rate: u32,
    channel_count: u16,
    callback_frame_count: usize,
) -> Vec<RuntimeMixRenderOutput> {
    let Some(first_step) = steps.first() else {
        return Vec::new();
    };
    let channel_count = usize::from(channel_count.max(1));
    let callback_sample_count = callback_frame_count.max(1).saturating_mul(channel_count);
    let first_plan = first_step.plan;
    let shared_transport = SharedTransportTimingState::new(
        first_plan.transport.is_transport_running,
        first_plan.transport.tempo_bpm,
        first_plan.transport.position_beats,
    );
    let shared_tr909 = SharedTr909RenderState::new(&first_plan.tr909_render);
    let shared_mc202 = SharedMc202RenderState::new(&first_plan.mc202_render);
    let shared_w30_preview = SharedW30PreviewRenderState::new(&first_plan.w30_preview_render);
    let shared_w30_resample = SharedW30ResampleTapState::new(&first_plan.w30_resample_tap);
    let shared_source_monitor =
        SharedSourceMonitorRenderState::new(&first_plan.source_monitor_render);
    let mut transport_state = TransportTimingCallbackState::default();
    let mut tr909_state = Tr909CallbackState::default();
    let mut w30_preview_state = W30PreviewCallbackState::with_sample_rate(sample_rate);
    let mut w30_resample_state = W30ResampleTapCallbackState::default();
    let mut source_monitor_callback_state = SourceMonitorCallbackState::default();

    steps
        .iter()
        .map(|step| {
            let plan = step.plan;
            shared_transport.update(
                plan.transport.is_transport_running,
                plan.transport.tempo_bpm,
                plan.transport.position_beats,
            );
            shared_tr909.update(&plan.tr909_render);
            shared_mc202.update(&plan.mc202_render);
            shared_w30_preview.update(&plan.w30_preview_render);
            shared_w30_resample.update(&plan.w30_resample_tap);
            shared_source_monitor.update(&plan.source_monitor_render);

            let mut output = vec![0.0; step.frame_count.saturating_mul(channel_count)];
            let mut pre_limiter = vec![0.0; output.len()];
            let mut limited_sample_count = 0;
            for (block, pre_limiter_block) in output
                .chunks_mut(callback_sample_count)
                .zip(pre_limiter.chunks_mut(callback_sample_count))
            {
                let block_frame_count = block.len() / channel_count;
                let timing = advance_transport_timing(
                    &shared_transport.snapshot(),
                    &mut transport_state,
                    sample_rate,
                    block_frame_count,
                );
                let mut tr909_render = shared_tr909.snapshot();
                tr909_render.is_transport_running = timing.is_transport_running;
                tr909_render.tempo_bpm = timing.tempo_bpm;
                tr909_render.position_beats = timing.render_position_beats;
                let mut mc202_render = shared_mc202.snapshot();
                mc202_render.is_transport_running = timing.is_transport_running;
                mc202_render.tempo_bpm = timing.tempo_bpm;
                mc202_render.position_beats = timing.render_position_beats;
                let mut w30_preview_render = shared_w30_preview.snapshot();
                w30_preview_render.is_transport_running = timing.is_transport_running;
                w30_preview_render.tempo_bpm = timing.tempo_bpm;
                w30_preview_render.position_beats = timing.render_position_beats;
                let mut w30_resample_render = shared_w30_resample.snapshot();
                w30_resample_render.is_transport_running = timing.is_transport_running;
                w30_resample_render.tempo_bpm = timing.tempo_bpm;
                w30_resample_render.position_beats = timing.render_position_beats;
                let mut source_monitor_render = shared_source_monitor
                    .render_snapshot_from_control(shared_source_monitor.control_snapshot());
                source_monitor_render.is_transport_running = timing.is_transport_running;
                source_monitor_render.tempo_bpm = timing.tempo_bpm;
                source_monitor_render.position_beats = timing.render_position_beats;

                render_mix_buffer(
                    block,
                    sample_rate,
                    channel_count,
                    &tr909_render,
                    &mc202_render,
                    &mut tr909_state,
                    &mut W30MixRenderState {
                        preview_render: &w30_preview_render,
                        preview_state: &mut w30_preview_state,
                        resample_render: &w30_resample_render,
                        resample_state: &mut w30_resample_state,
                    },
                );
                apply_source_monitor_policy_with_state_and_fill_focus(
                    block,
                    sample_rate,
                    channel_count,
                    &source_monitor_render,
                    FillFocusRenderState::from_tr909(&tr909_render),
                    &mut source_monitor_callback_state,
                );
                pre_limiter_block.copy_from_slice(block);
                limited_sample_count += apply_master_bus_soft_limiter(block);
            }
            RuntimeMixRenderOutput {
                limiter: MasterBusLimiterReport {
                    applied: limited_sample_count > 0,
                    threshold: master_bus_limiter_threshold(),
                    ceiling: master_bus_limiter_ceiling(),
                    limited_sample_count,
                    pre: signal_metrics(&pre_limiter),
                    post: signal_metrics(&output),
                },
                samples: output,
            }
        })
        .collect()
}
