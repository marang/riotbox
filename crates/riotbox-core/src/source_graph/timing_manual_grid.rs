use crate::transport::DEFAULT_BARS_PER_PHRASE;

use super::{
    BarSpan, BeatPoint, MeterHint, PhraseSpan, SourceGraph, TimingDegradedPolicy, TimingHypothesis,
    TimingHypothesisKind, TimingQuality, TimingWarning, TimingWarningCode,
};

pub const MANUAL_SOURCE_GRID_HYPOTHESIS_ID_PREFIX: &str = "manual-source-grid-v1";
const MIN_MANUAL_BPM: f32 = 20.0;
const MAX_MANUAL_BPM: f32 = 400.0;
const GRID_EPSILON_SECONDS: f32 = 1.0e-4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ManualSourceTimingGrid {
    pub bpm: f32,
    /// Seconds from source start to the first musician-declared downbeat.
    pub downbeat_seconds: f32,
}

/// Installs a musician-declared timing hypothesis while retaining analyzer
/// hypotheses and warnings as separate evidence.
///
/// The selected top-level grids are compatibility projections of the manual
/// hypothesis. The typed `Manual` kind and provenance prevent them from being
/// presented as analyzer-derived timing.
pub fn install_manual_source_timing_grid(
    graph: &mut SourceGraph,
    input: ManualSourceTimingGrid,
) -> Result<(), String> {
    validate_manual_grid_input(graph.source.duration_seconds, input)?;

    let meter = graph.timing.meter_hint.unwrap_or(MeterHint {
        beats_per_bar: 4,
        beat_unit: 4,
    });
    if meter.beats_per_bar == 0 || meter.beat_unit == 0 {
        return Err("manual source grid requires a non-zero meter".into());
    }

    let seconds_per_beat = 60.0 / input.bpm;
    let seconds_per_bar = seconds_per_beat * f32::from(meter.beats_per_bar);
    if input.downbeat_seconds + seconds_per_bar
        > graph.source.duration_seconds + GRID_EPSILON_SECONDS
    {
        return Err(format!(
            "manual source grid needs one full bar after downbeat {:.3}s at {:.3} BPM; source duration is {:.3}s",
            input.downbeat_seconds, input.bpm, graph.source.duration_seconds
        ));
    }

    let beat_grid = manual_beat_grid(
        graph.source.duration_seconds,
        input.downbeat_seconds,
        seconds_per_beat,
    );
    let bar_grid = manual_bar_grid(
        graph.source.duration_seconds,
        input.downbeat_seconds,
        seconds_per_bar,
    );
    let phrase_grid = manual_phrase_grid(&bar_grid, seconds_per_bar);
    let warnings = phrase_grid.is_empty().then(|| TimingWarning {
        code: TimingWarningCode::PhraseUncertain,
        message: "manual grid has insufficient source material for phrase evidence".into(),
    });
    let hypothesis_id = manual_source_grid_hypothesis_id(input);
    let hypothesis = TimingHypothesis {
        hypothesis_id: hypothesis_id.clone(),
        kind: TimingHypothesisKind::Manual,
        bpm: input.bpm,
        meter,
        confidence: 1.0,
        score: 0.0,
        beat_grid: beat_grid.clone(),
        bar_grid: bar_grid.clone(),
        phrase_grid: phrase_grid.clone(),
        anchors: Vec::new(),
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::Low,
        warnings: warnings.into_iter().collect(),
        provenance: vec![
            "musician-manual-source-grid.v1".into(),
            format!("source:{}", graph.source.source_id),
            format!("declared_bpm:{:.6}", input.bpm),
            format!("declared_downbeat_seconds:{:.6}", input.downbeat_seconds),
        ],
    };

    graph
        .timing
        .hypotheses
        .retain(|candidate| candidate.kind != TimingHypothesisKind::Manual);
    graph.timing.hypotheses.push(hypothesis);
    graph.timing.primary_hypothesis_id = Some(hypothesis_id);
    graph.timing.bpm_estimate = Some(input.bpm);
    graph.timing.meter_hint = Some(meter);
    graph.timing.beat_grid = beat_grid;
    graph.timing.bar_grid = bar_grid;
    graph.timing.phrase_grid = phrase_grid;
    graph.timing.degraded_policy = TimingDegradedPolicy::ManualConfirm;
    Ok(())
}

