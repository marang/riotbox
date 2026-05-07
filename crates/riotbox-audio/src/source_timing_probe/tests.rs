use tempfile::tempdir;

use super::*;
use crate::source_audio::{SourceAudioCache, write_interleaved_pcm16_wav};
use riotbox_core::source_graph::{
    MeterHint, SourceTimingCandidateConfidenceResult, SourceTimingProbeBpmCandidatePolicy,
    SourceTimingProbeDiagnosticPolicy, TimingDegradedPolicy, TimingHypothesisKind, TimingQuality,
    source_timing_candidate_confidence_report, timing_model_from_probe_bpm_candidates,
    timing_model_from_probe_diagnostics,
};

#[test]
fn source_timing_probe_detects_impulse_onsets_from_pcm_wav_cache() {
    let tempdir = tempdir().expect("create tempdir");
    let source_path = tempdir.path().join("impulses.wav");
    let samples = impulse_train_samples(2_000, &[100, 600, 1_100, 1_600], 12, 0.9);
    write_interleaved_pcm16_wav(&source_path, 1_000, 1, &samples).expect("write source");
    let source = SourceAudioCache::load_pcm_wav(&source_path).expect("load source");

    let probe = analyze_source_timing_probe(
        &source,
        SourceTimingProbeConfig {
            window_size_frames: 50,
            hop_size_frames: 50,
            onset_threshold_ratio: 0.30,
            min_onset_flux: 0.01,
        },
    );

    assert_eq!(probe.sample_rate, 1_000);
    assert_eq!(probe.channel_count, 1);
    assert!((probe.duration_seconds - 2.0).abs() < 0.001);
    assert_eq!(probe.windows.len(), 40);
    assert!(probe.peak_energy > 0.30);
    assert!(probe.peak_positive_flux > 0.30);
    assert!(probe.onset_count >= 4, "{probe:?}");
    assert!(probe.onset_density_per_second >= 2.0);
    assert!(
        probe
            .windows
            .iter()
            .any(|window| window.onset && window.start_frame == 100)
    );

    let timing = timing_model_from_probe_diagnostics(
        &probe.diagnostic_input("impulses"),
        SourceTimingProbeDiagnosticPolicy::default(),
    );
    assert_eq!(timing.effective_timing_quality(), TimingQuality::Low);
    assert_eq!(
        timing.effective_degraded_policy(),
        TimingDegradedPolicy::ManualConfirm
    );
    assert!(timing.bpm_estimate.is_none());

    let bpm_timing = timing_model_from_probe_bpm_candidates(
        &probe.bpm_candidate_input(
            "impulses",
            MeterHint {
                beats_per_bar: 4,
                beat_unit: 4,
            },
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );
    let bpm = bpm_timing.bpm_estimate.expect("bpm estimate");
    assert!((bpm - 120.0).abs() <= 0.01, "{bpm}");
    assert!(!bpm_timing.hypotheses.is_empty());
}

#[test]
fn source_timing_probe_candidate_fixture_seed_scores_pcm_wav_grid() {
    let tempdir = tempdir().expect("create tempdir");
    let source_path = tempdir.path().join("fixture_like_120.wav");
    let samples = fixture_like_break_samples();
    write_interleaved_pcm16_wav(&source_path, 1_000, 1, &samples).expect("write source");
    let source = SourceAudioCache::load_pcm_wav(&source_path).expect("load source");

    let probe = analyze_source_timing_probe(
        &source,
        SourceTimingProbeConfig {
            window_size_frames: 50,
            hop_size_frames: 50,
            onset_threshold_ratio: 0.45,
            min_onset_flux: 0.01,
        },
    );

    assert_eq!(probe.sample_rate, 1_000);
    assert!((probe.duration_seconds - 4.0).abs() < 0.001);
    assert!(probe.onset_count >= 8, "{probe:?}");
    assert!(probe.onset_density_per_second >= 2.0, "{probe:?}");
    let onset_times = probe
        .windows
        .iter()
        .filter(|window| window.onset)
        .map(|window| window.start_seconds)
        .collect::<Vec<_>>();

    let timing = timing_model_from_probe_bpm_candidates(
        &probe.bpm_candidate_input(
            "fixture-like-120",
            MeterHint {
                beats_per_bar: 4,
                beat_unit: 4,
            },
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );
    let report = source_timing_candidate_confidence_report(&timing);
    let primary = timing.primary_hypothesis().expect("primary hypothesis");

    let bpm = timing.bpm_estimate.expect("bpm estimate");
    assert!((bpm - 120.0).abs() <= 0.01, "{bpm} {onset_times:?}");
    assert_eq!(primary.kind, TimingHypothesisKind::Primary);
    assert!(primary.beat_grid.len() >= 8);
    assert!(
        primary
            .provenance
            .contains(&"source-timing-probe.beat-period-score.v0".into())
    );
    assert!(
        timing
            .hypotheses
            .iter()
            .any(|hypothesis| hypothesis.kind == TimingHypothesisKind::HalfTime)
    );
    assert!(
        timing
            .hypotheses
            .iter()
            .any(|hypothesis| hypothesis.kind == TimingHypothesisKind::DoubleTime)
    );
    assert_eq!(
        report.result,
        SourceTimingCandidateConfidenceResult::CandidateAmbiguous
    );
    assert!(report.requires_manual_confirm);
}

#[test]
fn source_timing_probe_stays_quiet_for_silence() {
    let tempdir = tempdir().expect("create tempdir");
    let source_path = tempdir.path().join("silence.wav");
    write_interleaved_pcm16_wav(&source_path, 1_000, 1, &vec![0.0; 1_000]).expect("write silence");
    let source = SourceAudioCache::load_pcm_wav(&source_path).expect("load source");

    let probe = analyze_source_timing_probe(
        &source,
        SourceTimingProbeConfig {
            window_size_frames: 100,
            hop_size_frames: 100,
            onset_threshold_ratio: 0.35,
            min_onset_flux: 0.01,
        },
    );

    assert_eq!(probe.windows.len(), 10);
    assert_eq!(probe.peak_energy, 0.0);
    assert_eq!(probe.peak_positive_flux, 0.0);
    assert_eq!(probe.onset_count, 0);
    assert_eq!(probe.onset_density_per_second, 0.0);
    assert!(probe.windows.iter().all(|window| !window.onset));

    let timing = timing_model_from_probe_diagnostics(
        &probe.diagnostic_input("silence"),
        SourceTimingProbeDiagnosticPolicy::default(),
    );
    assert_eq!(timing.effective_timing_quality(), TimingQuality::Unknown);
    assert_eq!(
        timing.effective_degraded_policy(),
        TimingDegradedPolicy::Disabled
    );
    assert!(timing.bpm_estimate.is_none());
}

fn fixture_like_break_samples() -> Vec<f32> {
    let mut samples = vec![0.0_f32; 4_000];
    let beat_starts = [100, 600, 1_100, 1_600, 2_100, 2_600, 3_100, 3_600];
    for (index, start) in beat_starts.iter().copied().enumerate() {
        let amplitude = if index % 4 == 0 { 0.95 } else { 0.72 };
        add_impulse(&mut samples, start, 16, amplitude);
    }

    for start in [350, 850, 1_850, 2_350, 2_850, 3_350] {
        add_impulse(&mut samples, start, 8, 0.05);
    }

    samples
}

fn impulse_train_samples(
    frame_count: usize,
    starts: &[usize],
    impulse_frames: usize,
    amplitude: f32,
) -> Vec<f32> {
    let mut samples = vec![0.0_f32; frame_count];
    for &start in starts {
        add_impulse(&mut samples, start, impulse_frames, amplitude);
    }
    samples
}

fn add_impulse(samples: &mut [f32], start: usize, impulse_frames: usize, amplitude: f32) {
    let end = start.saturating_add(impulse_frames).min(samples.len());
    for sample in samples.iter_mut().take(end).skip(start) {
        *sample = amplitude;
    }
}
