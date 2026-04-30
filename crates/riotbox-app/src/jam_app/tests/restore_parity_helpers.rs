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

struct SnapshotPayloadRestoreSpec<'a> {
    plan_label: &'a str,
    snapshot_id: &'a str,
    snapshot_label: &'a str,
    snapshot_created_at: &'a str,
    expected_plan_len: usize,
    anchor_plan_len: usize,
    target_plan_index: usize,
    anchor_label: &'a str,
    restore_expectation: &'a str,
}

fn run_snapshot_payload_restore_probe(
    replay_base_session: SessionFile,
    committed_state: &JamAppState,
    graph: SourceGraph,
    spec: SnapshotPayloadRestoreSpec<'_>,
    configure_replayed_state: impl FnOnce(&mut JamAppState),
) -> JamAppState {
    assert!(
        spec.target_plan_index >= spec.anchor_plan_len,
        "target action must be after the snapshot anchor"
    );

    let full_action_log = committed_state.session.action_log.clone();
    let committed_plan = riotbox_core::replay::build_committed_replay_plan(&full_action_log)
        .unwrap_or_else(|error| panic!("{}: {error:?}", spec.plan_label));
    assert_eq!(committed_plan.len(), spec.expected_plan_len);
    assert!(
        spec.target_plan_index < committed_plan.len(),
        "target action index outside committed replay plan"
    );

    let anchor_action_ids = committed_plan[..spec.anchor_plan_len]
        .iter()
        .map(|entry| entry.action.id)
        .collect::<Vec<_>>();
    let (anchor_action_cursor, anchor_session) = if let Some(anchor_action_id) =
        anchor_action_ids.last().copied()
    {
        (
            action_cursor_for(&full_action_log, anchor_action_id, "anchor"),
            materialize_replay_anchor_session(
                replay_base_session,
                full_action_log.clone(),
                &committed_plan[..spec.anchor_plan_len],
                anchor_action_ids.clone(),
                spec.anchor_label,
            ),
        )
    } else {
        let mut anchor_session = replay_base_session;
        anchor_session.action_log = full_action_log.clone();
        anchor_session.runtime_state = Default::default();
        (0, anchor_session)
    };

    run_snapshot_payload_restore_probe_from_anchor_runtime(
        committed_state,
        graph,
        spec,
        anchor_action_cursor,
        &anchor_session.runtime_state,
        configure_replayed_state,
    )
}

fn run_snapshot_payload_restore_probe_from_anchor_runtime(
    committed_state: &JamAppState,
    graph: SourceGraph,
    spec: SnapshotPayloadRestoreSpec<'_>,
    anchor_action_cursor: usize,
    anchor_runtime_state: &riotbox_core::session::RuntimeState,
    configure_replayed_state: impl FnOnce(&mut JamAppState),
) -> JamAppState {
    let full_action_log = committed_state.session.action_log.clone();
    let committed_plan = riotbox_core::replay::build_committed_replay_plan(&full_action_log)
        .unwrap_or_else(|error| panic!("{}: {error:?}", spec.plan_label));
    assert_eq!(committed_plan.len(), spec.expected_plan_len);
    assert!(
        spec.target_plan_index < committed_plan.len(),
        "target action index outside committed replay plan"
    );
    assert!(
        spec.target_plan_index >= spec.anchor_plan_len,
        "target action must be after the snapshot anchor"
    );

    let suffix_action_ids = committed_plan[spec.anchor_plan_len..=spec.target_plan_index]
        .iter()
        .map(|entry| entry.action.id)
        .collect::<Vec<_>>();
    let target_action_id = committed_plan[spec.target_plan_index].action.id;
    let target_action_cursor = action_cursor_for(&full_action_log, target_action_id, "target");
    assert!(
        anchor_action_cursor <= target_action_cursor,
        "snapshot payload anchor cursor must not be after the target cursor"
    );

    let mut restore_session = committed_state.session.clone();
    restore_session.runtime_state = Default::default();
    restore_session.snapshots = vec![snapshot_payload_for_anchor(
        spec.snapshot_id,
        spec.snapshot_label,
        spec.snapshot_created_at,
        anchor_action_cursor,
        anchor_runtime_state,
    )];

    let mut replayed_state = JamAppState::from_parts(restore_session, Some(graph), ActionQueue::new());
    configure_replayed_state(&mut replayed_state);
    let report = replayed_state
        .apply_restore_target_from_snapshot_payload(target_action_cursor)
        .unwrap_or_else(|error| panic!("{}: {error:?}", spec.restore_expectation));

    assert_restore_report_identity(
        &report,
        target_action_cursor,
        spec.snapshot_id,
        anchor_action_cursor,
        suffix_action_ids,
    );

    replayed_state
}
