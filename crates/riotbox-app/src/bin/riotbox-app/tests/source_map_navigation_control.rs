#[test]
fn navigates_source_map_and_reports_committed_transport_seek() {
    let graph = source_map_navigation_control_graph();
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-05-23T15:07:00Z");
    session.runtime_state.transport.is_playing = true;
    session.runtime_state.transport.position_beats = 4.0;
    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Ingest,
    );

    navigate_source_map(
        &mut shell,
        riotbox_app::jam_app::SourceMapNavigationIntent::NextPhrase,
        123,
    );

    assert_eq!(
        shell.status_message,
        "source map moved to phrase 2 bar 5 @ beat 16"
    );
    assert!(shell.app.queue.pending_actions().is_empty());
    assert!(shell.app.session.runtime_state.transport.is_playing);
    assert_eq!(shell.app.session.runtime_state.transport.position_beats, 16.0);

    let action = shell
        .app
        .session
        .action_log
        .actions
        .last()
        .expect("committed source map navigation action");
    assert_eq!(action.command, ActionCommand::TransportSeek);
    assert_eq!(
        action.params,
        riotbox_core::action::ActionParams::Transport {
            position_beats: Some(16),
        }
    );
    assert_eq!(action.committed_at, Some(123));
}

fn source_map_navigation_control_graph() -> SourceGraph {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-map-nav"),
            path: "source-map-nav.wav".into(),
            content_hash: "hash-map-nav".into(),
            duration_seconds: 16.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["fixture".into()],
            generated_at: "2026-05-23T15:07:00Z".into(),
            source_hash: "hash-map-nav".into(),
            analysis_seed: 12,
            run_notes: None,
        },
    );
    graph.timing.bpm_estimate = Some(120.0);
    graph.timing.meter_hint = Some(MeterHint {
        beats_per_bar: 4,
        beat_unit: 4,
    });
    graph.timing.quality = TimingQuality::High;
    graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
    graph.timing.primary_hypothesis_id = Some("primary-grid".into());
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
            .map(|index| riotbox_core::source_graph::BarSpan {
                bar_index: index + 1,
                start_seconds: index as f32 * 2.0,
                end_seconds: (index + 1) as f32 * 2.0,
                downbeat_confidence: 0.9,
                phrase_index: Some((index / 4) + 1),
            })
            .collect(),
        phrase_grid: vec![
            riotbox_core::source_graph::PhraseSpan {
                phrase_index: 1,
                start_bar: 1,
                end_bar: 4,
                confidence: 0.9,
            },
            riotbox_core::source_graph::PhraseSpan {
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
