use super::*;
use crate::source_graph::MeterHint;

fn guided_input(downbeat_seconds: f32, bpm: f32) -> SourceTimingProbeBpmCandidateInput {
    let seconds_per_beat = 60.0 / bpm;
    let mut times = Vec::new();
    let mut strengths = Vec::new();
    for beat in 0..32 {
        times.push(downbeat_seconds + beat as f32 * seconds_per_beat);
        strengths.push(if beat % 4 == 0 { 1.0 } else { 0.45 });
    }
    SourceTimingProbeBpmCandidateInput {
        source_id: "guided-test".into(),
        duration_seconds: downbeat_seconds + 32.0 * seconds_per_beat,
        onset_times_seconds: times,
        onset_strengths: strengths,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
    }
}

#[test]
fn selects_source_anchored_nonzero_phase_at_supplied_tempo() {
    let input = guided_input(0.137, 130.0);
    let result = tempo_guided_timing_hypothesis(&input, 130.0);

    assert_eq!(
        result.evidence.decision,
        TempoGuidedTimingDecision::Selected
    );
    assert!((result.evidence.selected_downbeat_seconds.unwrap() - 0.137).abs() < 0.001);
    let hypothesis = result.hypothesis.expect("selected hypothesis");
    assert_eq!(hypothesis.kind, TimingHypothesisKind::TempoGuided);
    assert!(input.onset_times_seconds.iter().any(|onset| {
        (*onset - result.evidence.selected_phase_anchor_seconds.unwrap()).abs() < 0.000_001
    }));
    assert!(hypothesis.bar_grid.iter().all(|bar| {
        bar.end_seconds <= input.duration_seconds + f32::EPSILON
            && (bar.end_seconds - bar.start_seconds - 4.0 * 60.0 / 130.0).abs() < 0.000_01
    }));
    assert!(
        hypothesis
            .provenance
            .iter()
            .any(|value| { value.starts_with("source_derived_downbeat_seconds:0.137") })
    );
}

#[test]
fn same_source_and_tempo_are_deterministic() {
    let input = guided_input(0.137, 130.0);

    assert_eq!(
        tempo_guided_timing_hypothesis(&input, 130.0),
        tempo_guided_timing_hypothesis(&input, 130.0)
    );
}

#[test]
fn changed_source_phase_changes_selected_grid() {
    let first = tempo_guided_timing_hypothesis(&guided_input(0.091, 140.0), 140.0);
    let second = tempo_guided_timing_hypothesis(&guided_input(0.233, 140.0), 140.0);

    assert_ne!(
        first.evidence.selected_downbeat_seconds,
        second.evidence.selected_downbeat_seconds
    );
    assert_ne!(
        first.hypothesis.unwrap().hypothesis_id,
        second.hypothesis.unwrap().hypothesis_id
    );
}

#[test]
fn flat_bar_accents_fail_as_ambiguous() {
    let mut input = guided_input(0.125, 120.0);
    input.onset_strengths.fill(1.0);

    let result = tempo_guided_timing_hypothesis(&input, 120.0);

    assert_eq!(
        result.evidence.decision,
        TempoGuidedTimingDecision::AmbiguousPhase
    );
    assert!(result.hypothesis.is_none());
}

#[test]
fn sparse_tonal_downbeats_can_select_phase_across_one_third_of_bars() {
    let bpm = 180.0;
    let seconds_per_beat = 60.0 / bpm;
    let downbeat_seconds = 0.083;
    let onset_times_seconds = (0..4)
        .map(|index| downbeat_seconds + index as f32 * 12.0 * seconds_per_beat)
        .flat_map(|downbeat| [downbeat, downbeat + seconds_per_beat])
        .collect::<Vec<_>>();
    let input = SourceTimingProbeBpmCandidateInput {
        source_id: "sparse-tonal".into(),
        duration_seconds: downbeat_seconds + 12.0 * 4.0 * seconds_per_beat,
        onset_strengths: vec![1.0, 0.35, 0.9, 0.3, 1.0, 0.35, 0.9, 0.3],
        onset_times_seconds,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
    };

    let result = tempo_guided_timing_hypothesis(&input, bpm);

    assert_eq!(
        result.evidence.decision,
        TempoGuidedTimingDecision::Selected
    );
    assert!(result.evidence.bar_coverage >= MIN_BAR_COVERAGE);
    assert!((result.evidence.selected_downbeat_seconds.unwrap() - 0.083).abs() < 0.001);
}

#[test]
fn insufficient_onsets_fail_closed() {
    let mut input = guided_input(0.125, 120.0);
    input.onset_times_seconds.truncate(4);
    input.onset_strengths.truncate(4);

    let result = tempo_guided_timing_hypothesis(&input, 120.0);

    assert_eq!(
        result.evidence.decision,
        TempoGuidedTimingDecision::InsufficientOnsets
    );
    assert!(result.hypothesis.is_none());
}

#[test]
fn selection_preserves_analyzer_hypotheses_and_warnings() {
    let input = guided_input(0.137, 130.0);
    let mut timing = TimingModel {
        hypotheses: vec![TimingHypothesis {
            hypothesis_id: "probe-primary".into(),
            kind: TimingHypothesisKind::Primary,
            bpm: 173.0,
            meter: MeterHint {
                beats_per_bar: 4,
                beat_unit: 4,
            },
            confidence: 0.55,
            score: 0.4,
            beat_grid: Vec::new(),
            bar_grid: Vec::new(),
            phrase_grid: Vec::new(),
            anchors: Vec::new(),
            drift: Vec::new(),
            groove: Vec::new(),
            quality: TimingQuality::Medium,
            warnings: Vec::new(),
            provenance: vec!["probe".into()],
        }],
        ..Default::default()
    };

    let evidence = install_tempo_guided_timing(&mut timing, &input, 130.0);

    assert_eq!(evidence.decision, TempoGuidedTimingDecision::Selected);
    assert_eq!(timing.hypotheses.len(), 2);
    assert_eq!(
        timing.primary_hypothesis().unwrap().kind,
        TimingHypothesisKind::TempoGuided
    );
    assert_eq!(
        timing.effective_degraded_policy(),
        TimingDegradedPolicy::Locked
    );
}
