use serde::{Deserialize, Serialize};

use crate::transport::DEFAULT_BARS_PER_PHRASE;

use super::{
    BarSpan, BeatPoint, PhraseSpan, SourceTimingAnchor, SourceTimingAnchorType,
    SourceTimingProbeBpmCandidateInput, TimingDegradedPolicy, TimingDriftReport, TimingHypothesis,
    TimingHypothesisKind, TimingModel, TimingQuality,
};

pub const TEMPO_GUIDED_HYPOTHESIS_ID_PREFIX: &str = "tempo-guided-source-grid-v1";

const MIN_BPM: f32 = 20.0;
const MAX_BPM: f32 = 400.0;
const MIN_ONSET_COUNT: usize = 8;
const MIN_ALIGNED_ONSET_RATIO: f32 = 0.40;
const MIN_ALIGNED_STRENGTH_SHARE: f32 = 0.50;
const MIN_BAR_COVERAGE: f32 = 0.30;
const MIN_PHASE_SCORE: f32 = 0.48;
const MIN_PHASE_SCORE_MARGIN: f32 = 0.020;
const MAX_MEAN_ABS_DRIFT_MS: f32 = 45.0;
const MAX_ABS_DRIFT_MS: f32 = 90.0;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TempoGuidedTimingDecision {
    #[default]
    NotEvaluated,
    Selected,
    InvalidTempo,
    InsufficientMaterial,
    InsufficientOnsets,
    InsufficientGridSupport,
    AmbiguousPhase,
    ExcessiveDrift,
}

