use ratatui::text::{Line, Span};
use riotbox_core::action::{ActionCommand, ActionStatus};

use super::{
    JamShellState, compact_scene_label, current_scene_target_compact_label,
    restore_scene_target_compact_label, style_confirmation_strong, style_low_emphasis,
    style_pending_detail, style_primary_control,
};

pub(super) fn scene_history_trail_line(shell: &JamShellState) -> Option<String> {
    let trail = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .filter(|action| {
            action.status == ActionStatus::Committed
                && matches!(
                    action.command,
                    ActionCommand::SceneLaunch | ActionCommand::SceneRestore
                )
        })
        .take(3)
        .map(|action| {
            let verb = match action.command {
                ActionCommand::SceneLaunch => "jump",
                ActionCommand::SceneRestore => "restore",
                _ => unreachable!("scene trail filter only matches launch/restore"),
            };
            let scene = action
                .result
                .as_ref()
                .and_then(|result| result.summary.split_whitespace().nth(2))
                .or(action
                    .target
                    .scene_id
                    .as_ref()
                    .map(|scene_id| scene_id.as_str()))
                .map(compact_scene_label)
                .unwrap_or_else(|| "none".into());
            format!("{verb} {scene}")
        })
        .collect::<Vec<_>>();

    if trail.is_empty() {
        None
    } else {
        Some(format!("trail {}", trail.join(" <- ")))
    }
}

pub(super) fn latest_landed_command(shell: &JamShellState) -> Option<&str> {
    shell
        .app
        .jam_view
        .recent_actions
        .first()
        .map(|action| action.command.as_str())
}

pub(super) fn scene_post_commit_cue_line(shell: &JamShellState) -> Option<Line<'static>> {
    let command = latest_landed_command(shell)?;
    if !matches!(command, "scene.launch" | "scene.restore") {
        return None;
    }

    let current_scene = current_scene_target_compact_label(shell);
    let restore_scene = restore_scene_target_compact_label(shell);
    let next_scene_key = if command == "scene.launch" {
        ("[Y]", " restore ")
    } else {
        ("[y]", " jump ")
    };

    let mut spans = vec![
        Span::styled("scene ", style_low_emphasis()),
        Span::styled(current_scene, style_confirmation_strong()),
        Span::styled(" | restore ", style_low_emphasis()),
        Span::styled(restore_scene, style_pending_detail()),
    ];

    if shell.app.runtime_view.tr909_render_support_accent == "scene" {
        spans.push(Span::styled(" | ", style_low_emphasis()));
        spans.push(Span::styled("909 lift", style_pending_detail()));
    }

    if let Some(movement) = shell.app.jam_view.scene.last_movement.as_ref() {
        spans.push(Span::styled(" | move ", style_low_emphasis()));
        spans.push(Span::styled(
            movement.direction.clone(),
            style_confirmation_strong(),
        ));
        spans.push(Span::styled(" 909 ", style_low_emphasis()));
        spans.push(Span::styled(
            movement.tr909_intent.clone(),
            style_pending_detail(),
        ));
        spans.push(Span::styled(" 202 ", style_low_emphasis()));
        spans.push(Span::styled(
            movement.mc202_intent.clone(),
            style_pending_detail(),
        ));
    }

    spans.extend([
        Span::styled(" | next ", style_low_emphasis()),
        Span::styled(next_scene_key.0, style_primary_control()),
        Span::raw(next_scene_key.1),
        Span::styled("[c]", style_primary_control()),
        Span::raw(" capture"),
    ]);

    Some(Line::from(spans))
}
