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
    let beat_grid = probe_candidate_beat_grid(input.duration_seconds, bpm, confidence);
    let bar_grid = probe_candidate_bar_grid(
        input.duration_seconds,
        bpm,
        confidence,
        input.meter,
        downbeat_offset_beats,
        scoring.downbeat_score,
    );
    let drift = probe_candidate_drift_reports(input, bpm, confidence);
    let phrase_grid = probe_candidate_phrase_grid(&bar_grid, scoring.downbeat_score, &drift);
    let anchors = probe_candidate_anchors(
        input,
        ProbeAnchorBuildContext {
            kind,
            bpm,
            confidence,
            downbeat_score: scoring.downbeat_score,
            downbeat_offset_beats,
        },
        &beat_grid,
        &bar_grid,
    );
    TimingHypothesis {
        hypothesis_id,
        kind,
        bpm,
        meter: input.meter,
        confidence,
        score: hypothesis_score,
        beat_grid,
        bar_grid,
        phrase_grid,
        anchors,
        drift,
        groove: Vec::new(),
        quality: TimingQuality::Medium,
        warnings: Vec::new(),
        provenance: vec![
            "source-timing-probe.bpm-candidate".into(),
            "source-timing-probe.beat-period-score.v0".into(),
            "source-timing-probe.downbeat-accent-score.v0".into(),
            "source-timing-probe.drift-report.v0".into(),
            "source-timing-probe.phrase-grid.v0".into(),
            input.source_id.clone(),
        ],
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ProbeAnchorBuildContext {
    kind: TimingHypothesisKind,
    bpm: f32,
    confidence: Confidence,
    downbeat_score: f32,
    downbeat_offset_beats: u8,
}

fn probe_candidate_anchors(
    input: &SourceTimingProbeBpmCandidateInput,
    context: ProbeAnchorBuildContext,
    beat_grid: &[BeatPoint],
    bar_grid: &[BarSpan],
) -> Vec<SourceTimingAnchor> {
    normalized_onset_evidence(input)
        .into_iter()
        .take(16)
        .enumerate()
        .map(|(index, onset)| {
            let placement = ProbeAnchorPlacement::from_onset(
                input.meter,
                context.bpm,
                onset,
                beat_grid,
                bar_grid,
            );
            let classified =
                classify_probe_anchor(context.kind, input.meter, context.downbeat_score, placement);
            SourceTimingAnchor {
                anchor_id: format!("{}:probe-onset-{}", input.source_id, index + 1),
                anchor_type: classified.anchor_type,
                time_seconds: onset.time_seconds,
                bar_index: placement.bar_index,
                beat_index: placement.beat_index,
                confidence: classified.confidence(context.confidence),
                strength: onset.strength,
                tags: probe_anchor_tags(context.downbeat_offset_beats, placement, classified),
            }
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ProbeAnchorPlacement {
    bar_index: Option<u32>,
    beat_index: Option<u32>,
    beat_in_bar: Option<u8>,
    aligned_to_grid: bool,
}

impl ProbeAnchorPlacement {
    fn from_onset(
        meter: MeterHint,
        bpm: f32,
        onset: NormalizedOnsetEvidence,
        beat_grid: &[BeatPoint],
        bar_grid: &[BarSpan],
    ) -> Self {
        let seconds_per_beat = 60.0 / bpm.max(1.0);
        let grid_tolerance_seconds = (seconds_per_beat * 0.18).clamp(0.035, 0.09);
        let beat = nearest_beat(onset.time_seconds, beat_grid, grid_tolerance_seconds);
        let bar = containing_bar(onset.time_seconds, bar_grid, grid_tolerance_seconds);
        let beat_in_bar = bar
            .and_then(|bar| beat_in_bar(onset.time_seconds, bar, seconds_per_beat, meter));
        Self {
            bar_index: bar.map(|bar| bar.bar_index),
            beat_index: beat.map(|beat| beat.beat_index),
            beat_in_bar,
            aligned_to_grid: beat.is_some() && bar.is_some() && beat_in_bar.is_some(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ProbeAnchorClassification {
    anchor_type: SourceTimingAnchorType,
    class_tag: Option<&'static str>,
    grid_role_tag: Option<&'static str>,
}

impl ProbeAnchorClassification {
    fn confidence(self, base_confidence: Confidence) -> Confidence {
        match self.anchor_type {
            SourceTimingAnchorType::Kick | SourceTimingAnchorType::Backbeat => {
                (base_confidence + 0.1).min(1.0)
            }
            _ => base_confidence,
        }
    }
}

fn classify_probe_anchor(
    kind: TimingHypothesisKind,
    meter: MeterHint,
    downbeat_score: f32,
    placement: ProbeAnchorPlacement,
) -> ProbeAnchorClassification {
    if kind != TimingHypothesisKind::Primary
        || !placement.aligned_to_grid
        || downbeat_score < MIN_STABLE_DOWNBEAT_PHASE_SCORE
    {
        return transient_probe_anchor();
    }

    match placement.beat_in_bar {
        Some(1) => ProbeAnchorClassification {
            anchor_type: SourceTimingAnchorType::Kick,
            class_tag: Some("kick_anchor"),
            grid_role_tag: Some("downbeat"),
        },
        Some(2 | 4) if meter.beats_per_bar == 4 => ProbeAnchorClassification {
            anchor_type: SourceTimingAnchorType::Backbeat,
            class_tag: Some("backbeat_anchor"),
            grid_role_tag: Some("snare_style"),
        },
        _ => transient_probe_anchor(),
    }
}

fn transient_probe_anchor() -> ProbeAnchorClassification {
    ProbeAnchorClassification {
        anchor_type: SourceTimingAnchorType::TransientCluster,
        class_tag: Some("transient_cluster"),
        grid_role_tag: None,
    }
}

fn probe_anchor_tags(
    downbeat_offset_beats: u8,
    placement: ProbeAnchorPlacement,
    classified: ProbeAnchorClassification,
) -> Vec<String> {
    let mut tags = vec![
        "probe_onset".into(),
        "bpm_candidate".into(),
        "period_scored".into(),
        "anchor_classified_v0".into(),
        format!("downbeat_phase_{}", downbeat_offset_beats + 1),
    ];
    if let Some(class_tag) = classified.class_tag {
        tags.push(class_tag.into());
    }
    if let Some(grid_role_tag) = classified.grid_role_tag {
        tags.push(grid_role_tag.into());
    }
    if let Some(beat_in_bar) = placement.beat_in_bar {
        tags.push(format!("beat_in_bar_{beat_in_bar}"));
    }
    if placement.aligned_to_grid {
        tags.push("grid_aligned".into());
    }
    tags
}

fn nearest_beat(
    time_seconds: f32,
    beat_grid: &[BeatPoint],
    tolerance_seconds: f32,
) -> Option<BeatPoint> {
    beat_grid
        .iter()
        .copied()
        .filter(|beat| (beat.time_seconds - time_seconds).abs() <= tolerance_seconds)
        .min_by(|left, right| {
            (left.time_seconds - time_seconds)
                .abs()
                .total_cmp(&(right.time_seconds - time_seconds).abs())
        })
}

fn containing_bar(
    time_seconds: f32,
    bar_grid: &[BarSpan],
    tolerance_seconds: f32,
) -> Option<&BarSpan> {
    bar_grid.iter().rev().find(|bar| {
        time_seconds + tolerance_seconds >= bar.start_seconds
            && time_seconds <= bar.end_seconds + tolerance_seconds
    })
}

fn beat_in_bar(
    time_seconds: f32,
    bar: &BarSpan,
    seconds_per_beat: f32,
    meter: MeterHint,
) -> Option<u8> {
    let beat_in_bar = ((time_seconds - bar.start_seconds) / seconds_per_beat).round() as i32 + 1;
    let beats_per_bar = i32::from(meter.beats_per_bar.max(1));
    if (1..=beats_per_bar).contains(&beat_in_bar) {
        u8::try_from(beat_in_bar).ok()
    } else {
        None
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
        TimingWarningCode::PhraseUncertain => "BPM candidate has uncertain phrase boundary scoring",
        TimingWarningCode::HalfTimePossible => "half-time BPM candidate preserved",
        TimingWarningCode::DoubleTimePossible => "double-time BPM candidate preserved",
        TimingWarningCode::LowTimingConfidence => "BPM candidate confidence is low",
        TimingWarningCode::WeakKickAnchor => "BPM candidate has no trusted kick anchor yet",
        TimingWarningCode::WeakBackbeatAnchor => "BPM candidate has no trusted backbeat anchor yet",
        TimingWarningCode::DriftHigh => {
            let _ = input;
            "BPM candidate drift is high"
        }
    }
}
