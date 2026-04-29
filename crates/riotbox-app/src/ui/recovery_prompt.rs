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
    ];

    lines.extend(
        surface
            .candidates
            .iter()
            .filter(|candidate| {
                !matches!(candidate.trust, RecoveryCandidateTrust::NormalLoadTarget)
            })
            .take(3)
            .map(|candidate| {
                Line::from(format!(
                    "{} | {} | {} | {}",
                    candidate.kind_label,
                    candidate.status_label,
                    candidate.action_hint,
                    recovery_candidate_file_label(candidate.path.as_path())
                ))
            }),
    );

    Some(lines)
}

fn recovery_candidate_file_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| path.display().to_string(), ToOwned::to_owned)
}
