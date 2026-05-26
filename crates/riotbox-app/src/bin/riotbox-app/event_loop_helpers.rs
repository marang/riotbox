fn accept_current_ghost_suggestion(shell: &mut JamShellState, requested_at: u64) {
    match shell.app.accept_current_ghost_suggestion(requested_at) {
        riotbox_app::jam_app::GhostSuggestionQueueResult::Enqueued(action_id) => {
            shell.set_error_status(format!(
                "accepted ghost suggestion | queued action {}",
                action_id.0
            ));
        }
        riotbox_app::jam_app::GhostSuggestionQueueResult::Rejected { reason } => {
            if reason == riotbox_app::jam_app::NO_CURRENT_GHOST_SUGGESTION_REASON
                && shell.app.refresh_current_ghost_suggestion_from_jam_state()
                && let Some(suggestion) = shell.app.runtime.current_ghost_suggestion.as_ref()
            {
                shell.set_error_status(format!("ghost suggestion ready: {}", suggestion.summary));
            } else {
                shell.set_error_status(format!("ghost accept ignored: {reason}"));
            }
        }
    }
}

fn reject_current_ghost_suggestion(shell: &mut JamShellState) {
    if shell.app.reject_current_ghost_suggestion() {
        shell.set_error_status("rejected current ghost suggestion");
    } else {
        shell.set_error_status("ghost reject ignored: no current ghost suggestion");
    }
}

fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn scene_select_unavailable_status(shell: &JamShellState) -> &'static str {
    match shell.app.jam_view.scene.scene_jump_availability {
        SceneJumpAvailabilityView::WaitingForMoreScenes => "scene jump waits for 2 scenes",
        SceneJumpAvailabilityView::Ready | SceneJumpAvailabilityView::Unknown => {
            "no next scene candidate available"
        }
    }
}
