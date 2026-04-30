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
    if surface
        .candidates
        .iter()
        .any(has_artifact_backed_replay_blocker_hint)
    {
        lines.push(Line::from(
            "Artifact note: audio present, but W-30 artifact replay hydration is not built yet",
        ));
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
            candidate.kind_label, candidate.status_label, candidate.artifact_availability_label,
        )));
        lines.push(Line::from(format!(
            "  {} | {} | {}",
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

fn has_artifact_backed_replay_blocker_hint(
    candidate: &crate::jam_app::SessionRecoveryCandidateView,
) -> bool {
    let unsupported_artifact_command = candidate.replay_unsupported_label.contains("w30.loop_freeze")
        || candidate
            .replay_unsupported_label
            .contains("promote.resample");
    candidate.artifact_availability_label.starts_with("artifacts ready:")
        && is_actionable_replay_unsupported(&candidate.replay_unsupported_label)
        && unsupported_artifact_command
}

fn recovery_candidate_file_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| path.display().to_string(), ToOwned::to_owned)
}
