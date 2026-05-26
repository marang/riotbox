use super::*;
use crate::{
    ids::{ActionId, SourceId},
    session::{SessionFile, SourceTimingGridConfirmationState},
    source_graph::{
        DecodeProfile, GraphProvenance, GrooveResidual, GrooveSubdivision, MeterHint,
        SourceDescriptor, SourceGraph, SourceTimingAnchor, SourceTimingAnchorType,
        TimingDegradedPolicy, TimingHypothesis, TimingHypothesisKind, TimingQuality,
        TimingWarning, TimingWarningCode,
    },
};

#[test]
fn default_summary_keeps_policy_and_cue_contract_aligned() {
    let timing = SourceTimingSummaryView::default();

    assert_eq!(timing.degraded_policy, "disabled");
    assert_eq!(timing.cue, "not available");
    assert_eq!(timing.actionability, "timing unavailable");
    assert_eq!(timing.grid_use, "unavailable");
    assert_eq!(timing.beat_status, "unknown");
    assert_eq!(timing.beat_count, 0);
    assert_eq!(timing.downbeat_status, "unknown");
    assert_eq!(timing.primary_downbeat_offset_beats, None);
    assert_eq!(timing.primary_downbeat_score, None);
    assert_eq!(timing.primary_downbeat_score_gap, None);
    assert_eq!(timing.alternate_downbeat_phase_count, 0);
    assert_eq!(timing.bar_count, 0);
    assert_eq!(timing.phrase_status, "unknown");
    assert_eq!(timing.phrase_count, 0);
}

#[test]
fn consumer_readiness_distinguishes_analyzer_user_and_manual_trust() {
    let locked = source_timing_graph(TimingQuality::High, TimingDegradedPolicy::Locked);
    let session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");
    assert_eq!(
        source_timing_consumer_readiness(Some(&locked), &session),
        SourceTimingConsumerReadiness::AnalyzerLocked
    );
    assert!(
        source_timing_consumer_readiness(Some(&locked), &session).can_use_source_window_grid()
    );

    let mut manual = source_timing_graph(TimingQuality::Low, TimingDegradedPolicy::ManualConfirm);
    manual.timing.primary_hypothesis_id = Some("primary".into());
    assert_eq!(
        source_timing_consumer_readiness(Some(&manual), &session),
        SourceTimingConsumerReadiness::NeedsUserConfirmation
    );
    assert!(
        !source_timing_consumer_readiness(Some(&manual), &session).can_use_source_window_grid()
    );

    let mut confirmed_session =
        SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");
    confirmed_session.runtime_state.source_timing.confirmed_grid =
        Some(SourceTimingGridConfirmationState {
            source_id: manual.source.source_id.clone(),
            hypothesis_id: manual.timing.primary_hypothesis_id.clone(),
            confirmed_by_action: ActionId(9),
            confirmed_at: 900,
        });
    assert_eq!(
        source_timing_consumer_readiness(Some(&manual), &confirmed_session),
        SourceTimingConsumerReadiness::UserConfirmed
    );
    assert!(
        source_timing_consumer_readiness(Some(&manual), &confirmed_session)
            .can_use_source_window_grid()
    );

    let fallback = source_timing_graph(TimingQuality::Low, TimingDegradedPolicy::FallbackGrid);
    assert_eq!(
        source_timing_consumer_readiness(Some(&fallback), &session),
        SourceTimingConsumerReadiness::FallbackGrid
    );
    confirmed_session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_mut()
        .expect("confirmed grid")
        .hypothesis_id = fallback.timing.primary_hypothesis_id.clone();
    assert_eq!(
        source_timing_consumer_readiness(Some(&fallback), &confirmed_session),
        SourceTimingConsumerReadiness::FallbackGrid
    );
    assert_eq!(
        source_timing_consumer_readiness(None, &session),
        SourceTimingConsumerReadiness::Unavailable
    );
}

