use crate::{
    mc202::Mc202RenderState,
    tr909::Tr909RenderState,
    w30::{W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30PreviewRenderState, W30ResampleTapState},
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

/// Heap-owned side information for the bounded RIOTBOX-1469 Development seam.
///
/// It is deliberately separate from `W30PreviewRenderState`: normal product
/// snapshots keep their established mono size and never select this path.
#[derive(Clone, Debug, PartialEq)]
pub struct W30StereoPadDevelopmentWindow {
    pub sample_count: usize,
    side_samples: Box<[f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN]>,
}

/// Project a stereo source into the exact existing W-30 sample indices while
/// retaining only the side component needed to reconstruct L/R around the
/// unchanged mono control window. Decode and projection happen off callback.
#[must_use]
pub fn project_w30_stereo_pad_development_window(
    interleaved_samples: &[f32],
    channel_count: usize,
) -> Option<W30StereoPadDevelopmentWindow> {
    if channel_count != 2
        || interleaved_samples.is_empty()
        || !interleaved_samples.len().is_multiple_of(channel_count)
        || interleaved_samples.iter().any(|sample| !sample.is_finite())
    {
        return None;
    }
    let frame_count = interleaved_samples.len() / channel_count;
    let sample_count = frame_count.min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
    let mut side_samples: Box<[f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN]> =
        vec![0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN]
            .into_boxed_slice()
            .try_into()
            .ok()?;
    for (index, side) in side_samples.iter_mut().take(sample_count).enumerate() {
        let frame_index = if sample_count <= 1 {
            0
        } else {
            index * (frame_count - 1) / (sample_count - 1)
        };
        let base = frame_index * channel_count;
        *side = (interleaved_samples[base] - interleaved_samples[base + 1]) / 2.0;
    }
    Some(W30StereoPadDevelopmentWindow {
        sample_count,
        side_samples,
    })
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
/// between simulated callbacks. Every step atomically publishes its complete source-monitor
/// snapshot, so a source replacement cannot combine stale PCM with new controls or anchors.
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
    render_runtime_mix_plan_sequence_with_w30_stereo_development(
        steps,
        sample_rate,
        channel_count,
        callback_frame_count,
        None,
    )
}

/// Render one exact RuntimeMix plan through the RIOTBOX-1469 stereo candidate.
///
/// This refuses mismatched windows and articulations rather than silently
/// changing the candidate. It is an offline Development seam, not product
/// behavior or a fallback path.
#[must_use]
pub fn render_runtime_mix_w30_stereo_development_offline_with_report(
    plan: &RuntimeMixRenderPlan,
    stereo: &W30StereoPadDevelopmentWindow,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
    callback_frame_count: usize,
) -> Option<RuntimeMixRenderOutput> {
    let pad = plan.w30_preview_render.pad_playback.as_ref()?;
    if channel_count != 2
        || pad.sample_count.min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN) != stereo.sample_count
        || pad.hook_articulation.is_some()
    {
        return None;
    }
    render_runtime_mix_plan_sequence_with_w30_stereo_development(
        &[RuntimeMixRenderSequenceStep::new(plan, frame_count)],
        sample_rate,
        channel_count,
        callback_frame_count,
        Some(stereo),
    )
    .pop()
}

