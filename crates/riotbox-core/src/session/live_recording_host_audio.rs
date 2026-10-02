use serde::{Deserialize, Serialize};

use crate::action::LiveRecordingDuration;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportLiveRecordingHostAudioRef {
    pub host: String,
    pub device: String,
    pub recording_duration_ms: u64,
    pub callback_gap_summary: ExportLiveRecordingCallbackGapSummary,
    pub stream_error_summary: ExportLiveRecordingStreamErrorSummary,
    #[serde(default)]
    pub timing_window: Option<ExportLiveRecordingTimingWindow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportLiveRecordingTimingWindow {
    pub confirmed_bpm_micros: u64,
    pub bar_grid_anchor_position_microbeats: u64,
    pub beat_span_per_frame_nanobeats: u64,
    pub requested_start_position_microbeats: u64,
    pub captured_start_position_microbeats: u64,
    pub captured_end_position_microbeats: u64,
    pub start_alignment_error_frame_micros: u64,
    pub duration_error_frame_micros: u64,
    pub beats_per_bar: u8,
    pub duration_beats: u32,
}

#[derive(Copy, Clone)]
enum TimingGeometry {
    LegacyIntegerMicroBpm,
    CanonicalRuntime,
}

impl ExportLiveRecordingTimingWindow {
    #[must_use]
    pub fn bar_aligned_two_bar_window_ready(&self, sample_rate_hz: u32, frame_count: u64) -> bool {
        self.bar_aligned_geometry_ready(
            sample_rate_hz,
            frame_count,
            LiveRecordingDuration::TwoBars,
            TimingGeometry::LegacyIntegerMicroBpm,
        )
    }

    /// Compatibility interface for the V2/V3 duration mapping. Versioned owners
    /// must select legacy V2 or canonical V3/V4 geometry explicitly instead.
    #[must_use]
    pub fn bar_aligned_window_ready(
        &self,
        sample_rate_hz: u32,
        frame_count: u64,
        duration: LiveRecordingDuration,
    ) -> bool {
        match duration {
            LiveRecordingDuration::TwoBars => {
                self.bar_aligned_two_bar_window_ready(sample_rate_hz, frame_count)
            }
            LiveRecordingDuration::EightBars | LiveRecordingDuration::SixteenBars => {
                self.canonical_runtime_bar_window_ready(sample_rate_hz, frame_count, duration)
            }
        }
    }

    /// V3/V4 timing evidence follows exact canonical runtime-f32 tempo, including
    /// explicit two-bar V4 windows. Historical V2 uses its separate frozen method.
    #[must_use]
    pub fn canonical_runtime_bar_window_ready(
        &self,
        sample_rate_hz: u32,
        frame_count: u64,
        duration: LiveRecordingDuration,
    ) -> bool {
        self.bar_aligned_geometry_ready(
            sample_rate_hz,
            frame_count,
            duration,
            TimingGeometry::CanonicalRuntime,
        )
    }

    fn bar_aligned_geometry_ready(
        &self,
        sample_rate_hz: u32,
        frame_count: u64,
        duration: LiveRecordingDuration,
        geometry: TimingGeometry,
    ) -> bool {
        let bar_microbeats = u64::from(self.beats_per_bar).saturating_mul(1_000_000);
        let start_error_nanobeats = self
            .captured_start_position_microbeats
            .checked_sub(self.requested_start_position_microbeats)
            .map(u128::from)
            .and_then(|delta| delta.checked_mul(1_000));
        let captured_duration_nanobeats = self
            .captured_end_position_microbeats
            .checked_sub(self.captured_start_position_microbeats)
            .map(u128::from)
            .and_then(|duration| duration.checked_mul(1_000));
        let expected_duration_nanobeats =
            u128::from(self.duration_beats).saturating_mul(1_000_000_000);
        let frame_span_nanobeats = u128::from(self.beat_span_per_frame_nanobeats);
        let frame_span_denominator = u128::from(sample_rate_hz).saturating_mul(60);
        let (expected_frame_span_nanobeats, expected_frame_count) =
            if matches!(geometry, TimingGeometry::LegacyIntegerMicroBpm) {
                // Frozen V2 geometry used integer micro-BPM and must remain unchanged.
                let span = u128::from(self.confirmed_bpm_micros)
                    .saturating_mul(1_000)
                    .saturating_add(frame_span_denominator / 2)
                    .checked_div(frame_span_denominator);
                let frame_numerator = u128::from(sample_rate_hz)
                    .saturating_mul(60)
                    .saturating_mul(u128::from(self.duration_beats))
                    .saturating_mul(1_000_000);
                let count = frame_numerator
                    .saturating_add(u128::from(self.confirmed_bpm_micros) / 2)
                    .checked_div(u128::from(self.confirmed_bpm_micros))
                    .and_then(|frames| u64::try_from(frames).ok());
                (span, count)
            } else {
                let span = LiveRecordingDuration::confirmed_runtime_bpm(self.confirmed_bpm_micros)
                    .filter(|_| sample_rate_hz > 0)
                    .map(|bpm| {
                        (f64::from(bpm) / 60.0 / f64::from(sample_rate_hz) * 1_000_000_000.0)
                            .round() as u128
                    });
                (
                    span,
                    duration.target_frame_count(sample_rate_hz, self.confirmed_bpm_micros),
                )
            };
        self.beats_per_bar == 4
            && self.duration_beats == duration.duration_beats()
            && self.confirmed_bpm_micros > 0
            && sample_rate_hz > 0
            && bar_microbeats > 0
            && self.beat_span_per_frame_nanobeats > 0
            && expected_frame_span_nanobeats == Some(u128::from(self.beat_span_per_frame_nanobeats))
            && expected_frame_count == Some(frame_count)
            && self
                .bar_grid_anchor_position_microbeats
                .is_multiple_of(1_000_000)
            && self
                .requested_start_position_microbeats
                .is_multiple_of(1_000_000)
            && self.requested_start_position_microbeats >= self.bar_grid_anchor_position_microbeats
            && (self.requested_start_position_microbeats - self.bar_grid_anchor_position_microbeats)
                .is_multiple_of(bar_microbeats)
            && self.captured_start_position_microbeats >= self.requested_start_position_microbeats
            && self.captured_end_position_microbeats > self.captured_start_position_microbeats
            && self.start_alignment_error_frame_micros <= 1_000_000
            && self.duration_error_frame_micros <= 500_001
            && start_error_nanobeats
                .is_some_and(|error| error <= frame_span_nanobeats.saturating_add(1_000))
            && captured_duration_nanobeats.is_some_and(|duration| {
                duration.abs_diff(expected_duration_nanobeats)
                    <= frame_span_nanobeats.div_ceil(2).saturating_add(1_000)
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportLiveRecordingCallbackGapSummary {
    #[serde(default)]
    pub max_gap_ms: Option<u64>,
    pub over_threshold_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportLiveRecordingStreamErrorSummary {
    pub error_count: u32,
    #[serde(default)]
    pub last_error: Option<String>,
}