#[test]
fn manual_confirm_summary_preserves_musician_cue_warning_and_anchor_counts() {
    let mut graph = source_timing_graph(TimingQuality::Low, TimingDegradedPolicy::ManualConfirm);
    graph.timing.primary_hypothesis_id = Some("primary".into());
    graph.timing.hypotheses.push(timing_hypothesis(
        "primary",
        TimingHypothesisKind::Primary,
        vec![
            source_anchor("kick-1", SourceTimingAnchorType::Kick),
            source_anchor("backbeat-1", SourceTimingAnchorType::Backbeat),
            source_anchor("transient-1", SourceTimingAnchorType::TransientCluster),
            source_anchor("fill-1", SourceTimingAnchorType::Fill),
        ],
    ));
    graph.timing.hypotheses[0].groove = vec![
        GrooveResidual {
            subdivision: GrooveSubdivision::Eighth,
            offset_ms: -12.5,
            confidence: 0.72,
        },
        GrooveResidual {
            subdivision: GrooveSubdivision::Sixteenth,
            offset_ms: 6.25,
            confidence: 0.61,
        },
    ];
    graph.timing.warnings.push(TimingWarning {
        code: TimingWarningCode::AmbiguousDownbeat,
        message: "downbeat candidates are close".into(),
    });

    let timing = SourceTimingSummaryView::from_graph(&graph);

    assert_eq!(timing.quality, "low");
    assert_eq!(timing.degraded_policy, "manual_confirm");
    assert_eq!(timing.cue, "needs confirm");
    assert_eq!(timing.actionability, "confirm grid first");
    assert_eq!(timing.grid_use, "manual_confirm_only");
    assert_eq!(timing.beat_status, "tempo_only");
    assert_eq!(timing.beat_count, 0);
    assert_eq!(timing.downbeat_status, "ambiguous");
    assert_eq!(timing.primary_warning.as_deref(), Some("ambiguous_downbeat"));
    assert_eq!(timing.primary_downbeat_offset_beats, None);
    assert_eq!(timing.primary_downbeat_score, None);
    assert_eq!(timing.primary_downbeat_score_gap, None);
    assert_eq!(timing.alternate_downbeat_phase_count, 0);
    assert_eq!(timing.bar_count, 0);
    assert_eq!(timing.phrase_status, "unknown");
    assert_eq!(timing.phrase_count, 0);
    assert_eq!(timing.primary_anchor_count, 4);
    assert_eq!(timing.primary_kick_anchor_count, 1);
    assert_eq!(timing.primary_backbeat_anchor_count, 1);
    assert_eq!(timing.primary_transient_anchor_count, 1);
    assert_eq!(timing.primary_anchor_cue, "anchors 4 | kick+backbeat");
    assert_eq!(timing.primary_groove_residual_count, 2);
    assert_eq!(timing.primary_max_abs_groove_offset_ms, 12.5);
    assert_eq!(timing.primary_groove_preview[0].subdivision, "eighth");
    assert_eq!(timing.primary_groove_preview[0].offset_ms, -12.5);
}

