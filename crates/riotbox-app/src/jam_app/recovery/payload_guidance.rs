use riotbox_core::replay::SnapshotPayloadReadiness;

use super::*;

pub(super) fn missing_snapshot_payload_guidance(
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
    let Ok(summary) = riotbox_core::replay::build_latest_snapshot_replay_convergence_summary(
        &session.action_log,
        &session.snapshots,
    ) else {
        return None;
    };
    if summary.anchor_payload_readiness != SnapshotPayloadReadiness::Missing {
        return None;
    }

    let snapshot = summary
        .anchor_snapshot_id
        .as_deref()
        .unwrap_or("unknown snapshot");
    let cursor = summary
        .anchor_action_cursor
        .map_or_else(|| "unknown cursor".into(), |cursor| cursor.to_string());

    Some(format!(
        "snapshot {snapshot} at cursor {cursor} has no payload; recovery would need full replay to target cursor {}",
        summary.target_action_cursor
    ))
}
