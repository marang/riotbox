use std::path::Path;

use crate::jam_app::RecoveryCandidateTrust;

fn recovery_warning_line(shell: &JamShellState) -> Option<String> {
    let surface = shell.recovery_surface.as_ref()?;
    if !surface.has_manual_candidates() {
        return None;
    }
    Some(format!(
        "recovery: {} | manual review only",
        surface.headline
    ))
}

fn recovery_help_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    let surface = shell.recovery_surface.as_ref()?;
    if !surface.has_manual_candidates() {
        return None;
    }

    let mut lines = vec![
        Line::from(""),
        Line::from("Session recovery"),
        Line::from(surface.headline.clone()),
        Line::from(surface.safety_note.clone()),
        Line::from("No candidate is selected here; reload an explicit reviewed path manually."),
        Line::from(format!(
            "Restore replay: {} | {} | {}",
            compact_restore_replay_label(&shell.app.runtime_view.replay_restore_status),
            compact_restore_replay_label(&shell.app.runtime_view.replay_restore_payload),
            restore_replay_help_scope_label(shell),
        )),
    ];
    if let Some(guidance) = surface
        .candidates
        .iter()
        .find_map(|candidate| candidate.guidance.as_ref())
    {
        lines.push(Line::from(guidance.help_label()));
    }

    for candidate in surface
        .candidates
        .iter()
        .filter(|candidate| !matches!(candidate.trust, RecoveryCandidateTrust::NormalLoadTarget))
        .take(3)
    {
        let mut replay_parts = vec![
            candidate.replay_readiness_label.clone(),
            candidate.payload_readiness_label.clone(),
        ];
        if is_actionable_replay_unsupported(&candidate.replay_unsupported_label) {
            replay_parts.push(compact_restore_replay_label(
                &candidate.replay_unsupported_label,
            ));
        } else {
            replay_parts.push(compact_restore_replay_label(&candidate.replay_suffix_label));
        }

        lines.push(Line::from(format!(
            "{} | {} | {}",
            candidate.kind_label,
            compact_recovery_decision_label(&candidate.decision_label),
            candidate.artifact_availability_label,
        )));
        lines.push(Line::from(format!(
            "  {} | {} | {} | {}",
            candidate.status_label,
            replay_parts.join(" | "),
            candidate.action_hint,
            recovery_candidate_file_label(candidate.path.as_path())
        )));
    }

    Some(lines)
}

fn restore_replay_help_scope_label(shell: &JamShellState) -> String {
    let runtime = &shell.app.runtime_view;
    if runtime.replay_restore_unsupported != "unsupported none" {
        return compact_restore_replay_label(&runtime.replay_restore_unsupported);
    }

    compact_restore_replay_label(&runtime.replay_restore_suffix)
}

fn is_actionable_replay_unsupported(label: &str) -> bool {
    label.starts_with("unsupported suffix") || label.starts_with("unsupported origin")
}

fn compact_recovery_decision_label(label: &str) -> &str {
    match label {
        "decision: normal load path" => "decision normal",
        "decision: broken candidate" => "decision broken",
        "decision: normal target missing" => "decision missing",
        "decision: blocked | replay unsupported" => "decision replay-blocked",
        "decision: blocked | replay hydration" => "decision hydration-blocked",
        "decision: blocked | replay hydration and artifacts" => "decision hydration+artifact",
        "decision: blocked | artifacts unavailable" => "decision artifact-blocked",
        "decision: blocked | replay and artifacts" => "decision multi-blocked",
        "decision: reviewable | full replay required" => "decision full-replay",
        "decision: reviewable | explicit user choice required" => "decision reviewable",
        _ => label,
    }
}

fn recovery_candidate_file_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| path.display().to_string(), ToOwned::to_owned)
}
