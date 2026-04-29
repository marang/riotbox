fn capture_or_recall_cue_label(shell: &JamShellState) -> String {
    shell
        .app
        .jam_view
        .pending_actions
        .iter()
        .find(|action| {
            matches!(
                action.command.as_str(),
                "w30.trigger_pad"
                    | "w30.step_focus"
                    | "w30.swap_bank"
                    | "w30.apply_damage_profile"
                    | "w30.loop_freeze"
                    | "w30.live_recall"
                    | "w30.audition_raw_capture"
                    | "w30.audition_promoted"
                    | "promote.resample"
            )
        })
        .or_else(|| {
            shell
                .app
                .jam_view
                .pending_actions
                .iter()
                .find(|action| is_capture_command_view(action.command.as_str()))
        })
        .map(|action| format!("{} @ {}", action.command, action.quantization))
        .unwrap_or_else(|| "no capture cue queued".into())
}

fn w30_pending_cue_label(shell: &JamShellState) -> String {
    if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_trigger_target
        .as_deref()
    {
        format!("trigger {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_focus_step_target
        .as_deref()
    {
        format!("step {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_audition
        .as_ref()
        .map(|pending| pending.target.as_str())
    {
        format!("audition {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_bank_swap_target
        .as_deref()
    {
        format!("bank {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_slice_pool_target
        .as_deref()
    {
        if shell
            .app
            .jam_view
            .lanes
            .w30_pending_slice_pool_reason
            .as_deref()
            == Some("feral")
        {
            let capture_id = shell
                .app
                .jam_view
                .lanes
                .w30_pending_slice_pool_capture_id
                .as_deref()
                .unwrap_or(target);
            format!("feral browse {capture_id}")
        } else {
            format!("browse {target}")
        }
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_damage_profile_target
        .as_deref()
    {
        format!("damage shred {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_loop_freeze_target
        .as_deref()
    {
        format!("freeze {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_recall_target
        .as_deref()
    {
        format!("recall {target}")
    } else if let Some(capture_id) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_resample_capture_id
        .as_deref()
    {
        format!("resample {capture_id}")
    } else {
        "idle".into()
    }
}

fn last_committed_w30_action(shell: &JamShellState) -> Option<&riotbox_core::action::Action> {
    shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| {
            action.status == riotbox_core::action::ActionStatus::Committed
                && matches!(
                    action.command,
                    riotbox_core::action::ActionCommand::W30TriggerPad
                        | riotbox_core::action::ActionCommand::W30StepFocus
                        | riotbox_core::action::ActionCommand::W30SwapBank
                        | riotbox_core::action::ActionCommand::W30BrowseSlicePool
                        | riotbox_core::action::ActionCommand::W30ApplyDamageProfile
                        | riotbox_core::action::ActionCommand::W30LoopFreeze
                        | riotbox_core::action::ActionCommand::W30LiveRecall
                        | riotbox_core::action::ActionCommand::W30AuditionRawCapture
                        | riotbox_core::action::ActionCommand::W30AuditionPromoted
                        | riotbox_core::action::ActionCommand::PromoteResample
                )
        })
}

fn short_w30_action_label(command: &riotbox_core::action::ActionCommand) -> &'static str {
    match command {
        riotbox_core::action::ActionCommand::W30TriggerPad => "trigger",
        riotbox_core::action::ActionCommand::W30StepFocus => "step",
        riotbox_core::action::ActionCommand::W30SwapBank => "bank",
        riotbox_core::action::ActionCommand::W30BrowseSlicePool => "browse",
        riotbox_core::action::ActionCommand::W30ApplyDamageProfile => "damage",
        riotbox_core::action::ActionCommand::W30LoopFreeze => "freeze",
        riotbox_core::action::ActionCommand::W30LiveRecall => "recall",
        riotbox_core::action::ActionCommand::W30AuditionRawCapture => "audition raw",
        riotbox_core::action::ActionCommand::W30AuditionPromoted => "audition",
        riotbox_core::action::ActionCommand::PromoteResample => "resample",
        _ => "other",
    }
}

fn pending_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let pending = shell.app.queue.pending_actions();
    if pending.is_empty() {
        return vec![Line::from("no queued or pending actions")];
    }

    let mut lines = Vec::new();
    for action in pending.into_iter().take(4) {
        lines.push(Line::from(format!(
            "{} {} {}",
            action.id, action.actor, action.command
        )));
        lines.push(Line::from(format!(
            "status {} | when {} | target {}",
            format!("{:?}", action.status).to_lowercase(),
            action.quantization,
            action_target_label(&action.target)
        )));
        lines.push(Line::from(format!(
            "requested {}{}",
            action.requested_at,
            action
                .explanation
                .as_ref()
                .map(|explanation| format!(" | {explanation}"))
                .unwrap_or_default()
        )));
        lines.push(Line::from(""));
    }

    lines
}

fn committed_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let committed: Vec<_> = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .filter(|action| action.status == riotbox_core::action::ActionStatus::Committed)
        .take(4)
        .collect();

    if committed.is_empty() {
        return vec![Line::from("no committed actions yet")];
    }

    action_entry_lines(committed)
}

fn exception_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let exceptions: Vec<_> = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .filter(|action| {
            matches!(
                action.status,
                riotbox_core::action::ActionStatus::Rejected
                    | riotbox_core::action::ActionStatus::Undone
                    | riotbox_core::action::ActionStatus::Failed
            )
        })
        .take(4)
        .collect();

    if exceptions.is_empty() {
        return vec![Line::from("no rejected, failed, or undone actions")];
    }

    action_entry_lines(exceptions)
}

fn action_entry_lines(actions: Vec<&riotbox_core::action::Action>) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for action in actions {
        lines.push(Line::from(format!(
            "{} {} {}",
            action.id, action.actor, action.command
        )));
        lines.push(Line::from(format!(
            "status {} | when {} | target {}",
            format!("{:?}", action.status).to_lowercase(),
            action.quantization,
            action_target_label(&action.target)
        )));
        lines.push(Line::from(format!(
            "requested {} | committed {}",
            action.requested_at,
            action
                .committed_at
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".into())
        )));
        if let Some(result) = &action.result {
            lines.push(Line::from(format!("result {}", result.summary)));
        } else if let Some(explanation) = &action.explanation {
            lines.push(Line::from(format!("note {explanation}")));
        }
        lines.push(Line::from(""));
    }

    lines
}

fn action_target_label(target: &riotbox_core::action::ActionTarget) -> String {
    let Some(scope) = &target.scope else {
        return "unset".into();
    };

    let detail = if let Some(scene_id) = &target.scene_id {
        scene_id.to_string()
    } else if let Some(bank_id) = &target.bank_id {
        match &target.pad_id {
            Some(pad_id) => format!("{bank_id}/{pad_id}"),
            None => bank_id.to_string(),
        }
    } else if let Some(loop_id) = &target.loop_id {
        loop_id.to_string()
    } else if let Some(object_id) = &target.object_id {
        object_id.clone()
    } else {
        String::new()
    };

    let scope = format!("{scope:?}").to_lowercase();
    if detail.is_empty() {
        scope
    } else {
        format!("{scope}:{detail}")
    }
}

fn is_capture_command_view(command: &str) -> bool {
    matches!(
        command,
        "capture.now"
            | "capture.loop"
            | "capture.bar_group"
            | "w30.capture_to_pad"
            | "promote.capture_to_pad"
            | "promote.capture_to_scene"
            | "w30.loop_freeze"
            | "promote.resample"
    )
}

fn log_warning_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let warnings: Vec<_> = shell
        .app
        .runtime_view
        .runtime_warnings
        .iter()
        .chain(shell.app.jam_view.warnings.iter())
        .take(2)
        .cloned()
        .collect();
    let restore_lines = restore_replay_log_lines(shell);
    if warnings.is_empty() && restore_lines.is_empty() {
        return vec![Line::from("no active runtime or trust warnings")];
    }

    let mut lines = restore_lines;
    lines.extend(warnings
        .into_iter()
        .map(|warning| Line::from(format!("warning {warning}")))
    );
    lines
}

fn restore_replay_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let runtime = &shell.app.runtime_view;
    if runtime.replay_restore_status == "ready: no replay entries" {
        return Vec::new();
    }

    let mut lines = vec![
        Line::from(compact_restore_replay_label(
            &runtime.replay_restore_status,
        )),
        Line::from(compact_restore_replay_label(
            &runtime.replay_restore_anchor,
        )),
    ];
    if runtime.replay_restore_unsupported != "unsupported none" {
        lines.push(Line::from(compact_restore_replay_label(
            &runtime.replay_restore_unsupported,
        )));
    } else {
        lines.push(Line::from(compact_restore_replay_label(
            &runtime.replay_restore_suffix,
        )));
    }
    lines
}

fn compact_restore_replay_label(label: &str) -> String {
    let mut compact = label.strip_prefix("ready: ").unwrap_or(label).to_owned();
    compact = compact
        .replace("suffix 1 action(s): ", "suffix ")
        .replace("unsupported suffix 1: ", "unsupported suffix ")
        .replace("unsupported origin 1: ", "unsupported origin ")
        .replace(" action(s)", "")
        .replace(" @ cursor ", "@");
    compact
}

fn source_identity_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!("source {}", graph.source.source_id)),
            Line::from(graph.source.path.clone()),
            Line::from(format!(
                "{:.2}s | {} Hz | {} ch | {}",
                graph.source.duration_seconds,
                graph.source.sample_rate,
                graph.source.channel_count,
                decode_profile_label(&graph.source.decode_profile)
            )),
            Line::from(format!("hash {}", graph.source.content_hash)),
        ],
        None => vec![Line::from("no source graph loaded")],
    }
}

fn source_timing_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!(
                "tempo {} | conf {:.2}",
                graph
                    .timing
                    .bpm_estimate
                    .map(|bpm| format!("{bpm:.1} BPM"))
                    .unwrap_or_else(|| "unknown".into()),
                graph.timing.bpm_confidence
            )),
            Line::from(format!(
                "meter {}",
                graph
                    .timing
                    .meter_hint
                    .as_ref()
                    .map(|meter| format!("{}/{}", meter.beats_per_bar, meter.beat_unit))
                    .unwrap_or_else(|| "unknown".into())
            )),
            Line::from(format!(
                "beats {} | bars {} | phrases {}",
                graph.timing.beat_grid.len(),
                graph.timing.bar_grid.len(),
                graph.timing.phrase_grid.len()
            )),
            Line::from(format!(
                "timing {} | sections {}",
                quality_label(&graph.analysis_summary.timing_quality),
                quality_label(&graph.analysis_summary.section_quality)
            )),
        ],
        None => vec![Line::from("no timing information available")],
    }
}

