use serde::{Deserialize, Serialize};

use crate::{action::CommitBoundary, ids::SceneId};

pub const DEFAULT_BEATS_PER_BAR: u64 = 4;
pub const DEFAULT_BARS_PER_PHRASE: u64 = 4;

/// Transport-grid identity derived from the zero-based continuous beat cursor.
///
/// `beat_cursor` stays zero-based for Session V1 compatibility, while bar and
/// phrase identities are one-based. In the default 4/4, four-bar phrase grid,
/// `position_beats == 4.0` is cursor 4 / bar 2 and `position_beats == 16.0` is
/// cursor 16 / bar 5 / phrase 2.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TransportGridPosition {
    pub beat_cursor: u64,
    pub bar_index: u64,
    pub phrase_index: u64,
}

/// Source-backed bar-grid phase expressed in transport coordinates.
///
/// Source timing beat identities are one-based, while Session V1 transport
/// cursors are zero-based. Keeping the converted cursor beside the source bar
/// identity prevents individual product paths from independently guessing the
/// downbeat phase.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TransportBarGridAnchor {
    pub beat_cursor: u64,
    pub bar_index: u64,
}

impl TransportGridPosition {
    #[must_use]
    pub fn from_zero_based_position_beats(
        position_beats: f64,
        beats_per_bar: u64,
        bars_per_phrase: u64,
    ) -> Self {
        let zero_based_beat = zero_based_beat_cursor_index(position_beats);
        let beats_per_bar = beats_per_bar.max(1);
        let bars_per_phrase = bars_per_phrase.max(1);
        let beats_per_phrase = beats_per_bar.saturating_mul(bars_per_phrase).max(1);

        Self {
            beat_cursor: zero_based_beat,
            bar_index: zero_based_beat
                .checked_div(beats_per_bar)
                .unwrap_or(0)
                .saturating_add(1),
            phrase_index: zero_based_beat
                .checked_div(beats_per_phrase)
                .unwrap_or(0)
                .saturating_add(1),
        }
    }

    #[must_use]
    pub fn from_zero_based_position_beats_with_bar_anchor(
        position_beats: f64,
        beats_per_bar: u64,
        bars_per_phrase: u64,
        anchor: TransportBarGridAnchor,
    ) -> Self {
        let beat_cursor = zero_based_beat_cursor_index(position_beats);
        let beats_per_bar = beats_per_bar.max(1);
        let bars_per_phrase = bars_per_phrase.max(1);
        let bar_index = if beat_cursor >= anchor.beat_cursor {
            anchor
                .bar_index
                .saturating_add((beat_cursor - anchor.beat_cursor) / beats_per_bar)
        } else {
            let beats_before_anchor = anchor.beat_cursor - beat_cursor;
            let bars_before_anchor = beats_before_anchor.div_ceil(beats_per_bar);
            anchor.bar_index.saturating_sub(bars_before_anchor).max(1)
        }
        .max(1);

        Self {
            beat_cursor,
            bar_index,
            phrase_index: (bar_index - 1) / bars_per_phrase + 1,
        }
    }
}

impl TransportBarGridAnchor {
    #[must_use]
    pub fn beat_cursor_for_bar(self, bar_index: u64, beats_per_bar: u64) -> Option<u64> {
        let beats_per_bar = beats_per_bar.max(1);
        if bar_index >= self.bar_index {
            return Some(
                self.beat_cursor
                    .saturating_add((bar_index - self.bar_index).saturating_mul(beats_per_bar)),
            );
        }
        self.beat_cursor
            .checked_sub((self.bar_index - bar_index).saturating_mul(beats_per_bar))
    }

    /// Returns the first phase-aligned bar cursor strictly after the current
    /// continuous transport position.
    #[must_use]
    pub fn next_bar_beat_cursor_after(self, position_beats: f64, beats_per_bar: u64) -> u64 {
        let beats_per_bar = beats_per_bar.max(1);
        let next_cursor = zero_based_beat_cursor_index(position_beats).saturating_add(1);
        if next_cursor <= self.beat_cursor {
            return self.beat_cursor;
        }
        let phase_distance = next_cursor - self.beat_cursor;
        self.beat_cursor
            .saturating_add(phase_distance.div_ceil(beats_per_bar) * beats_per_bar)
    }
}

