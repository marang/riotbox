use crate::mc202::Mc202SourcePhraseRenderPlan;
use crate::w30::{
    W30_PAD_CHOP_SLICE_COUNT, W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30_PREVIEW_SAMPLE_WINDOW_LEN,
    W30_RESAMPLE_SOURCE_WINDOW_LEN, W30PadPlaybackSampleWindow, W30PreviewSampleWindow,
    W30ResampleSourceWindow,
};

pub(super) fn runtime_mix_parity_source_plan() -> Mc202SourcePhraseRenderPlan {
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

pub(super) fn runtime_mix_parity_source_window() -> W30PreviewSampleWindow {
    let mut window = W30PreviewSampleWindow {
        source_start_frame: 4_096,
        source_end_frame: 4_096 + W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
        sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
        samples: [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN],
    };
    for (index, sample) in window.samples.iter_mut().enumerate() {
        let phase = index as f32 / W30_PREVIEW_SAMPLE_WINDOW_LEN as f32;
        *sample = ((phase * std::f32::consts::TAU * 3.0).sin() * 0.45)
            + ((phase * std::f32::consts::TAU * 11.0).sin() * 0.15);
    }
    window
}

pub(super) fn runtime_mix_resample_source() -> W30ResampleSourceWindow {
    let mut samples = [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        let phase = index as f32 / 41.0;
        *sample = phase.sin() * 0.32 + (phase * 2.3).sin() * 0.08;
    }
    W30ResampleSourceWindow {
        source_start_frame: 0,
        source_sample_rate: 48_000,
        source_frame_count: W30_RESAMPLE_SOURCE_WINDOW_LEN as u64,
        sample_count: W30_RESAMPLE_SOURCE_WINDOW_LEN,
        samples,
    }
}

pub(super) fn runtime_mix_duration_pad() -> W30PadPlaybackSampleWindow {
    let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        let phase = index as f32 / W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as f32;
        let transient = if index % 2_048 < 96 { 0.5 } else { 0.0 };
        *sample = ((phase * std::f32::consts::TAU * 7.0).sin() * 0.42) + transient;
    }
    W30PadPlaybackSampleWindow {
        source_start_frame: 0,
        source_end_frame: 96_000,
        source_sample_rate: 48_000,
        playback_frame_count: 96_000,
        sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
        loop_enabled: true,
        playback_rate: 1.0,
        reverse: false,
        gate_step_fraction: 0.0,
        loop_crossfade_sample_count: 128,
        chop_slice_count: 0,
        chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
        hook_articulation: None,
        samples,
    }
}
