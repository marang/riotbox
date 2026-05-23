fn commit_capture_length_change(shell: &mut JamShellState, requested_at: u64, next: bool) {
    let result = if next {
        shell.app.queue_next_capture_length_intent(requested_at)
    } else {
        shell.app.queue_previous_capture_length_intent(requested_at)
    };
    match result {
        riotbox_app::jam_app::QueueControlResult::Enqueued => {
            let transport = shell.app.runtime.transport.clone();
            let committed = shell.app.commit_ready_actions(
                riotbox_core::transport::CommitBoundaryState {
                    kind: riotbox_core::action::CommitBoundary::Immediate,
                    beat_index: transport.beat_index,
                    bar_index: transport.bar_index,
                    phrase_index: transport.phrase_index,
                    scene_id: transport.current_scene,
                },
                requested_at,
            );
            if committed.is_empty() {
                shell.set_error_status("capture length change queued");
            } else {
                shell.set_error_status(format!(
                    "capture length {}",
                    shell.app.session.runtime_state.capture.length_intent
                ));
            }
        }
        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
            shell.set_error_status("capture length change already queued");
        }
        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
            shell.set_error_status(format!(
                "capture length already {}",
                shell.app.session.runtime_state.capture.length_intent
            ));
        }
    }
}
