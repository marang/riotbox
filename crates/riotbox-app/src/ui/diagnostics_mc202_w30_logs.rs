fn material_inspect_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    vec![
        Line::from(format!(
            "captures {} | pending {}",
            capture.capture_count, capture.pending_capture_count
        )),
        Line::from(format!("w30 {}", w30_target_compact(shell))),
        Line::from(format!(
            "last {}",
            capture.last_capture_id.as_deref().unwrap_or("none")
        )),
        Line::from(format!(
            "target {}",
            capture
                .last_capture_target
                .as_deref()
                .unwrap_or("unassigned")
        )),
        Line::from(format!(
            "notes {}",
            capture
                .last_capture_notes
                .as_deref()
                .unwrap_or("no capture note yet")
        )),
    ]
}

fn jam_diagnostic_lines(shell: &JamShellState) -> Vec<Line<'static>> {
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
            "audio {} | sidecar {}",
            shell.app.runtime_view.audio_status, shell.app.runtime_view.sidecar_status
        )),
        Line::from(format!(
            "transport {} @ {:.1}",
            transport_label(shell),
            shell.app.runtime.transport.position_beats
        )),
        Line::from(format!("last boundary {last_boundary}")),
        Line::from(format!(
            "pending {} | landed {}",
            shell.app.jam_view.pending_actions.len(),
            shell.app.jam_view.recent_actions.len()
        )),
        Line::from(primary_warning_line(shell)),
    ]
}

fn mc202_pending_role_label(shell: &JamShellState) -> &'static str {
    if shell.app.jam_view.lanes.mc202_pending_role.is_some() {
        "voice queued"
    } else if shell.app.jam_view.lanes.mc202_pending_answer_generation {
        "answer queued"
    } else if shell.app.jam_view.lanes.mc202_pending_pressure_generation {
        "pressure queued"
    } else if shell.app.jam_view.lanes.mc202_pending_instigator_generation {
        "instigate queued"
    } else if shell.app.jam_view.lanes.mc202_pending_follower_generation {
        "follow queued"
    } else {
        "stable"
    }
}

fn jam_action_label(command: &str) -> String {
    match command {
        "mutate.scene" => GESTURE_MUTATE.into(),
        "scene.launch" => GESTURE_SCENE_JUMP.into(),
        "scene.restore" => GESTURE_RESTORE.into(),
        "mc202.set_role" => GESTURE_VOICE.into(),
        "mc202.generate_follower" => GESTURE_FOLLOW.into(),
        "mc202.generate_answer" => GESTURE_ANSWER.into(),
        "mc202.generate_pressure" => GESTURE_PRESSURE.into(),
        "mc202.generate_instigator" => GESTURE_INSTIGATE.into(),
        "mc202.mutate_phrase" => GESTURE_PHRASE.into(),
        "tr909.fill_next" => GESTURE_FILL.into(),
        "tr909.reinforce_break" => GESTURE_PUSH.into(),
        "tr909.set_slam" => GESTURE_SLAM.into(),
        "tr909.takeover" => GESTURE_TAKEOVER.into(),
        "tr909.scene_lock" => GESTURE_LOCK.into(),
        "tr909.release" => GESTURE_RELEASE.into(),
        "capture.now" | "capture.loop" | "capture.bar_group" => GESTURE_CAPTURE.into(),
        "promote.capture_to_pad" | "promote.capture_to_scene" => GESTURE_PROMOTE.into(),
        "w30.trigger_pad" => GESTURE_HIT.into(),
        "w30.step_focus" => GESTURE_NEXT_PAD.into(),
        "w30.swap_bank" => GESTURE_BANK.into(),
        "w30.browse_slice_pool" => GESTURE_BROWSE.into(),
        "w30.apply_damage_profile" => GESTURE_DAMAGE.into(),
        "w30.loop_freeze" => GESTURE_FREEZE.into(),
        "w30.live_recall" => GESTURE_RECALL.into(),
        "w30.audition_raw_capture" => GESTURE_AUDITION.into(),
        "w30.audition_promoted" => GESTURE_AUDITION.into(),
        "promote.resample" => GESTURE_RESAMPLE.into(),
        _ => command.to_string(),
    }
}

