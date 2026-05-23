const SOURCE_MAP_WIDTH: usize = 32;
const SOURCE_MAP_BLOCKS: [char; 5] = ['▁', '▂', '▅', '▇', '█'];

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
    pub playhead_column: Option<usize>,
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
            playhead_column: None,
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
        let mode = if source_map_can_use_bar_grid(&timing, &bars) {
            SourceMapModeView::BarGrid
        } else {
            SourceMapModeView::TimeFallback
        };
        let playhead_column = source_map_playhead_column(graph, session);

        Self {
            mode,
            trust_label: timing.cue,
            width: SOURCE_MAP_WIDTH,
            energy_row: source_map_energy_row(graph),
            peak_row: source_map_peak_row(graph),
            grid_row: source_map_grid_row(graph, mode, &bars),
            playhead_row: source_map_playhead_row(playhead_column),
            playhead_column,
            capture_hint: source_map_capture_hint(mode),
            section_labels: source_map_section_labels(graph),
        }
    }
}

fn source_map_can_use_bar_grid(
    timing: &SourceTimingSummaryView,
    bars: &[crate::source_graph::BarSpan],
) -> bool {
    timing.grid_use == "locked_grid" && !bars.is_empty()
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

fn source_map_capture_hint(mode: SourceMapModeView) -> String {
    match mode {
        SourceMapModeView::BarGrid => "cap next bar | map bar grid | 32 cols".into(),
        SourceMapModeView::TimeFallback => {
            "cap listen first | map time fallback | no bar-accurate claim".into()
        }
        SourceMapModeView::Missing => "cap unavailable | no source graph".into(),
    }
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

fn source_map_playhead_column(graph: &SourceGraph, session: &SessionFile) -> Option<usize> {
    let bpm = graph.timing.bpm_estimate?;
    if bpm <= 0.0 || !bpm.is_finite() || graph.source.duration_seconds <= 0.0 {
        return None;
    }

    let position_seconds = (session.transport().position_beats as f32 * 60.0 / bpm)
        .clamp(0.0, graph.source.duration_seconds);
    Some(source_map_column_for_time(graph, position_seconds))
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