#[must_use]
pub fn manual_source_grid_hypothesis_id(input: ManualSourceTimingGrid) -> String {
    format!(
        "{MANUAL_SOURCE_GRID_HYPOTHESIS_ID_PREFIX}-{:08x}-{:08x}",
        input.bpm.to_bits(),
        input.downbeat_seconds.to_bits()
    )
}

fn validate_manual_grid_input(
    duration_seconds: f32,
    input: ManualSourceTimingGrid,
) -> Result<(), String> {
    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return Err(format!(
            "manual source grid requires a finite positive source duration, got {duration_seconds}"
        ));
    }
    if !input.bpm.is_finite() || !(MIN_MANUAL_BPM..=MAX_MANUAL_BPM).contains(&input.bpm) {
        return Err(format!(
            "manual source grid BPM must be finite and inside {MIN_MANUAL_BPM}..={MAX_MANUAL_BPM}, got {}",
            input.bpm
        ));
    }
    if !input.downbeat_seconds.is_finite()
        || input.downbeat_seconds < 0.0
        || input.downbeat_seconds >= duration_seconds
    {
        return Err(format!(
            "manual source grid downbeat must be finite and inside 0..{duration_seconds}, got {}",
            input.downbeat_seconds
        ));
    }
    Ok(())
}

fn manual_beat_grid(
    duration_seconds: f32,
    downbeat_seconds: f32,
    seconds_per_beat: f32,
) -> Vec<BeatPoint> {
    let mut beats = Vec::new();
    let mut time_seconds = downbeat_seconds;
    while time_seconds <= duration_seconds + GRID_EPSILON_SECONDS {
        beats.push(BeatPoint {
            beat_index: u32::try_from(beats.len() + 1).unwrap_or(u32::MAX),
            time_seconds: time_seconds.min(duration_seconds),
            confidence: 1.0,
        });
        time_seconds += seconds_per_beat;
    }
    beats
}

fn manual_bar_grid(
    duration_seconds: f32,
    downbeat_seconds: f32,
    seconds_per_bar: f32,
) -> Vec<BarSpan> {
    let mut bars = Vec::new();
    let mut start_seconds = downbeat_seconds;
    while start_seconds < duration_seconds - GRID_EPSILON_SECONDS {
        let bar_index = u32::try_from(bars.len() + 1).unwrap_or(u32::MAX);
        bars.push(BarSpan {
            bar_index,
            start_seconds,
            end_seconds: (start_seconds + seconds_per_bar).min(duration_seconds),
            downbeat_confidence: 1.0,
            phrase_index: Some((bar_index - 1) / DEFAULT_BARS_PER_PHRASE as u32 + 1),
        });
        start_seconds += seconds_per_bar;
    }
    bars
}