#[test]
fn locked_summary_preserves_grid_locked_cue_without_primary_warning() {
    let mut graph = source_timing_graph(TimingQuality::High, TimingDegradedPolicy::Locked);
    graph.timing.primary_hypothesis_id = Some("primary".into());
    graph.timing.hypotheses.push(timing_hypothesis(
        "primary",
        TimingHypothesisKind::Primary,
        vec![source_anchor("kick-1", SourceTimingAnchorType::Kick)],
    ));
    graph.timing.hypotheses[0].bar_grid = vec![crate::source_graph::BarSpan {
        bar_index: 1,
        start_seconds: 0.9375,
        end_seconds: 2.8125,
        downbeat_confidence: 0.91,
        phrase_index: Some(1),
    }];

    let timing = SourceTimingSummaryView::from_graph(&graph);

    assert_eq!(timing.quality, "high");
    assert_eq!(timing.degraded_policy, "locked");
    assert_eq!(timing.cue, "grid locked");
    assert_eq!(timing.actionability, "grid can steer moves");
    assert_eq!(timing.grid_use, "locked_grid");
    assert_eq!(timing.beat_status, "tempo_only");
    assert_eq!(timing.downbeat_status, "bar_locked");
    assert_eq!(timing.primary_warning, None);
    assert_eq!(timing.primary_downbeat_offset_beats, Some(2));
    assert_eq!(timing.primary_downbeat_score, Some(0.91));
    assert_eq!(timing.primary_downbeat_score_gap, None);
    assert_eq!(timing.alternate_downbeat_phase_count, 0);
    assert_eq!(timing.bar_count, 1);
    assert_eq!(timing.primary_anchor_count, 1);
    assert_eq!(timing.primary_anchor_cue, "anchors 1 | kick");
    assert_eq!(timing.primary_groove_residual_count, 0);
    assert_eq!(timing.primary_groove_preview, Vec::new());
}

#[test]
fn summary_uses_primary_hypothesis_grid_counts_when_top_level_grids_are_empty() {
    let mut graph = source_timing_graph(TimingQuality::High, TimingDegradedPolicy::Locked);
    graph.timing.primary_hypothesis_id = Some("primary".into());
    graph.timing.hypotheses.push(timing_hypothesis(
        "primary",
        TimingHypothesisKind::Primary,
        Vec::new(),
    ));
    graph.timing.hypotheses[0].beat_grid = vec![
        crate::source_graph::BeatPoint {
            beat_index: 1,
            time_seconds: 0.0,
            confidence: 0.91,
        },
        crate::source_graph::BeatPoint {
            beat_index: 2,
            time_seconds: 0.468_75,
            confidence: 0.88,
        },
    ];
    graph.timing.hypotheses[0].bar_grid = vec![crate::source_graph::BarSpan {
        bar_index: 1,
        start_seconds: 0.0,
        end_seconds: 1.875,
        downbeat_confidence: 0.91,
        phrase_index: Some(1),
    }];
    graph.timing.hypotheses[0].phrase_grid = vec![crate::source_graph::PhraseSpan {
        phrase_index: 1,
        start_bar: 1,
        end_bar: 4,
        confidence: 0.83,
    }];

    let timing = SourceTimingSummaryView::from_graph(&graph);

    assert_eq!(timing.beat_status, "grid");
    assert_eq!(timing.beat_count, 2);
    assert_eq!(timing.downbeat_status, "bar_locked");
    assert_eq!(timing.bar_count, 1);
    assert_eq!(timing.phrase_status, "phrase_locked");
    assert_eq!(timing.phrase_count, 1);
}

#[test]
fn summary_falls_back_to_primary_kind_when_primary_id_is_missing() {
    let mut graph = source_timing_graph(TimingQuality::Medium, TimingDegradedPolicy::Cautious);
    graph.timing.hypotheses.push(timing_hypothesis(
        "alternate",
        TimingHypothesisKind::AlternateDownbeat,
        vec![source_anchor("alternate-kick", SourceTimingAnchorType::Kick)],
    ));
    graph.timing.hypotheses.push(timing_hypothesis(
        "primary-by-kind",
        TimingHypothesisKind::Primary,
        vec![source_anchor("primary-backbeat", SourceTimingAnchorType::Backbeat)],
    ));

    let timing = SourceTimingSummaryView::from_graph(&graph);

    assert_eq!(timing.quality, "medium");
    assert_eq!(timing.degraded_policy, "cautious");
    assert_eq!(timing.cue, "listen first");
    assert_eq!(timing.actionability, "listen first");
    assert_eq!(timing.grid_use, "manual_confirm_only");
    assert_eq!(timing.primary_anchor_count, 1);
    assert_eq!(timing.primary_kick_anchor_count, 0);
    assert_eq!(timing.primary_backbeat_anchor_count, 1);
    assert_eq!(timing.primary_anchor_cue, "anchors 1 | backbeat");
}

