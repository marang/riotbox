use tempfile::tempdir;

use super::*;
use crate::source_audio::{SourceAudioCache, write_interleaved_pcm16_wav};
use riotbox_core::source_graph::{
    MeterHint, SourceTimingCandidateConfidenceResult, SourceTimingCandidatePhraseStatus,
    SourceTimingProbeBeatEvidenceStatus, SourceTimingProbeBpmCandidatePolicy,
    SourceTimingProbeDownbeatEvidenceStatus, SourceTimingProbeReadinessStatus, TimingWarningCode,
    source_timing_probe_beat_evidence_report, source_timing_probe_readiness_report,
};

#[test]
fn source_timing_probe_preserves_real_loop_like_weak_readiness() {
    let tempdir = tempdir().expect("create tempdir");
    let source_path = tempdir.path().join("real_loop_like_128.wav");
    let samples = real_loop_like_flat_drum_samples_128_bpm();
    write_interleaved_pcm16_wav(&source_path, 32_000, 1, &samples).expect("write source");
    let source = SourceAudioCache::load_pcm_wav(&source_path).expect("load source");

    let probe = analyze_source_timing_probe(&source, SourceTimingProbeConfig::default());
    let candidate_input = probe.bpm_candidate_input(
        "real-loop-like-128",
        MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
    );
    let readiness = source_timing_probe_readiness_report(
        &candidate_input,
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert!(probe.onset_count >= 24, "{probe:?}");
    assert_bpm_near(readiness.primary_bpm, 128.0, 1.0);
    assert_eq!(
        readiness.beat_status,
        SourceTimingProbeBeatEvidenceStatus::Ambiguous
    );
    assert_eq!(
        readiness.downbeat_status,
        SourceTimingProbeDownbeatEvidenceStatus::Weak
    );
    assert_eq!(
        readiness.confidence_result,
        SourceTimingCandidateConfidenceResult::CandidateAmbiguous
    );
    assert_eq!(
        readiness.phrase_status,
        SourceTimingCandidatePhraseStatus::AmbiguousDownbeat
    );
    assert_eq!(readiness.readiness, SourceTimingProbeReadinessStatus::Weak);
    assert!(readiness.requires_manual_confirm);
    assert!(
        readiness
            .warning_codes
            .contains(&TimingWarningCode::AmbiguousDownbeat),
        "{readiness:?}"
    );
    assert!(
        readiness
            .warning_codes
            .contains(&TimingWarningCode::PhraseUncertain),
        "{readiness:?}"
    );
    assert!(
        readiness
            .warning_codes
            .contains(&TimingWarningCode::HalfTimePossible),
        "{readiness:?}"
    );
}

#[test]
fn source_timing_probe_accepts_real_loop_like_ready_readiness() {
    let tempdir = tempdir().expect("create tempdir");
    let source_path = tempdir.path().join("real_loop_like_ready_128.wav");
    let samples = real_loop_like_accented_drum_samples_128_bpm();
    write_interleaved_pcm16_wav(&source_path, 32_768, 1, &samples).expect("write source");
    let source = SourceAudioCache::load_pcm_wav(&source_path).expect("load source");

    let probe = analyze_source_timing_probe(&source, SourceTimingProbeConfig::default());
    let candidate_input = probe.bpm_candidate_input(
        "real-loop-like-ready-128",
        MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
    );
    let policy = SourceTimingProbeBpmCandidatePolicy::dance_loop_auto_readiness();
    let readiness = source_timing_probe_readiness_report(&candidate_input, policy);
    let beat_evidence = source_timing_probe_beat_evidence_report(&candidate_input, policy);

    assert!(probe.onset_count >= 24, "{probe:?}");
    assert_bpm_near(readiness.primary_bpm, 128.0, 1.0);
    assert_eq!(
        readiness.beat_status,
        SourceTimingProbeBeatEvidenceStatus::Stable,
        "{readiness:?} {beat_evidence:?}"
    );
    assert_eq!(
        readiness.downbeat_status,
        SourceTimingProbeDownbeatEvidenceStatus::Stable
    );
    assert_eq!(
        readiness.phrase_status,
        SourceTimingCandidatePhraseStatus::Stable
    );
    assert_eq!(readiness.readiness, SourceTimingProbeReadinessStatus::Ready);
    assert!(
        !readiness
            .warning_codes
            .contains(&TimingWarningCode::AmbiguousDownbeat),
        "{readiness:?}"
    );
    assert!(
        !readiness
            .warning_codes
            .contains(&TimingWarningCode::PhraseUncertain),
        "{readiness:?}"
    );
}

fn assert_bpm_near(actual: Option<f32>, expected: f32, tolerance: f32) {
    let actual = actual.expect("bpm estimate");
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {actual} near {expected} within {tolerance}"
    );
}

fn real_loop_like_flat_drum_samples_128_bpm() -> Vec<f32> {
    const BEAT_FRAMES: usize = 15_000;
    const BEATS: usize = 32;

    let mut samples = vec![0.0_f32; BEAT_FRAMES * BEATS];
    for beat in 0..BEATS {
        let start = beat * BEAT_FRAMES;
        add_impulse(&mut samples, start, 96, 0.82);
        add_impulse(&mut samples, start + BEAT_FRAMES / 2, 32, 0.09);
    }

    samples
}

fn real_loop_like_accented_drum_samples_128_bpm() -> Vec<f32> {
    const BEAT_FRAMES: usize = 15_360;
    const BEATS: usize = 32;

    let mut samples = vec![0.0_f32; BEAT_FRAMES * BEATS];
    for beat in 0..BEATS {
        let start = beat * BEAT_FRAMES;
        let amplitude = if beat % 4 == 0 { 1.0 } else { 0.45 };
        add_impulse(&mut samples, start, 96, amplitude);
        add_impulse(&mut samples, start + BEAT_FRAMES / 2, 32, 0.08);
    }

    samples
}

fn add_impulse(samples: &mut [f32], start: usize, impulse_frames: usize, amplitude: f32) {
    let end = start.saturating_add(impulse_frames).min(samples.len());
    for sample in samples.iter_mut().take(end).skip(start) {
        *sample = amplitude;
    }
}