fn manual_phrase_grid(bars: &[BarSpan], seconds_per_bar: f32) -> Vec<PhraseSpan> {
    let full_bars = bars
        .iter()
        .take_while(|bar| {
            (bar.end_seconds - bar.start_seconds) + GRID_EPSILON_SECONDS >= seconds_per_bar
        })
        .copied()
        .collect::<Vec<_>>();
    full_bars
        .as_chunks::<{ DEFAULT_BARS_PER_PHRASE as usize }>()
        .0
        .iter()
        .enumerate()
        .map(|(index, chunk)| PhraseSpan {
            phrase_index: u32::try_from(index + 1).unwrap_or(u32::MAX),
            start_bar: chunk.first().map_or(1, |bar| bar.bar_index),
            end_bar: chunk.last().map_or(1, |bar| bar.bar_index),
            confidence: 1.0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_graph::{DecodeProfile, GraphProvenance, SourceDescriptor};

    fn graph(duration_seconds: f32) -> SourceGraph {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: "src-tonal".into(),
                path: "tonal.wav".into(),
                content_hash: "sha256:tonal".into(),
                duration_seconds,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::Native,
            },
            GraphProvenance {
                sidecar_version: "test".into(),
                provider_set: vec!["test".into()],
                generated_at: "2026-07-21T00:00:00Z".into(),
                source_hash: "sha256:tonal".into(),
                analysis_seed: 1,
                run_notes: None,
            },
        );
        graph.timing.warnings.push(TimingWarning {
            code: TimingWarningCode::SparseOnsets,
            message: "analyzer evidence".into(),
        });
        graph
    }

    #[test]
    fn installs_deterministic_manual_grid_without_erasing_analyzer_warnings() {
        let mut graph = graph(4.0);
        install_manual_source_timing_grid(
            &mut graph,
            ManualSourceTimingGrid {
                bpm: 120.0,
                downbeat_seconds: 0.0,
            },
        )
        .expect("manual grid");

        let primary = graph.timing.primary_hypothesis().expect("manual primary");
        assert_eq!(primary.kind, TimingHypothesisKind::Manual);
        assert_eq!(primary.beat_grid.len(), 9);
        assert_eq!(primary.bar_grid.len(), 2);
        assert_eq!(primary.transport_bar_grid_anchor().unwrap().beat_cursor, 0);
        assert_eq!(
            graph.timing.degraded_policy,
            TimingDegradedPolicy::ManualConfirm
        );
        assert!(
            graph
                .timing
                .warnings
                .iter()
                .any(|warning| { warning.code == TimingWarningCode::SparseOnsets })
        );
        assert!(
            primary
                .provenance
                .contains(&"musician-manual-source-grid.v1".into())
        );
    }

    #[test]
    fn declared_phase_becomes_the_transport_bar_anchor() {
        let mut graph = graph(5.0);
        install_manual_source_timing_grid(
            &mut graph,
            ManualSourceTimingGrid {
                bpm: 120.0,
                downbeat_seconds: 0.5,
            },
        )
        .expect("manual phase grid");

        let primary = graph.timing.primary_hypothesis().expect("manual primary");
        assert_eq!(primary.beat_grid[0].time_seconds, 0.5);
        assert_eq!(primary.bar_grid[0].start_seconds, 0.5);
        assert_eq!(primary.transport_bar_grid_anchor().unwrap().beat_cursor, 0);
    }

    #[test]
    fn rejects_invalid_bpm_phase_and_too_short_source_window() {
        for (duration, bpm, phase) in [
            (4.0, 0.0, 0.0),
            (4.0, 401.0, 0.0),
            (4.0, 120.0, -0.1),
            (4.0, 120.0, 4.0),
            (1.5, 120.0, 0.0),
        ] {
            let error = install_manual_source_timing_grid(
                &mut graph(duration),
                ManualSourceTimingGrid {
                    bpm,
                    downbeat_seconds: phase,
                },
            )
            .expect_err("invalid manual grid");
            assert!(!error.is_empty());
        }
    }

    #[test]
    fn partial_fourth_bar_does_not_claim_phrase_evidence() {
        let mut graph = graph(7.0);
        install_manual_source_timing_grid(
            &mut graph,
            ManualSourceTimingGrid {
                bpm: 120.0,
                downbeat_seconds: 0.0,
            },
        )
        .expect("manual grid with partial fourth bar");

        let primary = graph.timing.primary_hypothesis().expect("manual primary");
        assert_eq!(primary.bar_grid.len(), 4);
        assert!(primary.phrase_grid.is_empty());
    }

    #[test]
    fn hypothesis_identity_changes_with_declared_bpm_or_phase() {
        let base = manual_source_grid_hypothesis_id(ManualSourceTimingGrid {
            bpm: 120.0,
            downbeat_seconds: 0.0,
        });
        assert_eq!(
            base,
            manual_source_grid_hypothesis_id(ManualSourceTimingGrid {
                bpm: 120.0,
                downbeat_seconds: 0.0,
            })
        );
        assert_ne!(
            base,
            manual_source_grid_hypothesis_id(ManualSourceTimingGrid {
                bpm: 120.0,
                downbeat_seconds: 0.25,
            })
        );
        assert_ne!(
            base,
            manual_source_grid_hypothesis_id(ManualSourceTimingGrid {
                bpm: 121.0,
                downbeat_seconds: 0.0,
            })
        );
    }
}
