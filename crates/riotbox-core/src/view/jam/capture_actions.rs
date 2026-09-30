use crate::session::SessionFile;

#[derive(Clone, Debug, PartialEq)]
pub struct CaptureSummaryView {
    pub capture_count: usize,
    pub pinned_capture_count: usize,
    pub promoted_capture_count: usize,
    pub unassigned_capture_count: usize,
    pub pending_capture_count: usize,
    pub pending_capture_items: Vec<PendingCaptureActionView>,
    pub last_capture_id: Option<String>,
    pub last_capture_target: Option<String>,
    pub last_capture_target_kind: Option<CaptureTargetKindView>,
    pub last_capture_handoff_readiness: Option<CaptureHandoffReadinessView>,
    pub last_capture_origin_count: usize,
    pub last_capture_notes: Option<String>,
    pub last_promotion_result: Option<String>,
    pub latest_w30_promoted_capture_label: Option<String>,
    pub recent_capture_rows: Vec<String>,
    pub latest_capture_provenance_lines: Vec<String>,
    pub pinned_capture_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureTargetKindView {
    W30Pad,
    Scene,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureHandoffReadinessView {
    Source,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingCaptureActionView {
    pub id: String,
    pub actor: String,
    pub command: String,
    pub quantization: String,
    pub target: String,
    pub explanation: Option<String>,
}

pub(super) fn is_capture_command(action: &crate::action::Action) -> bool {
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

pub(super) fn capture_action_target_label(action: &crate::action::Action) -> String {
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

pub(super) const fn capture_target_kind_view(
    target: &crate::session::CaptureTarget,
) -> CaptureTargetKindView {
    match target {
        crate::session::CaptureTarget::W30Pad { .. } => CaptureTargetKindView::W30Pad,
        crate::session::CaptureTarget::Scene(_) => CaptureTargetKindView::Scene,
    }
}

pub(super) const fn capture_handoff_readiness_view(
    capture: &crate::session::CaptureRef,
) -> CaptureHandoffReadinessView {
    if capture.source_window.is_some() {
        CaptureHandoffReadinessView::Source
    } else {
        CaptureHandoffReadinessView::Unavailable
    }
}

pub(super) fn latest_w30_promoted_capture_label(session: &SessionFile) -> Option<String> {
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

pub(super) fn recent_capture_rows(session: &SessionFile) -> Vec<String> {
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

pub(super) fn latest_capture_provenance_lines(session: &SessionFile) -> Vec<String> {
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
