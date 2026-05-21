#[derive(Clone, Debug, PartialEq)]
pub struct SourceTimingGrooveResidualView {
    pub subdivision: String,
    pub offset_ms: f32,
    pub confidence: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceTimingSummaryView {
    pub cue: String,
    pub actionability: String,
    pub quality: String,
    pub degraded_policy: String,
    pub grid_use: String,
    pub beat_status: String,
    pub beat_count: usize,
    pub downbeat_status: String,
    pub primary_warning: Option<String>,
    pub primary_downbeat_offset_beats: Option<u32>,
    pub bar_count: usize,
    pub phrase_status: String,
    pub phrase_count: usize,
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
            actionability: "timing unavailable".into(),
            quality: "unknown".into(),
            degraded_policy: "disabled".into(),
            grid_use: "unavailable".into(),
            beat_status: "unknown".into(),
            beat_count: 0,
            downbeat_status: "unknown".into(),
            primary_warning: None,
            primary_downbeat_offset_beats: None,
            bar_count: 0,
            phrase_status: "unknown".into(),
            phrase_count: 0,
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
            actionability: source_timing_actionability_label(degraded_policy).into(),
            quality: source_timing_quality_label(&graph.timing.effective_timing_quality()).into(),
            degraded_policy: degraded_policy.into(),
            grid_use: crate::source_graph::source_timing_grid_use_from_timing_model(&graph.timing)
                .label()
                .into(),
            beat_status: source_timing_beat_status_label(&graph.timing).into(),
            beat_count: graph.timing.beat_grid.len(),
            downbeat_status: source_timing_downbeat_status_label(&graph.timing).into(),
            primary_warning: primary_source_timing_warning(&graph.timing.warnings)
                .map(|warning| source_timing_warning_code_label(&warning.code).into()),
            primary_downbeat_offset_beats: primary_source_timing_downbeat_offset_beats(primary_hypothesis),
            bar_count: graph.timing.bar_grid.len(),
            phrase_status: source_timing_phrase_status_label(&graph.timing).into(),
            phrase_count: graph.timing.phrase_grid.len(),
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

fn source_timing_beat_status_label(timing: &crate::source_graph::TimingModel) -> &'static str {
    if !timing.beat_grid.is_empty() {
        return "grid";
    }
    if timing.bpm_estimate.is_some() {
        return "tempo_only";
    }
    "unknown"
}

fn source_timing_actionability_label(policy: &str) -> &'static str {
    match policy {
        "locked" => "grid can steer moves",
        "manual_confirm" => "confirm grid first",
        "cautious" => "listen first",
        "fallback_grid" => "using safe fallback grid",
        "disabled" => "timing unavailable",
        _ => "timing trust unknown",
    }
}

fn source_timing_downbeat_status_label(timing: &crate::source_graph::TimingModel) -> &'static str {
    if timing
        .warnings
        .iter()
        .any(|warning| warning.code == crate::source_graph::TimingWarningCode::AmbiguousDownbeat)
    {
        return "ambiguous";
    }
    if !timing.bar_grid.is_empty() {
        return "bar_locked";
    }
    "unknown"
}

fn source_timing_phrase_status_label(timing: &crate::source_graph::TimingModel) -> &'static str {
    if timing
        .warnings
        .iter()
        .any(|warning| warning.code == crate::source_graph::TimingWarningCode::PhraseUncertain)
    {
        return "uncertain";
    }
    if !timing.phrase_grid.is_empty() {
        return "phrase_locked";
    }
    "unknown"
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

fn primary_source_timing_downbeat_offset_beats(
    hypothesis: Option<&crate::source_graph::TimingHypothesis>,
) -> Option<u32> {
    let hypothesis = hypothesis?;
    let first_bar = hypothesis.bar_grid.first()?;
    if hypothesis.bpm <= 0.0 || !hypothesis.bpm.is_finite() {
        return None;
    }
    let beats_per_bar = u32::from(hypothesis.meter.beats_per_bar);
    if beats_per_bar == 0 {
        return None;
    }
    let beat_seconds = 60.0 / hypothesis.bpm;
    let offset_beats = (first_bar.start_seconds / beat_seconds).round() as i64;
    Some(offset_beats.rem_euclid(i64::from(beats_per_bar)) as u32)
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
        crate::source_graph::TimingWarningCode::SparseOnsets => 2,
        crate::source_graph::TimingWarningCode::DriftHigh => 0,
        crate::source_graph::TimingWarningCode::AmbiguousDownbeat => 1,
        crate::source_graph::TimingWarningCode::LowTimingConfidence => 3,
        crate::source_graph::TimingWarningCode::WeakKickAnchor => 4,
        crate::source_graph::TimingWarningCode::WeakBackbeatAnchor => 5,
        crate::source_graph::TimingWarningCode::HalfTimePossible => 6,
        crate::source_graph::TimingWarningCode::DoubleTimePossible => 7,
        crate::source_graph::TimingWarningCode::PhraseUncertain => 8,
    }
}

#[cfg(test)]
#[path = "source_timing_summary_tests.rs"]
mod source_timing_summary_tests;

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
        crate::source_graph::TimingWarningCode::SparseOnsets => "sparse_onsets",
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
