use crate::ui::recovery_prompt::recovery_warning_line;
use crate::ui::shell_labels::transport_label;
use crate::ui::shell_state::JamShellState;
use crate::ui::source_trust_summary::arrangement_inspect_lines;
use crate::ui::source_trust_summary::arrangement_proof_line;
use crate::ui::source_trust_summary::arrangement_taste_line;
use crate::ui::source_trust_summary::source_timing_warning_line;
use crate::ui::source_trust_summary::trust_summary;
use ratatui::text::Line;

pub(in crate::ui) fn jam_warning_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let trust = trust_summary(shell);
    let readiness = if trust.headline == "strong" || trust.headline == "usable" {
        "ready"
    } else {
        "tentative"
    };

    let mut lines = vec![Line::from(format!(
        "trust {} | {}",
        trust.headline, readiness
    ))];

    let recovery = recovery_warning_line(shell);
    if let Some(recovery) = recovery.as_ref() {
        lines.push(Line::from(recovery.clone()));
    }

    lines.extend([
        arrangement_taste_line(shell),
        Line::from(arrangement_proof_line(shell)),
        Line::from(source_timing_warning_line(shell)),
    ]);

    if recovery.is_none() {
        lines.push(Line::from(primary_warning_line(shell)));
    }

    lines.push(Line::from(format!(
        "audio {} | sidecar {}",
        shell.app.runtime_view.audio_status, shell.app.runtime_view.sidecar_status
    )));

    lines
}

pub(in crate::ui) fn primary_warning_line(shell: &JamShellState) -> String {
    if let Some(recovery) = recovery_warning_line(shell) {
        return recovery;
    }

    shell
        .app
        .runtime_view
        .runtime_warnings
        .iter()
        .chain(shell.app.jam_view.warnings.iter())
        .next()
        .map(|warning| warning.to_string())
        .unwrap_or_else(|| "no major warning".into())
}

pub(in crate::ui) fn jam_diagnostic_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let last_boundary = shell
        .app
        .runtime
        .last_commit_boundary
        .as_ref()
        .map(|boundary| {
            format!(
                "{:?} b{} p{}",
                boundary.kind, boundary.bar_index, boundary.phrase_index
            )
        })
        .unwrap_or_else(|| "none".into());

    let mut lines = arrangement_inspect_lines(shell);
    lines.extend([
        Line::from(format!(
            "audio {} | sidecar {}",
            shell.app.runtime_view.audio_status, shell.app.runtime_view.sidecar_status
        )),
        Line::from(format!(
            "transport {} @ {:.1}",
            transport_label(shell),
            shell.app.runtime.transport.position_beats
        )),
        Line::from(format!("last boundary {last_boundary}")),
        Line::from(format!(
            "pending {} | landed {}",
            shell.app.jam_view.pending_actions.len(),
            shell.app.jam_view.recent_actions.len()
        )),
    ]);
    lines.push(Line::from(primary_warning_line(shell)));
    lines
}

pub(in crate::ui) fn log_warning_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let warnings: Vec<_> = shell
        .app
        .runtime_view
        .runtime_warnings
        .iter()
        .chain(shell.app.jam_view.warnings.iter())
        .take(2)
        .cloned()
        .collect();
    let restore_lines = restore_replay_log_lines(shell);
    if warnings.is_empty() && restore_lines.is_empty() {
        return vec![Line::from("no active runtime or trust warnings")];
    }

    let mut lines = restore_lines;
    lines.extend(
        warnings
            .into_iter()
            .map(|warning| Line::from(format!("warning {warning}"))),
    );
    lines
}

pub(in crate::ui) fn restore_replay_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let runtime = &shell.app.runtime_view;
    if runtime.replay_restore_status == "ready: no replay entries" {
        return Vec::new();
    }

    let mut lines = vec![
        Line::from(compact_restore_replay_label(&runtime.replay_restore_status)),
        Line::from(compact_restore_replay_label(&runtime.replay_restore_anchor)),
        Line::from(compact_restore_replay_label(
            &runtime.replay_restore_payload,
        )),
    ];
    if runtime.replay_restore_unsupported != "unsupported none" {
        lines.push(Line::from(compact_restore_replay_label(
            &runtime.replay_restore_unsupported,
        )));
    } else {
        lines.push(Line::from(compact_restore_replay_label(
            &runtime.replay_restore_suffix,
        )));
    }
    lines
}

pub(in crate::ui) fn compact_restore_replay_label(label: &str) -> String {
    let mut compact = label.strip_prefix("ready: ").unwrap_or(label).to_owned();
    compact = compact
        .replace("suffix 1 action(s): ", "suffix ")
        .replace("unsupported suffix 1: ", "unsupported suffix ")
        .replace("unsupported origin 1: ", "unsupported origin ")
        .replace("suffix none | target cursor ", "suffix none@")
        .replace("payload ready | snapshot restore ok", "payload ready")
        .replace(
            "payload missing | snapshot restore blocked",
            "payload missing",
        )
        .replace("payload none | full replay", "payload none")
        .replace(" action(s)", "")
        .replace(" @ cursor ", "@");
    compact
}
