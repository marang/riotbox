fn confirm_source_timing_grid(shell: &mut JamShellState, requested_at: u64) {
    match shell
        .app
        .queue_source_timing_grid_confirmation(requested_at)
    {
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
                shell.set_error_status("source timing grid confirmation queued");
            } else {
                shell.set_error_status("confirmed source timing grid");
            }
        }
        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
            shell.set_error_status("source timing grid confirmation already queued");
        }
        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
            if shell.app.source_graph.is_some() {
                shell.set_error_status("source timing grid already confirmed");
            } else {
                shell.set_error_status("no source timing grid available to confirm");
            }
        }
    }
}