impl TempoGuidedTimingDecision {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotEvaluated => "not_evaluated",
            Self::Selected => "selected",
            Self::InvalidTempo => "invalid_tempo",
            Self::InsufficientMaterial => "insufficient_material",
            Self::InsufficientOnsets => "insufficient_onsets",
            Self::InsufficientGridSupport => "insufficient_grid_support",
            Self::AmbiguousPhase => "ambiguous_phase",
            Self::ExcessiveDrift => "excessive_drift",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TempoGuidedTimingEvidence {
    pub decision: TempoGuidedTimingDecision,
    pub requested_bpm: f32,
    pub onset_count: usize,
    pub phase_candidate_count: usize,
    pub selected_downbeat_seconds: Option<f32>,
    pub selected_phase_anchor_seconds: Option<f32>,
    pub aligned_onset_count: usize,
    pub aligned_onset_ratio: f32,
    pub aligned_strength_share: f32,
    pub downbeat_strength_share: f32,
    pub complete_bar_count: usize,
    pub downbeat_bar_hit_count: usize,
    pub bar_coverage: f32,
    pub primary_score: f32,
    pub primary_score_margin: f32,
    pub mean_abs_drift_ms: f32,
    pub max_abs_drift_ms: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TempoGuidedTimingResult {
    pub evidence: TempoGuidedTimingEvidence,
    pub hypothesis: Option<TimingHypothesis>,
}

#[derive(Clone, Copy, Debug)]
struct PhaseCandidate {
    downbeat_seconds: f32,
    anchor_seconds: f32,
    aligned_onset_count: usize,
    aligned_onset_ratio: f32,
    aligned_strength_share: f32,
    downbeat_strength_share: f32,
    complete_bar_count: usize,
    downbeat_bar_hit_count: usize,
    bar_coverage: f32,
    score: f32,
    mean_abs_drift_ms: f32,
    max_abs_drift_ms: f32,
    end_drift_ms: f32,
}

#[must_use]
pub fn tempo_guided_timing_hypothesis(
    input: &SourceTimingProbeBpmCandidateInput,
    requested_bpm: f32,
) -> TempoGuidedTimingResult {
    let mut evidence = TempoGuidedTimingEvidence {
        requested_bpm,
        onset_count: input.onset_times_seconds.len(),
        ..Default::default()
    };
    if !requested_bpm.is_finite() || !(MIN_BPM..=MAX_BPM).contains(&requested_bpm) {
        evidence.decision = TempoGuidedTimingDecision::InvalidTempo;
        return rejected(evidence);
    }

    let seconds_per_beat = 60.0 / requested_bpm;
    let seconds_per_bar = seconds_per_beat * f32::from(input.meter.beats_per_bar.max(1));
    if !input.duration_seconds.is_finite()
        || input.duration_seconds <= 0.0
        || input.duration_seconds + f32::EPSILON < seconds_per_bar
    {
        evidence.decision = TempoGuidedTimingDecision::InsufficientMaterial;
        return rejected(evidence);
    }

    let onsets = normalized_onsets(input);
    evidence.onset_count = onsets.len();
    if onsets.len() < MIN_ONSET_COUNT {
        evidence.decision = TempoGuidedTimingDecision::InsufficientOnsets;
        return rejected(evidence);
    }

    let tolerance_seconds = (seconds_per_beat * 0.18).clamp(0.035, 0.09);
    let phase_merge_tolerance = (seconds_per_beat * 0.08).clamp(0.015, 0.04);
    let phases = unique_source_onset_phases(&onsets, seconds_per_bar, phase_merge_tolerance);
    evidence.phase_candidate_count = phases.len();
    let mut candidates = phases
        .into_iter()
        .filter_map(|(phase, anchor)| {
            score_phase_candidate(
                &onsets,
                input.duration_seconds,
                seconds_per_beat,
                seconds_per_bar,
                tolerance_seconds,
                phase,
                anchor,
            )
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| right.bar_coverage.total_cmp(&left.bar_coverage))
            .then_with(|| left.downbeat_seconds.total_cmp(&right.downbeat_seconds))
    });
    let Some(selected) = candidates.first().copied() else {
        evidence.decision = TempoGuidedTimingDecision::InsufficientGridSupport;
        return rejected(evidence);
    };
    let margin = candidates.get(1).map_or(selected.score, |next| {
        (selected.score - next.score).max(0.0)
    });
    evidence.selected_downbeat_seconds = Some(selected.downbeat_seconds);
    evidence.selected_phase_anchor_seconds = Some(selected.anchor_seconds);
    evidence.aligned_onset_count = selected.aligned_onset_count;
    evidence.aligned_onset_ratio = selected.aligned_onset_ratio;
    evidence.aligned_strength_share = selected.aligned_strength_share;
    evidence.downbeat_strength_share = selected.downbeat_strength_share;
    evidence.complete_bar_count = selected.complete_bar_count;
    evidence.downbeat_bar_hit_count = selected.downbeat_bar_hit_count;
    evidence.bar_coverage = selected.bar_coverage;
    evidence.primary_score = selected.score;
    evidence.primary_score_margin = margin;
    evidence.mean_abs_drift_ms = selected.mean_abs_drift_ms;
    evidence.max_abs_drift_ms = selected.max_abs_drift_ms;

    if selected.aligned_onset_ratio < MIN_ALIGNED_ONSET_RATIO
        || selected.aligned_strength_share < MIN_ALIGNED_STRENGTH_SHARE
        || selected.bar_coverage < MIN_BAR_COVERAGE
        || selected.score < MIN_PHASE_SCORE
    {
        evidence.decision = TempoGuidedTimingDecision::InsufficientGridSupport;
        return rejected(evidence);
    }
    if margin < MIN_PHASE_SCORE_MARGIN {
        evidence.decision = TempoGuidedTimingDecision::AmbiguousPhase;
        return rejected(evidence);
    }
    if selected.mean_abs_drift_ms > MAX_MEAN_ABS_DRIFT_MS
        || selected.max_abs_drift_ms > MAX_ABS_DRIFT_MS
    {
        evidence.decision = TempoGuidedTimingDecision::ExcessiveDrift;
        return rejected(evidence);
    }

    evidence.decision = TempoGuidedTimingDecision::Selected;
    let hypothesis = build_hypothesis(input, requested_bpm, selected);
    TempoGuidedTimingResult {
        evidence,
        hypothesis: Some(hypothesis),
    }
}

pub fn install_tempo_guided_timing(
    timing: &mut TimingModel,
    input: &SourceTimingProbeBpmCandidateInput,
    requested_bpm: f32,
) -> TempoGuidedTimingEvidence {
    let result = tempo_guided_timing_hypothesis(input, requested_bpm);
    if let Some(hypothesis) = result.hypothesis {
        select_tempo_guided_timing(timing, hypothesis);
    }
    result.evidence
}

fn select_tempo_guided_timing(timing: &mut TimingModel, hypothesis: TimingHypothesis) {
    debug_assert_eq!(hypothesis.kind, TimingHypothesisKind::TempoGuided);
    timing
        .hypotheses
        .retain(|candidate| candidate.kind != TimingHypothesisKind::TempoGuided);
    timing.primary_hypothesis_id = Some(hypothesis.hypothesis_id.clone());
    timing.bpm_estimate = Some(hypothesis.bpm);
    timing.bpm_confidence = hypothesis.confidence;
    timing.meter_hint = Some(hypothesis.meter);
    timing.beat_grid = hypothesis.beat_grid.clone();
    timing.bar_grid = hypothesis.bar_grid.clone();
    timing.phrase_grid = hypothesis.phrase_grid.clone();
    timing.quality = TimingQuality::High;
    timing.degraded_policy = TimingDegradedPolicy::Locked;
    timing.hypotheses.push(hypothesis);
}

fn rejected(evidence: TempoGuidedTimingEvidence) -> TempoGuidedTimingResult {
    TempoGuidedTimingResult {
        evidence,
        hypothesis: None,
    }
}

fn normalized_onsets(input: &SourceTimingProbeBpmCandidateInput) -> Vec<(f32, f32)> {
    let mut onsets = input
        .onset_times_seconds
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, time)| {
            if !time.is_finite() || time < 0.0 || time > input.duration_seconds {
                return None;
            }
            let strength = input
                .onset_strengths
                .get(index)
                .copied()
                .filter(|strength| strength.is_finite())
                .unwrap_or(1.0)
                .max(0.0);
            Some((time, strength))
        })
        .collect::<Vec<_>>();
    onsets.sort_by(|left, right| left.0.total_cmp(&right.0));
    onsets.dedup_by(|left, right| (left.0 - right.0).abs() <= f32::EPSILON);
    onsets
}

