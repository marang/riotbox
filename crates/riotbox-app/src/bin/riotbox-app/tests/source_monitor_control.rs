use super::*;
use riotbox_audio::{runtime::SourceMonitorAudioRoute, source_audio::SourceAudioCache};

#[test]
fn source_monitor_mode_commits_immediately_without_audio_runtime() {
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("session-1", "0.1.0", "2026-07-15T00:00:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );

    let committed = queue_and_commit_source_monitor_mode(&mut shell, SourceMonitorMode::Blend, 123);

    assert_eq!(committed.len(), 1);
    assert_eq!(
        committed[0].boundary.kind,
        riotbox_core::action::CommitBoundary::Immediate
    );
    assert_eq!(
        shell.app.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Blend
    );
    assert!(shell.app.queue.pending_actions().is_empty());
    assert!(shell.status_message.contains("monitor blend landed"));
}

#[test]
fn source_monitor_m_then_u_restores_runtime_route_and_replay_converges() {
    let base_session = SessionFile::new("session-1", "0.1.0", "2026-07-15T00:00:00Z");
    let source_audio_cache =
        SourceAudioCache::from_interleaved_samples("source.wav", 44_100, 2, vec![0.25; 882])
            .expect("source audio cache");
    let mut shell = JamShellState::new(
        JamAppState::from_parts(base_session.clone(), None, ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.source_audio_cache = Some(source_audio_cache.clone());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.runtime.source_monitor_audio_route,
        SourceMonitorAudioRoute::SourceOnly
    );

    let ShellKeyOutcome::QueueSourceMonitorMode(mode) =
        shell.handle_key_code(crossterm::event::KeyCode::Char('M'))
    else {
        panic!("M should request the next source monitor mode");
    };
    let committed = queue_and_commit_source_monitor_mode(&mut shell, mode, 123);

    assert_eq!(committed.len(), 1);
    assert_eq!(
        shell.app.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Blend
    );
    assert_eq!(
        shell.app.runtime.source_monitor_audio_route,
        SourceMonitorAudioRoute::Blend
    );
    assert_eq!(
        shell
            .app
            .session
            .runtime_state
            .undo_state
            .source_monitor_snapshots
            .len(),
        1
    );

    assert_eq!(
        shell.handle_key_code(crossterm::event::KeyCode::Char('u')),
        ShellKeyOutcome::UndoLast
    );
    shell
        .app
        .undo_last_action(124)
        .expect("source monitor action should undo");

    assert_eq!(
        shell.app.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Source
    );
    assert_eq!(
        shell.app.runtime.source_monitor_audio_route,
        SourceMonitorAudioRoute::SourceOnly
    );
    assert!(
        shell
            .app
            .session
            .runtime_state
            .undo_state
            .source_monitor_snapshots
            .is_empty()
    );

    let plan = riotbox_core::replay::build_committed_replay_plan(&shell.app.session.action_log)
        .expect("undone source monitor commit remains valid replay history");
    assert!(plan.is_empty());

    let mut replayed_session = base_session;
    riotbox_core::replay::apply_replay_plan_to_session(&mut replayed_session, &plan)
        .expect("empty effective replay plan");
    let mut replayed_state = JamAppState::from_parts(replayed_session, None, ActionQueue::new());
    replayed_state.source_audio_cache = Some(source_audio_cache);
    replayed_state.refresh_view();

    assert_eq!(
        replayed_state.session.runtime_state.source_monitor.mode,
        shell.app.session.runtime_state.source_monitor.mode
    );
    assert_eq!(
        replayed_state.runtime.source_monitor_audio_route,
        shell.app.runtime.source_monitor_audio_route
    );
}

#[test]
fn source_monitor_m_then_u_survives_save_and_reload_as_typed_undo_history() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("session-typed-undo", "0.1.0", "2026-07-15T00:00:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );

    let committed =
        queue_and_commit_source_monitor_mode(&mut shell, SourceMonitorMode::Blend, 123);
    assert_eq!(committed.len(), 1);
    let action_id = committed[0].action_id;
    shell
        .app
        .undo_last_action(124)
        .expect("source monitor action should undo");
    assert_eq!(
        shell
            .app
            .session
            .action_log
            .actions
            .iter()
            .find(|action| action.id == action_id)
            .expect("source monitor action")
            .status,
        riotbox_core::action::ActionStatus::Undone
    );

    save_session_json(&session_path, &shell.app.session).expect("save typed undo history");
    let reloaded = JamAppState::from_json_files(
        &session_path,
        Option::<&std::path::Path>::None,
    )
    .expect("reload typed undo history");

    assert_eq!(
        reloaded.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Source
    );
    assert!(reloaded.session.action_log.commit_records.iter().any(|record| {
        record.action_id == action_id
    }));
    assert_eq!(
        reloaded
            .session
            .action_log
            .actions
            .iter()
            .find(|action| action.id == action_id)
            .expect("reloaded source monitor action")
            .status,
        riotbox_core::action::ActionStatus::Undone
    );
}

#[test]
fn immediate_commits_around_undo_share_one_persisted_boundary_sequence() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("session-undo-sequence", "0.1.0", "2026-07-16T00:00:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );

    let first = queue_and_commit_source_monitor_mode(&mut shell, SourceMonitorMode::Blend, 123);
    assert_eq!(first[0].commit_sequence, 1);
    shell
        .app
        .undo_last_action(124)
        .expect("first monitor commit should undo");
    let second =
        queue_and_commit_source_monitor_mode(&mut shell, SourceMonitorMode::Riotbox, 125);
    assert_eq!(second[0].commit_sequence, 3);

    let records = &shell.app.session.action_log.commit_records;
    assert_eq!(
        records
            .iter()
            .map(|record| record.commit_sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert!(
        records
            .windows(2)
            .all(|pair| pair[0].boundary == pair[1].boundary)
    );
    riotbox_core::replay::build_committed_replay_plan(&shell.app.session.action_log)
        .expect("same-boundary sequence remains replayable");

    riotbox_core::persistence::save_session_json(&session_path, &shell.app.session)
        .expect("save same-boundary history");
    let reloaded = JamAppState::from_json_files(
        &session_path,
        Option::<&std::path::Path>::None,
    )
    .expect("reload same-boundary history");
    assert_eq!(
        reloaded
            .session
            .action_log
            .commit_records
            .iter()
            .map(|record| record.commit_sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn source_monitor_mode_does_not_queue_when_already_landed() {
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("session-1", "0.1.0", "2026-07-15T00:00:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );

    let committed =
        queue_and_commit_source_monitor_mode(&mut shell, SourceMonitorMode::Source, 123);

    assert!(committed.is_empty());
    assert!(shell.app.queue.pending_actions().is_empty());
    assert_eq!(shell.status_message, "monitor already source");
}

#[test]
fn live_source_monitor_observer_records_key_before_commit_at_request_timestamp() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("events.ndjson");
    let mut observer = UserSessionObserver::open(&observer_path).expect("open observer");
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("session-1", "0.1.0", "2026-07-15T00:00:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    let outcome = shell.handle_key_code(crossterm::event::KeyCode::Char('M'));
    let ShellKeyOutcome::QueueSourceMonitorMode(mode) = outcome else {
        panic!("expected M to queue source monitor mode, got {outcome:?}");
    };
    let requested_at = 1_234;
    let committed = queue_and_commit_source_monitor_mode(&mut shell, mode, requested_at);

    record_key_outcome_then_immediate_commit(
        &mut observer,
        requested_at,
        "M",
        outcome,
        &shell,
        &committed,
    )
    .expect("record ordered observer events");
    drop(observer);

    let events = fs::read_to_string(observer_path)
        .expect("read observer events")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("parse observer event"))
        .collect::<Vec<_>>();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["event"], "key_outcome");
    assert_eq!(events[0]["key"], "M");
    assert_eq!(events[0]["outcome"], "queue_source_monitor_mode");
    assert_eq!(events[1]["event"], "transport_commit");
    assert_eq!(events[0]["timestamp_ms"], requested_at);
    assert_eq!(events[1]["timestamp_ms"], requested_at);
    assert_eq!(events[1]["committed"][0]["boundary"], "Immediate");
}
