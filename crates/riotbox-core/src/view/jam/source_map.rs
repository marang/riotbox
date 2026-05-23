const SOURCE_MAP_WIDTH: usize = 32;
const SOURCE_MAP_BLOCKS: [char; 5] = ['▁', '▂', '▅', '▇', '█'];
const SOURCE_MAP_CAPTURE_RANGE_FILL: char = '=';

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SourceMapModeView {
    BarGrid,
    TimeFallback,
    Missing,
}

impl SourceMapModeView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BarGrid => "bar grid",
            Self::TimeFallback => "time fallback",
            Self::Missing => "missing",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceMapView {
    pub mode: SourceMapModeView,
    pub trust_label: String,
    pub width: usize,
    pub energy_row: String,
    pub peak_row: String,
    pub grid_row: String,
    pub playhead_row: String,
    pub capture_range_row: String,
    pub playhead_column: Option<usize>,
    pub current_region_label: String,
    pub navigation_hint: String,
    pub capture_hint: String,
    pub section_labels: Vec<String>,
}

impl Default for SourceMapView {
    fn default() -> Self {
        Self {
            mode: SourceMapModeView::Missing,
            trust_label: "not available".into(),
            width: SOURCE_MAP_WIDTH,
            energy_row: SOURCE_MAP_BLOCKS[0].to_string().repeat(SOURCE_MAP_WIDTH),
            peak_row: ".".repeat(SOURCE_MAP_WIDTH),
            grid_row: ".".repeat(SOURCE_MAP_WIDTH),
            playhead_row: " ".repeat(SOURCE_MAP_WIDTH),
            capture_range_row: ".".repeat(SOURCE_MAP_WIDTH),
            playhead_column: None,
            current_region_label: "now unavailable".into(),
            navigation_hint: "nav unavailable".into(),
            capture_hint: "cap unavailable | no source graph".into(),
            section_labels: Vec::new(),
        }
    }
}

impl SourceMapView {
    #[must_use]
    pub fn from_graph(graph: &SourceGraph, session: &SessionFile) -> Self {
        let timing = SourceTimingSummaryView::from_graph(graph);
        let bars = source_map_bar_spans(graph);
        let readiness = source_timing_consumer_readiness(Some(graph), session);
        let mode = if source_map_can_use_bar_grid(&bars, readiness) {
            SourceMapModeView::BarGrid
        } else {
            SourceMapModeView::TimeFallback
        };
        let playhead_column = source_map_playhead_column(graph, session);

        Self {
            mode,
            trust_label: source_map_trust_label(&timing, readiness),
            width: SOURCE_MAP_WIDTH,
            energy_row: source_map_energy_row(graph),
            peak_row: source_map_peak_row(graph),
            grid_row: source_map_grid_row(graph, mode, &bars),
            playhead_row: source_map_playhead_row(playhead_column),
            capture_range_row: source_map_capture_range_row(graph, session, mode),
            playhead_column,
            current_region_label: source_map_current_region_label(graph, session),
            navigation_hint: source_map_navigation_hint(graph, mode),
            capture_hint: source_map_capture_hint(mode),
            section_labels: source_map_section_labels(graph),
        }
    }
}

fn source_map_can_use_bar_grid(
    bars: &[crate::source_graph::BarSpan],
    readiness: SourceTimingConsumerReadiness,
) -> bool {
    readiness.can_use_source_window_grid() && !bars.is_empty()
}

fn source_map_trust_label(
    timing: &SourceTimingSummaryView,
    readiness: SourceTimingConsumerReadiness,
) -> String {
    match readiness {
        SourceTimingConsumerReadiness::UserConfirmed => "grid confirmed".into(),
        _ => timing.cue.clone(),
    }
}

fn source_map_energy_row(graph: &SourceGraph) -> String {
    (0..SOURCE_MAP_WIDTH)
        .map(|column| {
            let time = source_map_column_midpoint_seconds(graph, column);
            let energy = graph
                .sections
                .iter()
                .find(|section| time >= section.start_seconds && time < section.end_seconds)
                .map_or(EnergyClass::Unknown, |section| section.energy_class);
            source_map_energy_block(energy)
        })
        .collect()
}