fn section_compact_label(section: &Section) -> String {
    format!(
        "{} bars {}-{}",
        section_label_hint_compact(&section.label_hint),
        section.bar_start,
        section.bar_end
    )
}

fn section_label_hint_compact(label_hint: &SectionLabelHint) -> &'static str {
    match label_hint {
        SectionLabelHint::Intro => "intro",
        SectionLabelHint::Build => "build",
        SectionLabelHint::Drop => "drop",
        SectionLabelHint::Break => "break",
        SectionLabelHint::Verse => "verse",
        SectionLabelHint::Chorus => "chorus",
        SectionLabelHint::Bridge => "bridge",
        SectionLabelHint::Outro => "outro",
        SectionLabelHint::Unknown => "unknown",
    }
}

fn source_section_items(shell: &JamShellState) -> Vec<ListItem<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) if !graph.sections.is_empty() => graph
            .sections
            .iter()
            .take(6)
            .map(|section| {
                ListItem::new(format!(
                    "{} | bars {}-{} | {:.2}s-{:.2}s | {} | conf {:.2}",
                    section_label(section),
                    section.bar_start,
                    section.bar_end,
                    section.start_seconds,
                    section.end_seconds,
                    energy_label(section),
                    section.confidence
                ))
            })
            .collect(),
        Some(_) => vec![ListItem::new("no sections available")],
        None => vec![ListItem::new("no source graph loaded")],
    }
}
