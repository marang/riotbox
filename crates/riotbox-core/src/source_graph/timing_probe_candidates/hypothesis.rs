#[derive(Clone, Copy, Debug, PartialEq)]
struct ProbeBpmHypothesisScoring {
    confidence: Confidence,
    beat_period_score: f32,
    downbeat_score: f32,
}

fn probe_bpm_hypothesis(
    hypothesis_id: String,
    kind: TimingHypothesisKind,
    bpm: f32,
    scoring: ProbeBpmHypothesisScoring,
    downbeat_offset_beats: u8,
    input: &SourceTimingProbeBpmCandidateInput,
) -> TimingHypothesis {
    let confidence = scoring.confidence;
    let hypothesis_score =
        confidence * scoring.beat_period_score.clamp(0.0, 1.0) * scoring.downbeat_score.max(0.0);
    TimingHypothesis {
        hypothesis_id,
        kind,
        bpm,
        meter: input.meter,
        confidence,
        score: hypothesis_score,
        beat_grid: probe_candidate_beat_grid(input.duration_seconds, bpm, confidence),
        bar_grid: probe_candidate_bar_grid(
            input.duration_seconds,
            bpm,
            confidence,
            input.meter,
            downbeat_offset_beats,
            scoring.downbeat_score,
        ),
        phrase_grid: Vec::new(),
        anchors: normalized_onset_evidence(input)
            .into_iter()
            .take(16)
            .enumerate()
            .map(|(index, onset)| SourceTimingAnchor {
                anchor_id: format!("{}:probe-onset-{}", input.source_id, index + 1),
                anchor_type: SourceTimingAnchorType::TransientCluster,
                time_seconds: onset.time_seconds,
                bar_index: None,
                beat_index: None,
                confidence,
                strength: onset.strength,
                tags: vec![
                    "probe_onset".into(),
                    "bpm_candidate".into(),
                    "period_scored".into(),
                    format!("downbeat_phase_{}", downbeat_offset_beats + 1),
                ],
            })
            .collect(),
        drift: probe_candidate_drift_reports(input, bpm, confidence),
        groove: Vec::new(),
        quality: TimingQuality::Medium,
        warnings: Vec::new(),
        provenance: vec![
            "source-timing-probe.bpm-candidate".into(),
            "source-timing-probe.beat-period-score.v0".into(),
            "source-timing-probe.downbeat-accent-score.v0".into(),
            "source-timing-probe.drift-report.v0".into(),
            input.source_id.clone(),
        ],
    }
}

fn normalized_onset_times(input: &SourceTimingProbeBpmCandidateInput) -> Vec<f32> {
    normalized_onset_evidence(input)
        .into_iter()
        .map(|onset| onset.time_seconds)
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NormalizedOnsetEvidence {
    time_seconds: f32,
    strength: f32,
}

fn normalized_onset_evidence(
    input: &SourceTimingProbeBpmCandidateInput,
) -> Vec<NormalizedOnsetEvidence> {
    let max_time = input.duration_seconds.max(0.0);
    let mut onsets = input
        .onset_times_seconds
        .iter()
        .enumerate()
        .filter_map(|(index, time_seconds)| {
            if !time_seconds.is_finite() || *time_seconds < 0.0 || *time_seconds > max_time {
                return None;
            }
            let strength = input
                .onset_strengths
                .get(index)
                .copied()
                .filter(|strength| strength.is_finite() && *strength > 0.0)
                .unwrap_or(1.0);
            Some(NormalizedOnsetEvidence {
                time_seconds: *time_seconds,
                strength,
            })
        })
        .collect::<Vec<_>>();
    onsets.sort_by(|left, right| {
        left.time_seconds
            .total_cmp(&right.time_seconds)
            .then_with(|| right.strength.total_cmp(&left.strength))
    });
    onsets
}

fn normalized_onset_times_and_strengths(
    input: &SourceTimingProbeBpmCandidateInput,
) -> Vec<(f32, f32)> {
    normalized_onset_evidence(input)
        .into_iter()
        .map(|onset| (onset.time_seconds, onset.strength))
        .collect()
}

fn probe_bpm_warning_message(
    code: TimingWarningCode,
    input: &SourceTimingProbeBpmCandidateInput,
) -> &'static str {
    match code {
        TimingWarningCode::AmbiguousDownbeat => {
            "BPM candidate has only preliminary downbeat scoring"
        }
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
