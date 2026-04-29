use riotbox_core::{
    action::ActionCommand, replay::build_latest_snapshot_replay_convergence_summary,
    session::SessionFile,
};

pub(super) fn derive_replay_summary_warnings(session: &SessionFile) -> Vec<String> {
    let Ok(summary) =
        build_latest_snapshot_replay_convergence_summary(&session.action_log, &session.snapshots)
    else {
        return Vec::new();
    };

    if summary.suffix_unsupported_action_count > 0 {
        return vec![format!(
            "replay cannot cover {} unsupported command(s) after snapshot: {}",
            summary.suffix_unsupported_action_count,
            replay_command_list(&summary.suffix_unsupported_commands)
        )];
    }

    if summary.origin_unsupported_action_count > 0 {
        return vec![format!(
            "replay from origin cannot cover {} unsupported command(s) already inside snapshot: {}",
            summary.origin_unsupported_action_count,
            replay_command_list(&summary.origin_unsupported_commands)
        )];
    }

    Vec::new()
}

fn replay_command_list(commands: &[ActionCommand]) -> String {
    let mut labels: Vec<_> = commands
        .iter()
        .take(3)
        .map(|command| command.as_str())
        .collect();
    if commands.len() > labels.len() {
        labels.push("...");
    }
    labels.join(", ")
}