fn mc202_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lanes = &shell.app.jam_view.lanes;
    let last_mc202_action = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| {
            matches!(
                action.command,
                riotbox_core::action::ActionCommand::Mc202SetRole
                    | riotbox_core::action::ActionCommand::Mc202GenerateFollower
                    | riotbox_core::action::ActionCommand::Mc202GenerateAnswer
                    | riotbox_core::action::ActionCommand::Mc202GeneratePressure
                    | riotbox_core::action::ActionCommand::Mc202GenerateInstigator
                    | riotbox_core::action::ActionCommand::Mc202MutatePhrase
            )
        })
        .map(|action| action.command.to_string())
        .unwrap_or_else(|| "none".into());

    vec![
        Line::from(format!(
            "role {} | next {}",
            lanes.mc202_role.as_deref().unwrap_or("unset"),
            lanes.mc202_pending_role.as_deref().unwrap_or("none")
        )),
        Line::from(format!(
            "phrase {} | variant {} | gen {}",
            lanes.mc202_phrase_ref.as_deref().unwrap_or("unset"),
            lanes.mc202_phrase_variant.as_deref().unwrap_or("base"),
            if lanes.mc202_pending_answer_generation {
                "queued answer"
            } else if lanes.mc202_pending_pressure_generation {
                "queued pressure"
            } else if lanes.mc202_pending_instigator_generation {
                "queued instigate"
            } else if lanes.mc202_pending_follower_generation {
                "queued"
            } else if lanes.mc202_pending_phrase_mutation {
                "queued mutation"
            } else {
                "idle"
            }
        )),
        Line::from(format!(
            "touch {:.2} | last {}",
            shell.app.jam_view.macros.mc202_touch, last_mc202_action
        )),
        Line::from(format!(
            "render {} | {}",
            shell.app.runtime_view.mc202_render_routing,
            shell.app.runtime_view.mc202_render_mix_summary
        )),
        Line::from(format!("diagnostic {}", mc202_pending_role_label(shell))),
    ]
}

fn w30_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lanes = &shell.app.jam_view.lanes;
    let recent = last_committed_w30_action(shell);
    let recent_label = recent
        .map(|action| short_w30_action_label(&action.command))
        .unwrap_or("none");
    let lineage_active = w30_resample_lineage_active(shell);
    let slice_pool_relevant = w30_slice_pool_relevant(shell);

    vec![
        Line::from(format!(
            "bank {}/{}",
            lanes.w30_active_bank.as_deref().unwrap_or("unset"),
            lanes.w30_focused_pad.as_deref().unwrap_or("unset")
        )),
        Line::from(format!(
            "cue {} | {recent_label}",
            w30_pending_cue_label(shell)
        )),
        Line::from(format!("prev {}", w30_preview_log_compact(shell))),
        Line::from(if lineage_active {
            format!("tapmix {}", w30_resample_mix_log_compact(shell))
        } else {
            format!(
                "mix {} {}",
                w30_mix_log_compact(shell),
                w30_operation_status_compact(shell),
            )
        }),
        if lineage_active {
            Line::from(w30_resample_log_focus_compact(shell))
        } else {
            if slice_pool_relevant {
                Line::from(format!("pool {}", w30_slice_pool_log_compact(shell)))
            } else {
                Line::from(w30_capture_log_compact(shell))
            }
        },
    ]
}

