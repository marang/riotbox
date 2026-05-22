use riotbox_core::action::Quantization;
use riotbox_core::view::jam::SceneJumpAvailabilityView;

use super::{JamShellState, energy_label, ghost_label, transport_label};

pub(super) fn now_line(shell: &JamShellState) -> String {
    let scene = shell
        .app
        .jam_view
        .scene
        .active_scene
        .as_deref()
        .unwrap_or("no scene");
    format!(
        "{} at beat {:.1} in {}",
        transport_label(shell),
        shell.app.jam_view.transport.position_beats,
        scene
    )
}

pub(super) fn next_action_line(shell: &JamShellState) -> String {
    if let Some(action) = shell.app.jam_view.pending_actions.first() {
        format!(
            "{} {} @ {}",
            action.actor, action.command, action.quantization
        )
    } else {
        "no pending action queued".into()
    }
}

pub(super) fn next_scene_jump_suggestion(shell: &JamShellState) -> String {
    let Some(scene_id) = shell.app.jam_view.scene.next_scene.as_deref() else {
        match shell.app.jam_view.scene.scene_jump_availability {
            SceneJumpAvailabilityView::WaitingForMoreScenes => {
                return "[y] jump waits for 2 scenes".into();
            }
            SceneJumpAvailabilityView::Ready | SceneJumpAvailabilityView::Unknown => {
                return "[y] jump".into();
            }
        }
    };

    let scene = compact_scene_label(scene_id);
    match compact_energy_delta_label(
        shell.app.jam_view.scene.active_scene_energy.as_deref(),
        shell.app.jam_view.scene.next_scene_energy.as_deref(),
    ) {
        Some(direction) => format!("[y] jump {scene} ({direction})"),
        None => format!("[y] jump {scene}"),
    }
}

pub(super) fn next_scene_target_compact_label(shell: &JamShellState) -> String {
    let Some(scene_id) = shell.app.jam_view.scene.next_scene.as_deref() else {
        match shell.app.jam_view.scene.scene_jump_availability {
            SceneJumpAvailabilityView::WaitingForMoreScenes => {
                return "waits for 2 scenes".into();
            }
            SceneJumpAvailabilityView::Ready | SceneJumpAvailabilityView::Unknown => {
                return "none".into();
            }
        }
    };

    let scene = compact_scene_label(scene_id);
    if let Some(energy) = shell.app.jam_view.scene.next_scene_energy.as_deref() {
        return format!("{scene}/{}", compact_energy_label(energy));
    }
    scene
}

pub(super) fn current_scene_compact_label(shell: &JamShellState) -> String {
    let scene_id = current_scene_id(shell).unwrap_or_else(|| "none".into());

    compact_scene_label(scene_id.as_str())
}

pub(super) fn current_scene_target_compact_label(shell: &JamShellState) -> String {
    let scene = current_scene_compact_label(shell);
    if let Some(energy) = shell.app.jam_view.scene.active_scene_energy.as_deref() {
        return format!("{scene}/{}", compact_energy_label(energy));
    }
    scene
}

fn current_scene_id(shell: &JamShellState) -> Option<String> {
    shell.app.jam_view.scene.active_scene.clone()
}

pub(super) fn scene_restore_contrast_line(shell: &JamShellState) -> String {
    let current_scene = current_scene_compact_label(shell);
    let current_energy = shell
        .app
        .jam_view
        .scene
        .active_scene_energy
        .as_deref()
        .map(compact_energy_label)
        .unwrap_or("unk");
    let restore_scene = compact_scene_label(restore_scene_label(shell).as_str());
    let restore_energy = shell
        .app
        .jam_view
        .scene
        .restore_scene_energy
        .as_deref()
        .map(compact_energy_label);
    let ghost = ghost_label(shell);

    if ghost.contains("blocked") || ghost.contains("accept/reject") {
        return format!("ghost {ghost}");
    }

    format!(
        "live {current_scene}/{current_energy} <> restore {} | ghost {}",
        match restore_energy {
            Some(restore_energy) => format!("{restore_scene}/{restore_energy}"),
            None => restore_scene,
        },
        ghost
    )
}

