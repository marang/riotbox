fn tr909_log_header_line(shell: &JamShellState) -> String {
    let scene = shell
        .app
        .runtime
        .transport
        .current_scene
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "none".into());

    if shell.app.runtime_view.tr909_render_support_reason == "feral break lift" {
        return format!("feral break lift | scene {scene}");
    }

    let transport = if shell.app.runtime.transport.is_playing {
        format!(
            "running @ {:.1}",
            shell.app.runtime.transport.position_beats
        )
    } else {
        format!(
            "stopped @ {:.1}",
            shell.app.runtime.transport.position_beats
        )
    };
    let boundary = shell
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
        .unwrap_or_else(|| "boundary none".into());

    format!("{transport} | scene {scene} | {boundary}")
}

fn tr909_compact_reason_line(shell: &JamShellState) -> Option<String> {
    match shell.app.runtime_view.tr909_render_support_reason.as_str() {
        "feral break lift" => Some("reason feral break lift".into()),
        _ => None,
    }
}

fn jam_pending_landed_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let first_pending_line = shell
        .app
        .jam_view
        .pending_actions
        .first()
        .map(|action| {
            format!(
                "next 1 {} {} @ {}",
                action.actor,
                jam_action_label(&action.command),
                action.quantization
            )
        })
        .unwrap_or_else(|| "next 1 none".into());

    let second_pending_line = shell
        .app
        .jam_view
        .pending_actions
        .get(1)
        .map(|action| {
            let mut line = format!(
                "next 2 {} {} @ {}",
                action.actor,
                jam_action_label(&action.command),
                action.quantization
            );
            let more_pending = shell.app.jam_view.pending_actions.len().saturating_sub(2);
            if more_pending > 0 {
                line.push_str(&format!(" | +{more_pending} more"));
            }
            line
        })
        .unwrap_or_else(|| "next 2 none".into());

    vec![
        Line::from(first_pending_line),
        Line::from(second_pending_line),
        latest_landed_line(shell),
        scene_post_commit_cue_line(shell)
            .unwrap_or_else(|| Line::from(format!("status {}", shell.status_message))),
    ]
}

fn latest_landed_line(shell: &JamShellState) -> Line<'static> {
    if let Some(action) = shell.app.jam_view.recent_actions.first() {
        let mut spans = vec![
            Span::styled("landed ", style_low_emphasis()),
            Span::styled(format!("{} ", action.actor), style_low_emphasis()),
            Span::styled(
                jam_action_label(&action.command),
                style_confirmation_strong(),
            ),
        ];

        if let Some(energy_delta) = landed_scene_energy_delta(shell, action.command.as_str()) {
            spans.push(Span::styled(" | ", style_low_emphasis()));
            spans.push(Span::styled(energy_delta, style_confirmation_strong()));
        }

        Line::from(spans)
    } else {
        Line::from(Span::styled("landed none yet", style_low_emphasis()))
    }
}

fn latest_landed_text(shell: &JamShellState) -> String {
    if let Some(action) = shell.app.jam_view.recent_actions.first() {
        let mut line = format!(
            "landed {} {}",
            action.actor,
            jam_action_label(&action.command)
        );
        if let Some(energy_delta) = landed_scene_energy_delta(shell, action.command.as_str()) {
            line.push_str(&format!(" | {energy_delta}"));
        }
        line
    } else {
        "landed none yet".into()
    }
}

