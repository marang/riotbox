#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct DownbeatPhaseScore {
    offset_beats: u8,
    score: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct DownbeatPhaseSelection {
    phase: DownbeatPhaseScore,
    used_loop_boundary_prior: bool,
}

const MAX_LOOP_BOUNDARY_BAR_COUNT: usize = 8;
const MAX_LOOP_BOUNDARY_FIT_ERROR_BARS: f32 = 0.03;
const MIN_LOOP_BOUNDARY_REPEAT_COVERAGE: f32 = 0.60;
const MIN_LOOP_BOUNDARY_STRENGTH_SIMILARITY: f32 = 0.55;

fn downbeat_phase_scores(
    input: &SourceTimingProbeBpmCandidateInput,
    bpm: f32,
) -> Vec<DownbeatPhaseScore> {
    let onsets = normalized_onset_times_and_strengths(input);
    let beats_per_bar = input.meter.beats_per_bar.max(1);
    let seconds_per_beat = 60.0 / bpm.max(1.0);
    let seconds_per_bar = seconds_per_beat * f32::from(beats_per_bar);
    if onsets.is_empty() || seconds_per_bar <= 0.0 {
        return vec![DownbeatPhaseScore::default()];
    }
    let total_strength = onsets
        .iter()
        .map(|(_, strength)| strength.max(0.0))
        .sum::<f32>()
        .max(f32::EPSILON);

    let tolerance_seconds = (seconds_per_beat * 0.2).clamp(0.02, 0.08);
    let mut scores = (0..beats_per_bar)
        .map(|offset_beats| {
            let phase_seconds = f32::from(offset_beats) * seconds_per_beat;
            let matching_strength = onsets
                .iter()
                .filter_map(|(time_seconds, strength)| {
                    (distance_to_repeating_phase(*time_seconds, phase_seconds, seconds_per_bar)
                        <= tolerance_seconds)
                        .then_some(strength.max(0.0))
                })
                .sum::<f32>();
            DownbeatPhaseScore {
                offset_beats,
                score: matching_strength / total_strength,
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

fn select_downbeat_phase(
    input: &SourceTimingProbeBpmCandidateInput,
    bpm: f32,
    phases: &[DownbeatPhaseScore],
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> DownbeatPhaseSelection {
    let accent_primary = phases.first().copied().unwrap_or_default();
    let Some(file_boundary) = phases.iter().copied().find(|phase| phase.offset_beats == 0) else {
        return DownbeatPhaseSelection {
            phase: accent_primary,
            used_loop_boundary_prior: false,
        };
    };
    let file_boundary_is_ambiguous_contender = file_boundary.score > 0.0
        && accent_primary.score - file_boundary.score <= policy.downbeat_ambiguity_margin;
    let use_loop_boundary = accent_primary.offset_beats != 0
        && file_boundary_is_ambiguous_contender
        && repeated_full_bar_loop_supports_file_boundary(input, bpm);

    DownbeatPhaseSelection {
        phase: if use_loop_boundary {
            file_boundary
        } else {
            accent_primary
        },
        used_loop_boundary_prior: use_loop_boundary,
    }
}

fn repeated_full_bar_loop_supports_file_boundary(
    input: &SourceTimingProbeBpmCandidateInput,
    bpm: f32,
) -> bool {
    let beats_per_bar = usize::from(input.meter.beats_per_bar.max(1));
    let seconds_per_beat = 60.0 / bpm.max(1.0);
    let seconds_per_bar = seconds_per_beat * beats_per_bar as f32;
    if !input.duration_seconds.is_finite() || seconds_per_bar <= 0.0 {
        return false;
    }
    let bar_count = (input.duration_seconds / seconds_per_bar).round();
    if !(2.0..=MAX_LOOP_BOUNDARY_BAR_COUNT as f32).contains(&bar_count) {
        return false;
    }
    let bar_fit_error = (input.duration_seconds - bar_count * seconds_per_bar).abs()
        / seconds_per_bar;
    if bar_fit_error > MAX_LOOP_BOUNDARY_FIT_ERROR_BARS {
        return false;
    }

    let tolerance_seconds = (seconds_per_beat * 0.18).clamp(0.035, 0.08);
    let onsets = normalized_onset_times_and_strengths(input);
    if !onsets
        .iter()
        .any(|(time_seconds, _)| time_seconds.abs() <= tolerance_seconds)
    {
        return false;
    }
    let reference = onset_pattern_for_bar(&onsets, 0, seconds_per_bar, tolerance_seconds);
    if reference.len() < beats_per_bar {
        return false;
    }

    (1..bar_count as usize).all(|bar_index| {
        let candidate =
            onset_pattern_for_bar(&onsets, bar_index, seconds_per_bar, tolerance_seconds);
        repeated_bar_pattern_matches(&reference, &candidate, tolerance_seconds)
    })
}

fn onset_pattern_for_bar(
    onsets: &[(f32, f32)],
    bar_index: usize,
    seconds_per_bar: f32,
    tolerance_seconds: f32,
) -> Vec<(f32, f32)> {
    let bar_start = bar_index as f32 * seconds_per_bar;
    onsets
        .iter()
        .filter_map(|(time_seconds, strength)| {
            let assigned_bar = ((*time_seconds + tolerance_seconds * 0.25) / seconds_per_bar)
                .floor()
                .max(0.0) as usize;
            (assigned_bar == bar_index).then(|| {
                let offset = (*time_seconds - bar_start).max(0.0);
                (offset, *strength)
            })
        })
        .collect()
}

fn repeated_bar_pattern_matches(
    reference: &[(f32, f32)],
    candidate: &[(f32, f32)],
    tolerance_seconds: f32,
) -> bool {
    if candidate.is_empty() {
        return false;
    }
    let mut used = vec![false; candidate.len()];
    let mut matched = 0_usize;
    let mut strength_similarity = 0.0_f32;
    for (reference_time, reference_strength) in reference {
        let Some((candidate_index, (_, candidate_strength))) = candidate
            .iter()
            .enumerate()
            .filter(|(index, (candidate_time, _))| {
                !used[*index] && (*candidate_time - *reference_time).abs() <= tolerance_seconds
            })
            .min_by(|(_, left), (_, right)| {
                (left.0 - *reference_time)
                    .abs()
                    .total_cmp(&(right.0 - *reference_time).abs())
            })
        else {
            continue;
        };
        used[candidate_index] = true;
        matched += 1;
        let stronger = reference_strength.max(*candidate_strength).max(f32::EPSILON);
        strength_similarity += reference_strength.min(*candidate_strength).max(0.0) / stronger;
    }

    let coverage = matched as f32 / reference.len().max(candidate.len()) as f32;
    let mean_strength_similarity = strength_similarity / matched.max(1) as f32;
    coverage >= MIN_LOOP_BOUNDARY_REPEAT_COVERAGE
        && mean_strength_similarity >= MIN_LOOP_BOUNDARY_STRENGTH_SIMILARITY
}

#[must_use]
pub fn source_timing_probe_downbeat_evidence_report(
    input: &SourceTimingProbeBpmCandidateInput,
    bpm: f32,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> SourceTimingProbeDownbeatEvidenceReport {
    let onsets = normalized_onset_times_and_strengths(input);
    let phases = if onsets.is_empty() || !bpm.is_finite() || bpm <= 0.0 {
        Vec::new()
    } else {
        downbeat_phase_scores(input, bpm)
    };
    let selection = (!phases.is_empty()).then(|| select_downbeat_phase(input, bpm, &phases, policy));
    let primary = selection.map(|selection| selection.phase);
    let primary_margin_to_next_phase = primary
        .and_then(|primary| {
            phases
                .iter()
                .copied()
                .filter(|phase| phase.offset_beats != primary.offset_beats)
                .max_by(|left, right| left.score.total_cmp(&right.score))
                .map(|next| (primary.score - next.score).max(0.0))
        });
    let alternate_phase_count = primary.map_or(0, |primary| {
        ambiguous_downbeat_phases(&phases, primary, policy).count()
    });
    let status = match primary {
        None => SourceTimingProbeDownbeatEvidenceStatus::Unavailable,
        Some(phase)
            if alternate_phase_count > 0 && phase.score >= MIN_AMBIGUOUS_DOWNBEAT_PHASE_SCORE =>
        {
            SourceTimingProbeDownbeatEvidenceStatus::Ambiguous
        }
        Some(phase) if phase.score < MIN_STABLE_DOWNBEAT_PHASE_SCORE => {
            SourceTimingProbeDownbeatEvidenceStatus::Weak
        }
        Some(_) if alternate_phase_count > 0 => SourceTimingProbeDownbeatEvidenceStatus::Ambiguous,
        Some(_) => SourceTimingProbeDownbeatEvidenceStatus::Stable,
    };

    SourceTimingProbeDownbeatEvidenceReport {
        schema: "riotbox.source_timing_probe_downbeat_evidence.v1",
        schema_version: 1,
        source_id: input.source_id.clone(),
        bpm,
        phase_count: phases.len(),
        primary_offset_beats: primary.map(|phase| phase.offset_beats),
        primary_score: primary.map(|phase| phase.score),
        primary_margin_to_next_phase,
        alternate_phase_count,
        status,
    }
}

fn best_downbeat_phase_selection(
    input: &SourceTimingProbeBpmCandidateInput,
    bpm: f32,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> DownbeatPhaseSelection {
    let phases = downbeat_phase_scores(input, bpm);
    select_downbeat_phase(input, bpm, &phases, policy)
}

fn ambiguous_downbeat_phases(
    phases: &[DownbeatPhaseScore],
    primary: DownbeatPhaseScore,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> impl Iterator<Item = DownbeatPhaseScore> + '_ {
    let best_score = phases.first().map_or(0.0, |phase| phase.score);
    phases.iter().copied().filter(move |phase| {
        phase.offset_beats != primary.offset_beats
            && phase.score > 0.0
            && best_score - phase.score <= policy.downbeat_ambiguity_margin
    })
}

fn distance_to_repeating_phase(time_seconds: f32, phase_seconds: f32, period_seconds: f32) -> f32 {
    let position = (time_seconds - phase_seconds).rem_euclid(period_seconds);
    position.min(period_seconds - position)
}
