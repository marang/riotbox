use riotbox_core::source_graph::{
    BeatPoint, BarSpan, MeterHint, PhraseSpan, TimingHypothesis, TimingHypothesisKind,
    TimingModel, TimingQuality,
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
    let phrase_grid = vec![
        PhraseSpan {
            phrase_index: 1,
            start_bar: 1,
            end_bar: 4,
            confidence: 0.92,
        },
        PhraseSpan {
            phrase_index: 2,
            start_bar: 5,
            end_bar: 8,
            confidence: 0.92,
        },
        PhraseSpan {
            phrase_index: 3,
            start_bar: 9,
            end_bar: 12,
            confidence: 0.92,
        },
    ];
    let bar_grid = (1..=12)
        .map(|bar_index| BarSpan {
            bar_index,
            start_seconds: (bar_index - 1) as f32 * 60.0 / DEFAULT_BPM * BEATS_PER_BAR as f32,
            end_seconds: bar_index as f32 * 60.0 / DEFAULT_BPM * BEATS_PER_BAR as f32,
            downbeat_confidence: 0.90,
            phrase_index: phrase_grid
                .iter()
                .find(|phrase| bar_index >= phrase.start_bar && bar_index <= phrase.end_bar)
                .map(|phrase| phrase.phrase_index),
        })
        .collect::<Vec<_>>();
    let beat_grid = (0..48)
        .map(|beat_index| BeatPoint {
            beat_index,
            time_seconds: beat_index as f32 * 60.0 / DEFAULT_BPM,
            confidence: 0.90,
        })
        .collect::<Vec<_>>();
    let meter = MeterHint {
        beats_per_bar: BEATS_PER_BAR as u8,
        beat_unit: 4,
    };

    TimingModel {
        bpm_estimate: Some(DEFAULT_BPM),
        bpm_confidence: 0.90,
        meter_hint: Some(meter),
        beat_grid: beat_grid.clone(),
        bar_grid: bar_grid.clone(),
        phrase_grid: phrase_grid.clone(),
        hypotheses: vec![TimingHypothesis {
            hypothesis_id: "lane-recipe-source-grid".to_string(),
            kind: TimingHypothesisKind::Primary,
            bpm: DEFAULT_BPM,
            meter,
            confidence: 0.90,
            score: 1.0,
            beat_grid,
            bar_grid,
            phrase_grid,
            anchors: Vec::new(),
            drift: Vec::new(),
            groove: Vec::new(),
            quality: TimingQuality::High,
            warnings: Vec::new(),
            provenance: vec!["lane-recipe-pack.synthetic-source-phrase-grid.v0".to_string()],
        }],
        primary_hypothesis_id: Some("lane-recipe-source-grid".to_string()),
        quality: TimingQuality::High,
        warnings: Vec::new(),
        degraded_policy: riotbox_core::source_graph::TimingDegradedPolicy::Locked,
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
