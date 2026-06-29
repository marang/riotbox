use crate::{
    mc202::Mc202RenderState,
    tr909::Tr909RenderState,
    w30::{W30PreviewRenderState, W30ResampleTapState},
};

use super::{
    public_api_shell::AudioRuntimeTimingSnapshot,
    shared_mc202_w30_preview::{SharedMc202RenderState, SharedW30PreviewRenderState},
    shared_transport_tr909::{RealtimeTransportTimingState, SharedTr909RenderState},
    shared_w30_resample_callback::{
        SharedW30ResampleTapState, Tr909CallbackState, TransportTimingCallbackState,
        W30MixRenderState, W30PreviewCallbackState, W30ResampleTapCallbackState,
        advance_transport_timing, render_mix_buffer,
    },
    source_monitor::{
        SharedSourceMonitorRenderState, SourceMonitorRenderState, apply_source_monitor_policy,
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

#[must_use]
pub fn render_runtime_mix_offline(
    plan: &RuntimeMixRenderPlan,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
) -> Vec<f32> {
    render_runtime_mix_in_callback_blocks(
        plan,
        sample_rate,
        channel_count,
        frame_count,
        frame_count.max(1),
    )
}

#[must_use]
pub fn render_runtime_mix_realtime_simulation_offline(
    plan: &RuntimeMixRenderPlan,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
    callback_frame_count: usize,
) -> Vec<f32> {
    render_runtime_mix_in_callback_blocks(
        plan,
        sample_rate,
        channel_count,
        frame_count,
        callback_frame_count.max(1),
    )
}

fn render_runtime_mix_in_callback_blocks(
    plan: &RuntimeMixRenderPlan,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
    callback_frame_count: usize,
) -> Vec<f32> {
    let channel_count = usize::from(channel_count.max(1));
    let mut output = vec![0.0; frame_count.saturating_mul(channel_count)];
    let transport = RealtimeTransportTimingState {
        is_transport_running: plan.transport.is_transport_running,
        tempo_bpm: plan.transport.tempo_bpm,
        position_beats: plan.transport.position_beats,
    };
    let shared_tr909 = SharedTr909RenderState::new(&plan.tr909_render);
    let shared_mc202 = SharedMc202RenderState::new(&plan.mc202_render);
    let shared_w30_preview = SharedW30PreviewRenderState::new(&plan.w30_preview_render);
    let shared_w30_resample = SharedW30ResampleTapState::new(&plan.w30_resample_tap);
    let shared_source_monitor = SharedSourceMonitorRenderState::new(&plan.source_monitor_render);
    let mut transport_state = TransportTimingCallbackState::default();
    let mut tr909_state = Tr909CallbackState::default();
    let mut w30_preview_state = W30PreviewCallbackState::default();
    let mut w30_resample_state = W30ResampleTapCallbackState::default();

    for block in output.chunks_mut(callback_frame_count.saturating_mul(channel_count)) {
        let block_frame_count = block.len() / channel_count;
        let timing = advance_transport_timing(
            &transport,
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
        apply_source_monitor_policy(block, sample_rate, channel_count, &source_monitor_render);
    }

    output
}
