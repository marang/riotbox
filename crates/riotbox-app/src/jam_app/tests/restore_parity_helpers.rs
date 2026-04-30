fn action_cursor_for(
    action_log: &riotbox_core::session::ActionLog,
    action_id: ActionId,
    label: &str,
) -> usize {
    action_log
        .actions
        .iter()
        .position(|action| action.id == action_id)
        .unwrap_or_else(|| panic!("{label} action exists in action log"))
        + 1
}

fn materialize_replay_anchor_session(
    mut base_session: SessionFile,
    full_action_log: riotbox_core::session::ActionLog,
    prefix: &[riotbox_core::replay::ReplayPlanEntry<'_>],
    expected_action_ids: Vec<ActionId>,
    label: &str,
) -> SessionFile {
    base_session.action_log = full_action_log;
    let anchor_report = riotbox_core::replay::apply_replay_plan_to_session(&mut base_session, prefix)
        .unwrap_or_else(|error| panic!("{label}: {error:?}"));
    assert_eq!(anchor_report.applied_action_ids, expected_action_ids);
    base_session
}

fn snapshot_payload_for_anchor(
    snapshot_id: &str,
    label: &str,
    created_at: &str,
    action_cursor: usize,
    runtime_state: &riotbox_core::session::RuntimeState,
) -> Snapshot {
    let snapshot_id = SnapshotId::from(snapshot_id);
    Snapshot {
        snapshot_id: snapshot_id.clone(),
        created_at: created_at.into(),
        label: label.into(),
        action_cursor,
        payload: Some(riotbox_core::session::SnapshotPayload {
            payload_version: riotbox_core::session::SnapshotPayloadVersion::V1,
            snapshot_id,
            action_cursor,
            runtime_state: runtime_state.clone(),
        }),
    }
}

fn assert_restore_report_identity(
    report: &riotbox_core::replay::ReplayTargetExecutionReport,
    target_action_cursor: usize,
    anchor_snapshot_id: &str,
    anchor_action_cursor: usize,
    applied_action_ids: Vec<ActionId>,
) {
    assert_eq!(report.target_action_cursor, target_action_cursor);
    assert_eq!(
        report.anchor_snapshot_id.as_deref(),
        Some(anchor_snapshot_id)
    );
    assert_eq!(report.anchor_action_cursor, Some(anchor_action_cursor));
    assert_eq!(report.applied_action_ids, applied_action_ids);
}
