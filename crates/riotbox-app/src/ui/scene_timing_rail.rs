fn pending_scene_transition(
    shell: &JamShellState,
) -> Option<(SceneTransitionKindView, &'static str, String, String)> {
    shell
        .app
        .queue
        .pending_actions()
        .iter()
        .find(|action| {
            matches!(
                action.command,
                riotbox_core::action::ActionCommand::SceneLaunch
                    | riotbox_core::action::ActionCommand::SceneRestore
            )
        })
        .and_then(|action| {
            let (kind, label) = match action.command {
                riotbox_core::action::ActionCommand::SceneLaunch => {
                    (SceneTransitionKindView::Launch, "launch")
                }
                riotbox_core::action::ActionCommand::SceneRestore => {
                    (SceneTransitionKindView::Restore, "restore")
                }
                _ => unreachable!("scene transition scan only matches launch and restore"),
            };
            action
                .target
                .scene_id
                .as_ref()
                .map(ToString::to_string)
                .or_else(|| match &action.params {
                    riotbox_core::action::ActionParams::Scene {
                        scene_id: Some(scene_id),
                    } => Some(scene_id.to_string()),
                    _ => None,
                })
                .map(|scene_id| {
                    (
                        kind,
                        label,
                        scene_id,
                        quantization_boundary_label(action.quantization).into(),
                    )
                })
        })
}

fn scene_pending_line(shell: &JamShellState) -> Line<'static> {
    let Some((kind, label, scene_id, boundary)) = pending_scene_transition(shell) else {
        return Line::from(Span::styled("scene transition idle", style_low_emphasis()));
    };

    let mut spans = vec![
        Span::styled(label, style_pending_cue()),
        Span::styled(" -> ", style_low_emphasis()),
        Span::styled(scene_id.clone(), style_pending_detail()),
        Span::styled(" @ ", style_low_emphasis()),
        Span::styled(boundary, style_pending_cue()),
    ];

    if let Some(policy) = pending_scene_transition_policy(shell, kind) {
        spans.push(Span::styled(" | policy ", style_low_emphasis()));
        spans.push(Span::styled(
            policy.direction.label(),
            style_confirmation_strong(),
        ));
        spans.push(Span::styled(" | 909 ", style_low_emphasis()));
        spans.push(Span::styled(
            policy.tr909_intent.label(),
            style_pending_detail(),
        ));
        spans.push(Span::styled(" | 202 ", style_low_emphasis()));
        spans.push(Span::styled(
            policy.mc202_intent.label(),
            style_pending_detail(),
        ));
    } else if let Some(energy_delta) = energy_delta_label(
        shell.app.jam_view.scene.active_scene_energy.as_deref(),
        scene_energy_label_for_scene_id(shell, scene_id.as_str()),
    ) {
        spans.push(Span::styled(" | ", style_low_emphasis()));
        spans.push(Span::styled(energy_delta, style_confirmation_strong()));
    }

    Line::from(spans)
}

fn pending_scene_transition_policy(
    shell: &JamShellState,
    kind: SceneTransitionKindView,
) -> Option<SceneTransitionPolicyView> {
    match kind {
        SceneTransitionKindView::Launch => shell.app.jam_view.scene.next_scene_policy,
        SceneTransitionKindView::Restore => shell.app.jam_view.scene.restore_scene_policy,
    }
}

fn landed_scene_energy_delta(shell: &JamShellState, command: &str) -> Option<&'static str> {
    if !matches!(command, "scene.launch" | "scene.restore") {
        return None;
    }

    energy_delta_label(
        shell.app.jam_view.scene.restore_scene_energy.as_deref(),
        shell.app.jam_view.scene.active_scene_energy.as_deref(),
    )
}

fn scene_commit_pulse_line(shell: &JamShellState) -> Option<Line<'static>> {
    pending_scene_transition(shell)?;
    let transport = &shell.app.runtime.transport;
    let countdown = scene_countdown_cue(transport.beat_index);

    Some(timing_rail_line(
        "pulse",
        countdown,
        None,
        format!(
            "| transport b{} bar{} p{}",
            transport.beat_index, transport.bar_index, transport.phrase_index
        ),
    ))
}

fn queued_timing_rail_line(shell: &JamShellState) -> Option<Line<'static>> {
    if let Some(scene_line) = scene_commit_pulse_line(shell) {
        return Some(scene_line);
    }

    let pending_actions = shell.app.queue.pending_actions();
    let action = pending_actions.first()?;
    let transport = &shell.app.runtime.transport;
    let countdown = quantization_countdown_cue(
        action.quantization,
        transport.beat_index,
        transport.bar_index,
    );

    Some(timing_rail_line(
        "wait",
        countdown,
        Some(quantization_boundary_label(action.quantization)),
        format!(
            "| transport b{} bar{} p{}",
            transport.beat_index, transport.bar_index, transport.phrase_index
        ),
    ))
}

fn timing_rail_line(
    prefix: &'static str,
    countdown: String,
    boundary: Option<&'static str>,
    tail: String,
) -> Line<'static> {
    let mut spans = vec![
        Span::styled(format!("{prefix} "), style_low_emphasis()),
        Span::styled(countdown, style_pending_cue()),
    ];

    if let Some(boundary) = boundary {
        spans.push(Span::styled(" ", style_low_emphasis()));
        spans.push(Span::styled(boundary, style_pending_cue()));
    }

    spans.push(Span::styled(format!(" {tail}"), style_low_emphasis()));

    Line::from(spans)
}

fn scene_countdown_cue(beat_index: u64) -> String {
    let slot = ((beat_index.saturating_sub(1) % 4) + 1) as usize;
    let mut chars = ['-'; 4];
    for ch in chars.iter_mut().take(slot.saturating_sub(1)) {
        *ch = '=';
    }
    chars[slot - 1] = '>';
    format!("[{}]", chars.iter().collect::<String>())
}

fn quantization_countdown_cue(
    quantization: riotbox_core::action::Quantization,
    beat_index: u64,
    bar_index: u64,
) -> String {
    match quantization {
        riotbox_core::action::Quantization::Immediate => "[now]".into(),
        riotbox_core::action::Quantization::NextBeat => "[>]".into(),
        riotbox_core::action::Quantization::NextHalfBar => {
            let slot = (((beat_index.saturating_sub(1) % 4) / 2) + 1) as usize;
            ascii_countdown(slot, 2)
        }
        riotbox_core::action::Quantization::NextBar => scene_countdown_cue(beat_index),
        riotbox_core::action::Quantization::NextPhrase => {
            let slot = ((bar_index.saturating_sub(1) % 8) + 1) as usize;
            ascii_countdown(slot, 8)
        }
        riotbox_core::action::Quantization::NextScene => "[scene]".into(),
    }
}

fn ascii_countdown(slot: usize, width: usize) -> String {
    let slot = slot.clamp(1, width);
    let mut chars = vec!['-'; width];
    for ch in chars.iter_mut().take(slot.saturating_sub(1)) {
        *ch = '=';
    }
    chars[slot - 1] = '>';
    format!("[{}]", chars.iter().collect::<String>())
}
