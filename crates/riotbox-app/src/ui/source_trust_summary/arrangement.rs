use ratatui::{
    style::Style,
    text::{Line, Span},
};

use super::{JamShellState, style_confirmation_strong, style_low_emphasis, style_pending_cue};

pub(crate) fn arrangement_taste_line(shell: &JamShellState) -> Line<'static> {
    let (cue, detail, cue_style) = arrangement_taste_parts(shell);

    Line::from(vec![
        Span::styled("taste ", style_low_emphasis()),
        Span::styled(cue, cue_style),
        Span::styled(" | ", style_low_emphasis()),
        Span::raw(detail),
    ])
}

pub(crate) fn arrangement_proof_line(shell: &JamShellState) -> String {
    let contract = &shell.app.jam_view.scene.arrangement_contract;

    if contract.has_pending_scene_transition {
        "proof pending scene move | wait commit + output".into()
    } else if contract.has_landed_movement {
        "proof landed movement | inspect replay/audio evidence".into()
    } else {
        "proof none yet | audible moves need output evidence".into()
    }
}

pub(crate) fn arrangement_inspect_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let contract = &shell.app.jam_view.scene.arrangement_contract;

    vec![
        Line::from(format!("scene contract {}", contract.readiness.label())),
        Line::from(match contract.truth_source.label() {
            "source_graph_session_actions_queue_commit" => "truth product spine",
            _ => "truth unknown",
        }),
        Line::from(format!(
            "timing {} | action {}",
            contract.timing_readiness.label(),
            arrangement_action_surface_label(contract.action_surface.label())
        )),
        Line::from(format!(
            "proof p012/p013/replay/output {}",
            if contract.requires_p012_source_grid_gate
                && contract.requires_p013_musical_quality_gate
                && contract.requires_replay_state_proof
                && contract.requires_output_path_proof_for_audible_changes
            {
                "yes"
            } else {
                "partial"
            }
        )),
        Line::from(format!(
            "scenes {} | active {} next {} restore {}",
            contract.scene_count,
            yes_no(contract.has_active_scene),
            yes_no(contract.has_next_scene),
            yes_no(contract.has_restore_scene)
        )),
    ]
}

fn arrangement_taste_parts(shell: &JamShellState) -> (&'static str, &'static str, Style) {
    match shell
        .app
        .jam_view
        .scene
        .arrangement_contract
        .readiness
        .label()
    {
        "ready" => (
            "scene-ready",
            "trusted grid can steer scene moves",
            style_confirmation_strong(),
        ),
        "needs_timing_confirmation" => (
            "cautious",
            "confirm grid before scene moves",
            style_pending_cue(),
        ),
        "fallback_timing_only" => ("sketch", "fallback timing only", style_pending_cue()),
        "needs_scene_material" => ("waiting", "needs two scenes", style_pending_cue()),
        "needs_timing_evidence" => ("unknown", "timing unavailable", style_low_emphasis()),
        "missing_source_graph" => ("unknown", "load source graph", style_low_emphasis()),
        _ => ("unknown", "arrangement trust unknown", style_low_emphasis()),
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn arrangement_action_surface_label(label: &str) -> &str {
    match label {
        "scene.launch_scene.restore" => "scene launch/restore",
        _ => label,
    }
}