#[test]
fn summary_picks_most_musically_urgent_primary_warning() {
    let mut graph = source_timing_graph(TimingQuality::Low, TimingDegradedPolicy::ManualConfirm);
    graph
        .timing
        .warnings
        .push(timing_warning(TimingWarningCode::PhraseUncertain));
    graph
        .timing
        .warnings
        .push(timing_warning(TimingWarningCode::WeakKickAnchor));
    graph
        .timing
        .warnings
        .push(timing_warning(TimingWarningCode::AmbiguousDownbeat));

    let timing = SourceTimingSummaryView::from_graph(&graph);

    assert_eq!(timing.primary_warning.as_deref(), Some("ambiguous_downbeat"));
}

#[test]
fn summary_prioritizes_drift_over_other_timing_warnings() {
    let mut graph = source_timing_graph(TimingQuality::Low, TimingDegradedPolicy::ManualConfirm);
    graph
        .timing
        .warnings
        .push(timing_warning(TimingWarningCode::AmbiguousDownbeat));
    graph
        .timing
        .warnings
        .push(timing_warning(TimingWarningCode::DriftHigh));
    graph
        .timing
        .warnings
        .push(timing_warning(TimingWarningCode::LowTimingConfidence));

    let timing = SourceTimingSummaryView::from_graph(&graph);

    assert_eq!(timing.primary_warning.as_deref(), Some("drift_high"));
}

#[test]
fn summary_prioritizes_sparse_onsets_over_generic_low_confidence() {
    let mut graph = source_timing_graph(TimingQuality::Low, TimingDegradedPolicy::Disabled);
    graph
        .timing
        .warnings
        .push(timing_warning(TimingWarningCode::LowTimingConfidence));
    graph
        .timing
        .warnings
        .push(timing_warning(TimingWarningCode::WeakKickAnchor));
    graph
        .timing
        .warnings
        .push(timing_warning(TimingWarningCode::SparseOnsets));

    let timing = SourceTimingSummaryView::from_graph(&graph);

    assert_eq!(timing.primary_warning.as_deref(), Some("sparse_onsets"));
}

#[test]
fn cautious_short_loop_summary_surfaces_short_loop_grid_use() {
    let mut graph = source_timing_graph(TimingQuality::Medium, TimingDegradedPolicy::Cautious);
    graph.timing.beat_grid = vec![crate::source_graph::BeatPoint {
        beat_index: 1,
        time_seconds: 0.0,
        confidence: 0.86,
    }];
    graph.timing.bar_grid = vec![crate::source_graph::BarSpan {
        bar_index: 1,
        start_seconds: 0.0,
        end_seconds: 2.0,
        downbeat_confidence: 0.72,
        phrase_index: None,
    }];
    graph
        .timing
        .warnings
        .push(timing_warning(TimingWarningCode::PhraseUncertain));

    let timing = SourceTimingSummaryView::from_graph(&graph);

    assert_eq!(timing.grid_use, "short_loop_manual_confirm");
    assert_eq!(timing.beat_status, "grid");
    assert_eq!(timing.beat_count, 1);
    assert_eq!(timing.downbeat_status, "bar_locked");
    assert_eq!(timing.bar_count, 1);
    assert_eq!(timing.phrase_status, "uncertain");
    assert_eq!(timing.primary_warning.as_deref(), Some("phrase_uncertain"));
    assert_eq!(timing.alternate_downbeat_phase_count, 0);
}

#[test]
fn fallback_grid_summary_surfaces_fallback_grid_use() {
    let graph = source_timing_graph(TimingQuality::Low, TimingDegradedPolicy::FallbackGrid);

    let timing = SourceTimingSummaryView::from_graph(&graph);

    assert_eq!(timing.cue, "fallback grid");
    assert_eq!(timing.actionability, "using safe fallback grid");
    assert_eq!(timing.grid_use, "fallback_grid");
}

