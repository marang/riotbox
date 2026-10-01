use riotbox_audio::runtime::OfflineAudioMetrics;
use riotbox_audio::runtime::signal_metrics;

pub(super) fn rms_delta(baseline: OfflineAudioMetrics, candidate: OfflineAudioMetrics) -> f32 {
    (baseline.rms - candidate.rms).abs()
}

pub(super) fn signal_delta_metrics(baseline: &[f32], candidate: &[f32]) -> OfflineAudioMetrics {
    debug_assert_eq!(
        baseline.len(),
        candidate.len(),
        "baseline and candidate renders should use the same frame count"
    );
    let delta = baseline
        .iter()
        .zip(candidate.iter())
        .map(|(baseline, candidate)| baseline - candidate)
        .collect::<Vec<_>>();
    signal_metrics(&delta)
}
