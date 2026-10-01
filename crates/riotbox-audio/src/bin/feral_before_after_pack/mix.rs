use super::config::{CHANNEL_COUNT, SILENCE_SECONDS};
use super::source_window::seconds_to_frames;
use riotbox_audio::runtime::{OfflineAudioMetrics, signal_metrics};

pub(super) fn feral_after_mix(
    source: &[f32],
    w30: &[f32],
    tr909: &[f32],
    mc202: &[f32],
) -> Vec<f32> {
    debug_assert_eq!(source.len(), w30.len());
    debug_assert_eq!(source.len(), tr909.len());
    debug_assert_eq!(source.len(), mc202.len());

    source
        .iter()
        .zip(w30.iter())
        .zip(tr909.iter())
        .zip(mc202.iter())
        .map(|(((source, w30), tr909), mc202)| {
            let mixed = source * 0.28 + w30 * 1.10 + tr909 * 1.20 + mc202 * 0.95;
            (mixed * 2.5).tanh() * 0.94
        })
        .collect()
}

pub(super) fn before_then_after(source: &[f32], after: &[f32]) -> Vec<f32> {
    let silence_len = seconds_to_frames(SILENCE_SECONDS) * usize::from(CHANNEL_COUNT);
    let mut output = Vec::with_capacity(source.len() + silence_len + after.len());
    output.extend_from_slice(source);
    output.extend(std::iter::repeat_n(0.0, silence_len));
    output.extend_from_slice(after);
    output
}

pub(super) fn signal_delta_metrics(baseline: &[f32], candidate: &[f32]) -> OfflineAudioMetrics {
    let delta = baseline
        .iter()
        .zip(candidate.iter())
        .map(|(baseline, candidate)| baseline - candidate)
        .collect::<Vec<_>>();
    signal_metrics(&delta)
}