fn unique_source_onset_phases(
    onsets: &[(f32, f32)],
    seconds_per_bar: f32,
    merge_tolerance: f32,
) -> Vec<(f32, f32)> {
    let mut phases = Vec::<(f32, f32, f32)>::new();
    for (time, strength) in onsets {
        let phase = time.rem_euclid(seconds_per_bar);
        if let Some(existing) = phases.iter_mut().find(|(existing, _, _)| {
            circular_distance(*existing, phase, seconds_per_bar) <= merge_tolerance
        }) {
            if *strength > existing.2 {
                *existing = (phase, *time, *strength);
            }
            continue;
        }
        phases.push((phase, *time, *strength));
    }
    phases
        .into_iter()
        .map(|(phase, anchor, _)| (phase, anchor))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn score_phase_candidate(
    onsets: &[(f32, f32)],
    duration_seconds: f32,
    seconds_per_beat: f32,
    seconds_per_bar: f32,
    tolerance_seconds: f32,
    downbeat_seconds: f32,
    anchor_seconds: f32,
) -> Option<PhaseCandidate> {
    let total_strength = onsets
        .iter()
        .map(|(_, strength)| *strength)
        .sum::<f32>()
        .max(f32::EPSILON);
    let complete_bar_count =
        ((duration_seconds - downbeat_seconds).max(0.0) / seconds_per_bar).floor() as usize;
    if complete_bar_count == 0 {
        return None;
    }
    let mut aligned_count = 0_usize;
    let mut aligned_strength = 0.0_f32;
    let mut downbeat_strength = 0.0_f32;
    let mut bar_hits = vec![false; complete_bar_count];
    let mut residuals = Vec::new();
    for (time, strength) in onsets {
        let beat_residual = signed_distance_to_phase(*time, downbeat_seconds, seconds_per_beat);
        if beat_residual.abs() <= tolerance_seconds {
            aligned_count += 1;
            aligned_strength += *strength;
            residuals.push(beat_residual);
        }
        let bar_residual = signed_distance_to_phase(*time, downbeat_seconds, seconds_per_bar);
        if bar_residual.abs() <= tolerance_seconds {
            downbeat_strength += *strength;
            let bar_index = ((*time - downbeat_seconds) / seconds_per_bar).round() as isize;
            if let Ok(bar_index) = usize::try_from(bar_index)
                && let Some(hit) = bar_hits.get_mut(bar_index)
            {
                *hit = true;
            }
        }
    }
    if aligned_count == 0 {
        return None;
    }
    let aligned_onset_ratio = aligned_count as f32 / onsets.len() as f32;
    let aligned_strength_share = aligned_strength / total_strength;
    let downbeat_strength_share = downbeat_strength / total_strength;
    let downbeat_bar_hit_count = bar_hits.iter().filter(|hit| **hit).count();
    let bar_coverage = downbeat_bar_hit_count as f32 / complete_bar_count as f32;
    let mean_abs_drift_ms =
        residuals.iter().map(|value| value.abs()).sum::<f32>() / residuals.len() as f32 * 1_000.0;
    let max_abs_drift_ms = residuals
        .iter()
        .map(|value| value.abs())
        .fold(0.0, f32::max)
        * 1_000.0;
    let end_drift_ms = residuals.last().copied().unwrap_or(0.0) * 1_000.0;
    let score = aligned_strength_share * 0.45
        + aligned_onset_ratio * 0.20
        + bar_coverage * 0.25
        + downbeat_strength_share * 0.10;
    Some(PhaseCandidate {
        downbeat_seconds,
        anchor_seconds,
        aligned_onset_count: aligned_count,
        aligned_onset_ratio,
        aligned_strength_share,
        downbeat_strength_share,
        complete_bar_count,
        downbeat_bar_hit_count,
        bar_coverage,
        score,
        mean_abs_drift_ms,
        max_abs_drift_ms,
        end_drift_ms,
    })
}

fn build_hypothesis(
    input: &SourceTimingProbeBpmCandidateInput,
    bpm: f32,
    selected: PhaseCandidate,
) -> TimingHypothesis {
    let confidence = (0.55 + selected.score * 0.45).clamp(0.0, 1.0);
    let seconds_per_beat = 60.0 / bpm;
    let seconds_per_bar = seconds_per_beat * f32::from(input.meter.beats_per_bar.max(1));
    let beat_grid = beat_grid(
        input.duration_seconds,
        selected.downbeat_seconds,
        seconds_per_beat,
        confidence,
    );
    let bar_grid = bar_grid(
        input.duration_seconds,
        selected.downbeat_seconds,
        seconds_per_bar,
        confidence,
        selected.score,
    );
    let phrase_grid = phrase_grid(&bar_grid, confidence);
    let drift = vec![TimingDriftReport {
        window_bars: u32::try_from(bar_grid.len()).unwrap_or(u32::MAX),
        max_drift_ms: selected.max_abs_drift_ms,
        mean_abs_drift_ms: selected.mean_abs_drift_ms,
        end_drift_ms: selected.end_drift_ms,
        confidence,
    }];
    let anchors = source_anchors(
        input,
        selected.downbeat_seconds,
        seconds_per_beat,
        seconds_per_bar,
        confidence,
    );
    TimingHypothesis {
        hypothesis_id: tempo_guided_hypothesis_id(
            bpm,
            selected.downbeat_seconds,
            selected.anchor_seconds,
        ),
        kind: TimingHypothesisKind::TempoGuided,
        bpm,
        meter: input.meter,
        confidence,
        score: selected.score,
        beat_grid,
        bar_grid,
        phrase_grid,
        anchors,
        drift,
        groove: Vec::new(),
        quality: TimingQuality::High,
        warnings: Vec::new(),
        provenance: vec![
            "source-timing-probe.tempo-guided-phase.v1".into(),
            format!("source:{}", input.source_id),
            format!("externally_supplied_bpm:{bpm:.6}"),
            format!(
                "source_derived_downbeat_seconds:{:.6}",
                selected.downbeat_seconds
            ),
            format!("source_phase_anchor_seconds:{:.6}", selected.anchor_seconds),
        ],
    }
}

fn tempo_guided_hypothesis_id(bpm: f32, downbeat_seconds: f32, anchor_seconds: f32) -> String {
    format!(
        "{TEMPO_GUIDED_HYPOTHESIS_ID_PREFIX}-{:08x}-{:08x}-{:08x}",
        bpm.to_bits(),
        downbeat_seconds.to_bits(),
        anchor_seconds.to_bits()
    )
}

fn beat_grid(
    duration_seconds: f32,
    start_seconds: f32,
    seconds_per_beat: f32,
    confidence: f32,
) -> Vec<BeatPoint> {
    let mut grid = Vec::new();
    let mut time = start_seconds;
    while time <= duration_seconds {
        grid.push(BeatPoint {
            beat_index: u32::try_from(grid.len() + 1).unwrap_or(u32::MAX),
            time_seconds: time,
            confidence,
        });
        time += seconds_per_beat;
    }
    grid
}

fn bar_grid(
    duration_seconds: f32,
    start_seconds: f32,
    seconds_per_bar: f32,
    confidence: f32,
    phase_score: f32,
) -> Vec<BarSpan> {
    let mut grid = Vec::new();
    let mut time = start_seconds;
    while time + seconds_per_bar <= duration_seconds + f32::EPSILON {
        grid.push(BarSpan {
            bar_index: u32::try_from(grid.len() + 1).unwrap_or(u32::MAX),
            start_seconds: time,
            end_seconds: time + seconds_per_bar,
            downbeat_confidence: confidence * phase_score.clamp(0.0, 1.0),
            phrase_index: None,
        });
        time += seconds_per_bar;
    }
    grid
}

fn phrase_grid(bars: &[BarSpan], confidence: f32) -> Vec<PhraseSpan> {
    let bars_per_phrase = u32::try_from(DEFAULT_BARS_PER_PHRASE).unwrap_or(4).max(1);
    let bar_count = u32::try_from(bars.len()).unwrap_or(u32::MAX);
    (0..(bar_count / bars_per_phrase))
        .map(|phrase| PhraseSpan {
            phrase_index: phrase + 1,
            start_bar: phrase * bars_per_phrase + 1,
            end_bar: (phrase + 1) * bars_per_phrase,
            confidence,
        })
        .collect()
}

fn source_anchors(
    input: &SourceTimingProbeBpmCandidateInput,
    downbeat_seconds: f32,
    seconds_per_beat: f32,
    seconds_per_bar: f32,
    confidence: f32,
) -> Vec<SourceTimingAnchor> {
    let tolerance = (seconds_per_beat * 0.18).clamp(0.035, 0.09);
    normalized_onsets(input)
        .into_iter()
        .take(16)
        .enumerate()
        .map(|(index, (time, strength))| {
            let beat_residual = signed_distance_to_phase(time, downbeat_seconds, seconds_per_beat);
            let bar_residual = signed_distance_to_phase(time, downbeat_seconds, seconds_per_bar);
            let aligned = beat_residual.abs() <= tolerance;
            let downbeat = bar_residual.abs() <= tolerance;
            let beat_index = aligned.then(|| {
                ((time - downbeat_seconds).max(0.0) / seconds_per_beat).round() as u32 + 1
            });
            let bar_index = downbeat
                .then(|| ((time - downbeat_seconds).max(0.0) / seconds_per_bar).round() as u32 + 1);
            let mut tags = vec!["tempo_guided".into(), "source_phase_evidence".into()];
            if aligned {
                tags.push("grid_aligned".into());
            }
            if downbeat {
                tags.push("downbeat".into());
            }
            SourceTimingAnchor {
                anchor_id: format!("{}:tempo-guided-onset-{}", input.source_id, index + 1),
                anchor_type: SourceTimingAnchorType::TransientCluster,
                time_seconds: time,
                bar_index,
                beat_index,
                confidence,
                strength,
                tags,
            }
        })
        .collect()
}

fn signed_distance_to_phase(time: f32, phase: f32, period: f32) -> f32 {
    let position = (time - phase).rem_euclid(period);
    if position <= period * 0.5 {
        position
    } else {
        position - period
    }
}

fn circular_distance(left: f32, right: f32, period: f32) -> f32 {
    let distance = (left - right).abs().rem_euclid(period);
    distance.min(period - distance)
}

#[cfg(test)]
mod tests;
