use tempfile::tempdir;

use super::*;
use crate::source_audio::{SourceAudioCache, write_interleaved_pcm16_wav};

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
}

fn impulse_train_samples(
    frame_count: usize,
    starts: &[usize],
    impulse_frames: usize,
    amplitude: f32,
) -> Vec<f32> {
    let mut samples = vec![0.0_f32; frame_count];
    for &start in starts {
        for sample in samples
            .iter_mut()
            .take(start.saturating_add(impulse_frames).min(frame_count))
            .skip(start)
        {
            *sample = amplitude;
        }
    }
    samples
}