pub(super) fn compact_scene_label(scene_id: &str) -> String {
    let mut parts = scene_id.splitn(3, '-');
    match (parts.next(), parts.next(), parts.next()) {
        (Some("scene"), Some(index), Some(label))
            if index.chars().all(|ch| ch.is_ascii_digit()) =>
        {
            label.to_string()
        }
        _ => scene_id.to_string(),
    }
}

pub(super) fn restore_scene_label(shell: &JamShellState) -> String {
    shell
        .app
        .jam_view
        .scene
        .restore_scene
        .clone()
        .unwrap_or_else(|| "none".into())
}

pub(super) fn scene_energy_label_for_scene_id<'a>(
    shell: &'a JamShellState,
    scene_id: &str,
) -> Option<&'a str> {
    let graph = shell.app.source_graph.as_ref()?;
    let scene_index = parse_projected_scene_index(scene_id)?;
    let mut sections = graph.sections.iter().collect::<Vec<_>>();
    sections.sort_by(|left, right| {
        left.bar_start
            .cmp(&right.bar_start)
            .then(left.bar_end.cmp(&right.bar_end))
            .then(left.section_id.as_str().cmp(right.section_id.as_str()))
    });
    sections
        .get(scene_index)
        .map(|section| energy_label(section))
}

fn parse_projected_scene_index(scene_id: &str) -> Option<usize> {
    let mut parts = scene_id.splitn(3, '-');
    match (parts.next(), parts.next()) {
        (Some("scene"), Some(index)) => index.parse::<usize>().ok()?.checked_sub(1),
        _ => None,
    }
}

fn energy_rank(label: &str) -> Option<u8> {
    match label {
        "low" => Some(0),
        "medium" => Some(1),
        "high" => Some(2),
        "peak" => Some(3),
        _ => None,
    }
}

fn compact_energy_label(label: &str) -> &'static str {
    match label {
        "low" => "low",
        "medium" => "med",
        "high" => "high",
        "peak" => "peak",
        _ => "unk",
    }
}

pub(super) fn energy_delta_label(from: Option<&str>, to: Option<&str>) -> Option<&'static str> {
    let from = energy_rank(from?)?;
    let to = energy_rank(to?)?;

    Some(match to.cmp(&from) {
        std::cmp::Ordering::Greater => "energy rise",
        std::cmp::Ordering::Less => "energy drop",
        std::cmp::Ordering::Equal => "energy hold",
    })
}

pub(super) fn compact_energy_delta_label(
    from: Option<&str>,
    to: Option<&str>,
) -> Option<&'static str> {
    let from = energy_rank(from?)?;
    let to = energy_rank(to?)?;

    Some(match to.cmp(&from) {
        std::cmp::Ordering::Greater => "rise",
        std::cmp::Ordering::Less => "drop",
        std::cmp::Ordering::Equal => "hold",
    })
}

pub(super) fn restore_scene_energy_direction_label(shell: &JamShellState) -> Option<&'static str> {
    compact_energy_delta_label(
        shell.app.jam_view.scene.active_scene_energy.as_deref(),
        shell.app.jam_view.scene.restore_scene_energy.as_deref(),
    )
}

pub(super) fn restore_scene_now_compact_label(shell: &JamShellState) -> String {
    let scene = compact_scene_label(restore_scene_label(shell).as_str());
    match restore_scene_energy_direction_label(shell) {
        Some(direction) => format!("{scene} now ({direction})"),
        None => format!("{scene} now"),
    }
}

pub(super) fn restore_scene_target_compact_label(shell: &JamShellState) -> String {
    let scene = compact_scene_label(restore_scene_label(shell).as_str());
    if let Some(energy) = shell.app.jam_view.scene.restore_scene_energy.as_deref() {
        return format!("{scene}/{}", compact_energy_label(energy));
    }
    scene
}

pub(super) fn quantization_boundary_label(quantization: Quantization) -> &'static str {
    match quantization {
        Quantization::Immediate => "immediately",
        Quantization::NextBeat => "next beat",
        Quantization::NextHalfBar => "next half bar",
        Quantization::NextBar => "next bar",
        Quantization::NextPhrase => "next phrase",
        Quantization::NextScene => "next scene",
    }
}
