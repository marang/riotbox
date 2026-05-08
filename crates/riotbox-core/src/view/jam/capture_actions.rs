fn is_capture_command(action: &crate::action::Action) -> bool {
    matches!(
        action.command,
        crate::action::ActionCommand::CaptureNow
            | crate::action::ActionCommand::CaptureLoop
            | crate::action::ActionCommand::CaptureBarGroup
            | crate::action::ActionCommand::W30CaptureToPad
            | crate::action::ActionCommand::PromoteCaptureToPad
            | crate::action::ActionCommand::PromoteCaptureToScene
            | crate::action::ActionCommand::W30LoopFreeze
            | crate::action::ActionCommand::PromoteResample
    )
}

fn capture_action_target_label(action: &crate::action::Action) -> String {
    match action.target.scope {
        Some(crate::action::TargetScope::LaneW30) => {
            if let (Some(bank_id), Some(pad_id)) = (
                action.target.bank_id.as_ref(),
                action.target.pad_id.as_ref(),
            ) {
                format!("lanew30:{bank_id}/{pad_id}")
            } else {
                "lanew30".into()
            }
        }
        Some(crate::action::TargetScope::Scene) => action
            .target
            .scene_id
            .as_ref()
            .map_or_else(|| "scene".into(), |scene_id| format!("scene:{scene_id}")),
        Some(crate::action::TargetScope::LaneMc202) => {
            action.target.object_id.as_ref().map_or_else(
                || "lanemc202".into(),
                |object_id| format!("lanemc202:{object_id}"),
            )
        }
        Some(crate::action::TargetScope::LaneTr909) => "lanetr909".into(),
        Some(crate::action::TargetScope::Global) => "global".into(),
        Some(crate::action::TargetScope::Mixer) => "mixer".into(),
        Some(crate::action::TargetScope::Ghost) => "ghost".into(),
        Some(crate::action::TargetScope::Session) | None => "session".into(),
    }
}

const fn capture_target_kind_view(target: &crate::session::CaptureTarget) -> CaptureTargetKindView {
    match target {
        crate::session::CaptureTarget::W30Pad { .. } => CaptureTargetKindView::W30Pad,
        crate::session::CaptureTarget::Scene(_) => CaptureTargetKindView::Scene,
    }
}

const fn capture_handoff_readiness_view(
    capture: &crate::session::CaptureRef,
) -> CaptureHandoffReadinessView {
    if capture.source_window.is_some() {
        CaptureHandoffReadinessView::Source
    } else {
        CaptureHandoffReadinessView::Fallback
    }
}

fn latest_w30_promoted_capture_label(session: &SessionFile) -> Option<String> {
    session
        .captures
        .iter()
        .rev()
        .find_map(|capture| match capture.assigned_target.as_ref() {
            Some(crate::session::CaptureTarget::W30Pad { bank_id, pad_id }) => {
                Some(format!("{} -> {bank_id}/{pad_id}", capture.capture_id))
            }
            _ => None,
        })
}

fn recent_capture_rows(session: &SessionFile) -> Vec<String> {
    session
        .captures
        .iter()
        .rev()
        .take(5)
        .map(|capture| {
            if let Some(source_window) = &capture.source_window {
                return format!(
                    "{} | {}{}",
                    capture.capture_id,
                    format_source_window_span(source_window),
                    if capture.is_pinned { " | pinned" } else { "" }
                );
            }

            let target = capture
                .assigned_target
                .as_ref()
                .map_or_else(|| "unassigned".into(), capture_recent_target_label);

            format!(
                "{} | {} | {} origins{}",
                capture.capture_id,
                target,
                capture.source_origin_refs.len(),
                if capture.is_pinned { " | pinned" } else { "" }
            )
        })
        .collect()
}

