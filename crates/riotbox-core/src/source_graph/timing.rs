use serde::{Deserialize, Serialize};

use crate::transport::{DEFAULT_BARS_PER_PHRASE, TransportBarGridAnchor, TransportGridPosition};

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

impl TimingHypothesis {
    /// Resolves the beat point that this hypothesis places at a bar
    /// start. This deliberately does not synthesize a beat index when the two
    /// grids disagree; callers can then distinguish absent bar evidence from
    /// inconsistent bar/beat evidence.
    #[must_use]
    pub fn bar_start_beat_point(&self, bar_index: u32) -> Option<BeatPoint> {
        let bar = self
            .bar_grid
            .iter()
            .find(|bar| bar.bar_index == bar_index)?;
        if !bar.start_seconds.is_finite() || !self.bpm.is_finite() || self.bpm <= 0.0 {
            return None;
        }

        let tolerance_seconds = (60.0 / self.bpm) * 0.25;
        self.beat_grid
            .iter()
            .copied()
            .filter(|beat| beat.time_seconds.is_finite())
            .min_by(|left, right| {
                (left.time_seconds - bar.start_seconds)
                    .abs()
                    .total_cmp(&(right.time_seconds - bar.start_seconds).abs())
            })
            .filter(|beat| (beat.time_seconds - bar.start_seconds).abs() <= tolerance_seconds)
    }

    /// Resolves the earliest evidenced source bar into the zero-based Session
    /// transport cursor. A populated but inconsistent bar grid returns `None`
    /// instead of silently inventing a zero-phase downbeat.
    #[must_use]
    pub fn transport_bar_grid_anchor(&self) -> Option<TransportBarGridAnchor> {
        let first_bar = self.bar_grid.iter().min_by_key(|bar| bar.bar_index)?;
        let beat = self.bar_start_beat_point(first_bar.bar_index)?;
        Some(TransportBarGridAnchor {
            beat_cursor: u64::from(beat.beat_index.checked_sub(1)?),
            bar_index: u64::from(first_bar.bar_index),
        })
    }

    #[must_use]
    pub fn transport_grid_position(&self, position_beats: f64) -> Option<TransportGridPosition> {
        let beats_per_bar = u64::from(self.meter.beats_per_bar).max(1);
        if self.bar_grid.is_empty() {
            return Some(TransportGridPosition::from_zero_based_position_beats(
                position_beats,
                beats_per_bar,
                DEFAULT_BARS_PER_PHRASE,
            ));
        }
        Some(
            TransportGridPosition::from_zero_based_position_beats_with_bar_anchor(
                position_beats,
                beats_per_bar,
                DEFAULT_BARS_PER_PHRASE,
                self.transport_bar_grid_anchor()?,
            ),
        )
    }

    #[must_use]
    pub fn next_bar_beat_cursor_after(&self, position_beats: f64) -> Option<u64> {
        let beats_per_bar = u64::from(self.meter.beats_per_bar).max(1);
        if self.bar_grid.is_empty() {
            let next_cursor =
                crate::transport::zero_based_beat_cursor_index(position_beats).saturating_add(1);
            return Some(next_cursor.div_ceil(beats_per_bar) * beats_per_bar);
        }
        Some(
            self.transport_bar_grid_anchor()?
                .next_bar_beat_cursor_after(position_beats, beats_per_bar),
        )
    }

    #[must_use]
    pub fn bar_start_beat_cursor(&self, bar_index: u64) -> Option<u64> {
        self.transport_bar_grid_anchor()?
            .beat_cursor_for_bar(bar_index, u64::from(self.meter.beats_per_bar).max(1))
    }
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
    SparseOnsets,
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