fn source_map_peak_row(graph: &SourceGraph) -> String {
    (0..SOURCE_MAP_WIDTH)
        .map(|column| {
            let start = source_map_column_start_seconds(graph, column);
            let end = source_map_column_end_seconds(graph, column);
            if source_map_bucket_has_anchor(graph, start, end) {
                SOURCE_MAP_BLOCKS[4]
            } else if source_map_bucket_has_asset(graph, start, end) {
                SOURCE_MAP_BLOCKS[3]
            } else {
                '.'
            }
        })
        .collect()
}

fn source_map_grid_row(
    graph: &SourceGraph,
    mode: SourceMapModeView,
    bars: &[crate::source_graph::BarSpan],
) -> String {
    if mode != SourceMapModeView::BarGrid {
        return ".".repeat(SOURCE_MAP_WIDTH);
    }

    let mut row = vec!['.'; SOURCE_MAP_WIDTH];
    for bar in bars {
        let column = source_map_column_for_time(graph, bar.start_seconds);
        row[column] = '|';
    }
    row.into_iter().collect()
}

fn source_map_playhead_row(playhead_column: Option<usize>) -> String {
    let mut row = vec![' '; SOURCE_MAP_WIDTH];
    if let Some(column) = playhead_column {
        row[column] = '^';
    }
    row.into_iter().collect()
}

fn source_map_capture_range_row(
    graph: &SourceGraph,
    session: &SessionFile,
    mode: SourceMapModeView,
) -> String {
    if mode != SourceMapModeView::BarGrid {
        return ".".repeat(SOURCE_MAP_WIDTH);
    }

    let Some((start_seconds, end_seconds)) = source_map_capture_range_seconds(graph, session) else {
        return ".".repeat(SOURCE_MAP_WIDTH);
    };
    let start_column = source_map_column_for_time(graph, start_seconds);
    let end_column = source_map_column_for_time(graph, end_seconds);
    let (left, right) = if start_column <= end_column {
        (start_column, end_column)
    } else {
        (end_column, start_column)
    };

    let mut row = vec!['.'; SOURCE_MAP_WIDTH];
    for cell in row.iter_mut().take(right + 1).skip(left) {
        *cell = SOURCE_MAP_CAPTURE_RANGE_FILL;
    }
    row[left] = '[';
    row[right] = ']';
    if left == right {
        row[left] = '*';
    }
    row.into_iter().collect()
}

fn source_map_capture_range_seconds(
    graph: &SourceGraph,
    session: &SessionFile,
) -> Option<(f32, f32)> {
    let start_beat = source_map_next_bar_capture_start_beat(graph, session);
    let end_beat = source_map_capture_end_beat(graph, session, start_beat)?;
    let start_seconds = source_map_seconds_for_beat(graph, start_beat)?
        .max(0.0)
        .min(graph.source.duration_seconds);
    let end_seconds = source_map_seconds_for_beat(graph, end_beat)
        .unwrap_or_else(|| source_map_seconds_for_beat_estimate(graph, end_beat))
        .min(graph.source.duration_seconds)
        .max(start_seconds);
    Some((start_seconds, end_seconds))
}

fn source_map_next_bar_capture_start_beat(graph: &SourceGraph, session: &SessionFile) -> u64 {
    let beats_per_bar = source_map_beats_per_bar(graph).max(1);
    let next_beat_after_position = (session.transport().position_beats.floor().max(0.0) as u64)
        .saturating_add(1);
    let remainder = next_beat_after_position % beats_per_bar;
    if remainder == 0 {
        next_beat_after_position
    } else {
        next_beat_after_position.saturating_add(beats_per_bar - remainder)
    }
}

fn source_map_capture_end_beat(
    graph: &SourceGraph,
    session: &SessionFile,
    start_beat: u64,
) -> Option<u64> {
    let beats_per_bar = source_map_beats_per_bar(graph);
    Some(match session.runtime_state.capture.length_intent {
        crate::action::CaptureLengthIntent::OneBeat => start_beat.saturating_add(1),
        crate::action::CaptureLengthIntent::OneBar => start_beat.saturating_add(beats_per_bar),
        crate::action::CaptureLengthIntent::FourBars => {
            start_beat.saturating_add(beats_per_bar.saturating_mul(4))
        }
        crate::action::CaptureLengthIntent::Phrase => {
            source_map_phrase_capture_end_beat(graph, start_beat, beats_per_bar)
                .unwrap_or_else(|| start_beat.saturating_add(beats_per_bar.saturating_mul(4)))
        }
    })
}

