#[test]
fn commits_grid_and_reports_status() {
    let graph = source_timing_confirm_graph();
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("session-1", "0.1.0", "2026-05-23T14:20:00Z"),
            Some(graph),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Ingest,
    );

    confirm_source_timing_grid(&mut shell, 123);

    assert_eq!(shell.status_message, "confirmed source timing grid");
    assert!(shell.app.queue.pending_actions().is_empty());

    let action = shell
        .app
        .session
        .action_log
        .actions
        .last()
        .expect("committed confirmation action");
    assert_eq!(action.command, ActionCommand::SourceTimingConfirmGrid);
    assert_eq!(action.committed_at, Some(123));
    assert_eq!(
        action.params,
        riotbox_core::action::ActionParams::SourceTimingGrid {
            source_id: Some(SourceId::from("src-confirm")),
            hypothesis_id: Some("primary-grid".into()),
        }
    );

    let confirmed = shell
        .app
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .expect("confirmed source timing grid");
    assert_eq!(confirmed.source_id, SourceId::from("src-confirm"));
    assert_eq!(confirmed.hypothesis_id.as_deref(), Some("primary-grid"));
    assert_eq!(confirmed.confirmed_by_action, action.id);
    assert_eq!(confirmed.confirmed_at, 123);

    confirm_source_timing_grid(&mut shell, 124);

    assert_eq!(
        shell.status_message,
        "source timing grid already confirmed"
    );
}

#[test]
fn reports_unavailable_without_graph() {
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("session-1", "0.1.0", "2026-05-23T14:21:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );

    confirm_source_timing_grid(&mut shell, 123);

    assert_eq!(
        shell.status_message,
        "no source timing grid available to confirm"
    );
    assert!(shell.app.queue.pending_actions().is_empty());
    assert!(shell.app.session.action_log.actions.is_empty());
    assert!(shell
        .app
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .is_none());
}

fn source_timing_confirm_graph() -> SourceGraph {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-confirm"),
            path: "source.wav".into(),
            content_hash: "hash-confirm".into(),
            duration_seconds: 8.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["fixture".into()],
            generated_at: "2026-05-23T14:20:00Z".into(),
            source_hash: "hash-confirm".into(),
            analysis_seed: 11,
            run_notes: None,
        },
    );
    graph.timing.bpm_estimate = Some(128.0);
    graph.timing.quality = TimingQuality::Low;
    graph.timing.degraded_policy = TimingDegradedPolicy::ManualConfirm;
    graph.timing.primary_hypothesis_id = Some("primary-grid".into());
    graph.timing.hypotheses.push(TimingHypothesis {
        hypothesis_id: "primary-grid".into(),
        kind: TimingHypothesisKind::Primary,
        bpm: 128.0,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.72,
        score: 0.68,
        beat_grid: Vec::new(),
        bar_grid: Vec::new(),
        phrase_grid: Vec::new(),
        anchors: Vec::new(),
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::Low,
        warnings: Vec::new(),
        provenance: vec!["fixture.source_timing_confirm".into()],
    });
    graph
}
