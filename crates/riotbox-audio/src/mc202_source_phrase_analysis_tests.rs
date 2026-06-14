use super::*;
use riotbox_core::source_graph::{BarSpan, MeterHint, TimingModel};

const SAMPLE_RATE: u32 = 8_000;

#[test]
fn measured_phrase_features_are_deterministic_for_same_source() {
    let timing = fixture_timing();
    let source = SourceAudioCache::from_interleaved_samples(
        "pressure.wav",
        SAMPLE_RATE,
        1,
        pressure_break_samples(),
    )
    .expect("source");

    let first = analyze_mc202_phrase_audio_features(&source, &timing);
    let second = analyze_mc202_phrase_audio_features(&source, &timing);

    assert_eq!(first, second);
    assert!(first[0].has_measured_evidence());
    assert!(
        first[0]
            .provenance_refs
            .contains(&"mc202.phrase-audio-features.v0".into())
    );
}

#[test]
fn measured_phrase_features_change_across_audio_sources() {
    let timing = fixture_timing();
    let pressure = SourceAudioCache::from_interleaved_samples(
        "pressure.wav",
        SAMPLE_RATE,
        1,
        pressure_break_samples(),
    )
    .expect("pressure source");
    let bright = SourceAudioCache::from_interleaved_samples(
        "bright.wav",
        SAMPLE_RATE,
        1,
        bright_offbeat_samples(),
    )
    .expect("bright source");

    let pressure_features = analyze_mc202_phrase_audio_features(&pressure, &timing);
    let bright_features = analyze_mc202_phrase_audio_features(&bright, &timing);

    assert!(pressure_features[0].low_band_rms > bright_features[0].low_band_rms);
    assert!(pressure_features[0].low_mid_ratio > bright_features[0].low_mid_ratio);
    assert!(bright_features[0].spectral_brightness > pressure_features[0].spectral_brightness);
    assert_ne!(pressure_features[0], bright_features[0]);
}

#[test]
fn measured_phrase_features_reject_silence_as_evidence() {
    let timing = fixture_timing();
    let source = SourceAudioCache::from_interleaved_samples(
        "silence.wav",
        SAMPLE_RATE,
        1,
        vec![0.0; SAMPLE_RATE as usize * 4],
    )
    .expect("silence source");

    let features = analyze_mc202_phrase_audio_features(&source, &timing);

    assert_eq!(features.len(), 1);
    assert!(!features[0].has_measured_evidence());
    assert!(features[0].confidence < 0.75);
}

fn fixture_timing() -> TimingModel {
    let mut timing = TimingModel {
        bpm_estimate: Some(120.0),
        bpm_confidence: 0.92,
        meter_hint: Some(MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        }),
        phrase_grid: vec![PhraseSpan {
            phrase_index: 0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.88,
        }],
        ..TimingModel::default()
    };
    timing.bar_grid = vec![
        BarSpan {
            bar_index: 1,
            start_seconds: 0.0,
            end_seconds: 2.0,
            downbeat_confidence: 0.9,
            phrase_index: Some(0),
        },
        BarSpan {
            bar_index: 2,
            start_seconds: 2.0,
            end_seconds: 4.0,
            downbeat_confidence: 0.9,
            phrase_index: Some(0),
        },
    ];
    timing
}

fn pressure_break_samples() -> Vec<f32> {
    let mut samples = vec![0.0; SAMPLE_RATE as usize * 4];
    for beat in 0..8 {
        let start = beat * SAMPLE_RATE as usize / 2;
        add_tone(&mut samples, start, 1600, 72.0, 0.55);
        add_impulse(&mut samples, start, 80, 0.7);
    }
    samples
}

fn bright_offbeat_samples() -> Vec<f32> {
    let mut samples = vec![0.0; SAMPLE_RATE as usize * 4];
    for beat in 0..8 {
        let start = beat * SAMPLE_RATE as usize / 2 + SAMPLE_RATE as usize / 4;
        add_tone(&mut samples, start, 480, 1_900.0, 0.42);
        add_impulse(&mut samples, start, 40, 0.5);
    }
    samples
}

fn add_tone(samples: &mut [f32], start: usize, frames: usize, hz: f32, amplitude: f32) {
    let end = start.saturating_add(frames).min(samples.len());
    for (offset, sample) in samples.iter_mut().enumerate().take(end).skip(start) {
        let phase = (offset - start) as f32 / SAMPLE_RATE as f32 * hz * std::f32::consts::TAU;
        *sample += phase.sin() * amplitude;
    }
}

fn add_impulse(samples: &mut [f32], start: usize, frames: usize, amplitude: f32) {
    let end = start.saturating_add(frames).min(samples.len());
    for sample in samples.iter_mut().take(end).skip(start) {
        *sample += amplitude;
    }
}
