#[derive(Clone, Debug, PartialEq, Eq)]
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
}

impl Default for SourceTimingSummaryView {
    fn default() -> Self {
        Self {
            cue: "not available".into(),
            quality: "unknown".into(),
            degraded_policy: "unknown".into(),
            primary_warning: None,
            primary_anchor_count: 0,
            primary_kick_anchor_count: 0,
            primary_backbeat_anchor_count: 0,
            primary_transient_anchor_count: 0,
            primary_anchor_cue: "anchors none".into(),
        }
    }
}

impl SourceTimingSummaryView {
    #[must_use]
    pub fn from_graph(graph: &SourceGraph) -> Self {
        let degraded_policy =
            source_timing_degraded_policy_label(&graph.timing.effective_degraded_policy());
        let anchors = graph
            .timing
            .primary_hypothesis_id
            .as_deref()
            .and_then(|primary_id| {
                graph
                    .timing
                    .hypotheses
                    .iter()
                    .find(|hypothesis| hypothesis.hypothesis_id == primary_id)
            })
            .or_else(|| {
                graph
                    .timing
                    .hypotheses
                    .iter()
                    .find(|hypothesis| {
                        hypothesis.kind
                            == crate::source_graph::TimingHypothesisKind::Primary
                    })
            })
            .map_or(&[][..], |hypothesis| hypothesis.anchors.as_slice());

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

        Self {
            cue: source_timing_policy_cue_label(degraded_policy).into(),
            quality: source_timing_quality_label(&graph.timing.effective_timing_quality()).into(),
            degraded_policy: degraded_policy.into(),
            primary_warning: graph
                .timing
                .warnings
                .first()
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
