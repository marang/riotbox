use super::*;

pub(super) fn supported_artifact_replay_hydration_blocker(
    candidate: &riotbox_core::persistence::SessionRecoveryCandidate,
) -> Option<String> {
    if !matches!(
        candidate.status,
        SessionRecoveryCandidateStatus::ParseableSession
    ) {
        return None;
    }

    let Ok(session) = load_session_json(&candidate.path) else {
        return None;
    };
    let target_action_cursor = session.action_log.actions.len();
    if target_action_cursor == 0 {
        return None;
    }

    match riotbox_core::replay::hydrate_replay_target_from_snapshot_payload(
        &session,
        target_action_cursor,
        None,
    ) {
        Err(riotbox_core::replay::SnapshotPayloadHydrationError::Execution(error)) => {
            artifact_hydration_blocker_label(&error)
        }
        Err(riotbox_core::replay::SnapshotPayloadHydrationError::MissingSnapshotAnchor {
            ..
        }) => {
            let mut replay_session = session;
            match riotbox_core::replay::apply_replay_target_suffix_to_session(
                &mut replay_session,
                target_action_cursor,
                None,
            ) {
                Err(error) => artifact_hydration_blocker_label(&error),
                Ok(_) => None,
            }
        }
        Ok(_) | Err(_) => None,
    }
}

fn artifact_hydration_blocker_label(
    error: &riotbox_core::replay::ReplayTargetExecutionError,
) -> Option<String> {
    let riotbox_core::replay::ReplayTargetExecutionError::Execution(
        riotbox_core::replay::ReplayExecutionError::ArtifactHydration {
            action_id,
            command,
            reason,
        },
    ) = error
    else {
        return None;
    };

    Some(format!(
        "{} action {} cannot hydrate persisted artifact: {}",
        command.as_str(),
        action_id,
        artifact_hydration_reason_label(reason)
    ))
}

fn artifact_hydration_reason_label(
    reason: &riotbox_core::replay::W30ArtifactReplayHydrationError,
) -> String {
    match reason {
        riotbox_core::replay::W30ArtifactReplayHydrationError::NotArtifactProducingW30Action {
            action_id,
            command,
        } => format!(
            "{} action {} is not an artifact-producing W-30 replay action",
            command.as_str(),
            action_id
        ),
        riotbox_core::replay::W30ArtifactReplayHydrationError::MissingSourceCaptureTarget {
            action_id,
            command,
        } => format!(
            "missing source capture target on {} action {}",
            command.as_str(),
            action_id
        ),
        riotbox_core::replay::W30ArtifactReplayHydrationError::MissingProducedCapture {
            action_id,
            command,
        } => format!(
            "missing produced capture for {} action {}",
            command.as_str(),
            action_id
        ),
        riotbox_core::replay::W30ArtifactReplayHydrationError::MissingSourceCapture {
            capture_id,
        } => format!("missing source capture {capture_id}"),
        riotbox_core::replay::W30ArtifactReplayHydrationError::AmbiguousProducedCapture {
            action_id,
            command,
            capture_count,
        } => format!(
            "ambiguous produced capture identity for {} action {}: {} captures",
            command.as_str(),
            action_id,
            capture_count
        ),
        riotbox_core::replay::W30ArtifactReplayHydrationError::MissingStoragePath {
            capture_id,
        } => format!("missing storage path for capture {capture_id}"),
        riotbox_core::replay::W30ArtifactReplayHydrationError::MissingSourceWindowForSourceBackedCapture {
            capture_id,
        } => format!("missing source-window identity for capture {capture_id}"),
        riotbox_core::replay::W30ArtifactReplayHydrationError::InvalidPadCaptureIdentity {
            capture_id,
        } => format!("invalid pad capture identity for capture {capture_id}"),
        riotbox_core::replay::W30ArtifactReplayHydrationError::InvalidLoopCaptureIdentity {
            capture_id,
        } => format!("invalid loop capture identity for capture {capture_id}"),
        riotbox_core::replay::W30ArtifactReplayHydrationError::InvalidResampleIdentity {
            capture_id,
        } => format!("invalid resample identity for capture {capture_id}"),
        riotbox_core::replay::W30ArtifactReplayHydrationError::SourceCaptureLineageMismatch {
            produced_capture_id,
            source_capture_id,
        } => format!(
            "lineage mismatch between produced capture {produced_capture_id} and source capture {source_capture_id}"
        ),
    }
}
