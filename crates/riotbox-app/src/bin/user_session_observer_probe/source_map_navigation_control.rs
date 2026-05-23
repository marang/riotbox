use riotbox_app::{
    jam_app::{SourceMapNavigationIntent, SourceMapNavigationResult},
    ui::JamShellState,
};
use riotbox_core::{
    action::CommitBoundary, queue::CommittedActionRef, transport::CommitBoundaryState,
};

pub(super) fn apply_source_map_navigation(
    shell: &mut JamShellState,
    intent: SourceMapNavigationIntent,
    requested_at: u64,
) -> Vec<CommittedActionRef> {
    match shell.app.queue_source_map_navigation(intent, requested_at) {
        SourceMapNavigationResult::Enqueued {
            target_label,
            target_position_beats,
        } => {
            let transport = shell.app.runtime.transport.clone();
            let committed = shell.app.commit_ready_actions(
                CommitBoundaryState {
                    kind: CommitBoundary::Immediate,
                    beat_index: transport.beat_index,
                    bar_index: transport.bar_index,
                    phrase_index: transport.phrase_index,
                    scene_id: transport.current_scene,
                },
                requested_at,
            );
            if committed.is_empty() {
                shell.set_error_status(format!("source map navigation queued to {target_label}"));
            } else {
                shell.set_error_status(format!(
                    "source map moved to {target_label} @ beat {target_position_beats}"
                ));
            }
            committed
        }
        SourceMapNavigationResult::AlreadyPending => {
            shell.set_error_status("source map navigation already queued");
            Vec::new()
        }
        SourceMapNavigationResult::AlreadyAtBoundary { target_label } => {
            shell.set_error_status(format!("source map already at {target_label}"));
            Vec::new()
        }
        SourceMapNavigationResult::Unavailable { reason } => {
            shell.set_error_status(reason);
            Vec::new()
        }
    }
}
