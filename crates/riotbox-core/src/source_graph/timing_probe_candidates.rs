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
}

impl Default for SourceTimingProbeBpmCandidatePolicy {
    fn default() -> Self {
        Self {
            min_onset_count: 4,
            min_bpm: 55.0,
            max_bpm: 240.0,
            primary_confidence: 0.55,
            alternative_confidence: 0.35,
        }
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

    let primary = probe_bpm_hypothesis(
        "probe-bpm-primary".into(),
        TimingHypothesisKind::Primary,
        primary_bpm,
        policy.primary_confidence,
        input,
    );
    let mut hypotheses = vec![primary];
    let mut warnings = vec![
        TimingWarningCode::AmbiguousDownbeat,
        TimingWarningCode::PhraseUncertain,
    ];

    if primary_bpm / 2.0 >= policy.min_bpm {
        hypotheses.push(probe_bpm_hypothesis(
            "probe-bpm-half-time".into(),
            TimingHypothesisKind::HalfTime,
            primary_bpm / 2.0,
            policy.alternative_confidence,
            input,
        ));
        warnings.push(TimingWarningCode::HalfTimePossible);
    }
    if primary_bpm * 2.0 <= policy.max_bpm {
        hypotheses.push(probe_bpm_hypothesis(
            "probe-bpm-double-time".into(),
            TimingHypothesisKind::DoubleTime,
            primary_bpm * 2.0,
            policy.alternative_confidence,
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
    input: &SourceTimingProbeBpmCandidateInput,
) -> TimingHypothesis {
    TimingHypothesis {
        hypothesis_id,
        kind,
        bpm,
        meter: input.meter,
        confidence,
        score: confidence,
        beat_grid: probe_candidate_beat_grid(input.duration_seconds, bpm, confidence),
        bar_grid: probe_candidate_bar_grid(input.duration_seconds, bpm, confidence, input.meter),
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
                tags: vec!["probe_onset".into(), "bpm_candidate".into()],
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
) -> Vec<BarSpan> {
    let seconds_per_bar = (60.0 / bpm.max(1.0)) * f32::from(meter.beats_per_bar.max(1));
    let mut bar_grid = Vec::new();
    let mut start_seconds = 0.0_f32;
    while start_seconds < duration_seconds.max(0.0) {
        bar_grid.push(BarSpan {
            bar_index: u32::try_from(bar_grid.len() + 1).unwrap_or(u32::MAX),
            start_seconds,
            end_seconds: (start_seconds + seconds_per_bar).min(duration_seconds.max(0.0)),
            downbeat_confidence: confidence * 0.5,
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
