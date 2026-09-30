use crate::ui::performer_cues::show_restore_ready_cue;
use crate::ui::scene_labels::compact_energy_delta_label;
use crate::ui::scene_labels::compact_scene_label;
use crate::ui::scene_labels::restore_scene_energy_direction_label;
use crate::ui::scene_labels::restore_scene_target_compact_label;
use crate::ui::scene_labels::scene_energy_label_for_scene_id;
use crate::ui::scene_timing::pending_scene_transition;
use crate::ui::scene_timing::pending_scene_transition_policy;
use crate::ui::scene_timing::scene_countdown_cue;
use crate::ui::shell_state::JamShellState;
use crate::ui::shell_state::ShellScreen;
use crate::ui::styles::style_confirmation;
use crate::ui::styles::style_low_emphasis;
use crate::ui::styles::style_primary_control;
use crate::ui::styles::style_warning_detail;
use crate::ui::styles::style_warning_label;
use ratatui::text::Line;
use ratatui::text::Span;

pub(in crate::ui) fn footer_status_line(status: &str) -> Line<'static> {
    Line::from(Span::styled(status.to_owned(), style_low_emphasis()))
}

pub(in crate::ui) fn footer_ok_line(message: &str) -> Line<'static> {
    Line::from(Span::styled(message.to_owned(), style_confirmation()))
}

pub(in crate::ui) fn footer_warning_line(warning: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("Warning:", style_warning_label()),
        Span::styled(format!(" {warning}"), style_warning_detail()),
    ])
}

pub(in crate::ui) fn spans_with_primary_gesture_keys(gestures: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    for (index, gesture) in gestures.split(" | ").enumerate() {
        if index > 0 {
            spans.push(Span::raw(" | "));
        }

        let Some((key, label)) = gesture.split_once(' ') else {
            spans.push(Span::styled(gesture.to_owned(), style_primary_control()));
            continue;
        };

        spans.push(Span::styled(key.to_owned(), style_primary_control()));
        spans.push(Span::raw(format!(" {label}")));
    }

    spans
}

pub(in crate::ui) fn spans_with_primary_legend_keys(legend: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    for (index, item) in legend.split(" | ").enumerate() {
        if index > 0 {
            spans.push(Span::raw(" | "));
        }

        let Some((key, label)) = item.split_once(' ') else {
            spans.push(Span::styled(item.to_owned(), style_primary_control()));
            continue;
        };

        if key == "[" && label.starts_with("] ") {
            spans.push(Span::styled("[ ]", style_primary_control()));
            spans.push(Span::raw(label[1..].to_owned()));
            continue;
        }

        spans.push(Span::styled(key.to_owned(), style_primary_control()));
        spans.push(Span::raw(format!(" {label}")));
    }

    spans
}

pub(in crate::ui) fn footer_scene_affordance_cue(shell: &JamShellState) -> Option<String> {
    if shell.active_screen != ShellScreen::Jam {
        return None;
    }

    if let Some((kind, label, scene_id, boundary)) = pending_scene_transition(shell) {
        let scene = compact_scene_label(scene_id.as_str());
        let tick = scene_countdown_cue(shell.app.runtime.transport.beat_index);
        if let Some(policy) = pending_scene_transition_policy(shell, kind) {
            return Some(format!(
                "{label} {scene} @ {boundary} | {} {tick} | 909 {} | 202 {} | 2 trail",
                policy.direction.label(),
                policy.tr909_intent.label(),
                policy.mc202_intent.label()
            ));
        }
        if let Some(direction) = compact_energy_delta_label(
            shell.app.jam_view.scene.active_scene_energy.as_deref(),
            scene_energy_label_for_scene_id(shell, scene_id.as_str()),
        ) {
            return Some(format!(
                "{label} {scene} @ {boundary} | {direction} {tick} | 2 trail"
            ));
        }
        return Some(format!(
            "{label} {scene} @ {boundary} | {tick} energy | 2 trail"
        ));
    }

    if show_restore_ready_cue(shell) {
        let restore_target = restore_scene_target_compact_label(shell);
        if let Some(direction) = restore_scene_energy_direction_label(shell) {
            return Some(format!(
                "restore {restore_target} ready | {direction} | Y brings back {restore_target}"
            ));
        }
        return Some(format!(
            "restore {restore_target} ready | Y brings back {restore_target}"
        ));
    }

    None
}
