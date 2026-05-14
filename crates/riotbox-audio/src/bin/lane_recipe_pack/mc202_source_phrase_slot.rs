use riotbox_core::source_graph::{
    MeterHint, PhraseSpan, SourceTimingProbeBpmCandidateInput,
    SourceTimingProbeBpmCandidatePolicy, TimingModel, timing_model_from_probe_bpm_candidates,
};

const MC202_SOURCE_PHRASE_SLOT_CONTRACT: &str = "source_graph_phrase_grid.v0";

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Mc202SourcePhraseSlotMetrics {
    contract: &'static str,
    source_hypothesis_id: Option<String>,
    phrase_grid_available: bool,
    phrase_index: Option<u32>,
    phrase_start_bar: Option<u32>,
    phrase_end_bar: Option<u32>,
    candidate_position_beats: f64,
    candidate_bar_index: u32,
    starts_on_source_phrase_boundary: bool,
    passed: bool,
}

fn mc202_source_phrase_slot_metrics(
    render_pair: &RenderPair,
    source_timing: &TimingModel,
) -> Option<Mc202SourcePhraseSlotMetrics> {
    let RenderPair::Mc202 { candidate, .. } = render_pair else {
        return None;
    };

    let phrase_grid = primary_phrase_grid(source_timing);
    let candidate_bar_index = bar_index_for_position(candidate.position_beats);
    let selected_phrase = phrase_grid
        .iter()
        .find(|phrase| candidate_bar_index >= phrase.start_bar && candidate_bar_index <= phrase.end_bar);
    let starts_on_source_phrase_boundary = selected_phrase.is_some_and(|phrase| {
        candidate_bar_index == phrase.start_bar
            && phrase_boundary_offset_beats(candidate.position_beats, phrase.start_bar) <= 0.0001
    });

    Some(Mc202SourcePhraseSlotMetrics {
        contract: MC202_SOURCE_PHRASE_SLOT_CONTRACT,
        source_hypothesis_id: source_timing.primary_hypothesis_id.clone(),
        phrase_grid_available: !phrase_grid.is_empty(),
        phrase_index: selected_phrase.map(|phrase| phrase.phrase_index),
        phrase_start_bar: selected_phrase.map(|phrase| phrase.start_bar),
        phrase_end_bar: selected_phrase.map(|phrase| phrase.end_bar),
        candidate_position_beats: candidate.position_beats,
        candidate_bar_index,
        starts_on_source_phrase_boundary,
        passed: selected_phrase.is_some() && starts_on_source_phrase_boundary,
    })
}

fn lane_recipe_source_timing_model() -> TimingModel {
    timing_model_from_probe_bpm_candidates(
        &lane_recipe_source_timing_candidate_input(),
        SourceTimingProbeBpmCandidatePolicy {
            min_bpm: 100.0,
            max_bpm: 150.0,
            ..SourceTimingProbeBpmCandidatePolicy::default()
        },
    )
}

fn lane_recipe_source_timing_candidate_input() -> SourceTimingProbeBpmCandidateInput {
    let seconds_per_beat = 60.0 / DEFAULT_BPM;
    let total_beats = 12 * BEATS_PER_BAR;
    let onset_times_seconds = (0..total_beats)
        .map(|beat| beat as f32 * seconds_per_beat)
        .collect::<Vec<_>>();
    let onset_strengths = (0..total_beats)
        .map(|beat| if beat % BEATS_PER_BAR == 0 { 1.0 } else { 0.36 })
        .collect::<Vec<_>>();

    SourceTimingProbeBpmCandidateInput {
        source_id: "lane-recipe-pack.generated-source-phrase-grid.v1".to_string(),
        duration_seconds: total_beats as f32 * seconds_per_beat,
        onset_times_seconds,
        onset_strengths,
        meter: MeterHint {
            beats_per_bar: BEATS_PER_BAR as u8,
            beat_unit: 4,
        },
    }
}

fn primary_phrase_grid(source_timing: &TimingModel) -> &[PhraseSpan] {
    source_timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.phrase_grid.as_slice())
        .filter(|phrase_grid| !phrase_grid.is_empty())
        .unwrap_or(source_timing.phrase_grid.as_slice())
}

fn bar_index_for_position(position_beats: f64) -> u32 {
    (position_beats / f64::from(BEATS_PER_BAR)).floor().max(0.0) as u32 + 1
}

fn phrase_boundary_offset_beats(position_beats: f64, phrase_start_bar: u32) -> f64 {
    let phrase_start_beats = f64::from(phrase_start_bar.saturating_sub(1) * BEATS_PER_BAR);
    (position_beats - phrase_start_beats).abs()
}
