pub type Confidence = f32;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct TimingModel {
    pub bpm_estimate: Option<f32>,
    pub bpm_confidence: Confidence,
    pub meter_hint: Option<MeterHint>,
    pub beat_grid: Vec<BeatPoint>,
    pub bar_grid: Vec<BarSpan>,
    pub phrase_grid: Vec<PhraseSpan>,
    #[serde(default)]
    pub hypotheses: Vec<TimingHypothesis>,
    #[serde(default)]
    pub primary_hypothesis_id: Option<String>,
    #[serde(default)]
    pub quality: TimingQuality,
    #[serde(default)]
    pub warnings: Vec<TimingWarning>,
    #[serde(default)]
    pub degraded_policy: TimingDegradedPolicy,
}

impl TimingModel {
    #[must_use]
    pub fn primary_hypothesis(&self) -> Option<&TimingHypothesis> {
        let primary_id = self.primary_hypothesis_id.as_deref()?;
        self.hypotheses
            .iter()
            .find(|hypothesis| hypothesis.hypothesis_id == primary_id)
    }

    #[must_use]
    pub fn effective_timing_quality(&self) -> TimingQuality {
        if self.quality != TimingQuality::Unknown {
            return self.quality;
        }

        match self.primary_hypothesis() {
            Some(hypothesis) => hypothesis.quality,
            None if self.bpm_confidence >= 0.8 => TimingQuality::High,
            None if self.bpm_confidence >= 0.5 => TimingQuality::Medium,
            None if self.bpm_estimate.is_some() => TimingQuality::Low,
            None => TimingQuality::Unknown,
        }
    }

    #[must_use]
    pub fn effective_degraded_policy(&self) -> TimingDegradedPolicy {
        if self.degraded_policy != TimingDegradedPolicy::Unknown {
            return self.degraded_policy;
        }

        match self.effective_timing_quality() {
            TimingQuality::High => TimingDegradedPolicy::Locked,
            TimingQuality::Medium => TimingDegradedPolicy::Cautious,
            TimingQuality::Low => TimingDegradedPolicy::ManualConfirm,
            TimingQuality::Unknown => TimingDegradedPolicy::Disabled,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeterHint {
    pub beats_per_bar: u8,
    pub beat_unit: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BeatPoint {
    pub beat_index: u32,
    pub time_seconds: f32,
    pub confidence: Confidence,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BarSpan {
    pub bar_index: u32,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub downbeat_confidence: Confidence,
    pub phrase_index: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhraseSpan {
    pub phrase_index: u32,
    pub start_bar: u32,
    pub end_bar: u32,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimingHypothesis {
    pub hypothesis_id: String,
    pub kind: TimingHypothesisKind,
    pub bpm: f32,
    pub meter: MeterHint,
    pub confidence: Confidence,
    pub score: f32,
    pub beat_grid: Vec<BeatPoint>,
    pub bar_grid: Vec<BarSpan>,
    pub phrase_grid: Vec<PhraseSpan>,
    pub anchors: Vec<SourceTimingAnchor>,
    pub drift: Vec<TimingDriftReport>,
    pub groove: Vec<GrooveResidual>,
    pub quality: TimingQuality,
    pub warnings: Vec<TimingWarning>,
    pub provenance: Vec<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimingHypothesisKind {
    Primary,
    HalfTime,
    DoubleTime,
    AlternateDownbeat,
    Ambiguous,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TimingQuality {
    Low,
    Medium,
    High,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceTimingAnchor {
    pub anchor_id: String,
    pub anchor_type: SourceTimingAnchorType,
    pub time_seconds: f32,
    pub bar_index: Option<u32>,
    pub beat_index: Option<u32>,
    pub confidence: Confidence,
    pub strength: f32,
    pub tags: Vec<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceTimingAnchorType {
    Kick,
    Snare,
    Backbeat,
    Fill,
    LoopWindow,
    AnswerSlot,
    CaptureCandidate,
    TransientCluster,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimingDriftReport {
    pub window_bars: u32,
    pub max_drift_ms: f32,
    pub mean_abs_drift_ms: f32,
    pub end_drift_ms: f32,
    pub confidence: Confidence,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GrooveResidual {
    pub subdivision: GrooveSubdivision,
    pub offset_ms: f32,
    pub confidence: Confidence,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrooveSubdivision {
    Eighth,
    Triplet,
    Sixteenth,
    ThirtySecond,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimingWarning {
    pub code: TimingWarningCode,
    pub message: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimingWarningCode {
    WeakKickAnchor,
    WeakBackbeatAnchor,
    AmbiguousDownbeat,
    HalfTimePossible,
    DoubleTimePossible,
    DriftHigh,
    PhraseUncertain,
    LowTimingConfidence,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TimingDegradedPolicy {
    Locked,
    Cautious,
    ManualConfirm,
    FallbackGrid,
    Disabled,
    #[default]
    Unknown,
}
