use riotbox_audio::mc202::Mc202RenderState;
use riotbox_audio::runtime::OfflineAudioMetrics;
use riotbox_audio::tr909::Tr909RenderState;
use serde::Serialize;

#[derive(Clone, Debug)]
pub(super) struct PackCase {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) recipe_refs: &'static str,
    pub(super) baseline_label: &'static str,
    pub(super) candidate_label: &'static str,
    pub(super) render_pair: RenderPair,
    pub(super) min_rms_delta: f32,
    pub(super) min_signal_delta_rms: f32,
    pub(super) note: &'static str,
}

#[derive(Clone, Debug)]
pub(super) enum RenderPair {
    Tr909 {
        baseline: Tr909RenderState,
        candidate: Tr909RenderState,
    },
    Mc202 {
        baseline: Mc202RenderState,
        candidate: Mc202RenderState,
    },
}

#[derive(Debug)]
pub(super) struct CaseReport {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) recipe_refs: &'static str,
    pub(super) baseline_label: &'static str,
    pub(super) candidate_label: &'static str,
    pub(super) baseline_metrics: OfflineAudioMetrics,
    pub(super) candidate_metrics: OfflineAudioMetrics,
    pub(super) signal_delta_metrics: OfflineAudioMetrics,
    pub(super) mc202_phrase_grid: Option<Mc202PhraseGridTimingMetrics>,
    pub(super) mc202_source_phrase_slot: Option<Mc202SourcePhraseSlotMetrics>,
    pub(super) min_rms_delta: f32,
    pub(super) min_signal_delta_rms: f32,
    pub(super) passed: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub(super) struct Mc202PhraseGridTimingMetrics {
    pub(super) resolution: &'static str,
    pub(super) phrase_length_steps: u32,
    pub(super) phrase_length_beats: f64,
    pub(super) position_beats: f64,
    pub(super) starts_on_phrase_boundary: bool,
    pub(super) candidate_onset_count: usize,
    pub(super) grid_aligned_onset_count: usize,
    pub(super) hit_ratio: f64,
    pub(super) max_onset_offset_ms: f64,
    pub(super) max_allowed_onset_offset_ms: f64,
    pub(super) passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct Mc202SourcePhraseSlotMetrics {
    pub(super) contract: &'static str,
    pub(super) source_hypothesis_id: Option<String>,
    pub(super) phrase_grid_available: bool,
    pub(super) phrase_index: Option<u32>,
    pub(super) phrase_start_bar: Option<u32>,
    pub(super) phrase_end_bar: Option<u32>,
    pub(super) candidate_position_beats: f64,
    pub(super) candidate_bar_index: u32,
    pub(super) starts_on_source_phrase_boundary: bool,
    pub(super) passed: bool,
}
