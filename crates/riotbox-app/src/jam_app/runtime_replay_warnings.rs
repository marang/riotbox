use riotbox_core::{
    action::ActionCommand, replay::build_latest_snapshot_replay_convergence_summary,
    session::SessionFile,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReplayReadinessLabels {
    pub status: String,
    pub anchor: String,
    pub payload: String,
    pub suffix: String,
    pub unsupported: String,
}

pub(super) fn derive_replay_readiness_labels(session: &SessionFile) -> ReplayReadinessLabels {
    let Ok(summary) =
        build_latest_snapshot_replay_convergence_summary(&session.action_log, &session.snapshots)
    else {
        return ReplayReadinessLabels {
            status: "restore replay unavailable".into(),
            anchor: "anchor invalid".into(),
            payload: "payload unknown".into(),
            suffix: "suffix unknown".into(),
            unsupported: "unsupported unknown".into(),
        };
    };

    let status = if summary.suffix_unsupported_action_count > 0 {
        format!(
            "blocked: {} unsupported suffix action(s)",
            summary.suffix_unsupported_action_count
        )
    } else if summary.origin_unsupported_action_count > 0 {
        format!(
            "blocked: {} unsupported origin action(s)",
            summary.origin_unsupported_action_count
        )
    } else if summary.origin_replay_entry_count == 0 && summary.anchor_snapshot_id.is_none() {
        "ready: no replay entries".into()
    } else if !summary.needs_replay {
        "ready: snapshot current".into()
    } else if summary.needs_full_replay {
        format!(
            "ready: full replay {} action(s)",
            summary.suffix_action_count
        )
    } else {
        format!(
            "ready: replay {} suffix action(s)",
            summary.suffix_action_count
        )
    };

    let anchor = match (
        summary.anchor_snapshot_id.as_deref(),
        summary.anchor_action_cursor,
    ) {
        (Some(snapshot_id), Some(cursor)) => format!("anchor {snapshot_id} @ cursor {cursor}"),
        (Some(snapshot_id), None) => format!("anchor {snapshot_id}"),
        (None, _) => "anchor none | full replay".into(),
    };

    let payload = match (
        summary.anchor_snapshot_id.as_deref(),
        summary.anchor_action_cursor,
    ) {
        (Some(snapshot_id), Some(cursor)) => session
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot.snapshot_id.as_str() == snapshot_id && snapshot.action_cursor == cursor
            })
            .and_then(|snapshot| snapshot.payload.as_ref().map(|payload| (snapshot, payload)))
            .map_or_else(
                || "payload missing | snapshot restore blocked".into(),
                |(snapshot, payload)| {
                    if payload.snapshot_id == snapshot.snapshot_id
                        && payload.action_cursor == snapshot.action_cursor
                    {
                        "payload ready | snapshot restore ok".into()
                    } else {
                        "payload invalid | snapshot restore blocked".into()
                    }
                },
            ),
        (Some(_), None) => "payload unknown | anchor cursor missing".into(),
        (None, _) => "payload none | full replay".into(),
    };

    let suffix = if summary.suffix_action_count == 0 {
        format!(
            "suffix none | target cursor {}",
            summary.target_action_cursor
        )
    } else {
        format!(
            "suffix {} action(s): {}",
            summary.suffix_action_count,
            replay_command_list(&summary.suffix_commands)
        )
    };

    let unsupported = if summary.suffix_unsupported_action_count > 0 {
        format!(
            "unsupported suffix {}: {}",
            summary.suffix_unsupported_action_count,
            replay_command_list(&summary.suffix_unsupported_commands)
        )
    } else if summary.origin_unsupported_action_count > 0 {
        format!(
            "unsupported origin {}: {}",
            summary.origin_unsupported_action_count,
            replay_command_list(&summary.origin_unsupported_commands)
        )
    } else {
        "unsupported none".into()
    };

    ReplayReadinessLabels {
        status,
        anchor,
        payload,
        suffix,
        unsupported,
    }
}

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
