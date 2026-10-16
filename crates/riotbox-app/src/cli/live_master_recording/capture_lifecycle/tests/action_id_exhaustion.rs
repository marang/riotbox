//! ID exhaustion exercises the production cleanup path with generated controls.
use std::collections::BTreeSet;

use riotbox_core::{ids::ActionId, queue::ActionIdExhausted};

use super::{Case, Event, LiveRecordingDuration, execute_with_control, fixture};

#[test]
fn exhausted_initial_play_stops_device_without_mutating_session_or_publishing() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    state
        .queue
        .reserve_action_ids_after(Some(ActionId(u64::MAX)));
    let session_before = state.session.clone();
    let queue_before = format!("{:?}", state.queue);
    let error = execute_with_control(&launch, &mut state, &mut control).unwrap_err();
    assert!(matches!(
        error.downcast_ref::<crate::jam_app::JamAppError>(),
        Some(crate::jam_app::JamAppError::ActionIdExhausted(
            ActionIdExhausted
        ))
    ));
    assert_eq!(state.session, session_before);
    assert_eq!(format!("{:?}", state.queue), queue_before);
    assert_eq!(*control.events.borrow(), [Event::Stop]);
    assert!(control.stopped.get());
    assert!(!control.playing.get());
    control.assert_unpublished();
}

#[test]
fn exhaustion_after_play_or_capture_still_stops_and_never_publishes() {
    for last_reserved in [u64::MAX - 1, u64::MAX - 2] {
        let (_temp, launch, mut state, mut control) =
            fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
        state
            .queue
            .reserve_action_ids_after(Some(ActionId(last_reserved)));
        let session_path = state.files.as_ref().unwrap().session_path.clone();
        let persisted_before = std::fs::read(&session_path).unwrap();
        let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
        assert_eq!(summary["ready"], false);
        assert_eq!(summary["runtime_stopped"], true);
        assert!(
            summary["failure_reason"]
                .as_str()
                .unwrap()
                .contains("action ID space exhausted")
        );
        assert!(
            control.stopped.get(),
            "Pause refusal must not skip device stop"
        );
        assert!(
            state.runtime.transport.is_playing,
            "never fabricate a committed Pause"
        );
        assert!(state.session.export_receipts.is_empty());
        assert!(state.queue.pending_actions().is_empty());
        let events = control.events.borrow();
        assert!(events.contains(&Event::Stop));
        assert!(!events.contains(&Event::Finish));
        if last_reserved == u64::MAX - 2 {
            assert_eq!(events[0], Event::Begin);
            assert!(events.contains(&Event::Abort));
        } else {
            assert!(!events.contains(&Event::Begin));
        }
        let actions = &state.session.action_log.actions;
        assert_eq!(state.queue.history().last().unwrap().id, ActionId(u64::MAX));
        assert_eq!(
            state
                .queue
                .history()
                .iter()
                .map(|action| action.id)
                .collect::<BTreeSet<_>>()
                .len(),
            state.queue.history().len(),
        );
        assert_eq!(
            actions
                .iter()
                .map(|action| action.id)
                .collect::<BTreeSet<_>>()
                .len(),
            actions.len()
        );
        assert_eq!(std::fs::read(&session_path).unwrap(), persisted_before);
        control.assert_unpublished();
    }
}

#[test]
fn final_three_ids_complete_play_export_pause_and_reload_without_duplicates() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    state
        .queue
        .reserve_action_ids_after(Some(ActionId(u64::MAX - 3)));
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_eq!(summary["ready"], true);
    assert_eq!(
        *control.events.borrow(),
        [Event::Begin, Event::Stop, Event::Finish]
    );
    assert!(!state.runtime.transport.is_playing);
    let path = &state.files.as_ref().unwrap().session_path;
    let mut restored =
        crate::jam_app::JamAppState::from_json_files(path, None::<&std::path::Path>).unwrap();
    assert_eq!(restored.session.action_log.actions.len(), 3);
    let ids = restored
        .session
        .action_log
        .actions
        .iter()
        .map(|action| action.id)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        ids,
        [
            ActionId(u64::MAX - 2),
            ActionId(u64::MAX - 1),
            ActionId(u64::MAX)
        ]
        .into()
    );
    assert!(matches!(
        restored.commit_transport_toggle(10_000),
        Err(crate::jam_app::JamAppError::ActionIdExhausted(
            ActionIdExhausted
        ))
    ));
}