#[test]
fn ambiguous_downbeat_summary_surfaces_alternates_and_score_gap() {
    let mut graph = source_timing_graph(TimingQuality::Medium, TimingDegradedPolicy::Cautious);
    graph.timing.primary_hypothesis_id = Some("primary".into());
    graph.timing.hypotheses.push(timing_hypothesis(
        "primary",
        TimingHypothesisKind::Primary,
        vec![source_anchor(
            "transient-primary",
            SourceTimingAnchorType::TransientCluster,
        )],
    ));
    graph.timing.hypotheses[0].score = 0.151;
    graph.timing.hypotheses[0].bar_grid = vec![crate::source_graph::BarSpan {
        bar_index: 1,
        start_seconds: 0.9375,
        end_seconds: 2.8125,
        downbeat_confidence: 0.273,
        phrase_index: None,
    }];
    for (index, score) in [0.146, 0.144, 0.140].into_iter().enumerate() {
        let mut alternate = timing_hypothesis(
            &format!("alt-downbeat-{}", index + 1),
            TimingHypothesisKind::AlternateDownbeat,
            vec![source_anchor(
                &format!("transient-alt-{}", index + 1),
                SourceTimingAnchorType::TransientCluster,
            )],
        );
        alternate.score = score;
        graph.timing.hypotheses.push(alternate);
    }
    graph
        .timing
        .warnings
        .push(timing_warning(TimingWarningCode::AmbiguousDownbeat));

    let timing = SourceTimingSummaryView::from_graph(&graph);

    assert_eq!(timing.grid_use, "manual_confirm_only");
    assert_eq!(timing.downbeat_status, "ambiguous");
    assert_eq!(timing.primary_downbeat_offset_beats, Some(2));
    assert_eq!(timing.primary_downbeat_score, Some(0.273));
    assert!(
        timing.primary_downbeat_score_gap.is_some_and(|gap| (gap - 0.005).abs() < 0.0001),
        "{:?}",
        timing.primary_downbeat_score_gap
    );
    assert_eq!(timing.alternate_downbeat_phase_count, 3);
    assert_eq!(timing.primary_warning.as_deref(), Some("ambiguous_downbeat"));
}

fn source_timing_graph(
    quality: TimingQuality,
    degraded_policy: TimingDegradedPolicy,
) -> SourceGraph {
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
            sidecar_version: "test".into(),
            provider_set: vec!["source_timing_summary_tests".into()],
            generated_at: "2026-05-08T00:00:00Z".into(),
            source_hash: "hash".into(),
            analysis_seed: 674,
            run_notes: None,
        },
    );
    graph.timing.bpm_estimate = Some(128.0);
    graph.timing.bpm_confidence = 0.86;
    graph.timing.quality = quality;
    graph.timing.degraded_policy = degraded_policy;
    graph
}

fn timing_hypothesis(
    hypothesis_id: &str,
    kind: TimingHypothesisKind,
    anchors: Vec<SourceTimingAnchor>,
) -> TimingHypothesis {
    TimingHypothesis {
        hypothesis_id: hypothesis_id.into(),
        kind,
        bpm: 128.0,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.86,
        score: 0.82,
        beat_grid: Vec::new(),
        bar_grid: Vec::new(),
        phrase_grid: Vec::new(),
        anchors,
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::Medium,
        warnings: Vec::new(),
        provenance: vec!["source_timing_summary_tests".into()],
    }
}

fn source_anchor(anchor_id: &str, anchor_type: SourceTimingAnchorType) -> SourceTimingAnchor {
    SourceTimingAnchor {
        anchor_id: anchor_id.into(),
        anchor_type,
        time_seconds: 0.0,
        bar_index: Some(1),
        beat_index: Some(1),
        confidence: 0.82,
        strength: 0.95,
        tags: Vec::new(),
    }
}

fn timing_warning(code: TimingWarningCode) -> TimingWarning {
    TimingWarning {
        code,
        message: format!("{code:?}"),
    }
}
