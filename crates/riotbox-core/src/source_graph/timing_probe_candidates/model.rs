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

fn onset_density(onset_count: usize, duration_seconds: f32) -> f32 {
    if duration_seconds <= 0.0 {
        return 0.0;
    }
    onset_count as f32 / duration_seconds
}
