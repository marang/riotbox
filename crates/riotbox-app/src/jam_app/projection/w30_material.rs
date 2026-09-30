use riotbox_audio::source_audio::{SourceAudioCache, SourceAudioWindow};
use riotbox_audio::w30::{
    W30_PAD_CHOP_SLICE_COUNT, W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30_PREVIEW_SAMPLE_WINDOW_LEN,
    W30PadPlaybackSampleWindow, W30PreviewSampleWindow,
};
use riotbox_core::ids::CaptureId;
use riotbox_core::source_graph::SourceGraph;
use std::collections::BTreeMap;

pub(super) fn build_w30_capture_artifact_playback(
    capture: &riotbox_core::session::CaptureRef,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
    transform: W30PadPlaybackTransform,
) -> Option<W30PadPlaybackSampleWindow> {
    let cache = capture_audio_cache?.get(&capture.capture_id)?;
    pad_playback_from_interleaved(
        cache.interleaved_samples(),
        usize::from(cache.channel_count),
        cache.sample_rate,
        0,
        cache.frame_count().try_into().unwrap_or(u64::MAX),
        transform,
    )
}

pub(super) fn build_w30_capture_artifact_preview(
    capture: &riotbox_core::session::CaptureRef,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
) -> Option<W30PreviewSampleWindow> {
    let cache = capture_audio_cache?.get(&capture.capture_id)?;
    source_preview_from_interleaved(
        cache.interleaved_samples(),
        usize::from(cache.channel_count),
        0,
        cache.frame_count().try_into().unwrap_or(u64::MAX),
    )
}

pub(super) fn build_w30_source_window_preview(
    capture: &riotbox_core::session::CaptureRef,
    source_graph: Option<&SourceGraph>,
    source_audio_cache: Option<&SourceAudioCache>,
) -> Option<W30PreviewSampleWindow> {
    let source_window = capture.source_window.as_ref()?;
    let graph = source_graph?;
    if source_window.source_id != graph.source.source_id {
        return None;
    }

    let cache = source_audio_cache?;
    let start_frame = usize::try_from(source_window.start_frame).unwrap_or(usize::MAX);
    let end_frame = usize::try_from(source_window.end_frame).unwrap_or(usize::MAX);
    let frame_count = end_frame.saturating_sub(start_frame);
    let window = SourceAudioWindow {
        start_frame,
        frame_count,
    };
    let samples = cache.window_samples(window);
    source_preview_from_interleaved(
        samples,
        usize::from(cache.channel_count),
        source_window.start_frame,
        source_window.end_frame,
    )
}

fn source_preview_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_start_frame: u64,
    source_end_frame: u64,
) -> Option<W30PreviewSampleWindow> {
    let channel_count = channel_count.max(1);
    let frame_count = samples.len() / channel_count;
    if frame_count == 0 {
        return None;
    }

    let sample_count = frame_count.min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    let stride = (frame_count / sample_count).max(1);
    let mut preview = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];

    for (index, slot) in preview.iter_mut().take(sample_count).enumerate() {
        let frame_index = (index * stride).min(frame_count - 1);
        let base = frame_index * channel_count;
        let sum: f32 = samples[base..base + channel_count].iter().sum();
        *slot = sum / channel_count as f32;
    }

    Some(W30PreviewSampleWindow {
        source_start_frame,
        source_end_frame,
        sample_count,
        samples: preview,
    })
}

fn pad_playback_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_sample_rate: u32,
    source_start_frame: u64,
    source_end_frame: u64,
    transform: W30PadPlaybackTransform,
) -> Option<W30PadPlaybackSampleWindow> {
    let channel_count = channel_count.max(1);
    let frame_count = samples.len() / channel_count;
    if frame_count == 0 {
        return None;
    }

    let sample_count = frame_count.min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
    let mut playback = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    for (index, slot) in playback.iter_mut().take(sample_count).enumerate() {
        let frame_index = if sample_count <= 1 {
            0
        } else {
            index * (frame_count - 1) / (sample_count - 1)
        };
        let base = frame_index * channel_count;
        let sum: f32 = samples[base..base + channel_count].iter().sum();
        *slot = sum / channel_count as f32;
    }
    let chop_slice_starts = derive_transient_chop_plan(&playback[..sample_count]);

    Some(W30PadPlaybackSampleWindow {
        source_start_frame,
        source_end_frame,
        source_sample_rate,
        playback_frame_count: frame_count.try_into().unwrap_or(u64::MAX),
        sample_count,
        loop_enabled: true,
        playback_rate: transform.playback_rate,
        reverse: transform.reverse,
        gate_step_fraction: transform.gate_step_fraction,
        loop_crossfade_sample_count: sample_count.min(128).min(sample_count / 4),
        chop_slice_count: W30_PAD_CHOP_SLICE_COUNT,
        chop_slice_starts,
        hook_articulation: None,
        samples: playback,
    })
}

