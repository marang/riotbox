fn now_line(shell: &JamShellState) -> String {
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

fn next_action_line(shell: &JamShellState) -> String {
    if let Some(action) = shell.app.jam_view.pending_actions.first() {
        format!(
            "{} {} @ {}",
            action.actor, action.command, action.quantization
        )
    } else {
        "no pending action queued".into()
    }
}

fn next_scene_jump_suggestion(shell: &JamShellState) -> String {
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

fn next_scene_target_compact_label(shell: &JamShellState) -> String {
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

fn current_scene_compact_label(shell: &JamShellState) -> String {
    let scene_id = current_scene_id(shell).unwrap_or_else(|| "none".into());

    compact_scene_label(scene_id.as_str())
}

fn current_scene_target_compact_label(shell: &JamShellState) -> String {
    let scene = current_scene_compact_label(shell);
    if let Some(energy) = shell.app.jam_view.scene.active_scene_energy.as_deref() {
        return format!("{scene}/{}", compact_energy_label(energy));
    }
    scene
}

fn current_scene_id(shell: &JamShellState) -> Option<String> {
    shell.app.jam_view.scene.active_scene.clone()
}

fn scene_restore_contrast_line(shell: &JamShellState) -> String {
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

fn compact_scene_label(scene_id: &str) -> String {
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

fn restore_scene_label(shell: &JamShellState) -> String {
    shell
        .app
        .jam_view
        .scene
        .restore_scene
        .clone()
        .unwrap_or_else(|| "none".into())
}

fn scene_energy_label_for_scene_id<'a>(
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

fn energy_delta_label(from: Option<&str>, to: Option<&str>) -> Option<&'static str> {
    let from = energy_rank(from?)?;
    let to = energy_rank(to?)?;

    Some(match to.cmp(&from) {
        std::cmp::Ordering::Greater => "energy rise",
        std::cmp::Ordering::Less => "energy drop",
        std::cmp::Ordering::Equal => "energy hold",
    })
}

fn compact_energy_delta_label(from: Option<&str>, to: Option<&str>) -> Option<&'static str> {
    let from = energy_rank(from?)?;
    let to = energy_rank(to?)?;

    Some(match to.cmp(&from) {
        std::cmp::Ordering::Greater => "rise",
        std::cmp::Ordering::Less => "drop",
        std::cmp::Ordering::Equal => "hold",
    })
}

fn restore_scene_energy_direction_label(shell: &JamShellState) -> Option<&'static str> {
    compact_energy_delta_label(
        shell.app.jam_view.scene.active_scene_energy.as_deref(),
        shell.app.jam_view.scene.restore_scene_energy.as_deref(),
    )
}

fn restore_scene_now_compact_label(shell: &JamShellState) -> String {
    let scene = compact_scene_label(restore_scene_label(shell).as_str());
    match restore_scene_energy_direction_label(shell) {
        Some(direction) => format!("{scene} now ({direction})"),
        None => format!("{scene} now"),
    }
}

fn restore_scene_target_compact_label(shell: &JamShellState) -> String {
    let scene = compact_scene_label(restore_scene_label(shell).as_str());
    if let Some(energy) = shell.app.jam_view.scene.restore_scene_energy.as_deref() {
        return format!("{scene}/{}", compact_energy_label(energy));
    }
    scene
}

fn quantization_boundary_label(quantization: riotbox_core::action::Quantization) -> &'static str {
    match quantization {
        riotbox_core::action::Quantization::Immediate => "immediately",
        riotbox_core::action::Quantization::NextBeat => "next beat",
        riotbox_core::action::Quantization::NextHalfBar => "next half bar",
        riotbox_core::action::Quantization::NextBar => "next bar",
        riotbox_core::action::Quantization::NextPhrase => "next phrase",
        riotbox_core::action::Quantization::NextScene => "next scene",
    }
}

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
            "b{} | b{} | p{}",
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
            "| b{} | bar{} | p{}",
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
