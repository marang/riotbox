#[must_use]
pub fn source_timing_probe_readiness_report(
    input: &SourceTimingProbeBpmCandidateInput,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> SourceTimingProbeReadinessReport {
    let timing = timing_model_from_probe_bpm_candidates(input, policy);
    let confidence = source_timing_candidate_confidence_report(&timing);
    let beat = source_timing_probe_beat_evidence_report(input, policy);
    let downbeat = source_timing_probe_downbeat_evidence_report(
        input,
        confidence.primary_bpm.unwrap_or(f32::NAN),
        policy,
    );
    let alternate_evidence_count = confidence.alternate_downbeat_count
        + confidence.half_time_count
        + confidence.double_time_count
        + beat.alternate_candidate_count
        + downbeat.alternate_phase_count;
    let requires_manual_confirm = confidence.requires_manual_confirm
        || beat.status != SourceTimingProbeBeatEvidenceStatus::Stable
        || downbeat.status != SourceTimingProbeDownbeatEvidenceStatus::Stable;
    let readiness = timing_probe_readiness_status(&confidence, &beat, &downbeat);

    SourceTimingProbeReadinessReport {
        schema: "riotbox.source_timing_probe_readiness.v1",
        schema_version: 1,
        source_id: input.source_id.clone(),
        primary_bpm: confidence.primary_bpm,
        primary_downbeat_offset_beats: downbeat.primary_offset_beats,
        beat_status: beat.status,
        downbeat_status: downbeat.status,
        confidence_result: confidence.result,
        drift_status: confidence.primary_drift_status,
        phrase_status: confidence.primary_phrase_status,
        alternate_evidence_count,
        warning_codes: confidence.warning_codes,
        requires_manual_confirm,
        readiness,
    }
}

fn timing_probe_readiness_status(
    confidence: &SourceTimingCandidateConfidenceReport,
    beat: &SourceTimingProbeBeatEvidenceReport,
    downbeat: &SourceTimingProbeDownbeatEvidenceReport,
) -> SourceTimingProbeReadinessStatus {
    if confidence.result == SourceTimingCandidateConfidenceResult::Degraded
        || beat.status == SourceTimingProbeBeatEvidenceStatus::Unavailable
        || downbeat.status == SourceTimingProbeDownbeatEvidenceStatus::Unavailable
    {
        SourceTimingProbeReadinessStatus::Unavailable
    } else if beat.status == SourceTimingProbeBeatEvidenceStatus::Weak
        || downbeat.status == SourceTimingProbeDownbeatEvidenceStatus::Weak
    {
        SourceTimingProbeReadinessStatus::Weak
    } else if confidence.result == SourceTimingCandidateConfidenceResult::CandidateAmbiguous
        || beat.status == SourceTimingProbeBeatEvidenceStatus::Ambiguous
        || downbeat.status == SourceTimingProbeDownbeatEvidenceStatus::Ambiguous
    {
        SourceTimingProbeReadinessStatus::NeedsReview
    } else {
        SourceTimingProbeReadinessStatus::Ready
    }
}
