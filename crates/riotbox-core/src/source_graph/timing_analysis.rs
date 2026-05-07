#[derive(Clone, Debug, PartialEq)]
pub struct SourceTimingAnalysisSeed {
    pub fixture_id: String,
    pub duration_seconds: f32,
    pub primary_bpm: f32,
    pub meter: MeterHint,
    pub quality: TimingQuality,
    pub degraded_policy: TimingDegradedPolicy,
    pub beat_hit_tolerance_ms: f32,
    pub downbeat_tolerance_ms: f32,
    pub expected_beat_count_min: u32,
    pub expected_bar_count_min: u32,
    pub expected_phrase_count_min: u32,
    pub confidence_floor: Confidence,
    pub warnings: Vec<TimingWarningCode>,
    pub alternatives: Vec<SourceTimingAlternativeSeed>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceTimingAlternativeSeed {
    pub kind: TimingHypothesisKind,
    pub bpm: f32,
    pub confidence_floor: Confidence,
}

#[must_use]
pub fn analyze_source_timing_seed(seed: &SourceTimingAnalysisSeed) -> TimingModel {
    let primary_hypothesis = timing_hypothesis_from_seed(
        "primary".into(),
        TimingHypothesisKind::Primary,
        seed.primary_bpm,
        seed.confidence_floor,
        seed.quality,
        "fixture-timing-skeleton.primary",
        seed,
    );

    let mut hypotheses = vec![primary_hypothesis];
    hypotheses.extend(seed.alternatives.iter().map(|alternative| {
        timing_hypothesis_from_seed(
            hypothesis_id_for_alternative(alternative),
            alternative.kind,
            alternative.bpm,
            alternative.confidence_floor,
            TimingQuality::Low,
            "fixture-timing-skeleton.alternative",
            seed,
        )
    }));

    TimingModel {
        bpm_estimate: Some(seed.primary_bpm),
        bpm_confidence: seed.confidence_floor,
        meter_hint: Some(seed.meter),
        beat_grid: hypotheses[0].beat_grid.clone(),
        bar_grid: hypotheses[0].bar_grid.clone(),
        phrase_grid: hypotheses[0].phrase_grid.clone(),
        hypotheses,
        primary_hypothesis_id: Some("primary".into()),
        quality: seed.quality,
        warnings: timing_warnings(seed),
        degraded_policy: seed.degraded_policy,
    }
}

fn timing_hypothesis_from_seed(
    hypothesis_id: String,
    kind: TimingHypothesisKind,
    bpm: f32,
    confidence: Confidence,
    quality: TimingQuality,
    provenance: &str,
    seed: &SourceTimingAnalysisSeed,
) -> TimingHypothesis {
    TimingHypothesis {
        hypothesis_id,
        kind,
        bpm,
        meter: seed.meter,
        confidence,
        score: confidence,
        beat_grid: beat_grid(seed.duration_seconds, bpm, confidence, seed.expected_beat_count_min),
        bar_grid: bar_grid(seed.duration_seconds, bpm, confidence, seed),
        phrase_grid: phrase_grid(confidence, seed),
        anchors: timing_anchors(confidence, seed),
        drift: timing_drift(seed, confidence),
        groove: Vec::new(),
        quality,
        warnings: timing_warnings(seed),
        provenance: vec![provenance.into(), seed.fixture_id.clone()],
    }
}

fn beat_grid(
    duration_seconds: f32,
    bpm: f32,
    confidence: Confidence,
    expected_beat_count_min: u32,
) -> Vec<BeatPoint> {
    let seconds_per_beat = seconds_per_beat(bpm);
    (0..expected_beat_count_min)
        .filter_map(|beat_index| {
            let time_seconds = beat_index as f32 * seconds_per_beat;
            (time_seconds <= duration_seconds).then_some(BeatPoint {
                beat_index: beat_index + 1,
                time_seconds,
                confidence,
            })
        })
        .collect()
}

fn bar_grid(
    duration_seconds: f32,
    bpm: f32,
    confidence: Confidence,
    seed: &SourceTimingAnalysisSeed,
) -> Vec<BarSpan> {
    let seconds_per_bar = seconds_per_beat(bpm) * f32::from(seed.meter.beats_per_bar);
    let phrase_count = seed.expected_phrase_count_min.max(1);

    (0..seed.expected_bar_count_min)
        .filter_map(|bar_index| {
            let start_seconds = bar_index as f32 * seconds_per_bar;
            if start_seconds >= duration_seconds {
                return None;
            }

            let phrase_index = if seed.expected_phrase_count_min == 0 {
                None
            } else {
                Some((bar_index / bars_per_phrase(seed.expected_bar_count_min, phrase_count)) + 1)
            };
            Some(BarSpan {
                bar_index: bar_index + 1,
                start_seconds,
                end_seconds: bounded_time(
                    (bar_index + 1) as f32 * seconds_per_bar,
                    duration_seconds,
                ),
                downbeat_confidence: confidence,
                phrase_index,
            })
        })
        .collect()
}

fn phrase_grid(confidence: Confidence, seed: &SourceTimingAnalysisSeed) -> Vec<PhraseSpan> {
    if seed.expected_phrase_count_min == 0 {
        return Vec::new();
    }

    let bars_per_phrase = bars_per_phrase(
        seed.expected_bar_count_min,
        seed.expected_phrase_count_min.max(1),
    );
    (0..seed.expected_phrase_count_min)
        .map(|phrase_index| {
            let start_bar = (phrase_index * bars_per_phrase) + 1;
            let end_bar = ((phrase_index + 1) * bars_per_phrase).min(seed.expected_bar_count_min);
            PhraseSpan {
                phrase_index: phrase_index + 1,
                start_bar,
                end_bar,
                confidence,
            }
        })
        .collect()
}

fn timing_anchors(
    confidence: Confidence,
    seed: &SourceTimingAnalysisSeed,
) -> Vec<SourceTimingAnchor> {
    vec![SourceTimingAnchor {
        anchor_id: format!("{}:downbeat-1", seed.fixture_id),
        anchor_type: SourceTimingAnchorType::LoopWindow,
        time_seconds: 0.0,
        bar_index: Some(1),
        beat_index: Some(1),
        confidence,
        strength: confidence,
        tags: vec!["fixture_seed".into(), "timing_skeleton".into()],
    }]
}

fn timing_drift(
    seed: &SourceTimingAnalysisSeed,
    confidence: Confidence,
) -> Vec<TimingDriftReport> {
    vec![TimingDriftReport {
        window_bars: seed.expected_bar_count_min.clamp(1, 8),
        max_drift_ms: seed.downbeat_tolerance_ms,
        mean_abs_drift_ms: seed.beat_hit_tolerance_ms / 2.0,
        end_drift_ms: 0.0,
        confidence,
    }]
}

fn timing_warnings(seed: &SourceTimingAnalysisSeed) -> Vec<TimingWarning> {
    seed.warnings
        .iter()
        .map(|code| TimingWarning {
            code: *code,
            message: timing_warning_message(*code).into(),
        })
        .collect()
}

fn hypothesis_id_for_alternative(alternative: &SourceTimingAlternativeSeed) -> String {
    match alternative.kind {
        TimingHypothesisKind::HalfTime => "half-time".into(),
        TimingHypothesisKind::DoubleTime => "double-time".into(),
        TimingHypothesisKind::AlternateDownbeat => "alternate-downbeat".into(),
        TimingHypothesisKind::Ambiguous => "ambiguous".into(),
        TimingHypothesisKind::Primary => "primary-alternative".into(),
    }
}

fn bars_per_phrase(bar_count: u32, phrase_count: u32) -> u32 {
    bar_count.div_ceil(phrase_count).max(1)
}

fn seconds_per_beat(bpm: f32) -> f32 {
    60.0 / bpm.max(1.0)
}

fn bounded_time(time_seconds: f32, duration_seconds: f32) -> f32 {
    time_seconds.min(duration_seconds.max(0.0))
}

fn timing_warning_message(code: TimingWarningCode) -> &'static str {
    match code {
        TimingWarningCode::WeakKickAnchor => "weak kick anchor",
        TimingWarningCode::WeakBackbeatAnchor => "weak backbeat anchor",
        TimingWarningCode::AmbiguousDownbeat => "ambiguous downbeat",
        TimingWarningCode::HalfTimePossible => "half-time possible",
        TimingWarningCode::DoubleTimePossible => "double-time possible",
        TimingWarningCode::DriftHigh => "timing drift high",
        TimingWarningCode::PhraseUncertain => "phrase boundary uncertain",
        TimingWarningCode::LowTimingConfidence => "low timing confidence",
    }
}