fn w30_preview_mode_profile_compact(shell: &JamShellState) -> String {
    let render = &shell.app.runtime.w30_preview;
    let mode = match render.mode {
        W30PreviewRenderMode::Idle => "idle",
        W30PreviewRenderMode::LiveRecall => "recall",
        W30PreviewRenderMode::RawCaptureAudition => "audition raw",
        W30PreviewRenderMode::PromotedAudition => "audition",
    };
    let profile = match render.source_profile {
        None => "unset",
        Some(riotbox_audio::w30::W30PreviewSourceProfile::PinnedRecall) => "pinned",
        Some(riotbox_audio::w30::W30PreviewSourceProfile::PromotedRecall) => "promoted",
        Some(riotbox_audio::w30::W30PreviewSourceProfile::SlicePoolBrowse) => "browse",
        Some(riotbox_audio::w30::W30PreviewSourceProfile::RawCaptureAudition) => "raw",
        Some(riotbox_audio::w30::W30PreviewSourceProfile::PromotedAudition) => "audition",
    };

    if matches!(render.mode, W30PreviewRenderMode::RawCaptureAudition) {
        return format!(
            "{mode}/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::PromotedAudition) {
        return format!(
            "{mode}/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::LiveRecall) {
        return format!(
            "{mode}/{profile}/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    format!("{mode}/{profile}")
}

fn w30_preview_log_compact(shell: &JamShellState) -> String {
    let render = &shell.app.runtime.w30_preview;
    if matches!(render.mode, W30PreviewRenderMode::RawCaptureAudition) {
        return format!(
            "raw/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::PromotedAudition) {
        return format!(
            "audition/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::LiveRecall) {
        return format!(
            "recall/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    w30_preview_mode_profile_compact(shell)
}

fn w30_preview_source_suffix(
    render: &riotbox_audio::w30::W30PreviewRenderState,
) -> Option<&'static str> {
    if matches!(render.mode, W30PreviewRenderMode::Idle) {
        return None;
    }

    if render.source_window_preview.is_some() {
        Some("src")
    } else {
        Some("fallback")
    }
}

fn w30_preview_source_readiness(shell: &JamShellState) -> Option<&'static str> {
    let render = &shell.app.runtime.w30_preview;
    if matches!(render.mode, W30PreviewRenderMode::RawCaptureAudition) {
        return match w30_preview_source_suffix(render)? {
            "src" => Some("source-backed"),
            "fallback" => Some("fallback"),
            _ => None,
        };
    }

    if render.source_window_preview.is_some() {
        Some("source-backed")
    } else {
        None
    }
}

fn w30_target_compact(shell: &JamShellState) -> String {
    format!(
        "{}/{}",
        shell
            .app
            .jam_view
            .lanes
            .w30_active_bank
            .as_deref()
            .unwrap_or("unset"),
        shell
            .app
            .jam_view
            .lanes
            .w30_focused_pad
            .as_deref()
            .unwrap_or("unset")
    )
}

fn w30_resample_tap_compact(shell: &JamShellState) -> String {
    let tap = &shell.app.runtime.w30_resample_tap;
    if matches!(tap.mode, riotbox_audio::w30::W30ResampleTapMode::Idle) {
        return "idle/silent".into();
    }

    let profile = match tap.source_profile {
        None => "unset",
        Some(riotbox_audio::w30::W30ResampleTapSourceProfile::RawCapture) => "raw",
        Some(riotbox_audio::w30::W30ResampleTapSourceProfile::PromotedCapture) => "promoted",
        Some(riotbox_audio::w30::W30ResampleTapSourceProfile::PinnedCapture) => "pinned",
    };

    format!("ready/{profile} g{}", tap.generation_depth)
}

fn w30_capture_lineage_compact(shell: &JamShellState) -> String {
    let Some(capture_id) = shell
        .app
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .as_ref()
    else {
        return "lineage none".into();
    };

    let Some(capture) = shell
        .app
        .session
        .captures
        .iter()
        .find(|capture| &capture.capture_id == capture_id)
    else {
        return format!("lineage missing {capture_id}");
    };

    let lineage_chain = if capture.lineage_capture_refs.is_empty() {
        capture.capture_id.to_string()
    } else {
        format!(
            "{}>{}",
            capture
                .lineage_capture_refs
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(">"),
            capture.capture_id
        )
    };

    format!("{lineage_chain} | g{}", capture.resample_generation_depth)
}

fn w30_resample_route_compact(shell: &JamShellState) -> &'static str {
    match shell.app.runtime.w30_resample_tap.routing {
        riotbox_audio::w30::W30ResampleTapRouting::Silent => "silent",
        riotbox_audio::w30::W30ResampleTapRouting::InternalCaptureTap => "internal",
    }
}

fn w30_resample_source_compact(shell: &JamShellState) -> String {
    let tap = &shell.app.runtime.w30_resample_tap;
    match tap.source_capture_id.as_deref() {
        Some(capture_id) => format!(
            "src {capture_id} g{}/l{}",
            tap.generation_depth, tap.lineage_capture_count
        ),
        None => format!(
            "src unset g{}/l{}",
            tap.generation_depth, tap.lineage_capture_count
        ),
    }
}

fn w30_resample_log_focus_compact(shell: &JamShellState) -> String {
    let tap = &shell.app.runtime.w30_resample_tap;
    let capture_id = tap.source_capture_id.as_deref().unwrap_or("unset");
    let route = match tap.routing {
        riotbox_audio::w30::W30ResampleTapRouting::Silent => "sil",
        riotbox_audio::w30::W30ResampleTapRouting::InternalCaptureTap => "int",
    };

    format!(
        "tap {capture_id} g{}/l{} {route}",
        tap.generation_depth, tap.lineage_capture_count
    )
}

fn w30_resample_lineage_active(shell: &JamShellState) -> bool {
    let tap = &shell.app.runtime.w30_resample_tap;
    tap.generation_depth > 0 || tap.lineage_capture_count > 0
}

fn w30_mix_log_compact(shell: &JamShellState) -> String {
    format!(
        "{:.2}/{:.2}",
        shell.app.runtime.w30_preview.music_bus_level, shell.app.runtime.w30_preview.grit_level
    )
}

fn w30_resample_mix_log_compact(shell: &JamShellState) -> String {
    format!(
        "{:.2}/{:.2}",
        shell.app.runtime.w30_resample_tap.music_bus_level,
        shell.app.runtime.w30_resample_tap.grit_level
    )
}

