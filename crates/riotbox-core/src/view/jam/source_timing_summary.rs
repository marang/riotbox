#[derive(Clone, Debug, PartialEq)]
pub struct SourceTimingGrooveResidualView {
    pub subdivision: String,
    pub offset_ms: f32,
    pub confidence: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceTimingSummaryView {
    pub cue: String,
    pub quality: String,
    pub degraded_policy: String,
    pub primary_warning: Option<String>,
    pub primary_anchor_count: usize,
    pub primary_kick_anchor_count: usize,
    pub primary_backbeat_anchor_count: usize,
    pub primary_transient_anchor_count: usize,
    pub primary_anchor_cue: String,
    pub primary_groove_residual_count: usize,
    pub primary_max_abs_groove_offset_ms: f32,
    pub primary_groove_preview: Vec<SourceTimingGrooveResidualView>,
}

impl Default for SourceTimingSummaryView {
    fn default() -> Self {
        Self {
            cue: "not available".into(),
            quality: "unknown".into(),
            degraded_policy: "disabled".into(),
            primary_warning: None,
            primary_anchor_count: 0,
            primary_kick_anchor_count: 0,
            primary_backbeat_anchor_count: 0,
            primary_transient_anchor_count: 0,
            primary_anchor_cue: "anchors none".into(),
            primary_groove_residual_count: 0,
            primary_max_abs_groove_offset_ms: 0.0,
            primary_groove_preview: Vec::new(),
        }
    }
}

impl SourceTimingSummaryView {
    #[must_use]
    pub fn from_graph(graph: &SourceGraph) -> Self {
        let degraded_policy =
            source_timing_degraded_policy_label(&graph.timing.effective_degraded_policy());
        let primary_hypothesis = primary_source_timing_hypothesis(&graph.timing);
        let anchors = primary_hypothesis.map_or(&[][..], |hypothesis| hypothesis.anchors.as_slice());
        let groove = primary_hypothesis.map_or(&[][..], |hypothesis| hypothesis.groove.as_slice());

        let primary_anchor_count = anchors.len();
        let primary_kick_anchor_count =
            count_source_timing_anchor_type(anchors, crate::source_graph::SourceTimingAnchorType::Kick);
        let primary_backbeat_anchor_count = count_source_timing_anchor_type(
            anchors,
            crate::source_graph::SourceTimingAnchorType::Backbeat,
        );
        let primary_transient_anchor_count = count_source_timing_anchor_type(
            anchors,
            crate::source_graph::SourceTimingAnchorType::TransientCluster,
        );
        let primary_groove_residual_count = groove.len();
        let primary_max_abs_groove_offset_ms = groove
            .iter()
            .map(|residual| residual.offset_ms.abs())
            .fold(0.0_f32, f32::max);

        Self {
            cue: source_timing_policy_cue_label(degraded_policy).into(),
            quality: source_timing_quality_label(&graph.timing.effective_timing_quality()).into(),
            degraded_policy: degraded_policy.into(),
            primary_warning: primary_source_timing_warning(&graph.timing.warnings)
                .map(|warning| source_timing_warning_code_label(&warning.code).into()),
            primary_anchor_count,
            primary_kick_anchor_count,
            primary_backbeat_anchor_count,
            primary_transient_anchor_count,
            primary_anchor_cue: source_timing_anchor_cue(
                primary_anchor_count,
                primary_kick_anchor_count,
                primary_backbeat_anchor_count,
                primary_transient_anchor_count,
            ),
            primary_groove_residual_count,
            primary_max_abs_groove_offset_ms,
            primary_groove_preview: groove
                .iter()
                .take(4)
                .map(source_timing_groove_residual_view)
                .collect(),
        }
    }
}

fn primary_source_timing_hypothesis(
    timing: &crate::source_graph::TimingModel,
) -> Option<&crate::source_graph::TimingHypothesis> {
    timing.primary_hypothesis().or_else(|| {
        timing
            .hypotheses
            .iter()
            .find(|hypothesis| hypothesis.kind == crate::source_graph::TimingHypothesisKind::Primary)
    })
}

fn source_timing_groove_residual_view(
    residual: &crate::source_graph::GrooveResidual,
) -> SourceTimingGrooveResidualView {
    SourceTimingGrooveResidualView {
        subdivision: source_timing_groove_subdivision_label(residual.subdivision).into(),
        offset_ms: residual.offset_ms,
        confidence: residual.confidence,
    }
}

fn source_timing_groove_subdivision_label(
    subdivision: crate::source_graph::GrooveSubdivision,
) -> &'static str {
    match subdivision {
        crate::source_graph::GrooveSubdivision::Eighth => "eighth",
        crate::source_graph::GrooveSubdivision::Triplet => "triplet",
        crate::source_graph::GrooveSubdivision::Sixteenth => "sixteenth",
        crate::source_graph::GrooveSubdivision::ThirtySecond => "thirty_second",
    }
}

fn primary_source_timing_warning(
    warnings: &[crate::source_graph::TimingWarning],
) -> Option<&crate::source_graph::TimingWarning> {
    warnings
        .iter()
        .min_by_key(|warning| source_timing_warning_priority(&warning.code))
}

