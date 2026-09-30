use crate::ui::shell_state::JamShellState;
use crate::ui::shell_state::ShellScreen;

pub(in crate::ui) fn screen_context_label(shell: &JamShellState) -> String {
    match shell.active_screen {
        ShellScreen::Jam => format!("jam/{}", shell.jam_mode.label()),
        _ => shell.active_screen.label().into(),
    }
}

pub(in crate::ui) fn transport_label(shell: &JamShellState) -> &'static str {
    if shell.app.jam_view.transport.is_playing {
        "playing"
    } else {
        "idle"
    }
}

pub(in crate::ui) fn ghost_label(shell: &JamShellState) -> String {
    let ghost = &shell.app.jam_view.ghost;
    let mode = if ghost.is_read_only {
        format!("{} ro", ghost.mode)
    } else {
        ghost.mode.clone()
    };
    let blocker = ghost.active_blocker.as_deref().map_or_else(
        || ghost.safety.clone(),
        |blocker| format!("blocked {blocker}"),
    );

    let status = ghost.latest_status.as_deref().unwrap_or("idle");
    let decision = ghost.decision_hint.as_deref().unwrap_or("no suggestion");

    if ghost.is_blocked {
        format!("{blocker} | {status}")
    } else {
        format!("{mode} {decision} | {status}")
    }
}
