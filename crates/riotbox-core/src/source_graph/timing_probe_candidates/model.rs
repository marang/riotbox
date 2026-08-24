#[must_use]
pub fn timing_model_from_probe_bpm_candidates(
    input: &SourceTimingProbeBpmCandidateInput,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> TimingModel {
    let period_scores = beat_period_scores(input, policy);
    let Some(primary_period) = period_scores
        .iter()
        .copied()
        .find(|score| score.score >= policy.min_beat_period_score)
    else {
        let mut timing = timing_model_from_probe_diagnostics(
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
        if !input.onset_times_seconds.is_empty()
            && input.onset_times_seconds.len() < policy.min_onset_count
            && !timing
                .warnings
                .iter()
                .any(|warning| warning.code == TimingWarningCode::SparseOnsets)
        {
            timing.warnings.insert(1, TimingWarning {
                code: TimingWarningCode::SparseOnsets,
                message: "BPM candidate has too few timing onsets".into(),
            });
        }
        return timing;
    };
    let primary_bpm = primary_period.bpm;

    let downbeat_phases = downbeat_phase_scores(input, primary_bpm);
    let primary_selection = select_downbeat_phase(input, primary_bpm, &downbeat_phases, policy);
    let primary_phase = primary_selection.phase;
    let ambiguous_phases =
        ambiguous_downbeat_phases(&downbeat_phases, primary_phase, policy).collect::<Vec<_>>();
    let mut primary = probe_bpm_hypothesis(
        "probe-bpm-primary".into(),
        TimingHypothesisKind::Primary,
        primary_bpm,
        ProbeBpmHypothesisScoring {
            confidence: policy.primary_confidence,
            beat_period_score: primary_period.score,
            downbeat_score: primary_phase.score,
        },
        primary_phase.offset_beats,
        input,
    );
    record_loop_boundary_provenance(&mut primary, primary_selection);
    let primary_drift_high = has_high_drift(&primary.drift);
    let primary_phrase_uncertain = primary.phrase_grid.is_empty()
        || !ambiguous_phases.is_empty()
        || primary_phase.score < MIN_STABLE_DOWNBEAT_PHASE_SCORE
        || primary_drift_high;
    let mut hypotheses = vec![primary];
    let mut warnings = Vec::new();
    if primary_phrase_uncertain {
        warnings.push(TimingWarningCode::PhraseUncertain);
    }
    if !ambiguous_phases.is_empty() || primary_phase.score < MIN_STABLE_DOWNBEAT_PHASE_SCORE {
        warnings.push(TimingWarningCode::AmbiguousDownbeat);
    }
    if primary_drift_high {
        warnings.push(TimingWarningCode::DriftHigh);
    }

    for phase in ambiguous_phases {
        hypotheses.push(probe_bpm_hypothesis(
            format!("probe-bpm-alt-downbeat-{}", phase.offset_beats + 1),
            TimingHypothesisKind::AlternateDownbeat,
            primary_bpm,
            ProbeBpmHypothesisScoring {
                confidence: policy.alternative_confidence,
                beat_period_score: primary_period.score,
                downbeat_score: phase.score,
            },
            phase.offset_beats,
            input,
        ));
    }

    for period in ambiguous_beat_period_scores(&period_scores, policy) {
        let kind = beat_period_hypothesis_kind(period, primary_period);
        if kind != TimingHypothesisKind::Ambiguous {
            continue;
        }
        if hypotheses
            .iter()
            .any(|hypothesis| hypothesis.kind == kind && (hypothesis.bpm - period.bpm).abs() < 0.01)
        {
            continue;
        }

        let selection = best_downbeat_phase_selection(input, period.bpm, policy);
        let mut hypothesis = probe_bpm_hypothesis(
            format!("probe-bpm-period-{:.2}", period.bpm),
            kind,
            period.bpm,
            ProbeBpmHypothesisScoring {
                confidence: policy.alternative_confidence,
                beat_period_score: period.score,
                downbeat_score: selection.phase.score,
            },
            selection.phase.offset_beats,
            input,
        );
        record_loop_boundary_provenance(&mut hypothesis, selection);
        hypotheses.push(hypothesis);
    }

    const BPM_BOUNDARY_EPSILON: f32 = 0.01;

    if primary_bpm / 2.0 + BPM_BOUNDARY_EPSILON >= policy.min_bpm {
        let half_bpm = (primary_bpm / 2.0).max(policy.min_bpm);
        let half_selection = best_downbeat_phase_selection(input, half_bpm, policy);
        let half_period_score = period_score_for_bpm(&period_scores, half_bpm)
            .map_or(primary_period.score, |score| score.score);
        let mut hypothesis = probe_bpm_hypothesis(
            "probe-bpm-half-time".into(),
            TimingHypothesisKind::HalfTime,
            half_bpm,
            ProbeBpmHypothesisScoring {
                confidence: policy.alternative_confidence,
                beat_period_score: half_period_score,
                downbeat_score: half_selection.phase.score,
            },
            half_selection.phase.offset_beats,
            input,
        );
        record_loop_boundary_provenance(&mut hypothesis, half_selection);
        hypotheses.push(hypothesis);
        warnings.push(TimingWarningCode::HalfTimePossible);
    }
    if primary_bpm * 2.0 <= policy.max_bpm + BPM_BOUNDARY_EPSILON {
        let double_bpm = (primary_bpm * 2.0).min(policy.max_bpm);
        let double_selection = best_downbeat_phase_selection(input, double_bpm, policy);
        let double_period_score = period_score_for_bpm(&period_scores, double_bpm)
            .map_or(primary_period.score, |score| score.score);
        let mut hypothesis = probe_bpm_hypothesis(
            "probe-bpm-double-time".into(),
            TimingHypothesisKind::DoubleTime,
            double_bpm,
            ProbeBpmHypothesisScoring {
                confidence: policy.alternative_confidence,
                beat_period_score: double_period_score,
                downbeat_score: double_selection.phase.score,
            },
            double_selection.phase.offset_beats,
            input,
        );
        record_loop_boundary_provenance(&mut hypothesis, double_selection);
        hypotheses.push(hypothesis);
        warnings.push(TimingWarningCode::DoubleTimePossible);
    }

    let (quality, degraded_policy) =
        timing_model_quality_and_policy(primary_bpm, &hypotheses, &warnings);

    TimingModel {
        bpm_estimate: Some(primary_bpm),
        bpm_confidence: policy.primary_confidence,
        meter_hint: Some(input.meter),
        beat_grid: hypotheses[0].beat_grid.clone(),
        bar_grid: hypotheses[0].bar_grid.clone(),
        phrase_grid: hypotheses[0].phrase_grid.clone(),
        hypotheses,
        primary_hypothesis_id: Some("probe-bpm-primary".into()),
        quality,
        warnings: warnings
            .into_iter()
            .map(|code| TimingWarning {
                code,
                message: probe_bpm_warning_message(
                    code,
                    input,
                    primary_selection.used_loop_boundary_prior,
                )
                .into(),
            })
            .collect(),
        degraded_policy,
    }
}

fn record_loop_boundary_provenance(
    hypothesis: &mut TimingHypothesis,
    selection: DownbeatPhaseSelection,
) {
    if selection.used_loop_boundary_prior {
        hypothesis
            .provenance
            .push("source-timing-probe.repeated-loop-boundary-prior.v1".into());
    }
}

fn timing_model_quality_and_policy(
    primary_bpm: f32,
    hypotheses: &[TimingHypothesis],
    warnings: &[TimingWarningCode],
) -> (TimingQuality, TimingDegradedPolicy) {
    if has_strict_stable_timing_evidence(primary_bpm, hypotheses, warnings) {
        (TimingQuality::High, TimingDegradedPolicy::Locked)
    } else {
        (TimingQuality::Medium, TimingDegradedPolicy::Cautious)
    }
}

fn has_strict_stable_timing_evidence(
    primary_bpm: f32,
    hypotheses: &[TimingHypothesis],
    warnings: &[TimingWarningCode],
) -> bool {
    const MAX_LOCKED_DRIFT_MS: f32 = 35.0;
    const MIN_LOCKED_SCORE: f32 = 0.25;
    // Bar confidence is probe confidence scaled by downbeat score, so this is
    // intentionally stricter than weak evidence but below the raw phase score.
    const MIN_DOWNBEAT_CONFIDENCE: f32 = 0.3;

    if !primary_bpm.is_finite() || primary_bpm <= 0.0 || !warnings.is_empty() {
        return false;
    }

    let [primary] = hypotheses else {
        return false;
    };
    if primary.kind != TimingHypothesisKind::Primary
        || primary.score < MIN_LOCKED_SCORE
        || primary.beat_grid.is_empty()
        || primary.bar_grid.len() < 8
        || primary.phrase_grid.is_empty()
        || primary.drift.is_empty()
        || has_high_drift(&primary.drift)
    {
        return false;
    }

    if primary
        .bar_grid
        .first()
        .is_none_or(|bar| bar.downbeat_confidence < MIN_DOWNBEAT_CONFIDENCE)
    {
        return false;
    }

    primary.drift.iter().all(|drift| {
        drift.max_drift_ms <= MAX_LOCKED_DRIFT_MS
            && drift.end_drift_ms.abs() <= MAX_LOCKED_DRIFT_MS
    })
}

fn beat_period_hypothesis_kind(
    period: BeatPeriodScore,
    primary_period: BeatPeriodScore,
) -> TimingHypothesisKind {
    if (period.period_seconds - primary_period.period_seconds * 2.0).abs() <= 0.01 {
        TimingHypothesisKind::HalfTime
    } else if (period.period_seconds * 2.0 - primary_period.period_seconds).abs() <= 0.01 {
        TimingHypothesisKind::DoubleTime
    } else {
        TimingHypothesisKind::Ambiguous
    }
}

fn period_score_for_bpm(scores: &[BeatPeriodScore], bpm: f32) -> Option<BeatPeriodScore> {
    scores
        .iter()
        .copied()
        .find(|score| (score.bpm - bpm).abs() <= 0.01)
}

fn onset_density(onset_count: usize, duration_seconds: f32) -> f32 {
    if duration_seconds <= 0.0 {
        return 0.0;
    }
    onset_count as f32 / duration_seconds
}