fn source_timing_warning_priority(code: &crate::source_graph::TimingWarningCode) -> u8 {
    match code {
        crate::source_graph::TimingWarningCode::DriftHigh => 0,
        crate::source_graph::TimingWarningCode::AmbiguousDownbeat => 1,
        crate::source_graph::TimingWarningCode::LowTimingConfidence => 2,
        crate::source_graph::TimingWarningCode::WeakKickAnchor => 3,
        crate::source_graph::TimingWarningCode::WeakBackbeatAnchor => 4,
        crate::source_graph::TimingWarningCode::HalfTimePossible => 5,
        crate::source_graph::TimingWarningCode::DoubleTimePossible => 6,
        crate::source_graph::TimingWarningCode::PhraseUncertain => 7,
    }
}

#[cfg(test)]
mod source_timing_summary_tests {
    use super::*;
    use crate::{
        ids::SourceId,
        source_graph::{
            DecodeProfile, GraphProvenance, MeterHint, SourceDescriptor, SourceGraph,
            GrooveResidual, GrooveSubdivision, SourceTimingAnchor, SourceTimingAnchorType,
            TimingDegradedPolicy, TimingHypothesis, TimingHypothesisKind, TimingQuality,
            TimingWarning, TimingWarningCode,
        },
    };

    #[test]
    fn default_summary_keeps_policy_and_cue_contract_aligned() {
        let timing = SourceTimingSummaryView::default();

        assert_eq!(timing.degraded_policy, "disabled");
        assert_eq!(timing.cue, "not available");
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
        assert_eq!(timing.primary_warning.as_deref(), Some("ambiguous_downbeat"));
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

        let timing = SourceTimingSummaryView::from_graph(&graph);

        assert_eq!(timing.quality, "high");
        assert_eq!(timing.degraded_policy, "locked");
        assert_eq!(timing.cue, "grid locked");
        assert_eq!(timing.primary_warning, None);
        assert_eq!(timing.primary_anchor_count, 1);
        assert_eq!(timing.primary_anchor_cue, "anchors 1 | kick");
        assert_eq!(timing.primary_groove_residual_count, 0);
        assert_eq!(timing.primary_groove_preview, Vec::new());
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
        assert_eq!(timing.primary_anchor_count, 1);
        assert_eq!(timing.primary_kick_anchor_count, 0);
        assert_eq!(timing.primary_backbeat_anchor_count, 1);
        assert_eq!(timing.primary_anchor_cue, "anchors 1 | backbeat");
    }

    #[test]
    fn summary_picks_most_musically_urgent_primary_warning() {
        let mut graph = source_timing_graph(TimingQuality::Low, TimingDegradedPolicy::ManualConfirm);
        graph.timing.warnings.push(timing_warning(TimingWarningCode::PhraseUncertain));
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
}

fn count_source_timing_anchor_type(
    anchors: &[crate::source_graph::SourceTimingAnchor],
    anchor_type: crate::source_graph::SourceTimingAnchorType,
) -> usize {
    anchors
        .iter()
        .filter(|anchor| anchor.anchor_type == anchor_type)
        .count()
}

fn source_timing_anchor_cue(
    total: usize,
    kick: usize,
    backbeat: usize,
    transient: usize,
) -> String {
    let label = if kick > 0 && backbeat > 0 {
        "kick+backbeat"
    } else if kick > 0 {
        "kick"
    } else if backbeat > 0 {
        "backbeat"
    } else if transient > 0 {
        "transient"
    } else {
        "none"
    };

    if total == 0 {
        "anchors none".into()
    } else {
        format!("anchors {total} | {label}")
    }
}

fn source_timing_quality_label(quality: &crate::source_graph::TimingQuality) -> &'static str {
    match quality {
        crate::source_graph::TimingQuality::Low => "low",
        crate::source_graph::TimingQuality::Medium => "medium",
        crate::source_graph::TimingQuality::High => "high",
        crate::source_graph::TimingQuality::Unknown => "unknown",
    }
}

fn source_timing_degraded_policy_label(
    policy: &crate::source_graph::TimingDegradedPolicy,
) -> &'static str {
    match policy {
        crate::source_graph::TimingDegradedPolicy::Locked => "locked",
        crate::source_graph::TimingDegradedPolicy::Cautious => "cautious",
        crate::source_graph::TimingDegradedPolicy::ManualConfirm => "manual_confirm",
        crate::source_graph::TimingDegradedPolicy::FallbackGrid => "fallback_grid",
        crate::source_graph::TimingDegradedPolicy::Disabled => "disabled",
        crate::source_graph::TimingDegradedPolicy::Unknown => "unknown",
    }
}

fn source_timing_policy_cue_label(policy: &str) -> &'static str {
    match policy {
        "locked" => "grid locked",
        "manual_confirm" => "needs confirm",
        "cautious" => "listen first",
        "fallback_grid" => "fallback grid",
        "disabled" => "not available",
        _ => "unknown",
    }
}

fn source_timing_warning_code_label(
    code: &crate::source_graph::TimingWarningCode,
) -> &'static str {
    match code {
        crate::source_graph::TimingWarningCode::WeakKickAnchor => "weak_kick_anchor",
        crate::source_graph::TimingWarningCode::WeakBackbeatAnchor => "weak_backbeat_anchor",
        crate::source_graph::TimingWarningCode::AmbiguousDownbeat => "ambiguous_downbeat",
        crate::source_graph::TimingWarningCode::HalfTimePossible => "half_time_possible",
        crate::source_graph::TimingWarningCode::DoubleTimePossible => "double_time_possible",
        crate::source_graph::TimingWarningCode::DriftHigh => "drift_high",
        crate::source_graph::TimingWarningCode::PhraseUncertain => "phrase_uncertain",
        crate::source_graph::TimingWarningCode::LowTimingConfidence => "low_timing_confidence",
    }
}
