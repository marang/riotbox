use serde::{Deserialize, Serialize};

use crate::{action::CommitBoundary, ids::SceneId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransportClockState {
    pub is_playing: bool,
    pub position_beats: f64,
    pub beat_index: u64,
    pub bar_index: u64,
    pub phrase_index: u64,
    pub current_scene: Option<SceneId>,
}

impl Default for TransportClockState {
    fn default() -> Self {
        Self {
            is_playing: false,
            position_beats: 0.0,
            beat_index: 0,
            bar_index: 0,
            phrase_index: 0,
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
    pub beat_index: u64,
    pub bar_index: u64,
    pub phrase_index: u64,
    pub scene_id: Option<SceneId>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
