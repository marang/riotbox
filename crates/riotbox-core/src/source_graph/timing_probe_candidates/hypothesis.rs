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
