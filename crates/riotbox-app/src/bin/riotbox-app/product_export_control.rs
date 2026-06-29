fn queue_product_mix_export(shell: &mut JamShellState, requested_at: u64) {
    match shell.app.queue_product_mix_export(requested_at, None) {
        crate::jam_app::QueueControlResult::Enqueued => {
            shell.set_error_status("queued export full_grid_mix | proof handoff writes receipt");
        }
        crate::jam_app::QueueControlResult::AlreadyPending => {
            shell.set_error_status("export full_grid_mix already queued");
        }
        crate::jam_app::QueueControlResult::AlreadyInState => {
            shell.set_error_status("export full_grid_mix already available");
        }
    }
}
