//! Control-side preparation of the exact processor used by the CPAL data callback.

use crate::runtime::fill_focus::FillFocusRenderState;
use crate::runtime::public_api_shell::apply_master_bus_soft_limiter;
use crate::runtime::shared_transport_tr909::AudioRuntimeSharedState;
use crate::runtime::shared_transport_tr909::callback_scratch_sample_count;
use crate::runtime::shared_w30_resample_callback::Tr909CallbackState;
use crate::runtime::shared_w30_resample_callback::TransportTimingCallbackState;
use crate::runtime::shared_w30_resample_callback::W30MixRenderState;
use crate::runtime::shared_w30_resample_callback::W30PreviewCallbackState;
use crate::runtime::shared_w30_resample_callback::W30ResampleTapCallbackState;
use crate::runtime::shared_w30_resample_callback::advance_transport_timing;
use crate::runtime::shared_w30_resample_callback::render_mix_buffer;
use crate::runtime::source_monitor::SourceMonitorCallbackState;
use crate::runtime::source_monitor::apply_source_monitor_policy_with_state_and_fill_focus;
use crate::runtime::w30_preview_snapshot::W30PreviewSnapshotCache;
use std::sync::Arc;
use std::time::Instant;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CallbackPreparationError {
    SourceReaderAlreadyTaken,
    CaptureReaderAlreadyTaken,
}

impl CallbackPreparationError {
    pub(super) const fn message(self) -> &'static str {
        match self {
            Self::SourceReaderAlreadyTaken => "source-monitor callback reader already acquired",
            Self::CaptureReaderAlreadyTaken => {
                "live-master capture callback reader already acquired"
            }
        }
    }
}

pub(super) fn prepare_output_callback<T>(
    config: &cpal::StreamConfig,
    shared: AudioRuntimeSharedState,
    start: Instant,
) -> Result<impl FnMut(&mut [T]) + Send + 'static + use<T>, CallbackPreparationError>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    // Unique readers are acquired on control, before the worker is created.
    // Never substitute observer snapshots (which lock) in the data callback.
    let mut source_reader = shared
        .source_monitor
        .take_callback_reader()
        .ok_or(CallbackPreparationError::SourceReaderAlreadyTaken)?;
    let mut capture_reader = shared
        .live_master_capture
        .take_callback_reader()
        .ok_or(CallbackPreparationError::CaptureReaderAlreadyTaken)?;
    let callback_telemetry = Arc::clone(&shared.telemetry);
    let callback_transport = Arc::clone(&shared.transport);
    let mut render_state = Tr909CallbackState::default();
    let mut transport_state = TransportTimingCallbackState::default();
    let channel_count = usize::from(config.channels.max(1));
    let mut w30_preview_state =
        W30PreviewCallbackState::with_sample_rate_and_channels(config.sample_rate, channel_count);
    let mut w30_resample_state = W30ResampleTapCallbackState::default();
    let mut source_monitor_callback_state = SourceMonitorCallbackState::default();
    let sample_rate = config.sample_rate;
    let mut mix_buffer = vec![0.0; callback_scratch_sample_count(config, channel_count)];
    let mut last_transport_snapshot = callback_transport.snapshot();
    let mut last_tr909_render_snapshot = shared.tr909_render.snapshot();
    let mut last_mc202_render_snapshot = shared.mc202_render.snapshot();
    let mut w30_preview_snapshot = W30PreviewSnapshotCache::new(&shared.w30_preview);
    let mut last_w30_resample_snapshot = shared.w30_resample_tap.snapshot();

    Ok(move |data: &mut [T]| {
        let frame_count = data.len() / channel_count.max(1);
        let transport_snapshot = callback_transport.snapshot_or_previous(&last_transport_snapshot);
        last_transport_snapshot = transport_snapshot;
        let callback_timing = advance_transport_timing(
            &transport_snapshot,
            &mut transport_state,
            sample_rate,
            frame_count,
        );
        let Some(mix_buffer) = mix_buffer.get_mut(..data.len()) else {
            for output in data.iter_mut() {
                *output = T::from_sample(0.0);
            }
            let now = start.elapsed().as_micros() as u64;
            capture_reader.record_scratch_overflow(&callback_timing, now);
            callback_telemetry.record_callback_scratch_overflow_at(now, &callback_timing);
            return;
        };
        let mut tr909_render_state = shared
            .tr909_render
            .snapshot_or_previous(&last_tr909_render_snapshot);
        last_tr909_render_snapshot = tr909_render_state;
        tr909_render_state.is_transport_running = callback_timing.is_transport_running;
        tr909_render_state.tempo_bpm = callback_timing.tempo_bpm;
        tr909_render_state.position_beats = callback_timing.render_position_beats;
        let mut mc202_render_state = shared
            .mc202_render
            .snapshot_or_previous(&last_mc202_render_snapshot);
        last_mc202_render_snapshot = mc202_render_state;
        mc202_render_state.is_transport_running = callback_timing.is_transport_running;
        mc202_render_state.tempo_bpm = callback_timing.tempo_bpm;
        mc202_render_state.position_beats = callback_timing.render_position_beats;
        let w30_preview_render_state = w30_preview_snapshot.refresh(&shared.w30_preview);
        w30_preview_render_state.is_transport_running = callback_timing.is_transport_running;
        w30_preview_render_state.tempo_bpm = callback_timing.tempo_bpm;
        w30_preview_render_state.position_beats = callback_timing.render_position_beats;
        let mut w30_resample_render_state = shared
            .w30_resample_tap
            .snapshot_or_previous(&last_w30_resample_snapshot);
        last_w30_resample_snapshot = w30_resample_render_state;
        w30_resample_render_state.is_transport_running = callback_timing.is_transport_running;
        w30_resample_render_state.tempo_bpm = callback_timing.tempo_bpm;
        w30_resample_render_state.position_beats = callback_timing.render_position_beats;
        let source_monitor_snapshot = source_reader.snapshot();
        let mut source_monitor_state = source_monitor_snapshot.render_state();
        source_monitor_state.is_transport_running = callback_timing.is_transport_running;
        source_monitor_state.tempo_bpm = callback_timing.tempo_bpm;
        source_monitor_state.position_beats = callback_timing.render_position_beats;

        render_mix_buffer(
            mix_buffer,
            sample_rate,
            channel_count,
            &tr909_render_state,
            &mc202_render_state,
            &mut render_state,
            &mut W30MixRenderState {
                preview_render: w30_preview_render_state,
                preview_state: &mut w30_preview_state,
                resample_render: &w30_resample_render_state,
                resample_state: &mut w30_resample_state,
            },
        );
        apply_source_monitor_policy_with_state_and_fill_focus(
            mix_buffer,
            sample_rate,
            channel_count,
            &source_monitor_state,
            FillFocusRenderState::from_tr909(&tr909_render_state),
            &mut source_monitor_callback_state,
        );
        apply_master_bus_soft_limiter(mix_buffer);
        let now = start.elapsed().as_micros() as u64;
        capture_reader.record_callback(mix_buffer, &callback_timing, now);
        for (output, sample) in data.iter_mut().zip(mix_buffer.iter().copied()) {
            *output = T::from_sample(sample);
        }

        callback_telemetry.record_callback_at(now, &callback_timing);
    })
}