fn scene_history_trail_line(shell: &JamShellState) -> Option<String> {
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

fn latest_landed_command(shell: &JamShellState) -> Option<&str> {
    shell
        .app
        .jam_view
        .recent_actions
        .first()
        .map(|action| action.command.as_str())
}

fn scene_post_commit_cue_line(shell: &JamShellState) -> Option<Line<'static>> {
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

fn suggested_gesture_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    if let Some(suggestion) = shell.app.runtime.current_ghost_suggestion.as_ref() {
        let ghost_action_line = if !matches!(shell.app.session.ghost_state.mode, GhostMode::Assist)
        {
            "[Enter] needs Assist  [N] reject"
        } else if suggestion.is_blocked() {
            "[Enter] blocked  [N] reject"
        } else if suggestion.suggested_action.is_none() {
            "[Enter] no action  [N] reject"
        } else {
            "[Enter] accept  [N] reject"
        };
        return vec![
            Line::from(format!("ghost: {}", suggestion.summary)),
            line_with_primary_keys(ghost_action_line),
            line_with_primary_keys("[2] log  [?] help"),
        ];
    }

    if !shell.app.jam_view.transport.is_playing {
        return vec![
            line_with_primary_keys("[Space] play"),
            line_with_primary_keys(format!("{}  [f] fill", next_scene_jump_suggestion(shell))),
            line_with_primary_keys("[c] capture"),
        ];
    }

    if !shell.app.jam_view.pending_actions.is_empty() {
        return vec![
            Line::from("let it land"),
            line_with_primary_keys("[2] log  [u] undo"),
            line_with_primary_keys("[c] capture if good"),
        ];
    }

    if show_restore_readiness_cue(shell) {
        let third_line =
            ghost_assist_request_line(shell).unwrap_or_else(|| line_with_primary_keys("[c] capture"));
        return vec![
            line_with_primary_keys("[y] jump first"),
            line_with_primary_keys("[Y] restore waits for one landed jump"),
            third_line,
        ];
    }

    if show_restore_ready_cue(shell) {
        return vec![
            line_with_primary_keys(format!(
                "[Y] restore {}",
                restore_scene_now_compact_label(shell)
            )),
            line_with_primary_keys("[y] jump  [c] capture"),
            line_with_primary_keys("[2] trail  [u] undo"),
        ];
    }

    if shell.app.jam_view.source.feral_scorecard.readiness == "ready" {
        let third_line = ghost_assist_request_line(shell)
            .unwrap_or_else(|| line_with_primary_keys("[c] capture if it bites"));
        return vec![
            line_with_primary_keys("feral ready: [j] browse  [f] fill"),
            line_with_primary_keys("[g] follow  [a] answer"),
            third_line,
        ];
    }

    if !shell.app.jam_view.recent_actions.is_empty() {
        return vec![
            Line::from(format!("what changed: {}", latest_landed_text(shell))),
            line_with_primary_keys("what next: [c] capture  [u] undo"),
            line_with_primary_keys(format!(
                "then try: {}  [g] follow",
                next_scene_jump_suggestion(shell)
            )),
        ];
    }

    let third_line =
        ghost_assist_request_line(shell).unwrap_or_else(|| line_with_primary_keys("[c] capture  [w] hit"));

    vec![
        line_with_primary_keys(format!("{}  [g] follow", next_scene_jump_suggestion(shell))),
        line_with_primary_keys("[a] answer  [f] fill"),
        third_line,
    ]
}

fn ghost_assist_request_line(shell: &JamShellState) -> Option<Line<'static>> {
    ghost_assist_request_is_useful(shell)
        .then(|| line_with_primary_keys("ghost assist: [Enter] ask"))
}

fn ghost_assist_request_is_useful(shell: &JamShellState) -> bool {
    shell.active_screen == ShellScreen::Jam
        && first_run_onramp_stage(shell).is_none()
        && shell.app.jam_view.transport.is_playing
        && shell.app.jam_view.pending_actions.is_empty()
        && shell.app.jam_view.recent_actions.is_empty()
        && shell.app.can_refresh_current_ghost_suggestion_from_jam_state()
}

fn line_with_primary_keys(text: impl Into<String>) -> Line<'static> {
    let text = text.into();
    let mut spans = Vec::new();
    let mut rest = text.as_str();

    while let Some(start) = rest.find('[') {
        let (prefix, key_and_tail) = rest.split_at(start);
        if !prefix.is_empty() {
            spans.push(Span::raw(prefix.to_owned()));
        }

        let Some(end) = key_and_tail.find(']') else {
            spans.push(Span::raw(key_and_tail.to_owned()));
            return Line::from(spans);
        };
        let key_end = end + 1;
        let (key, tail) = key_and_tail.split_at(key_end);
        spans.push(Span::styled(key.to_owned(), style_primary_control()));
        rest = tail;
    }

    if !rest.is_empty() || spans.is_empty() {
        spans.push(Span::raw(rest.to_owned()));
    }

    Line::from(spans)
}

