fn footer_status_line(status: &str) -> Line<'static> {
    Line::from(Span::styled(status.to_owned(), style_low_emphasis()))
}

fn footer_ok_line(message: &str) -> Line<'static> {
    Line::from(Span::styled(message.to_owned(), style_confirmation()))
}

fn footer_warning_line(warning: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("Warning:", style_warning_label()),
        Span::styled(format!(" {warning}"), style_warning_detail()),
    ])
}

fn spans_with_primary_gesture_keys(gestures: &str) -> Vec<Span<'static>> {
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

fn spans_with_primary_legend_keys(legend: &str) -> Vec<Span<'static>> {
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

fn footer_scene_affordance_cue(shell: &JamShellState) -> Option<String> {
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

fn render_help_overlay(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let popup = centered_rect(60, 55, area);
    let mut lines = vec![
        Line::from("Jam shell keys"),
        line_with_primary_key_prefixes("q or Esc: quit"),
        line_with_primary_key_prefixes("? or h: toggle help"),
        line_with_primary_key_prefixes(
            "1: Jam screen | 2: Log screen | 3: Source screen | 4: Capture screen | Tab: next screen",
        ),
        line_with_primary_key_prefixes(
            "i: open inspect from Jam | press i again to return to perform",
        ),
    ];

    if let Some(stage) = first_run_onramp_stage(shell) {
        lines.push(Line::from(""));
        lines.push(Line::from("First run"));
        match stage {
            FirstRunOnrampStage::Start => {
                lines.push(line_with_primary_key_prefixes("space: start transport"));
                lines.push(line_with_primary_key_prefixes("f: queue one first fill"));
                lines.push(line_with_primary_key_prefixes(
                    "2: switch to Log and watch it land",
                ));
            }
            FirstRunOnrampStage::QueuedFirstMove => {
                lines.push(Line::from("let transport cross the next bar"));
                lines.push(line_with_primary_key_prefixes(
                    "2: confirm the first landed action in Log",
                ));
                lines.push(line_with_primary_key_prefixes("c: capture it | u: undo it"));
            }
            FirstRunOnrampStage::FirstResult => {
                lines.push(line_with_primary_key_prefixes(
                    "c: capture the first keeper",
                ));
                lines.push(line_with_primary_key_prefixes("u: undo it if it missed"));
                lines.push(line_with_primary_key_prefixes(
                    "y / g / w: try one more gesture",
                ));
            }
        }
    }

    if let Some(scene_help_lines) = pending_scene_help_lines(shell) {
        lines.extend(scene_help_lines);
    }
    if let Some(scene_restore_help_lines) = scene_restore_help_lines(shell) {
        lines.extend(scene_restore_help_lines);
    }
    if let Some(capture_help_lines) = capture_help_lines(shell) {
        lines.extend(capture_help_lines);
    }

    lines.extend([
        Line::from(""),
        Line::from("Primary gestures"),
        line_with_primary_key_prefixes(format!(
            "space: play / pause | {}",
            render_help_primary_gesture_items(shell)
        )),
        line_with_primary_key_prefixes(format!(
            "{} | 2: confirm in Log",
            render_gesture_items(HELP_PRIMARY_CONFIRM_GESTURES, ": ")
        )),
        Line::from(""),
        Line::from("After first loop: docs/jam_recipes.md -> Recipe 2 / Recipe 5"),
        Line::from(""),
        Line::from("Advanced / lane gestures"),
        line_with_primary_key_prefixes(format!("r: {}", shell.launch_mode.refresh_verb())),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_A, ": ")),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_B, ": ")),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_C, ": ")),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_D, ": ")),
        line_with_primary_key_prefixes("[ / ]: drum bus | < / >: MC-202 touch | v: pin latest"),
        Line::from(""),
        Line::from(format!("Current mode: {}", shell.launch_mode.label())),
        Line::from(format!("Jam view: {}", shell.jam_mode.label())),
        Line::from(format!("Current screen: {}", shell.active_screen.label())),
        Line::from(shell.status_message.clone()),
    ]);

    let help = Paragraph::new(lines)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, popup);
    frame.render_widget(help, popup);
}

fn pending_scene_help_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    let (_kind, label, scene_id, boundary) = pending_scene_transition(shell)?;
    let scene = compact_scene_label(scene_id.as_str());

    Some(vec![
        Line::from(""),
        Line::from("Scene timing"),
        Line::from(format!("{label} {scene}: lands at {boundary}")),
        Line::from("Jam: read launch/restore, pulse, live/restore energy"),
        line_with_primary_key_prefixes("2: confirm the landed trail on Log"),
    ])
}

fn scene_restore_help_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    if show_restore_readiness_cue(shell) {
        return Some(vec![
            Line::from(""),
            Line::from("Scene restore"),
            Line::from("Y waits for one landed jump"),
            Line::from("land one jump, then Y can restore the last scene"),
        ]);
    }

    if show_restore_ready_cue(shell) {
        let restore_target = restore_scene_target_compact_label(shell);
        let direction = restore_scene_energy_direction_label(shell)
            .map(|direction| format!(" ({direction})"))
            .unwrap_or_default();
        return Some(vec![
            Line::from(""),
            Line::from("Scene restore"),
            Line::from(format!("Y is live now for {restore_target}{direction}")),
            Line::from(format!(
                "press Y to bring {restore_target} back on the next bar"
            )),
        ]);
    }

    None
}

