//! Existing QA trigger data shared by trigger policy and accent evidence.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct W30SourceTriggerEvent {
    pub(super) beat_position: f32,
    pub(super) velocity: f32,
    pub(super) source_energy_score: f32,
    pub(super) source_offset_samples: usize,
}