fn line_with_primary_key_prefixes(text: impl Into<String>) -> Line<'static> {
    let text = text.into();
    let mut spans = Vec::new();

    for (index, segment) in text.split(" | ").enumerate() {
        if index > 0 {
            spans.push(Span::raw(" | "));
        }

        let Some(colon) = segment.find(':') else {
            spans.push(Span::raw(segment.to_owned()));
            continue;
        };

        let (key, detail) = segment.split_at(colon);
        spans.push(Span::styled(key.to_owned(), style_primary_control()));
        spans.push(Span::raw(detail.to_owned()));
    }

    Line::from(spans)
}

fn show_restore_readiness_cue(shell: &JamShellState) -> bool {
    let recent_command_allows_readiness =
        matches!(latest_landed_command(shell), None | Some("undo.last"));

    shell.app.jam_view.transport.is_playing
        && shell.app.jam_view.pending_actions.is_empty()
        && recent_command_allows_readiness
        && shell
            .app
            .session
            .runtime_state
            .scene_state
            .restore_scene
            .is_none()
        && shell.app.session.runtime_state.scene_state.scenes.len() > 1
}

fn show_restore_ready_cue(shell: &JamShellState) -> bool {
    shell.app.jam_view.transport.is_playing
        && shell.app.jam_view.pending_actions.is_empty()
        && shell
            .app
            .session
            .runtime_state
            .scene_state
            .restore_scene
            .is_some()
}

fn jam_warning_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let trust = trust_summary(shell);
    let readiness = if trust.headline == "strong" || trust.headline == "usable" {
        "ready"
    } else {
        "tentative"
    };

    vec![
        Line::from(format!("trust {} | {}", trust.headline, readiness)),
        Line::from(source_timing_warning_line(shell)),
        Line::from(primary_warning_line(shell)),
        Line::from(format!(
            "audio {} | sidecar {}",
            shell.app.runtime_view.audio_status, shell.app.runtime_view.sidecar_status
        )),
    ]
}

fn primary_warning_line(shell: &JamShellState) -> String {
    if let Some(recovery) = recovery_warning_line(shell) {
        return recovery;
    }

    shell
        .app
        .runtime_view
        .runtime_warnings
        .iter()
        .chain(shell.app.jam_view.warnings.iter())
        .next()
        .map(|warning| warning.to_string())
        .unwrap_or_else(|| "no major warning".into())
}

fn source_inspect_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let source = &shell.app.jam_view.source;
    let first_section = shell
        .app
        .source_graph
        .as_ref()
        .and_then(|graph| graph.sections.first())
        .map(section_compact_label)
        .unwrap_or_else(|| "first none".into());
    let second_section = shell
        .app
        .source_graph
        .as_ref()
        .and_then(|graph| graph.sections.get(1))
        .map(section_compact_label)
        .unwrap_or_else(|| "next none".into());

    vec![
        Line::from(format!(
            "tempo {:.1} | trust {} | feral {}",
            source.bpm_estimate.unwrap_or(0.0),
            trust_summary(shell).headline,
            source.feral_scorecard.readiness
        )),
        Line::from(format!(
            "sections {} | loops {} | hooks {}",
            source.section_count, source.loop_candidate_count, source.hook_candidate_count
        )),
        Line::from(source_timing_readiness_line(shell)),
        Line::from(source_timing_warning_line(shell)),
        Line::from(first_section),
        Line::from(second_section),
        source_warning_lines(shell)
            .into_iter()
            .next()
            .unwrap_or_else(|| Line::from("warnings clear")),
    ]
}
