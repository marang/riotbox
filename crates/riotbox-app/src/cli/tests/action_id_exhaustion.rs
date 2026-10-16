//! Source-free checks of the existing musician-facing error boundary.
use crate::{
    cli::controls::{
        commit_capture_length_change, commit_transport_toggle, execute_product_mix_export,
        queue_and_commit_performance_preset, queue_and_commit_source_monitor_mode,
    },
    jam_app::JamAppState,
    ui::{JamShellState, ShellLaunchMode},
};
use riotbox_core::{
    action::SourceMonitorMode, ids::ActionId, queue::ActionQueue, session::SessionFile,
    style::PerformancePresetId,
};

fn shell() -> JamShellState {
    JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("generated-exhaustion", "test", "2026-10-16T00:00:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    )
}

#[test]
fn control_errors_are_visible_without_fake_success_or_product_mutation() {
    let operations: [fn(&mut JamShellState); 5] = [
        |shell| assert!(commit_transport_toggle(shell, 100).is_empty()),
        |shell| {
            assert!(
                queue_and_commit_source_monitor_mode(shell, SourceMonitorMode::Blend, 100)
                    .is_empty()
            )
        },
        |shell| {
            assert!(
                queue_and_commit_performance_preset(
                    shell,
                    PerformancePresetId::FeralBreakAlphaV2,
                    100
                )
                .is_empty()
            )
        },
        |shell| commit_capture_length_change(shell, 100, true),
        |shell| execute_product_mix_export(shell, None, 100),
    ];
    for operation in operations {
        let mut shell = shell();
        shell
            .app
            .queue
            .reserve_action_ids_after(Some(ActionId(u64::MAX)));
        let session = shell.app.session.clone();
        let runtime = shell.app.runtime.clone();
        let queue = format!("{:?}", shell.app.queue);
        let jam_view = shell.app.jam_view.clone();
        let runtime_view = shell.app.runtime_view.clone();
        operation(&mut shell);
        assert_eq!(shell.status_message, "action ID space exhausted");
        assert_eq!(shell.app.session, session);
        assert_eq!(shell.app.runtime, runtime);
        assert_eq!(format!("{:?}", shell.app.queue), queue);
        assert_eq!(shell.app.jam_view, jam_view);
        assert_eq!(shell.app.runtime_view, runtime_view);
    }
}

#[test]
fn already_pending_keeps_its_status_even_after_last_id_was_issued() {
    let mut shell = shell();
    shell
        .app
        .queue
        .reserve_action_ids_after(Some(ActionId(u64::MAX - 1)));
    shell
        .app
        .queue_source_monitor_mode(SourceMonitorMode::Blend, 100)
        .unwrap();
    let queue = format!("{:?}", shell.app.queue);
    assert!(
        queue_and_commit_source_monitor_mode(&mut shell, SourceMonitorMode::Blend, 200).is_empty()
    );
    assert_eq!(shell.status_message, "source monitor change already queued");
    assert_eq!(format!("{:?}", shell.app.queue), queue);
}
