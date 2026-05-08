#[test]
fn observer_snapshot_records_source_timing_readiness_when_graph_is_attached() {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-timing"),
            path: "source.wav".into(),
            content_hash: "hash".into(),
            duration_seconds: 8.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["fixture".into()],
            generated_at: "2026-05-08T00:00:00Z".into(),
            source_hash: "hash".into(),
            analysis_seed: 7,
            run_notes: None,
        },
    );
    graph.timing.bpm_estimate = Some(128.0);
    graph.timing.bpm_confidence = 0.72;
    graph.timing.quality = TimingQuality::Low;
    graph.timing.degraded_policy = TimingDegradedPolicy::ManualConfirm;
    graph.timing.primary_hypothesis_id = Some("probe-primary".into());
    graph.timing.warnings.push(TimingWarning {
        code: TimingWarningCode::AmbiguousDownbeat,
        message: "downbeat candidates are close".into(),
    });
    graph.timing.warnings.push(TimingWarning {
        code: TimingWarningCode::PhraseUncertain,
        message: "phrase grid needs confirmation".into(),
    });

    let shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("session-1", "0.1.0", "2026-05-08T00:00:00Z"),
            Some(graph),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Ingest,
    );

    let snapshot = observer_snapshot(&shell);
    let source_timing = &snapshot["source_timing"];

    assert_eq!(source_timing["present"], true);
    assert_eq!(source_timing["source_id"], "src-timing");
    assert_eq!(source_timing["quality"], "low");
    assert_eq!(source_timing["degraded_policy"], "manual_confirm");
    assert_eq!(source_timing["cue"], "needs confirm");
    assert_eq!(source_timing["beat_status"], "tempo_only");
    assert_eq!(source_timing["beat_count"], 0);
    assert_eq!(source_timing["downbeat_status"], "ambiguous");
    assert_eq!(source_timing["bar_count"], 0);
    assert_eq!(source_timing["phrase_status"], "uncertain");
    assert_eq!(source_timing["phrase_count"], 0);
    assert_eq!(source_timing["primary_hypothesis_id"], "probe-primary");
    assert_eq!(source_timing["primary_warning_code"], "ambiguous_downbeat");
    assert_eq!(source_timing["warning_codes"][1], "phrase_uncertain");
}
