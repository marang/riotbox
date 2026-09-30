use crate::mc202::Mc202SourcePhraseRenderPlan;
use crate::runtime::shared_w30_resample_callback::RealtimeW30ResampleSourceWindow;
use crate::runtime::w30_preview_snapshot::RealtimeW30PreviewSampleWindow;
use crate::w30::{W30_PREVIEW_SAMPLE_WINDOW_LEN, W30_RESAMPLE_SOURCE_WINDOW_LEN};

pub(super) fn fill_positive_preview_ramp(samples: &mut [f32; W30_PREVIEW_SAMPLE_WINDOW_LEN]) {
    let denominator = W30_PREVIEW_SAMPLE_WINDOW_LEN.saturating_sub(1).max(1) as f32;
    for (index, sample) in samples.iter_mut().enumerate() {
        let progress = index as f32 / denominator;
        *sample = 0.18 + progress * 0.12;
    }
}

pub(super) fn positive_realtime_source_window() -> RealtimeW30PreviewSampleWindow {
    let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    fill_positive_preview_ramp(&mut samples);
    RealtimeW30PreviewSampleWindow {
        source_start_frame: 0,
        source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
        sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
        samples,
    }
}

pub(super) fn positive_realtime_resample_source() -> RealtimeW30ResampleSourceWindow {
    let mut samples = [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        let phase = index as f32 / 37.0;
        *sample = phase.sin() * 0.34 + (phase * 2.7).sin() * 0.09;
    }
    RealtimeW30ResampleSourceWindow {
        source_start_frame: 0,
        source_sample_rate: 44_100,
        source_frame_count: W30_RESAMPLE_SOURCE_WINDOW_LEN as u64,
        sample_count: W30_RESAMPLE_SOURCE_WINDOW_LEN,
        samples,
    }
}

pub(super) fn mc202_source_plan() -> Mc202SourcePhraseRenderPlan {
    Mc202SourcePhraseRenderPlan {
        active_mask: 0b0001_0001_0010_0101,
        semitones: [-12, 0, -7, 0, 0, -5, 0, 0, -10, 0, 0, 0, -3, 0, 0, 0],
        accent_mask: 0b0001_0000_0000_0001,
        destructive_mask: 0b0000_0000_0001_0000,
        pressure: 0.70,
        contrast: 0.56,
        bass_weight: 0.72,
        stab_bite: 0.26,
        gate_snap: 0.22,
    }
}
