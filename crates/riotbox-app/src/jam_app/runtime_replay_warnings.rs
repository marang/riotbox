use riotbox_core::{
    action::ActionCommand,
    replay::{SnapshotPayloadReadiness, build_latest_snapshot_replay_convergence_summary},
    session::SessionFile,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReplayReadinessLabels {
    pub status: String,
    pub anchor: String,
    pub payload: String,
    pub suffix: String,
    pub family: String,
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
            family: "families unknown".into(),
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

    let payload = match summary.anchor_payload_readiness {
        SnapshotPayloadReadiness::NoAnchor => "payload none | full replay".into(),
        SnapshotPayloadReadiness::Missing => "payload missing | snapshot restore blocked".into(),
        SnapshotPayloadReadiness::Ready => "payload ready | snapshot restore ok".into(),
        SnapshotPayloadReadiness::Invalid => "payload invalid | snapshot restore blocked".into(),
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

    let family = if summary.suffix_action_count == 0 {
        if summary.origin_replay_entry_count == 0 {
            "families none | no replay".into()
        } else if !summary.needs_replay {
            "families none | snapshot current".into()
        } else {
            format!(
                "families none | target cursor {}",
                summary.target_action_cursor
            )
        }
    } else {
        format!(
            "families {} | suffix {}",
            replay_command_family_list(&summary.suffix_commands),
            summary.suffix_action_count
        )
    };

    ReplayReadinessLabels {
        status,
        anchor,
        payload,
        suffix,
        family,
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

fn replay_command_family_list(commands: &[ActionCommand]) -> String {
    let mut labels = Vec::new();
    for command in commands {
        let family = replay_command_family(*command);
        if !labels.contains(&family) {
            labels.push(family);
        }
    }
    if labels.is_empty() {
        "none".into()
    } else {
        labels.join(", ")
    }
}

const fn replay_command_family(command: ActionCommand) -> &'static str {
    match command {
        ActionCommand::SceneLaunch
        | ActionCommand::SceneRestore
        | ActionCommand::SceneRegenerate
        | ActionCommand::SceneReinterpret
        | ActionCommand::MutateScene
        | ActionCommand::PromoteCaptureToScene => "Scene",
        ActionCommand::Mc202GenerateFollower
        | ActionCommand::Mc202GenerateAnswer
        | ActionCommand::Mc202GeneratePressure
        | ActionCommand::Mc202GenerateInstigator
        | ActionCommand::Mc202MutatePhrase
        | ActionCommand::Mc202SetRole => "MC-202",
        ActionCommand::Tr909FillNext
        | ActionCommand::Tr909SetSlam
        | ActionCommand::Tr909ReinforceBreak
        | ActionCommand::Tr909Takeover
        | ActionCommand::Tr909SceneLock
        | ActionCommand::Tr909Release => "TR-909",
        ActionCommand::CaptureNow
        | ActionCommand::CaptureLoop
        | ActionCommand::CaptureBarGroup
        | ActionCommand::PromoteCaptureToPad
        | ActionCommand::PromoteResample
        | ActionCommand::W30CaptureToPad
        | ActionCommand::W30LiveRecall
        | ActionCommand::W30TriggerPad
        | ActionCommand::W30AuditionRawCapture
        | ActionCommand::W30AuditionPromoted
        | ActionCommand::W30SwapBank
        | ActionCommand::W30BrowseSlicePool
        | ActionCommand::W30StepFocus
        | ActionCommand::W30ApplyDamageProfile
        | ActionCommand::W30LoopFreeze => "W-30",
        ActionCommand::TransportPlay
        | ActionCommand::TransportPause
        | ActionCommand::TransportStop
        | ActionCommand::TransportSeek => "Transport",
        ActionCommand::GhostSetMode
        | ActionCommand::GhostAcceptSuggestion
        | ActionCommand::GhostRejectSuggestion
        | ActionCommand::GhostExecuteTool => "Ghost",
        ActionCommand::LockObject | ActionCommand::UnlockObject => "Lock",
        ActionCommand::SnapshotSave | ActionCommand::SnapshotLoad => "Snapshot",
        ActionCommand::UndoLast | ActionCommand::RedoLast => "Undo",
        ActionCommand::RestoreSource => "Source",
        ActionCommand::MutateLane
        | ActionCommand::MutateLoop
        | ActionCommand::MutatePattern
        | ActionCommand::MutateHook => "Mutation",
    }
}
