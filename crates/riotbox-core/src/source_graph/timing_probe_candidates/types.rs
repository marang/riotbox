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
    pub min_beat_period_score: f32,
    pub beat_period_ambiguity_margin: f32,
    pub downbeat_ambiguity_margin: f32,
}

impl Default for SourceTimingProbeBpmCandidatePolicy {
    fn default() -> Self {
        Self {
            min_onset_count: 4,
            min_bpm: 55.0,
            max_bpm: 240.0,
            primary_confidence: 0.55,
            alternative_confidence: 0.35,
            min_beat_period_score: 0.45,
            beat_period_ambiguity_margin: 0.08,
            downbeat_ambiguity_margin: 0.05,
        }
    }
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