fn derive_transient_chop_plan(samples: &[f32]) -> [u32; W30_PAD_CHOP_SLICE_COUNT] {
    if samples.len() < W30_PAD_CHOP_SLICE_COUNT * 2 {
        return [0; W30_PAD_CHOP_SLICE_COUNT];
    }

    let bin_len = samples.len() / W30_PAD_CHOP_SLICE_COUNT;
    let mut candidates = [(0_usize, 0.0_f32); W30_PAD_CHOP_SLICE_COUNT];
    const ONSET_WINDOW: usize = 64;
    const ONSET_HOP: usize = 32;
    for (bin, candidate) in candidates.iter_mut().enumerate() {
        let start = bin * bin_len;
        let end = if bin + 1 == W30_PAD_CHOP_SLICE_COUNT {
            samples.len()
        } else {
            (bin + 1) * bin_len
        };
        let search_start = (start + ONSET_WINDOW).min(end);
        let search_end = end.saturating_sub(ONSET_WINDOW);
        let mut strongest = (start, 0.0_f32);
        for index in (search_start..search_end).step_by(ONSET_HOP) {
            let before = samples[index - ONSET_WINDOW..index]
                .iter()
                .map(|sample| sample.abs())
                .sum::<f32>()
                / ONSET_WINDOW as f32;
            let after = samples[index..index + ONSET_WINDOW]
                .iter()
                .map(|sample| sample.abs())
                .sum::<f32>()
                / ONSET_WINDOW as f32;
            let onset = (after - before).max(0.0) + after * 0.05;
            if onset > strongest.1 {
                strongest = (index, onset);
            }
        }
        *candidate = strongest;
    }
    candidates.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });

    // A compact sampler riff: the strongest source onset becomes the anchor while
    // three other source-derived attacks supply call/response variation.
    let ranked = [
        candidates[0].0,
        candidates[1].0,
        candidates[2].0,
        candidates[3].0,
    ];
    let order = [0, 1, 0, 2, 0, 3, 1, 2];
    std::array::from_fn(|index| ranked[order[index]].try_into().unwrap_or(u32::MAX))
}

#[cfg(test)]
mod transient_chop_plan_tests {
    use super::derive_transient_chop_plan;
    use std::collections::BTreeSet;

    #[test]
    fn chop_plan_repeats_the_strongest_source_onset_as_a_riff_anchor() {
        let mut samples = vec![0.0; 8_192];
        for (index, amplitude) in [(700, 0.4), (1_500, 0.7), (3_000, 1.0), (6_700, 0.6)] {
            samples[index] = amplitude;
        }

        let plan = derive_transient_chop_plan(&samples);

        assert_eq!(plan[0], plan[2]);
        assert_eq!(plan[0], plan[4]);
        assert!(plan[0].abs_diff(3_000) <= 64);
        assert!(plan.iter().copied().collect::<BTreeSet<_>>().len() >= 4);
    }

    #[test]
    fn chop_plan_changes_when_source_transients_move() {
        let mut first = vec![0.0; 8_192];
        let mut second = vec![0.0; 8_192];
        for index in [400, 1_600, 3_200, 5_900] {
            first[index] = 0.8;
        }
        for index in [800, 2_300, 4_700, 7_200] {
            second[index] = 0.8;
        }

        assert_ne!(
            derive_transient_chop_plan(&first),
            derive_transient_chop_plan(&second)
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct W30PadPlaybackTransform {
    pub(super) playback_rate: f32,
    pub(super) reverse: bool,
    pub(super) gate_step_fraction: f32,
}

impl Default for W30PadPlaybackTransform {
    fn default() -> Self {
        Self {
            playback_rate: 1.0,
            reverse: false,
            gate_step_fraction: 0.0,
        }
    }
}