fn latest_capture_provenance_lines(session: &SessionFile) -> Vec<String> {
    let Some(capture) = session.captures.last() else {
        return Vec::new();
    };

    let mut lines = vec![
        format!("file {}", capture.storage_path),
        format!(
            "from action {}",
            capture
                .created_from_action
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "manual or unknown".into())
        ),
        format!(
            "origins {}",
            if capture.source_origin_refs.is_empty() {
                "none".into()
            } else {
                capture.source_origin_refs.join(", ")
            }
        ),
    ];

    if let Some(source_window) = &capture.source_window {
        lines.push(format_source_window_provenance(source_window));
    }

    lines
}

fn capture_recent_target_label(target: &crate::session::CaptureTarget) -> String {
    match target {
        crate::session::CaptureTarget::W30Pad { bank_id, pad_id } => {
            format!("{bank_id}/{pad_id}")
        }
        crate::session::CaptureTarget::Scene(scene_id) => scene_id.to_string(),
    }
}

fn format_source_window_span(source_window: &crate::session::CaptureSourceWindow) -> String {
    format!(
        "{:.2}-{:.2}s",
        source_window.start_seconds, source_window.end_seconds
    )
}

fn format_source_window_provenance(source_window: &crate::session::CaptureSourceWindow) -> String {
    format!(
        "win {} {}",
        source_window.source_id,
        format_source_window_span(source_window)
    )
}

trait SessionAccessors {
    fn transport(&self) -> &crate::session::TransportRuntimeState;
}

