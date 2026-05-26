use riotbox_core::source_graph::{BarSpan, PhraseSpan};

#[test]
fn source_map_navigation_queues_commits_and_restores_transport_position() {
    let graph = source_map_navigation_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.transport.is_playing = true;
    session.runtime_state.transport.position_beats = 4.0;
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_source_map_navigation(SourceMapNavigationIntent::NextPhrase, 100),
        SourceMapNavigationResult::Enqueued {
            target_position_beats: 16,
            target_label: "phrase 2 bar 5".into(),
        }
    );
    assert_eq!(
        state.queue_source_map_navigation(SourceMapNavigationIntent::NextBar, 101),
        SourceMapNavigationResult::AlreadyPending
    );

    let committed = state.commit_ready_actions(immediate_boundary(), 120);

    assert_eq!(committed.len(), 1);
    assert!(state.session.runtime_state.transport.is_playing);
    assert_eq!(state.session.runtime_state.transport.position_beats, 16.0);
    assert_eq!(state.runtime.transport.position_beats, 16.0);
    assert_eq!(state.jam_view.source.source_map.playhead_column, Some(16));
    assert_eq!(
        state.jam_view.source.source_map.current_region_label,
        "now bar 5 | break"
    );
    assert_eq!(
        state.session.action_log.actions.last().map(|action| action.command),
        Some(ActionCommand::TransportSeek)
    );
}

#[test]
fn source_map_navigation_clamps_at_source_map_edges() {
    let graph = source_map_navigation_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.transport.position_beats = 0.0;
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_source_map_navigation(SourceMapNavigationIntent::PreviousBar, 100),
        SourceMapNavigationResult::AlreadyAtBoundary {
            target_label: "bar 1".into(),
        }
    );
}

fn source_map_navigation_graph() -> SourceGraph {
    let mut graph = sample_graph();
    graph.source.duration_seconds = 16.0;
    graph.sections.clear();
    graph.sections.push(Section {
        section_id: SectionId::from("section-a"),
        label_hint: SectionLabelHint::Intro,
        start_seconds: 0.0,
        end_seconds: 8.0,
        bar_start: 1,
        bar_end: 4,
        energy_class: EnergyClass::Medium,
        confidence: 0.75,
        tags: Vec::new(),
    });
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Break,
        start_seconds: 8.0,
        end_seconds: 16.0,
        bar_start: 5,
        bar_end: 8,
        energy_class: EnergyClass::High,
        confidence: 0.9,
        tags: Vec::new(),
    });
    graph.timing.bpm_estimate = Some(120.0);
    graph.timing.meter_hint = Some(MeterHint {
        beats_per_bar: 4,
        beat_unit: 4,
    });
    graph.timing.quality = TimingQuality::High;
    graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
    graph.timing.primary_hypothesis_id = Some("primary-grid".into());
    graph.timing.hypotheses.clear();
    graph.timing.hypotheses.push(TimingHypothesis {
        hypothesis_id: "primary-grid".into(),
        kind: TimingHypothesisKind::Primary,
        bpm: 120.0,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.9,
        score: 0.9,
        beat_grid: Vec::new(),
        bar_grid: (0..8)
            .map(|index| BarSpan {
                bar_index: index + 1,
                start_seconds: index as f32 * 2.0,
                end_seconds: (index + 1) as f32 * 2.0,
                downbeat_confidence: 0.9,
                phrase_index: Some((index / 4) + 1),
            })
            .collect(),
        phrase_grid: vec![
            PhraseSpan {
                phrase_index: 1,
                start_bar: 1,
                end_bar: 4,
                confidence: 0.9,
            },
            PhraseSpan {
                phrase_index: 2,
                start_bar: 5,
                end_bar: 8,
                confidence: 0.9,
            },
        ],
        anchors: Vec::new(),
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::High,
        warnings: Vec::new(),
        provenance: vec!["fixture.source_map_navigation".into()],
    });
    graph
}
