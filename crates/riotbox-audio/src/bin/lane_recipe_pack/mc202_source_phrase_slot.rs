use riotbox_core::source_graph::{
    MeterHint, PhraseSpan, SourceTimingProbeBpmCandidatePolicy, TimingModel,
    timing_model_from_probe_bpm_candidates,
};

use riotbox_audio::{
    source_audio::SourceAudioCache,
    source_timing_probe::{SourceTimingProbeConfig, analyze_source_timing_probe},
};

const MC202_SOURCE_PHRASE_SLOT_CONTRACT: &str = "source_graph_phrase_grid.v0";
const LANE_RECIPE_SOURCE_TIMING_SAMPLE_RATE: u32 = 4_096;
const LANE_RECIPE_SOURCE_TIMING_CHANNELS: u16 = 1;
const LANE_RECIPE_SOURCE_TIMING_IMPULSE_FRAMES: usize = 64;

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
    let source = lane_recipe_source_timing_audio();
    let probe = analyze_source_timing_probe(
        &source,
        SourceTimingProbeConfig {
            window_size_frames: 256,
            hop_size_frames: 256,
            onset_threshold_ratio: 0.20,
            min_onset_flux: 0.01,
        },
    );
    let candidate_input = probe.bpm_candidate_input(
        "lane-recipe-pack.generated-source-audio-phrase-grid.v1",
        MeterHint {
            beats_per_bar: BEATS_PER_BAR as u8,
            beat_unit: 4,
        },
    );

    timing_model_from_probe_bpm_candidates(
        &candidate_input,
        SourceTimingProbeBpmCandidatePolicy {
            min_bpm: 100.0,
            max_bpm: 150.0,
            ..SourceTimingProbeBpmCandidatePolicy::default()
        },
    )
}

fn lane_recipe_source_timing_audio() -> SourceAudioCache {
    SourceAudioCache::from_interleaved_samples(
        "lane-recipe-generated-source-audio.wav",
        LANE_RECIPE_SOURCE_TIMING_SAMPLE_RATE,
        LANE_RECIPE_SOURCE_TIMING_CHANNELS,
        lane_recipe_source_timing_samples(),
    )
    .expect("generated lane recipe source audio must be valid")
}

fn lane_recipe_source_timing_samples() -> Vec<f32> {
    let frames_per_beat =
        (LANE_RECIPE_SOURCE_TIMING_SAMPLE_RATE as f32 * 60.0 / DEFAULT_BPM).round() as usize;
    let total_beats = 12 * BEATS_PER_BAR;
    let mut samples = vec![0.0; frames_per_beat * total_beats as usize];
    for beat in 0..total_beats {
        let amplitude = if beat % BEATS_PER_BAR == 0 { 1.0 } else { 0.36 };
        add_impulse(
            &mut samples,
            beat as usize * frames_per_beat,
            LANE_RECIPE_SOURCE_TIMING_IMPULSE_FRAMES,
            amplitude,
        );
    }
    samples
}

fn add_impulse(samples: &mut [f32], start: usize, impulse_frames: usize, amplitude: f32) {
    let end = start.saturating_add(impulse_frames).min(samples.len());
    for sample in samples.iter_mut().take(end).skip(start) {
        *sample = amplitude;
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
