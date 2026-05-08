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
    graph.timing.hypotheses.push(TimingHypothesis {
        hypothesis_id: "probe-primary".into(),
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
        anchors: vec![
            SourceTimingAnchor {
                anchor_id: "kick-1".into(),
                anchor_type: SourceTimingAnchorType::Kick,
                time_seconds: 0.0,
                bar_index: Some(1),
                beat_index: Some(1),
                confidence: 0.82,
                strength: 0.95,
                tags: vec!["kick_anchor".into()],
            },
            SourceTimingAnchor {
                anchor_id: "backbeat-1".into(),
                anchor_type: SourceTimingAnchorType::Backbeat,
                time_seconds: 0.47,
                bar_index: Some(1),
                beat_index: Some(2),
                confidence: 0.74,
                strength: 0.8,
                tags: vec!["backbeat_anchor".into()],
            },
            SourceTimingAnchor {
                anchor_id: "transient-1".into(),
                anchor_type: SourceTimingAnchorType::TransientCluster,
                time_seconds: 0.94,
                bar_index: Some(1),
                beat_index: Some(3),
                confidence: 0.69,
                strength: 0.7,
                tags: vec!["transient_cluster".into()],
            },
        ],
        drift: Vec::new(),
        groove: vec![
            GrooveResidual {
                subdivision: GrooveSubdivision::Eighth,
                offset_ms: -9.5,
                confidence: 0.72,
            },
            GrooveResidual {
                subdivision: GrooveSubdivision::Sixteenth,
                offset_ms: 4.25,
                confidence: 0.61,
            },
        ],
        quality: TimingQuality::Low,
        warnings: Vec::new(),
        provenance: vec!["fixture.source_timing_observer".into()],
    });
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
    assert_eq!(
        source_timing["anchor_evidence"]["primary_anchor_count"],
        3
    );
    assert_eq!(
        source_timing["anchor_evidence"]["primary_kick_anchor_count"],
        1
    );
    assert_eq!(
        source_timing["anchor_evidence"]["primary_backbeat_anchor_count"],
        1
    );
    assert_eq!(
        source_timing["anchor_evidence"]["primary_transient_anchor_count"],
        1
    );
    assert_eq!(
        source_timing["groove_evidence"]["primary_groove_residual_count"],
        2
    );
    assert_eq!(
        source_timing["groove_evidence"]["primary_max_abs_offset_ms"],
        9.5
    );
    assert_eq!(
        source_timing["groove_evidence"]["primary_groove_preview"][0]["subdivision"],
        "eighth"
    );
    assert_eq!(
        source_timing["groove_evidence"]["primary_groove_preview"][0]["offset_ms"],
        -9.5
    );
    assert_eq!(source_timing["primary_warning_code"], "ambiguous_downbeat");
    assert_eq!(source_timing["warning_codes"][1], "phrase_uncertain");
}

#[test]
fn observer_snapshot_uses_shared_source_timing_summary_for_musician_cues() {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-summary"),
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
            analysis_seed: 8,
            run_notes: None,
        },
    );
    graph.timing.bpm_estimate = Some(128.0);
    graph.timing.quality = TimingQuality::Low;
    graph.timing.degraded_policy = TimingDegradedPolicy::ManualConfirm;
    graph.timing.primary_hypothesis_id = Some("probe-primary".into());
    graph.timing.hypotheses.push(TimingHypothesis {
        hypothesis_id: "probe-primary".into(),
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
        anchors: vec![SourceTimingAnchor {
            anchor_id: "kick-1".into(),
            anchor_type: SourceTimingAnchorType::Kick,
            time_seconds: 0.0,
            bar_index: Some(1),
            beat_index: Some(1),
            confidence: 0.82,
            strength: 0.95,
            tags: vec!["kick_anchor".into()],
        }],
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::Low,
        warnings: Vec::new(),
        provenance: vec!["fixture.source_timing_observer".into()],
    });
    graph.timing.warnings.push(TimingWarning {
        code: TimingWarningCode::AmbiguousDownbeat,
        message: "downbeat candidates are close".into(),
    });

    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("session-1", "0.1.0", "2026-05-08T00:00:00Z"),
            Some(graph),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Ingest,
    );

    let graph = shell
        .app
        .source_graph
        .as_mut()
        .expect("observer fixture should keep source graph");
    graph.timing.quality = TimingQuality::High;
    graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
    graph.timing.warnings.clear();
    graph.timing.hypotheses[0].anchors.clear();

    let snapshot = observer_snapshot(&shell);
    let source_timing = &snapshot["source_timing"];

    assert_eq!(source_timing["quality"], "low");
    assert_eq!(source_timing["degraded_policy"], "manual_confirm");
    assert_eq!(source_timing["cue"], "needs confirm");
    assert_eq!(source_timing["primary_warning_code"], "ambiguous_downbeat");
    assert_eq!(
        source_timing["anchor_evidence"]["primary_anchor_count"],
        1
    );
    assert_eq!(
        source_timing["groove_evidence"]["primary_groove_residual_count"],
        0
    );
    assert_eq!(source_timing["warning_codes"].as_array().unwrap().len(), 0);
}