fn render_runtime_mix_plan_sequence_with_w30_stereo_development(
    steps: &[RuntimeMixRenderSequenceStep<'_>],
    sample_rate: u32,
    channel_count: u16,
    callback_frame_count: usize,
    stereo: Option<&W30StereoPadDevelopmentWindow>,
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
    let mut w30_preview_state =
        W30PreviewCallbackState::with_sample_rate_and_channels(sample_rate, channel_count);
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
            shared_source_monitor.replace_source_and_controls(&plan.source_monitor_render);

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
                let source_monitor_snapshot = shared_source_monitor.snapshot();
                let mut source_monitor_render = source_monitor_snapshot.render_state();
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
                        stereo_side_samples: stereo.map(|window| window.side_samples.as_ref()),
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

#[cfg(test)]
mod stereo_pad_development_tests {
    use super::*;
    use crate::w30::{
        W30_PAD_CHOP_SLICE_COUNT, W30PadPlaybackSampleWindow, W30PreviewRenderMode,
        W30PreviewRenderRouting, W30PreviewSourceProfile,
    };

    fn public_test_plan(interleaved: &[f32]) -> RuntimeMixRenderPlan {
        let frame_count = interleaved.len() / 2;
        let mut mono = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
        for index in 0..frame_count {
            mono[index] = (interleaved[index * 2] + interleaved[index * 2 + 1]) / 2.0;
        }
        RuntimeMixRenderPlan {
            transport: AudioRuntimeTimingSnapshot {
                is_transport_running: true,
                tempo_bpm: 120.0,
                position_beats: 0.0,
            },
            w30_preview_render: W30PreviewRenderState {
                mode: W30PreviewRenderMode::LiveRecall,
                routing: W30PreviewRenderRouting::MusicBusPreview,
                source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
                active_bank_id: Some("bank-a".into()),
                focused_pad_id: Some("pad-01".into()),
                capture_id: Some("capture-stereo".into()),
                trigger_revision: 1,
                trigger_velocity: 0.8,
                source_window_preview: None,
                pad_playback: Some(W30PadPlaybackSampleWindow {
                    source_start_frame: 0,
                    source_end_frame: frame_count as u64,
                    source_sample_rate: 48_000,
                    playback_frame_count: frame_count as u64,
                    sample_count: frame_count,
                    loop_enabled: true,
                    playback_rate: 1.0,
                    reverse: false,
                    gate_step_fraction: 0.0,
                    loop_crossfade_sample_count: 0,
                    chop_slice_count: 0,
                    chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
                    hook_articulation: None,
                    samples: mono,
                }),
                music_bus_level: 0.58,
                grit_level: 0.64,
                is_transport_running: true,
                tempo_bpm: 120.0,
                position_beats: 0.0,
            },
            ..RuntimeMixRenderPlan::default()
        }
    }

    #[test]
    fn public_stereo_development_seam_is_partition_exact_and_distinct() {
        let mut interleaved = Vec::with_capacity(2_048);
        for index in 0..1_024 {
            let phase = index as f32 / 1_024.0;
            interleaved.push((phase * std::f32::consts::TAU * 7.0).sin() * 0.42);
            interleaved.push((phase * std::f32::consts::TAU * 11.0).sin() * 0.31);
        }
        let stereo =
            project_w30_stereo_pad_development_window(&interleaved, 2).expect("stereo projection");
        let plan = public_test_plan(&interleaved);
        let control = render_runtime_mix_realtime_simulation_offline(&plan, 48_000, 2, 4_096, 128);
        let candidate_128 = render_runtime_mix_w30_stereo_development_offline_with_report(
            &plan, &stereo, 48_000, 2, 4_096, 128,
        )
        .expect("128-frame stereo render");
        let candidate_257 = render_runtime_mix_w30_stereo_development_offline_with_report(
            &plan, &stereo, 48_000, 2, 4_096, 257,
        )
        .expect("257-frame stereo render");

        assert_eq!(candidate_128.samples, candidate_257.samples);
        assert_ne!(candidate_128.samples, control);
        assert_eq!(candidate_128.limiter.limited_sample_count, 0);
    }

    #[test]
    fn stereo_development_projection_refuses_non_stereo_or_nonfinite_input() {
        assert!(project_w30_stereo_pad_development_window(&[0.1, 0.2], 1).is_none());
        assert!(project_w30_stereo_pad_development_window(&[0.1, f32::NAN], 2).is_none());
        assert!(project_w30_stereo_pad_development_window(&[0.1], 2).is_none());
    }
}