#[must_use]
pub fn zero_based_beat_cursor_index(position_beats: f64) -> u64 {
    if position_beats.is_finite() {
        position_beats.max(0.0).floor() as u64
    } else {
        0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransportClockState {
    pub is_playing: bool,
    pub position_beats: f64,
    /// Zero-based integral beat cursor retained by Session V1.
    pub beat_index: u64,
    /// One-based musical bar identity.
    pub bar_index: u64,
    /// One-based musical phrase identity.
    pub phrase_index: u64,
    pub current_scene: Option<SceneId>,
}

impl Default for TransportClockState {
    fn default() -> Self {
        Self {
            is_playing: false,
            position_beats: 0.0,
            beat_index: 0,
            bar_index: 1,
            phrase_index: 1,
            current_scene: None,
        }
    }
}

impl TransportClockState {
    #[must_use]
    pub fn boundary_state(&self, kind: CommitBoundary) -> CommitBoundaryState {
        CommitBoundaryState {
            kind,
            beat_index: self.beat_index,
            bar_index: self.bar_index,
            phrase_index: self.phrase_index,
            scene_id: self.current_scene.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitBoundaryState {
    pub kind: CommitBoundary,
    /// Session V1 compatibility field: zero-based integral transport beat cursor.
    pub beat_index: u64,
    /// One-based musical bar identity.
    pub bar_index: u64,
    /// One-based musical phrase identity.
    pub phrase_index: u64,
    pub scene_id: Option<SceneId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_clock_uses_the_first_one_based_bar_and_phrase() {
        let clock = TransportClockState::default();

        assert_eq!(clock.position_beats, 0.0);
        assert_eq!(clock.beat_index, 0);
        assert_eq!(clock.bar_index, 1);
        assert_eq!(clock.phrase_index, 1);
    }

    #[test]
    fn derives_boundary_state_from_transport_clock() {
        let clock = TransportClockState {
            is_playing: true,
            position_beats: 16.0,
            beat_index: 64,
            bar_index: 17,
            phrase_index: 3,
            current_scene: Some(SceneId::from("scene-a")),
        };

        let boundary = clock.boundary_state(CommitBoundary::Bar);

        assert_eq!(boundary.kind, CommitBoundary::Bar);
        assert_eq!(boundary.beat_index, 64);
        assert_eq!(boundary.bar_index, 17);
        assert_eq!(boundary.phrase_index, 3);
        assert_eq!(
            boundary.scene_id.as_ref().map(ToString::to_string),
            Some("scene-a".into())
        );
    }

    #[test]
    fn maps_zero_based_cursor_to_one_based_bar_and_phrase_boundaries() {
        for (position_beats, beat_cursor, bar_index, phrase_index) in [
            (0.0, 0, 1, 1),
            (3.999, 3, 1, 1),
            (4.0, 4, 2, 1),
            (8.0, 8, 3, 1),
            (16.0, 16, 5, 2),
        ] {
            assert_eq!(
                TransportGridPosition::from_zero_based_position_beats(
                    position_beats,
                    DEFAULT_BEATS_PER_BAR,
                    DEFAULT_BARS_PER_PHRASE,
                ),
                TransportGridPosition {
                    beat_cursor,
                    bar_index,
                    phrase_index,
                },
                "position {position_beats}",
            );
        }
    }

    #[test]
    fn maps_transport_against_a_nonzero_source_downbeat_phase() {
        let anchor = TransportBarGridAnchor {
            beat_cursor: 3,
            bar_index: 1,
        };

        for (position_beats, bar_index, phrase_index) in [
            (0.0, 1, 1),
            (3.0, 1, 1),
            (6.999, 1, 1),
            (7.0, 2, 1),
            (19.0, 5, 2),
            (20.0, 5, 2),
        ] {
            let position = TransportGridPosition::from_zero_based_position_beats_with_bar_anchor(
                position_beats,
                4,
                4,
                anchor,
            );
            assert_eq!(position.bar_index, bar_index, "position {position_beats}");
            assert_eq!(
                position.phrase_index, phrase_index,
                "position {position_beats}"
            );
        }
    }

    #[test]
    fn rounds_next_bar_to_the_source_downbeat_phase() {
        let anchor = TransportBarGridAnchor {
            beat_cursor: 3,
            bar_index: 1,
        };

        assert_eq!(anchor.next_bar_beat_cursor_after(0.0, 4), 3);
        assert_eq!(anchor.next_bar_beat_cursor_after(3.0, 4), 7);
        assert_eq!(anchor.next_bar_beat_cursor_after(18.2, 4), 19);
        assert_eq!(anchor.next_bar_beat_cursor_after(19.0, 4), 23);
        assert_eq!(anchor.beat_cursor_for_bar(1, 4), Some(3));
        assert_eq!(anchor.beat_cursor_for_bar(5, 4), Some(19));
    }

    #[test]
    fn preserves_session_v1_zero_based_commit_boundary_beat_index() {
        let boundary: CommitBoundaryState = serde_json::from_str(
            r#"{"kind":"Bar","beat_index":36,"bar_index":9,"phrase_index":2,"scene_id":null}"#,
        )
        .expect("legacy Session V1 commit boundary");

        assert_eq!(boundary.beat_index, 36);
        assert_eq!(boundary.bar_index, 9);
        assert_eq!(boundary.phrase_index, 2);
    }
}