fn source_map_beats_per_bar(graph: &SourceGraph) -> u64 {
    graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.meter)
        .or(graph.timing.meter_hint)
        .map_or(4_u64, |meter| u64::from(meter.beats_per_bar))
}

fn source_map_phrase_capture_end_beat(
    graph: &SourceGraph,
    start_beat: u64,
    beats_per_bar: u64,
) -> Option<u64> {
    let start_bar = start_beat / beats_per_bar + 1;
    let phrase_grid = graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.phrase_grid.as_slice())
        .filter(|phrases| !phrases.is_empty())
        .unwrap_or(graph.timing.phrase_grid.as_slice());
    phrase_grid
        .iter()
        .find(|phrase| {
            u64::from(phrase.start_bar) <= start_bar && start_bar <= u64::from(phrase.end_bar)
        })
        .or_else(|| {
            phrase_grid
                .iter()
                .find(|phrase| u64::from(phrase.start_bar) >= start_bar)
        })
        .map(|phrase| u64::from(phrase.end_bar).saturating_mul(beats_per_bar))
        .filter(|end_beat| *end_beat > start_beat)
}

fn source_map_capture_hint(mode: SourceMapModeView) -> String {
    match mode {
        SourceMapModeView::BarGrid => "cap next bar | map bar grid | 32 cols".into(),
        SourceMapModeView::TimeFallback => {
            "cap listen first | map time fallback | no bar-accurate claim".into()
        }
        SourceMapModeView::Missing => "cap unavailable | no source graph".into(),
    }
}

fn source_map_navigation_hint(graph: &SourceGraph, mode: SourceMapModeView) -> String {
    if mode != SourceMapModeView::BarGrid {
        return "nav listen first | confirm grid before bar seek".into();
    }

    let phrase_count = graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.phrase_grid.len())
        .filter(|count| *count > 0)
        .or_else(|| (!graph.timing.phrase_grid.is_empty()).then_some(graph.timing.phrase_grid.len()))
        .unwrap_or_else(|| {
            source_map_bar_spans(graph)
                .into_iter()
                .filter_map(|bar| bar.phrase_index)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        });
    if phrase_count > 0 {
        "nav Left/Right bar | Up/Down phrase".into()
    } else {
        "nav Left/Right bar | phrase unavailable".into()
    }
}

fn source_map_current_region_label(graph: &SourceGraph, session: &SessionFile) -> String {
    let Some(position_seconds) = source_map_position_seconds(graph, session) else {
        return "now unavailable".into();
    };
    let bar = source_map_bar_at_time(graph, position_seconds)
        .map_or_else(|| "bar -".into(), |bar_index| format!("bar {bar_index}"));
    let section = source_map_section_at_time(graph, position_seconds)
        .map_or_else(|| "section -".into(), |(index, section)| {
            if section.confidence >= 0.8 && section.label_hint != SectionLabelHint::Unknown {
                source_map_section_label_hint(section.label_hint).into()
            } else {
                format!("section {}", source_map_section_letter(index))
            }
        });
    format!("now {bar} | {section}")
}

fn source_map_section_labels(graph: &SourceGraph) -> Vec<String> {
    sorted_sections(graph)
        .into_iter()
        .take(4)
        .enumerate()
        .map(|(index, section)| {
            let label = if section.confidence >= 0.8 && section.label_hint != SectionLabelHint::Unknown
            {
                source_map_section_label_hint(section.label_hint).into()
            } else {
                format!("section {}", source_map_section_letter(index))
            };
            format!("{label} {}-{}", section.bar_start, section.bar_end)
        })
        .collect()
}

fn source_map_bar_spans(graph: &SourceGraph) -> Vec<crate::source_graph::BarSpan> {
    graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.bar_grid.clone())
        .filter(|bars| !bars.is_empty())
        .unwrap_or_else(|| graph.timing.bar_grid.clone())
}

fn source_map_bar_at_time(graph: &SourceGraph, position_seconds: f32) -> Option<u32> {
    source_map_bar_spans(graph)
        .into_iter()
        .find(|bar| position_seconds >= bar.start_seconds && position_seconds < bar.end_seconds)
        .map(|bar| bar.bar_index)
}

fn source_map_section_at_time(
    graph: &SourceGraph,
    position_seconds: f32,
) -> Option<(usize, &crate::source_graph::Section)> {
    sorted_sections(graph)
        .into_iter()
        .enumerate()
        .find(|(_, section)| {
            position_seconds >= section.start_seconds && position_seconds < section.end_seconds
        })
}