fn capture_help_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    if shell.active_screen != ShellScreen::Capture {
        return None;
    }

    Some(vec![
        Line::from(""),
        Line::from("Capture path"),
        Line::from("Do Next: read capture -> promote -> hit"),
        line_with_primary_keys("src means source-backed; fallback is safe preview"),
        line_with_primary_keys("hear fallback: [o] raw, then [p]->[w]"),
        line_with_primary_key_prefixes("2: confirm promote, hit, and audition results in Log"),
    ])
}

fn screen_context_label(shell: &JamShellState) -> String {
    match shell.active_screen {
        ShellScreen::Jam => format!("jam/{}", shell.jam_mode.label()),
        _ => shell.active_screen.label().into(),
    }
}

fn mc202_perform_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lanes = &shell.app.jam_view.lanes;
    let next = if let Some(role) = lanes.mc202_pending_role.as_deref() {
        format!("next voice {role}")
    } else if lanes.mc202_pending_answer_generation {
        "next answer".into()
    } else if lanes.mc202_pending_pressure_generation {
        "next pressure".into()
    } else if lanes.mc202_pending_instigator_generation {
        "next instigate".into()
    } else if lanes.mc202_pending_follower_generation {
        "next follow".into()
    } else if lanes.mc202_pending_phrase_mutation {
        "next phrase mutation".into()
    } else {
        "next none".into()
    };

    vec![
        Line::from(format!(
            "current voice {}",
            lanes.mc202_role.as_deref().unwrap_or("unset")
        )),
        Line::from(next),
        Line::from(format!(
            "current phrase {}",
            lanes.mc202_phrase_ref.as_deref().unwrap_or("unset")
        )),
        Line::from(format!(
            "variant {}",
            lanes.mc202_phrase_variant.as_deref().unwrap_or("base")
        )),
        Line::from(format!(
            "sound {} / {} | touch {:.2}",
            shell.app.runtime_view.mc202_render_mode,
            shell.app.runtime_view.mc202_render_phrase_shape,
            shell.app.runtime.mc202_render.touch
        )),
    ]
}

fn w30_perform_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let next = if w30_pending_cue_label(shell) != "idle" {
        format!("next {}", w30_pending_cue_label(shell))
    } else {
        format!("next {}", w30_operation_status_compact(shell))
    };

    vec![
        Line::from(format!("current pad {}", w30_target_compact(shell))),
        Line::from(format!(
            "current preview {}",
            w30_preview_mode_profile_compact(shell)
        )),
        Line::from(next),
    ]
}

fn tr909_perform_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let next = tr909_next_line(shell);

    vec![
        Line::from(format!(
            "current mode {}",
            if shell.app.jam_view.lanes.tr909_takeover_enabled {
                "takeover"
            } else {
                "support"
            }
        )),
        Line::from(format!(
            "current fill {} | slam {:.2}",
            if shell.app.jam_view.lanes.tr909_fill_armed_next_bar {
                "armed"
            } else {
                "idle"
            },
            shell.app.jam_view.macros.tr909_slam
        )),
        Line::from(format!("next {next}")),
    ]
}

fn tr909_next_line(shell: &JamShellState) -> String {
    use riotbox_core::action::ActionCommand::{
        Tr909FillNext, Tr909ReinforceBreak, Tr909Release, Tr909SceneLock, Tr909SetSlam,
        Tr909Takeover,
    };

    shell
        .app
        .queue
        .pending_actions()
        .iter()
        .find_map(|action| match action.command {
            Tr909FillNext => Some("fill".into()),
            Tr909ReinforceBreak => Some("push".into()),
            Tr909SetSlam => Some("slam".into()),
            Tr909Takeover => Some("takeover".into()),
            Tr909SceneLock => Some("lock".into()),
            Tr909Release => Some("release".into()),
            _ => None,
        })
        .unwrap_or_else(|| {
            if shell.app.jam_view.lanes.tr909_fill_armed_next_bar {
                "fill armed".into()
            } else {
                "none".into()
            }
        })
}

fn tr909_inspect_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let render = &shell.app.runtime_view;
    let last_boundary = shell
        .app
        .runtime
        .last_commit_boundary
        .as_ref()
        .map(|boundary| {
            format!(
                "{:?} b{} p{}",
                boundary.kind, boundary.bar_index, boundary.phrase_index
            )
        })
        .unwrap_or_else(|| "none".into());

    vec![
        Line::from(format!(
            "mode {} | next {}",
            render.tr909_render_mode,
            tr909_next_line(shell)
        )),
        Line::from(format!(
            "profile {} | context {} | accent {} | reason {} | route {}",
            render.tr909_render_profile,
            render.tr909_render_support_context,
            render.tr909_render_support_accent,
            render.tr909_render_support_reason,
            render.tr909_render_routing
        )),
        Line::from(format!(
            "{} | {}",
            render.tr909_render_pattern_adoption, render.tr909_render_phrase_variation
        )),
        Line::from(render.tr909_render_mix_summary.clone()),
        Line::from(format!(
            "{} | boundary {last_boundary}",
            render.tr909_render_alignment
        )),
    ]
}
