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

    let mut session = SessionFile::new("session-1", "0.1.0", "2026-05-08T00:00:00Z");
    session.runtime_state.source_timing.confirmed_grid =
        Some(riotbox_core::session::SourceTimingGridConfirmationState {
            source_id: SourceId::from("src-timing"),
            hypothesis_id: Some("probe-primary".into()),
            confirmed_by_action: ActionId(9),
            confirmed_at: 1_777_777,
        });

    let shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Ingest,
    );

    let snapshot = observer_snapshot(&shell);
    let source_timing = &snapshot["source_timing"];

    assert_eq!(source_timing["present"], true);
    assert_eq!(source_timing["source_id"], "src-timing");
    assert_eq!(source_timing["quality"], "low");
    assert_eq!(source_timing["degraded_policy"], "manual_confirm");
    assert_eq!(source_timing["cue"], "needs confirm");
    assert_eq!(source_timing["actionability"], "confirm grid first");
    assert_eq!(source_timing["grid_use"], "manual_confirm_only");
    assert_eq!(source_timing["beat_status"], "tempo_only");
    assert_eq!(source_timing["beat_count"], 0);
    assert_eq!(source_timing["downbeat_status"], "ambiguous");
    assert_eq!(source_timing["primary_downbeat_score"], serde_json::Value::Null);
    assert_eq!(
        source_timing["primary_downbeat_score_gap"],
        serde_json::Value::Null
    );
    assert_eq!(source_timing["alternate_downbeat_phase_count"], 0);
    assert_eq!(source_timing["bar_count"], 0);
    assert_eq!(source_timing["phrase_status"], "uncertain");
    assert_eq!(source_timing["phrase_count"], 0);
    assert_eq!(source_timing["primary_hypothesis_id"], "probe-primary");
    assert_eq!(source_timing["grid_confirmed"], true);
    assert_eq!(source_timing["confirmed_grid_source_id"], "src-timing");
    assert_eq!(source_timing["confirmed_grid_hypothesis_id"], "probe-primary");
    assert_eq!(source_timing["confirmed_grid_action_id"], 9);
    assert_eq!(source_timing["confirmed_grid_at"], 1_777_777);
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
        source_timing["primary_anchor_cue"],
        "anchors 3 | kick+backbeat"
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
fn observer_snapshot_records_source_map_capture_range_projection() {
    let graph = observer_source_map_graph(TimingDegradedPolicy::Locked, TimingQuality::High);
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");
    session.runtime_state.transport.position_beats = 4.0;
    session.runtime_state.capture.length_intent = CaptureLengthIntent::OneBar;
    let shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Ingest,
    );

    let snapshot = observer_snapshot(&shell);
    let source_map = &snapshot["source_map"];

    assert_eq!(source_map["present"], true);
    assert_eq!(source_map["mode"], "bar grid");
    assert_eq!(source_map["trust_label"], "grid locked");
    assert_eq!(source_map["playhead_column"], 8);
    assert_eq!(source_map["capture_range_available"], true);
    assert_eq!(
        source_map["capture_range_row"],
        "................[=======]......."
    );
    assert_eq!(source_map["capture_hint"], "cap next bar | map bar grid | 32 cols");
}

#[test]
fn observer_snapshot_records_committed_capture_source_window() {
    let graph = observer_source_map_graph(TimingDegradedPolicy::Locked, TimingQuality::High);
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");
    session.runtime_state.capture.length_intent = CaptureLengthIntent::OneBar;
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.queue_capture_bar(100);
    let committed = state.commit_ready_actions(
        riotbox_core::transport::CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 8,
            bar_index: 2,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        },
        200,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        committed[0].boundary.kind,
        riotbox_core::action::CommitBoundary::Bar
    );
    let shell = JamShellState::new(state, ShellLaunchMode::Ingest);
    let snapshot = observer_snapshot(&shell);
    let capture = &snapshot["capture"];

    assert_eq!(snapshot["runtime"]["capture_length_intent"], "1 bar");
    assert_eq!(capture["present"], true);
    assert_eq!(capture["capture_count"], 1);
    assert_eq!(capture["latest_capture_id"], "cap-01");
    assert_eq!(capture["created_from_action"], 1);
    assert_eq!(capture["source_window_available"], true);
    assert_eq!(capture["source_window"]["source_id"], "src-map-observer");
    assert_eq!(capture["source_window"]["start_seconds"], 4.0);
    assert_eq!(capture["source_window"]["end_seconds"], 6.0);
    assert_eq!(capture["source_window"]["duration_seconds"], 2.0);
    assert_eq!(capture["source_window"]["start_frame"], 176_400);
    assert_eq!(capture["source_window"]["end_frame"], 264_600);
}