fn source_map_playhead_column(graph: &SourceGraph, session: &SessionFile) -> Option<usize> {
    source_map_position_seconds(graph, session)
        .map(|position_seconds| source_map_column_for_time(graph, position_seconds))
}

fn source_map_position_seconds(graph: &SourceGraph, session: &SessionFile) -> Option<f32> {
    let bpm = graph.timing.bpm_estimate?;
    if bpm <= 0.0 || !bpm.is_finite() || graph.source.duration_seconds <= 0.0 {
        return None;
    }

    Some(
        (session.transport().position_beats as f32 * 60.0 / bpm)
            .clamp(0.0, graph.source.duration_seconds),
    )
}

fn source_map_seconds_for_beat(graph: &SourceGraph, beat_index: u64) -> Option<f32> {
    graph
        .timing
        .beat_grid
        .iter()
        .find(|beat| u64::from(beat.beat_index) == beat_index)
        .map(|beat| beat.time_seconds)
        .or_else(|| {
            graph
                .timing
                .bpm_estimate
                .map(|_| source_map_seconds_for_beat_estimate(graph, beat_index))
        })
}

fn source_map_seconds_for_beat_estimate(graph: &SourceGraph, beat_index: u64) -> f32 {
    let bpm = graph.timing.bpm_estimate.unwrap_or(120.0).max(1.0);
    beat_index as f32 * 60.0 / bpm
}

fn source_map_bucket_has_anchor(graph: &SourceGraph, start: f32, end: f32) -> bool {
    graph
        .timing
        .primary_hypothesis()
        .into_iter()
        .flat_map(|hypothesis| hypothesis.anchors.iter())
        .any(|anchor| anchor.time_seconds >= start && anchor.time_seconds < end)
}

fn source_map_bucket_has_asset(graph: &SourceGraph, start: f32, end: f32) -> bool {
    graph
        .assets
        .iter()
        .any(|asset| asset.start_seconds < end && asset.end_seconds > start)
}

fn source_map_column_midpoint_seconds(graph: &SourceGraph, column: usize) -> f32 {
    (source_map_column_start_seconds(graph, column) + source_map_column_end_seconds(graph, column))
        * 0.5
}

fn source_map_column_start_seconds(graph: &SourceGraph, column: usize) -> f32 {
    graph.source.duration_seconds.max(0.0) * column as f32 / SOURCE_MAP_WIDTH as f32
}

fn source_map_column_end_seconds(graph: &SourceGraph, column: usize) -> f32 {
    graph.source.duration_seconds.max(0.0) * (column + 1) as f32 / SOURCE_MAP_WIDTH as f32
}

fn source_map_column_for_time(graph: &SourceGraph, time_seconds: f32) -> usize {
    if graph.source.duration_seconds <= 0.0 {
        return 0;
    }
    let normalized = (time_seconds / graph.source.duration_seconds).clamp(0.0, 1.0);
    ((normalized * SOURCE_MAP_WIDTH as f32).floor() as usize).min(SOURCE_MAP_WIDTH - 1)
}

const fn source_map_energy_block(energy: EnergyClass) -> char {
    match energy {
        EnergyClass::Low => SOURCE_MAP_BLOCKS[1],
        EnergyClass::Medium => SOURCE_MAP_BLOCKS[2],
        EnergyClass::High => SOURCE_MAP_BLOCKS[3],
        EnergyClass::Peak => SOURCE_MAP_BLOCKS[4],
        EnergyClass::Unknown => SOURCE_MAP_BLOCKS[0],
    }
}

const fn source_map_section_letter(index: usize) -> char {
    match index {
        0 => 'A',
        1 => 'B',
        2 => 'C',
        3 => 'D',
        _ => '?',
    }
}

const fn source_map_section_label_hint(label_hint: SectionLabelHint) -> &'static str {
    match label_hint {
        SectionLabelHint::Intro => "intro",
        SectionLabelHint::Build => "build",
        SectionLabelHint::Drop => "drop",
        SectionLabelHint::Break => "break",
        SectionLabelHint::Verse => "verse",
        SectionLabelHint::Chorus => "chorus",
        SectionLabelHint::Bridge => "bridge",
        SectionLabelHint::Outro => "outro",
        SectionLabelHint::Unknown => "section",
    }
}

#[cfg(test)]
#[path = "source_map_tests.rs"]
mod source_map_tests;
