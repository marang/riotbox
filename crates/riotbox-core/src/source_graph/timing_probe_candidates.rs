#[derive(Clone, Debug, PartialEq)]
pub struct SourceTimingProbeBpmCandidateInput {
    pub source_id: String,
    pub duration_seconds: f32,
    pub onset_times_seconds: Vec<f32>,
    pub meter: MeterHint,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceTimingProbeBpmCandidatePolicy {
    pub min_onset_count: usize,
    pub min_bpm: f32,
    pub max_bpm: f32,
    pub primary_confidence: Confidence,
    pub alternative_confidence: Confidence,
    pub downbeat_ambiguity_margin: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceTimingCandidateConfidenceReport {
    pub schema: &'static str,
    pub schema_version: u32,
    pub primary_bpm: Option<f32>,
    pub bpm_confidence: Confidence,
    pub timing_quality: TimingQuality,
    pub degraded_policy: TimingDegradedPolicy,
    pub hypothesis_count: usize,
    pub alternate_downbeat_count: usize,
    pub half_time_count: usize,
    pub double_time_count: usize,
    pub primary_downbeat_confidence: Option<Confidence>,
    pub warning_codes: Vec<TimingWarningCode>,
    pub requires_manual_confirm: bool,
    pub result: SourceTimingCandidateConfidenceResult,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceTimingCandidateConfidenceResult {
    Degraded,
    CandidateCautious,
    CandidateAmbiguous,
}

impl Default for SourceTimingProbeBpmCandidatePolicy {
    fn default() -> Self {
        Self {
            min_onset_count: 4,
            min_bpm: 55.0,
            max_bpm: 240.0,
            primary_confidence: 0.55,
            alternative_confidence: 0.35,
            downbeat_ambiguity_margin: 0.05,
        }
    }
}

#[must_use]
pub fn source_timing_candidate_confidence_report(
    timing: &TimingModel,
) -> SourceTimingCandidateConfidenceReport {
    let alternate_downbeat_count = count_hypotheses(timing, TimingHypothesisKind::AlternateDownbeat);
    let half_time_count = count_hypotheses(timing, TimingHypothesisKind::HalfTime);
    let double_time_count = count_hypotheses(timing, TimingHypothesisKind::DoubleTime);
    let warning_codes = timing
        .warnings
        .iter()
        .map(|warning| warning.code)
        .collect::<Vec<_>>();
    let primary_downbeat_confidence = timing
        .primary_hypothesis()
        .and_then(|hypothesis| hypothesis.bar_grid.first())
        .map(|bar| bar.downbeat_confidence);
    let degraded_policy = timing.effective_degraded_policy();
    let requires_manual_confirm = degraded_policy != TimingDegradedPolicy::Locked
        || !warning_codes.is_empty()
        || alternate_downbeat_count > 0
        || half_time_count > 0
        || double_time_count > 0;
    let result = if timing.bpm_estimate.is_none() || timing.primary_hypothesis().is_none() {
        SourceTimingCandidateConfidenceResult::Degraded
    } else if alternate_downbeat_count > 0
        || half_time_count > 0
        || double_time_count > 0
        || warning_codes.contains(&TimingWarningCode::AmbiguousDownbeat)
    {
        SourceTimingCandidateConfidenceResult::CandidateAmbiguous
    } else {
        SourceTimingCandidateConfidenceResult::CandidateCautious
    };

    SourceTimingCandidateConfidenceReport {
        schema: "riotbox.source_timing_candidate_confidence.v1",
        schema_version: 1,
        primary_bpm: timing.bpm_estimate,
        bpm_confidence: timing.bpm_confidence,
        timing_quality: timing.effective_timing_quality(),
        degraded_policy,
        hypothesis_count: timing.hypotheses.len(),
        alternate_downbeat_count,
        half_time_count,
        double_time_count,
        primary_downbeat_confidence,
        warning_codes,
        requires_manual_confirm,
        result,
    }
}

#[must_use]
pub fn timing_model_from_probe_bpm_candidates(
    input: &SourceTimingProbeBpmCandidateInput,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> TimingModel {
    let Some(primary_bpm) = primary_bpm_candidate(input, policy) else {
        return timing_model_from_probe_diagnostics(
            &SourceTimingProbeDiagnosticInput {
                source_id: input.source_id.clone(),
                duration_seconds: input.duration_seconds,
                peak_energy: 0.0,
                peak_positive_flux: 0.0,
                onset_count: input.onset_times_seconds.len(),
                onset_density_per_second: onset_density(
                    input.onset_times_seconds.len(),
                    input.duration_seconds,
                ),
            },
            SourceTimingProbeDiagnosticPolicy::default(),
        );
    };

    let downbeat_phases = downbeat_phase_scores(input, primary_bpm);
    let primary_phase = downbeat_phases.first().copied().unwrap_or_default();
    let primary = probe_bpm_hypothesis(
        "probe-bpm-primary".into(),
        TimingHypothesisKind::Primary,
        primary_bpm,
        policy.primary_confidence,
        primary_phase.offset_beats,
        primary_phase.score,
        input,
    );
    let mut hypotheses = vec![primary];
    let mut warnings = vec![
        TimingWarningCode::AmbiguousDownbeat,
        TimingWarningCode::PhraseUncertain,
    ];

    for phase in ambiguous_downbeat_phases(&downbeat_phases, policy) {
        hypotheses.push(probe_bpm_hypothesis(
            format!("probe-bpm-alt-downbeat-{}", phase.offset_beats + 1),
            TimingHypothesisKind::AlternateDownbeat,
            primary_bpm,
            policy.alternative_confidence,
            phase.offset_beats,
            phase.score,
            input,
        ));
    }

    if primary_bpm / 2.0 >= policy.min_bpm {
        let half_bpm = primary_bpm / 2.0;
        let half_phase = best_downbeat_phase(input, half_bpm);
        hypotheses.push(probe_bpm_hypothesis(
            "probe-bpm-half-time".into(),
            TimingHypothesisKind::HalfTime,
            half_bpm,
            policy.alternative_confidence,
            half_phase.offset_beats,
            half_phase.score,
            input,
        ));
        warnings.push(TimingWarningCode::HalfTimePossible);
    }
    if primary_bpm * 2.0 <= policy.max_bpm {
        let double_bpm = primary_bpm * 2.0;
        let double_phase = best_downbeat_phase(input, double_bpm);
        hypotheses.push(probe_bpm_hypothesis(
            "probe-bpm-double-time".into(),
            TimingHypothesisKind::DoubleTime,
            double_bpm,
            policy.alternative_confidence,
            double_phase.offset_beats,
            double_phase.score,
            input,
        ));
        warnings.push(TimingWarningCode::DoubleTimePossible);
    }

    TimingModel {
        bpm_estimate: Some(primary_bpm),
        bpm_confidence: policy.primary_confidence,
        meter_hint: Some(input.meter),
        beat_grid: hypotheses[0].beat_grid.clone(),
        bar_grid: hypotheses[0].bar_grid.clone(),
        phrase_grid: Vec::new(),
        hypotheses,
        primary_hypothesis_id: Some("probe-bpm-primary".into()),
        quality: TimingQuality::Medium,
        warnings: warnings
            .into_iter()
            .map(|code| TimingWarning {
                code,
                message: probe_bpm_warning_message(code, input).into(),
            })
            .collect(),
        degraded_policy: TimingDegradedPolicy::Cautious,
    }
}

fn count_hypotheses(timing: &TimingModel, kind: TimingHypothesisKind) -> usize {
    timing
        .hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.kind == kind)
        .count()
}

fn primary_bpm_candidate(
    input: &SourceTimingProbeBpmCandidateInput,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> Option<f32> {
    if input.duration_seconds <= 0.0 {
        return None;
    }

    let onset_times = normalized_onset_times(input);
    if onset_times.len() < policy.min_onset_count {
        return None;
    }

    let mut deltas = onset_times
        .windows(2)
        .filter_map(|times| {
            let delta = times[1] - times[0];
            (delta.is_finite() && delta >= 0.05).then_some(delta)
        })
        .collect::<Vec<_>>();
    if deltas.is_empty() {
        return None;
    }
    deltas.sort_by(f32::total_cmp);
    let median_delta = deltas[deltas.len() / 2];
    let mut bpm = 60.0 / median_delta;
    while bpm < policy.min_bpm {
        bpm *= 2.0;
    }
    while bpm > policy.max_bpm {
        bpm /= 2.0;
    }

    (bpm >= policy.min_bpm && bpm <= policy.max_bpm).then_some(bpm)
}

fn probe_bpm_hypothesis(
    hypothesis_id: String,
    kind: TimingHypothesisKind,
    bpm: f32,
    confidence: Confidence,
    downbeat_offset_beats: u8,
    downbeat_score: f32,
    input: &SourceTimingProbeBpmCandidateInput,
) -> TimingHypothesis {
    TimingHypothesis {
        hypothesis_id,
        kind,
        bpm,
        meter: input.meter,
        confidence,
        score: confidence * downbeat_score.max(0.0),
        beat_grid: probe_candidate_beat_grid(input.duration_seconds, bpm, confidence),
        bar_grid: probe_candidate_bar_grid(
            input.duration_seconds,
            bpm,
            confidence,
            input.meter,
            downbeat_offset_beats,
            downbeat_score,
        ),
        phrase_grid: Vec::new(),
        anchors: normalized_onset_times(input)
            .into_iter()
            .take(16)
            .enumerate()
            .map(|(index, time_seconds)| SourceTimingAnchor {
                anchor_id: format!("{}:probe-onset-{}", input.source_id, index + 1),
                anchor_type: SourceTimingAnchorType::TransientCluster,
                time_seconds,
                bar_index: None,
                beat_index: None,
                confidence,
                strength: confidence,
                tags: vec![
                    "probe_onset".into(),
                    "bpm_candidate".into(),
                    format!("downbeat_phase_{}", downbeat_offset_beats + 1),
                ],
            })
            .collect(),
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::Medium,
        warnings: Vec::new(),
        provenance: vec!["source-timing-probe.bpm-candidate".into(), input.source_id.clone()],
    }
}

fn normalized_onset_times(input: &SourceTimingProbeBpmCandidateInput) -> Vec<f32> {
    let max_time = input.duration_seconds.max(0.0);
    let mut onset_times = input
        .onset_times_seconds
        .iter()
        .copied()
        .filter(|time_seconds| {
            time_seconds.is_finite() && *time_seconds >= 0.0 && *time_seconds <= max_time
        })
        .collect::<Vec<_>>();
    onset_times.sort_by(f32::total_cmp);
    onset_times
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct DownbeatPhaseScore {
    offset_beats: u8,
    score: f32,
}

fn downbeat_phase_scores(input: &SourceTimingProbeBpmCandidateInput, bpm: f32) -> Vec<DownbeatPhaseScore> {
    let onset_times = normalized_onset_times(input);
    let beats_per_bar = input.meter.beats_per_bar.max(1);
    let seconds_per_beat = 60.0 / bpm.max(1.0);
    let seconds_per_bar = seconds_per_beat * f32::from(beats_per_bar);
    if onset_times.is_empty() || seconds_per_bar <= 0.0 {
        return vec![DownbeatPhaseScore::default()];
    }

    let tolerance_seconds = (seconds_per_beat * 0.2).clamp(0.02, 0.08);
    let mut scores = (0..beats_per_bar)
        .map(|offset_beats| {
            let phase_seconds = f32::from(offset_beats) * seconds_per_beat;
            let matching_onsets = onset_times
                .iter()
                .filter(|time_seconds| {
                    distance_to_repeating_phase(**time_seconds, phase_seconds, seconds_per_bar)
                        <= tolerance_seconds
                })
                .count();
            DownbeatPhaseScore {
                offset_beats,
                score: matching_onsets as f32 / onset_times.len() as f32,
            }
        })
        .collect::<Vec<_>>();
    scores.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.offset_beats.cmp(&right.offset_beats))
    });
    scores
}

fn best_downbeat_phase(input: &SourceTimingProbeBpmCandidateInput, bpm: f32) -> DownbeatPhaseScore {
    downbeat_phase_scores(input, bpm)
        .first()
        .copied()
        .unwrap_or_default()
}

fn ambiguous_downbeat_phases(
    phases: &[DownbeatPhaseScore],
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> impl Iterator<Item = DownbeatPhaseScore> + '_ {
    let best_score = phases.first().map_or(0.0, |phase| phase.score);
    phases.iter().copied().skip(1).filter(move |phase| {
        phase.score > 0.0 && best_score - phase.score <= policy.downbeat_ambiguity_margin
    })
}

fn distance_to_repeating_phase(time_seconds: f32, phase_seconds: f32, period_seconds: f32) -> f32 {
    let position = (time_seconds - phase_seconds).rem_euclid(period_seconds);
    position.min(period_seconds - position)
}

fn probe_candidate_beat_grid(
    duration_seconds: f32,
    bpm: f32,
    confidence: Confidence,
) -> Vec<BeatPoint> {
    let seconds_per_beat = 60.0 / bpm.max(1.0);
    let mut beat_grid = Vec::new();
    let mut time_seconds = 0.0_f32;
    while time_seconds <= duration_seconds.max(0.0) {
        beat_grid.push(BeatPoint {
            beat_index: u32::try_from(beat_grid.len() + 1).unwrap_or(u32::MAX),
            time_seconds,
            confidence,
        });
        time_seconds += seconds_per_beat;
    }
    beat_grid
}

fn probe_candidate_bar_grid(
    duration_seconds: f32,
    bpm: f32,
    confidence: Confidence,
    meter: MeterHint,
    downbeat_offset_beats: u8,
    downbeat_score: f32,
) -> Vec<BarSpan> {
    let seconds_per_beat = 60.0 / bpm.max(1.0);
    let seconds_per_bar = seconds_per_beat * f32::from(meter.beats_per_bar.max(1));
    let mut bar_grid = Vec::new();
    let mut start_seconds = f32::from(downbeat_offset_beats) * seconds_per_beat;
    while start_seconds < duration_seconds.max(0.0) {
        bar_grid.push(BarSpan {
            bar_index: u32::try_from(bar_grid.len() + 1).unwrap_or(u32::MAX),
            start_seconds,
            end_seconds: (start_seconds + seconds_per_bar).min(duration_seconds.max(0.0)),
            downbeat_confidence: confidence * downbeat_score.clamp(0.0, 1.0),
            phrase_index: None,
        });
        start_seconds += seconds_per_bar;
    }
    bar_grid
}

fn probe_bpm_warning_message(
    code: TimingWarningCode,
    input: &SourceTimingProbeBpmCandidateInput,
) -> &'static str {
    match code {
        TimingWarningCode::AmbiguousDownbeat => "BPM candidate has no downbeat scoring yet",
        TimingWarningCode::PhraseUncertain => "BPM candidate has no phrase boundary scoring yet",
        TimingWarningCode::HalfTimePossible => "half-time BPM candidate preserved",
        TimingWarningCode::DoubleTimePossible => "double-time BPM candidate preserved",
        TimingWarningCode::LowTimingConfidence => "BPM candidate confidence is low",
        TimingWarningCode::WeakKickAnchor => "BPM candidate has no trusted kick anchor yet",
        TimingWarningCode::WeakBackbeatAnchor => "BPM candidate has no trusted backbeat anchor yet",
        TimingWarningCode::DriftHigh => {
            let _ = input;
            "BPM candidate has no drift model yet"
        }
    }
}

fn onset_density(onset_count: usize, duration_seconds: f32) -> f32 {
    if duration_seconds <= 0.0 {
        return 0.0;
    }
    onset_count as f32 / duration_seconds
}

#[cfg(test)]
mod probe_candidate_tests;
