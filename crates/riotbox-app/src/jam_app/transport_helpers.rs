use riotbox_core::view::jam::source_timing_consumer_readiness;
use riotbox_core::{
    ids::SceneId,
    session::SessionFile,
    source_graph::{SectionLabelHint, SourceGraph},
    transport::{
        CommitBoundaryState, DEFAULT_BARS_PER_PHRASE, DEFAULT_BEATS_PER_BAR, TransportClockState,
        TransportGridPosition,
    },
};

pub(in crate::jam_app) fn trusted_source_timing_bpm(
    session: &SessionFile,
    source_graph: Option<&SourceGraph>,
) -> Option<f32> {
    let graph = source_graph?;
    source_timing_consumer_readiness(Some(graph), session)
        .can_use_source_window_grid()
        .then(|| {
            graph
                .timing
                .primary_hypothesis()
                .map(|hypothesis| hypothesis.bpm)
                .or(graph.timing.bpm_estimate)
        })
        .flatten()
        .filter(|bpm| bpm.is_finite() && *bpm > 0.0)
}

pub(in crate::jam_app) fn normalize_scene_candidates(
    session: &mut SessionFile,
    source_graph: Option<&SourceGraph>,
) {
    if session.runtime_state.scene_state.scenes.is_empty()
        && let Some(graph) = source_graph
    {
        session.runtime_state.scene_state.scenes = derive_scene_candidates(graph);
    }

    if session.runtime_state.scene_state.active_scene.is_none() {
        session.runtime_state.scene_state.active_scene =
            session.runtime_state.transport.current_scene.clone();
    }

    if session.runtime_state.transport.current_scene.is_none() {
        session.runtime_state.transport.current_scene =
            session.runtime_state.scene_state.active_scene.clone();
    }

    if session.runtime_state.scene_state.active_scene.is_none()
        && let Some(first_scene) = session.runtime_state.scene_state.scenes.first().cloned()
    {
        session.runtime_state.scene_state.active_scene = Some(first_scene.clone());
        session.runtime_state.transport.current_scene = Some(first_scene);
    }
}

fn derive_scene_candidates(graph: &SourceGraph) -> Vec<SceneId> {
    let mut sections = graph.sections.iter().collect::<Vec<_>>();
    sections.sort_by(|left, right| {
        left.bar_start
            .cmp(&right.bar_start)
            .then(left.bar_end.cmp(&right.bar_end))
            .then(left.section_id.as_str().cmp(right.section_id.as_str()))
    });

    sections
        .into_iter()
        .enumerate()
        .map(|(index, section)| {
            SceneId::from(format!(
                "scene-{:02}-{}",
                index + 1,
                section_label_slug(section.label_hint)
            ))
        })
        .collect()
}

const fn section_label_slug(label: SectionLabelHint) -> &'static str {
    match label {
        SectionLabelHint::Intro => "intro",
        SectionLabelHint::Build => "build",
        SectionLabelHint::Drop => "drop",
        SectionLabelHint::Break => "break",
        SectionLabelHint::Verse => "verse",
        SectionLabelHint::Chorus => "chorus",
        SectionLabelHint::Bridge => "bridge",
        SectionLabelHint::Outro => "outro",
        SectionLabelHint::Unknown => "unknown",
    }
}

pub(in crate::jam_app) fn transport_clock_from_state(
    session: &SessionFile,
    source_graph: Option<&SourceGraph>,
) -> TransportClockState {
    let trusted_graph =
        source_graph.filter(|_| trusted_source_timing_bpm(session, source_graph).is_some());
    transport_clock_for_state(
        session.runtime_state.transport.position_beats,
        session.runtime_state.transport.is_playing,
        session.runtime_state.transport.current_scene.clone(),
        trusted_graph,
    )
}

pub(in crate::jam_app) fn transport_clock_for_state(
    position_beats: f64,
    is_playing: bool,
    current_scene: Option<SceneId>,
    source_graph: Option<&SourceGraph>,
) -> TransportClockState {
    let grid_position = source_graph
        .and_then(|graph| graph.timing.primary_hypothesis())
        .and_then(|hypothesis| hypothesis.transport_grid_position(position_beats))
        .unwrap_or_else(|| {
            let beats_per_bar = source_graph
                .and_then(|graph| graph.timing.meter_hint)
                .map(|meter| u64::from(meter.beats_per_bar))
                .filter(|beats| *beats > 0)
                .unwrap_or(DEFAULT_BEATS_PER_BAR);
            TransportGridPosition::from_zero_based_position_beats(
                position_beats,
                beats_per_bar,
                DEFAULT_BARS_PER_PHRASE,
            )
        });
    let phrase_index = source_graph
        .and_then(|graph| {
            let phrase_grid = graph
                .timing
                .primary_hypothesis()
                .map(|hypothesis| hypothesis.phrase_grid.as_slice())
                .filter(|phrases| !phrases.is_empty())
                .unwrap_or(graph.timing.phrase_grid.as_slice());
            phrase_grid
                .iter()
                .find(|phrase| {
                    let start_bar = u64::from(phrase.start_bar);
                    let end_bar = u64::from(phrase.end_bar);
                    grid_position.bar_index >= start_bar && grid_position.bar_index <= end_bar
                })
                .map(|phrase| u64::from(phrase.phrase_index))
        })
        .unwrap_or(grid_position.phrase_index);

    TransportClockState {
        is_playing,
        position_beats,
        beat_index: grid_position.beat_cursor,
        bar_index: grid_position.bar_index,
        phrase_index,
        current_scene,
    }
}

pub(in crate::jam_app) fn crossed_commit_boundary(
    previous: &TransportClockState,
    next: &TransportClockState,
) -> Option<CommitBoundaryState> {
    if next.phrase_index > previous.phrase_index {
        return Some(next.boundary_state(riotbox_core::action::CommitBoundary::Phrase));
    }

    if next.bar_index > previous.bar_index {
        return Some(next.boundary_state(riotbox_core::action::CommitBoundary::Bar));
    }

    if next.beat_index > previous.beat_index {
        return Some(next.boundary_state(riotbox_core::action::CommitBoundary::Beat));
    }

    None
}