#[test]
fn observer_snapshot_keeps_source_map_capture_range_unavailable_for_untrusted_timing() {
    let graph = observer_source_map_graph(TimingDegradedPolicy::ManualConfirm, TimingQuality::Low);
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");
    session.runtime_state.transport.position_beats = 4.0;
    session.runtime_state.capture.length_intent = CaptureLengthIntent::OneBar;
    let shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Ingest,
    );

    let snapshot = observer_snapshot(&shell);
    let source_map = &snapshot["source_map"];

    assert_eq!(source_map["present"], true);
    assert_eq!(source_map["mode"], "time fallback");
    assert_eq!(source_map["trust_label"], "needs confirm");
    assert_eq!(source_map["capture_range_available"], false);
    assert_eq!(source_map["capture_range_row"], ".".repeat(32));
    assert_eq!(
        source_map["capture_hint"],
        "cap listen first | map time fallback | no bar-accurate claim"
    );
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
    assert_eq!(source_timing["actionability"], "confirm grid first");
    assert_eq!(source_timing["grid_use"], "manual_confirm_only");
    assert_eq!(source_timing["primary_warning_code"], "ambiguous_downbeat");
    assert_eq!(source_timing["grid_confirmed"], false);
    assert_eq!(source_timing["alternate_downbeat_phase_count"], 0);
    assert_eq!(
        source_timing["anchor_evidence"]["primary_anchor_count"],
        1
    );
    assert_eq!(source_timing["primary_anchor_cue"], "anchors 1 | kick");
    assert_eq!(
        source_timing["groove_evidence"]["primary_groove_residual_count"],
        0
    );
    assert_eq!(source_timing["warning_codes"].as_array().unwrap().len(), 0);
}

fn observer_source_map_graph(
    policy: TimingDegradedPolicy,
    quality: TimingQuality,
) -> SourceGraph {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-map-observer"),
            path: "source-map-observer.wav".into(),
            content_hash: "hash-map-observer".into(),
            duration_seconds: 8.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["fixture.source_map_observer".into()],
            generated_at: "2026-05-23T00:00:00Z".into(),
            source_hash: "hash-map-observer".into(),
            analysis_seed: 982,
            run_notes: None,
        },
    );
    graph.timing.bpm_estimate = Some(120.0);
    graph.timing.bpm_confidence = 0.9;
    graph.timing.quality = quality;
    graph.timing.degraded_policy = policy;
    graph.timing.primary_hypothesis_id = Some("map-primary".into());
    graph.timing.hypotheses.push(TimingHypothesis {
        hypothesis_id: "map-primary".into(),
        kind: TimingHypothesisKind::Primary,
        bpm: 120.0,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.9,
        score: 0.9,
        beat_grid: Vec::new(),
        bar_grid: (0..4)
            .map(|index| riotbox_core::source_graph::BarSpan {
                bar_index: index + 1,
                start_seconds: index as f32 * 2.0,
                end_seconds: (index + 1) as f32 * 2.0,
                downbeat_confidence: 0.9,
                phrase_index: Some(1),
            })
            .collect(),
        phrase_grid: Vec::new(),
        anchors: Vec::new(),
        drift: Vec::new(),
        groove: Vec::new(),
        quality,
        warnings: Vec::new(),
        provenance: vec!["fixture.source_map_observer".into()],
    });
    graph
}
