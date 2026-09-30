use super::fixtures::ghost_capture_candidate_graph;
use crate::cli::controls::accept_current_ghost_suggestion;
use crate::cli::controls::reject_current_ghost_suggestion;
use crate::cli::controls::scene_select_unavailable_status;
use crate::jam_app::JamAppState;
use crate::ui::JamShellState;
use crate::ui::ShellLaunchMode;
use riotbox_core::action::ActionCommand;
use riotbox_core::action::ActionTarget;
use riotbox_core::action::GhostMode;
use riotbox_core::action::Quantization;
use riotbox_core::action::TargetScope;
use riotbox_core::ghost::GhostSuggestedAction;
use riotbox_core::ghost::GhostSuggestionConfidence;
use riotbox_core::ghost::GhostSuggestionSafety;
use riotbox_core::ghost::GhostWatchSuggestion;
use riotbox_core::ghost::GhostWatchTool;
use riotbox_core::ids::SceneId;
use riotbox_core::queue::ActionQueue;
use riotbox_core::session::SessionFile;

#[test]
fn scene_select_unavailable_status_explains_waiting_for_scene_material() {
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-25T00:00:00Z");
    session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-01-intro")];
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));

    let shell = JamShellState::new(
        JamAppState::from_parts(session, None, ActionQueue::new()),
        ShellLaunchMode::Load,
    );

    assert_eq!(
        scene_select_unavailable_status(&shell),
        "scene jump waits for 2 scenes"
    );
}

#[test]
fn ghost_accept_control_reports_queue_or_read_only_status() {
    let mut assist_shell = ghost_shell(GhostMode::Assist);

    accept_current_ghost_suggestion(&mut assist_shell, 123);

    assert!(
        assist_shell
            .status_message
            .starts_with("accepted ghost suggestion | queued action "),
        "{}",
        assist_shell.status_message
    );
    assert!(assist_shell.app.runtime.current_ghost_suggestion.is_none());
    assert_eq!(assist_shell.app.queue.pending_actions().len(), 1);

    let mut watch_shell = ghost_shell(GhostMode::Watch);

    accept_current_ghost_suggestion(&mut watch_shell, 123);

    assert_eq!(
        watch_shell.status_message,
        "ghost accept ignored: ghost accept requires assist mode"
    );
    assert!(watch_shell.app.runtime.current_ghost_suggestion.is_some());
    assert!(watch_shell.app.queue.pending_actions().is_empty());
}

#[test]
fn ghost_reject_control_reports_clear_or_noop_status() {
    let mut shell = ghost_shell(GhostMode::Assist);

    reject_current_ghost_suggestion(&mut shell);

    assert_eq!(shell.status_message, "rejected current ghost suggestion");
    assert!(shell.app.runtime.current_ghost_suggestion.is_none());
    assert!(shell.app.session.ghost_state.suggestion_history[0].rejected);

    reject_current_ghost_suggestion(&mut shell);

    assert_eq!(
        shell.status_message,
        "ghost reject ignored: no current ghost suggestion"
    );
}

#[test]
fn ghost_accept_control_can_request_then_accept_jam_state_suggestion() {
    let mut shell = ghost_feed_shell();

    accept_current_ghost_suggestion(&mut shell, 123);

    assert_eq!(
        shell.status_message,
        "ghost suggestion ready: capture the current source-backed hit"
    );
    assert!(shell.app.runtime.current_ghost_suggestion.is_some());
    assert!(shell.app.queue.pending_actions().is_empty());

    accept_current_ghost_suggestion(&mut shell, 124);

    assert!(
        shell
            .status_message
            .starts_with("accepted ghost suggestion | queued action "),
        "{}",
        shell.status_message
    );
    assert!(shell.app.runtime.current_ghost_suggestion.is_none());
    assert_eq!(shell.app.queue.pending_actions().len(), 1);
    assert_eq!(
        shell.app.queue.pending_actions()[0].command,
        ActionCommand::CaptureNow
    );
}

fn ghost_shell(mode: GhostMode) -> JamShellState {
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-29T00:00:00Z");
    session.ghost_state.mode = mode;
    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, None, ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell
        .app
        .set_current_ghost_suggestion(sample_ghost_fill_suggestion(mode));
    shell
}

fn ghost_feed_shell() -> JamShellState {
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-29T00:00:00Z");
    session.ghost_state.mode = GhostMode::Assist;
    JamShellState::new(
        JamAppState::from_parts(
            session,
            Some(ghost_capture_candidate_graph()),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    )
}

fn sample_ghost_fill_suggestion(mode: GhostMode) -> GhostWatchSuggestion {
    GhostWatchSuggestion {
        proposal_id: "ghost-fill-1".into(),
        mode,
        tool_name: GhostWatchTool::SuggestMacroShift,
        summary: "add a next-bar drum answer".into(),
        rationale: "the current loop has room before the next scene move".into(),
        suggested_action: Some(GhostSuggestedAction {
            command: ActionCommand::Tr909FillNext,
            target: ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
            quantization: Quantization::NextBar,
            intent: "add a next-bar drum answer".into(),
        }),
        confidence: GhostSuggestionConfidence::Medium,
        safety: GhostSuggestionSafety::NeedsAssistAcceptance,
        blockers: Vec::new(),
        created_at: "2026-04-29T17:00:00Z".into(),
    }
}