impl SessionAccessors for SessionFile {
    fn transport(&self) -> &crate::session::TransportRuntimeState {
        &self.runtime_state.transport
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct JamTransportView {
    pub is_playing: bool,
    pub position_beats: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SourceSummaryView {
    pub source_id: String,
    pub bpm_estimate: Option<f32>,
    pub bpm_confidence: f32,
    pub timing: SourceTimingSummaryView,
    pub section_count: usize,
    pub loop_candidate_count: usize,
    pub hook_candidate_count: usize,
    pub feral_scorecard: FeralScorecardView,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeralScorecardView {
    pub readiness: String,
    pub break_rebuild_potential: String,
    pub hook_fragment_count: usize,
    pub break_support_count: usize,
    pub quote_risk_count: usize,
    pub capture_candidate_count: usize,
    pub top_reason: String,
    pub warnings: Vec<String>,
}

impl Default for FeralScorecardView {
    fn default() -> Self {
        Self {
            readiness: "unknown".into(),
            break_rebuild_potential: "unknown".into(),
            hook_fragment_count: 0,
            break_support_count: 0,
            quote_risk_count: 0,
            capture_candidate_count: 0,
            top_reason: "no feral source graph".into(),
            warnings: Vec::new(),
        }
    }
}

impl FeralScorecardView {
    #[must_use]
    pub fn from_graph(graph: &SourceGraph) -> Self {
        let break_rebuild_potential =
            quality_class_label(graph.analysis_summary.break_rebuild_potential).to_string();
        let hook_fragment_count = graph
            .assets
            .iter()
            .filter(|asset| asset.asset_type == AssetType::HookFragment)
            .count();
        let break_support_count = graph
            .relationships
            .iter()
            .filter(|relationship| {
                relationship.relation_type == RelationshipType::SupportsBreakRebuild
            })
            .count();
        let quote_risk_count = graph
            .relationships
            .iter()
            .filter(|relationship| {
                relationship.relation_type == RelationshipType::HighQuoteRiskWith
            })
            .count();
        let capture_candidate_count = graph
            .candidates
            .iter()
            .filter(|candidate| candidate.candidate_type == CandidateType::CaptureCandidate)
            .count();
        let readiness = feral_readiness(
            graph,
            hook_fragment_count,
            break_support_count,
            capture_candidate_count,
        )
        .to_string();
        let top_reason = feral_top_reason(
            graph.analysis_summary.break_rebuild_potential,
            hook_fragment_count,
            break_support_count,
            quote_risk_count,
            capture_candidate_count,
        )
        .to_string();
        let warnings = feral_scorecard_warnings(graph, hook_fragment_count, quote_risk_count);

        Self {
            readiness,
            break_rebuild_potential,
            hook_fragment_count,
            break_support_count,
            quote_risk_count,
            capture_candidate_count,
            top_reason,
            warnings,
        }
    }
}

fn feral_readiness(
    graph: &SourceGraph,
    hook_fragment_count: usize,
    break_support_count: usize,
    capture_candidate_count: usize,
) -> &'static str {
    if graph.has_feral_break_support_evidence() {
        "ready"
    } else if graph.analysis_summary.break_rebuild_potential == QualityClass::High
        && break_support_count == 0
    {
        "needs support"
    } else if graph.analysis_summary.break_rebuild_potential == QualityClass::High
        && hook_fragment_count == 0
        && capture_candidate_count == 0
        && graph.analysis_summary.hook_candidate_count == 0
        && graph.hook_candidate_count() == 0
    {
        "needs hook/capture"
    } else {
        "not ready"
    }
}

fn quality_class_label(quality: QualityClass) -> &'static str {
    match quality {
        QualityClass::Low => "low",
        QualityClass::Medium => "medium",
        QualityClass::High => "high",
        QualityClass::Unknown => "unknown",
    }
}

fn feral_top_reason(
    break_rebuild_potential: QualityClass,
    hook_fragment_count: usize,
    break_support_count: usize,
    quote_risk_count: usize,
    capture_candidate_count: usize,
) -> &'static str {
    if quote_risk_count > 0 && capture_candidate_count > 0 {
        "use capture before quoting"
    } else if quote_risk_count > 0 {
        "quote guard needed"
    } else if break_rebuild_potential == QualityClass::High && break_support_count > 0 {
        "break rebuild ready"
    } else if capture_candidate_count > 0 {
        "capture candidates ready"
    } else if hook_fragment_count > 0 {
        "hook fragments ready"
    } else {
        "feral evidence sparse"
    }
}

fn feral_scorecard_warnings(
    graph: &SourceGraph,
    hook_fragment_count: usize,
    quote_risk_count: usize,
) -> Vec<String> {
    let mut warnings = graph
        .analysis_summary
        .warnings
        .iter()
        .map(|warning| warning.code.clone())
        .collect::<Vec<_>>();

    if quote_risk_count > 0 {
        warnings.push(format!("quote risk {quote_risk_count}"));
    }

    if hook_fragment_count == 0 {
        warnings.push("no hook fragments".into());
    }

    warnings
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneSummaryView {
    pub active_scene: Option<String>,
    pub restore_scene: Option<String>,
    pub next_scene: Option<String>,
    pub scene_jump_availability: SceneJumpAvailabilityView,
    pub active_scene_energy: Option<String>,
    pub restore_scene_energy: Option<String>,
    pub next_scene_energy: Option<String>,
    pub next_scene_policy: Option<SceneTransitionPolicyView>,
    pub restore_scene_policy: Option<SceneTransitionPolicyView>,
    pub last_movement: Option<SceneMovementView>,
    pub scene_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneMovementView {
    pub kind: String,
    pub direction: String,
    pub tr909_intent: String,
    pub mc202_intent: String,
    pub intensity: f32,
    pub from_scene: Option<String>,
    pub to_scene: String,
    pub committed_bar_index: u64,
    pub committed_phrase_index: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneTransitionKindView {
    Launch,
    Restore,
}

impl SceneTransitionKindView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Launch => "launch",
            Self::Restore => "restore",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneTransitionDirectionView {
    Rise,
    Drop,
    Hold,
}

impl SceneTransitionDirectionView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Rise => "rise",
            Self::Drop => "drop",
            Self::Hold => "hold",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneTransitionLaneIntentView {
    Drive,
    Lift,
    Release,
    Anchor,
}

impl SceneTransitionLaneIntentView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Drive => "drive",
            Self::Lift => "lift",
            Self::Release => "release",
            Self::Anchor => "anchor",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneTransitionPolicyView {
    pub kind: SceneTransitionKindView,
    pub direction: SceneTransitionDirectionView,
    pub tr909_intent: SceneTransitionLaneIntentView,
    pub mc202_intent: SceneTransitionLaneIntentView,
    pub intensity: f32,
}
