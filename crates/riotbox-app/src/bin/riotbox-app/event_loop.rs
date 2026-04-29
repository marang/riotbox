fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut shell: JamShellState,
    launch: AppLaunch,
    mut audio_runtime: Option<&mut AudioRuntimeShell>,
    mut observer: Option<&mut UserSessionObserver>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        if let Some(audio_runtime) = audio_runtime.as_deref_mut() {
            let now = timestamp_now();
            let committed = shell
                .app
                .apply_audio_timing_snapshot(audio_runtime.timing_snapshot(), now);
            if !committed.is_empty() {
                shell.set_error_status("committed queued actions on transport boundary");
                if let Some(observer) = observer.as_deref_mut() {
                    observer.record_transport_commit(now, &committed, &shell)?;
                }
            }

            audio_runtime.update_transport_state(
                shell.app.runtime.transport.is_playing,
                shell.app.runtime.tr909_render.tempo_bpm,
                shell.app.runtime.transport.position_beats,
            );
            audio_runtime.update_tr909_render_state(&shell.app.runtime.tr909_render);
            audio_runtime.update_mc202_render_state(&shell.app.runtime.mc202_render);
            audio_runtime.update_w30_preview_render_state(&shell.app.runtime.w30_preview);
            audio_runtime.update_w30_resample_tap_state(&shell.app.runtime.w30_resample_tap);
            shell.app.set_audio_health(audio_runtime.health_snapshot());
        }

        terminal.draw(|frame| render_jam_shell(frame, &shell))?;

        if event::poll(INPUT_POLL)?
            && let Event::Key(key) = event::read()?
        {
            let key_label = key_code_label(key.code);
            let outcome = shell.handle_key_code(key.code);
            match outcome {
                ShellKeyOutcome::Quit => {
                    if let Some(observer) = observer.as_deref_mut() {
                        observer.record_key_event(
                            timestamp_now(),
                            &key_label,
                            shell_key_outcome_label(outcome),
                            &shell,
                        )?;
                    }
                    return Ok(());
                }
                ShellKeyOutcome::Continue => {}
                ShellKeyOutcome::ToggleTransport => {
                    let next_is_playing = !shell.app.runtime.transport.is_playing;
                    shell.app.set_transport_playing(next_is_playing);
                    shell.set_error_status(if next_is_playing {
                        "transport started"
                    } else {
                        "transport paused"
                    });
                }
                ShellKeyOutcome::QueueSceneMutation => {
                    shell.app.queue_scene_mutation(timestamp_now());
                    shell.set_error_status("queued scene mutation for next bar");
                }
                ShellKeyOutcome::QueueSceneSelect => {
                    match shell.app.queue_scene_select(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status("queued scene select for next bar");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("scene transition already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status(scene_select_unavailable_status(&shell));
                        }
                    }
                }
                ShellKeyOutcome::QueueSceneRestore => {
                    match shell.app.queue_scene_restore(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status("queued scene restore for next bar");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("scene transition already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("no restore scene available");
                        }
                    }
                }
                ShellKeyOutcome::QueueMc202RoleToggle => {
                    match shell.app.queue_mc202_role_toggle(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status("queued MC-202 role change for next phrase");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("MC-202 role change already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("MC-202 role already set");
                        }
                    }
                }
                ShellKeyOutcome::QueueMc202GenerateFollower => {
                    match shell.app.queue_mc202_generate_follower(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status(
                                "queued MC-202 follower generation for next phrase",
                            );
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("MC-202 follower generation already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("MC-202 follower already in state");
                        }
                    }
                }
                ShellKeyOutcome::QueueMc202GenerateAnswer => {
                    match shell.app.queue_mc202_generate_answer(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status(
                                "queued MC-202 answer generation for next phrase",
                            );
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("MC-202 answer generation already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("MC-202 answer already in state");
                        }
                    }
                }
                ShellKeyOutcome::QueueMc202GeneratePressure => {
                    match shell.app.queue_mc202_generate_pressure(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status(
                                "queued MC-202 pressure generation for next phrase",
                            );
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("MC-202 pressure generation already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("MC-202 pressure already in state");
                        }
                    }
                }
                ShellKeyOutcome::QueueMc202GenerateInstigator => {
                    match shell.app.queue_mc202_generate_instigator(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status(
                                "queued MC-202 instigator generation for next phrase",
                            );
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("MC-202 phrase control already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("MC-202 instigator already in state");
                        }
                    }
                }
                ShellKeyOutcome::QueueMc202MutatePhrase => {
                    match shell.app.queue_mc202_mutate_phrase(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status("queued MC-202 phrase mutation for next phrase");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("MC-202 phrase control already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("set an MC-202 voice before mutating phrase");
                        }
                    }
                }
                ShellKeyOutcome::QueueTr909Fill => {
                    shell.app.queue_tr909_fill(timestamp_now());
                    shell.set_error_status("queued TR-909 fill for next bar");
                }
                ShellKeyOutcome::QueueTr909Reinforce => {
                    shell.app.queue_tr909_reinforce(timestamp_now());
                    shell.set_error_status("queued TR-909 reinforcement for next phrase");
                }
                ShellKeyOutcome::QueueTr909Slam => {
                    if shell.app.queue_tr909_slam_toggle(timestamp_now()) {
                        shell.set_error_status("queued TR-909 slam change for next beat");
                    } else {
                        shell.set_error_status("TR-909 slam change already queued");
                    }
                }
                ShellKeyOutcome::QueueTr909Takeover => {
                    match shell.app.queue_tr909_takeover(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status("queued TR-909 takeover for next phrase");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("TR-909 takeover change already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("TR-909 takeover already active");
                        }
                    }
                }
                ShellKeyOutcome::QueueTr909SceneLock => {
                    match shell.app.queue_tr909_scene_lock(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status(
                                "queued TR-909 scene-lock variation for next phrase",
                            );
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("TR-909 takeover change already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("TR-909 scene-lock variation already active");
                        }
                    }
                }
                ShellKeyOutcome::QueueTr909Release => {
                    match shell.app.queue_tr909_release(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status("queued TR-909 release for next phrase");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("TR-909 takeover change already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("TR-909 takeover already released");
                        }
                    }
                }
                ShellKeyOutcome::QueueCaptureBar => {
                    shell.app.queue_capture_bar(timestamp_now());
                    shell.set_error_status("queued capture for next phrase");
                }
                ShellKeyOutcome::PromoteLastCapture => {
                    if shell.app.queue_promote_last_capture(timestamp_now()) {
                        shell.set_error_status("queued promotion for latest capture");
                    } else {
                        shell.set_error_status("no promotable capture or W-30 target available");
                    }
                }
                ShellKeyOutcome::QueueW30TriggerPad => {
                    match shell.app.queue_w30_trigger_pad(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 pad trigger for next beat");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 pad trigger already in state");
                        }
                        None => {
                            shell.set_error_status("no committed W-30 pad available to trigger")
                        }
                    }
                }
                ShellKeyOutcome::QueueW30StepFocus => {
                    match shell.app.queue_w30_step_focus(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 focus step for next beat");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 focus already on the next stepped pad");
                        }
                        None => shell
                            .set_error_status("no promoted W-30 pads available to step through"),
                    }
                }
                ShellKeyOutcome::QueueW30SwapBank => {
                    match shell.app.queue_w30_swap_bank(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 bank swap for next bar");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 bank swap already on the next bank");
                        }
                        None => {
                            shell.set_error_status("no alternate W-30 bank available to swap to")
                        }
                    }
                }
                ShellKeyOutcome::QueueW30BrowseSlicePool => {
                    match shell.app.queue_w30_browse_slice_pool(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 slice-pool browse for next beat");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell
                                .set_error_status("W-30 slice pool already on the current capture");
                        }
                        None => shell.set_error_status(
                            "no alternate capture in the current W-30 slice pool",
                        ),
                    }
                }
                ShellKeyOutcome::QueueW30ApplyDamageProfile => {
                    match shell.app.queue_w30_apply_damage_profile(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 damage profile for next bar");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 damage profile already active");
                        }
                        None => shell.set_error_status("no W-30 pad available for damage profile"),
                    }
                }
                ShellKeyOutcome::QueueW30LoopFreeze => {
                    match shell.app.queue_w30_loop_freeze(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 loop freeze for next phrase");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 loop freeze already in state");
                        }
                        None => shell.set_error_status("no committed W-30 pad available to freeze"),
                    }
                }
                ShellKeyOutcome::QueueW30LiveRecall => {
                    match shell.app.queue_w30_live_recall(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 live recall for next bar");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 live recall already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 live recall already in state");
                        }
                        None => shell.set_error_status(
                            "no pinned or promoted W-30 capture available to recall",
                        ),
                    }
                }
                ShellKeyOutcome::QueueW30Audition => {
                    match shell.app.queue_w30_audition(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 audition for next bar");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 audition already in state");
                        }
                        None => {
                            shell.set_error_status("no W-30 or raw capture available to audition")
                        }
                    }
                }
                ShellKeyOutcome::QueueW30Resample => {
                    match shell.app.queue_w30_internal_resample(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 internal resample for next phrase");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 internal resample already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 internal resample already in state");
                        }
                        None => shell
                            .set_error_status("no committed W-30 capture available to resample"),
                    }
                }
                ShellKeyOutcome::TogglePinLatestCapture => {
                    match shell.app.toggle_pin_latest_capture() {
                        Some(true) => shell.set_error_status("pinned latest capture"),
                        Some(false) => shell.set_error_status("unpinned latest capture"),
                        None => shell.set_error_status("no capture available to pin"),
                    }
                }
                ShellKeyOutcome::LowerDrumBusLevel => {
                    let level = shell.app.adjust_drum_bus_level(-0.1);
                    shell.set_error_status(format!("drum bus level {:.2}", level));
                }
                ShellKeyOutcome::RaiseDrumBusLevel => {
                    let level = shell.app.adjust_drum_bus_level(0.1);
                    shell.set_error_status(format!("drum bus level {:.2}", level));
                }
                ShellKeyOutcome::LowerMc202Touch => {
                    let touch = shell.app.adjust_mc202_touch(-0.08);
                    shell.set_error_status(format!("MC-202 touch {:.2}", touch));
                }
                ShellKeyOutcome::RaiseMc202Touch => {
                    let touch = shell.app.adjust_mc202_touch(0.08);
                    shell.set_error_status(format!("MC-202 touch {:.2}", touch));
                }
                ShellKeyOutcome::AcceptCurrentGhostSuggestion => {
                    accept_current_ghost_suggestion(&mut shell, timestamp_now());
                }
                ShellKeyOutcome::RejectCurrentGhostSuggestion => {
                    reject_current_ghost_suggestion(&mut shell);
                }
                ShellKeyOutcome::UndoLast => {
                    if shell.app.undo_last_action(timestamp_now()).is_some() {
                        shell.set_error_status("undid most recent action");
                    } else {
                        shell.set_error_status("no undoable action available");
                    }
                }
                ShellKeyOutcome::RequestRefresh => match load_state(launch.mode.clone()) {
                    Ok(state) => shell.replace_app_state(state),
                    Err(error) => shell.set_error_status(format!("refresh failed: {error}")),
                },
            }

            if let Some(observer) = observer.as_deref_mut() {
                observer.record_key_event(
                    timestamp_now(),
                    &key_label,
                    shell_key_outcome_label(outcome),
                    &shell,
                )?;
            }
        }
    }
}

fn accept_current_ghost_suggestion(shell: &mut JamShellState, requested_at: u64) {
    match shell.app.accept_current_ghost_suggestion(requested_at) {
        riotbox_app::jam_app::GhostSuggestionQueueResult::Enqueued(action_id) => {
            shell.set_error_status(format!(
                "accepted ghost suggestion | queued action {}",
                action_id.0
            ));
        }
        riotbox_app::jam_app::GhostSuggestionQueueResult::Rejected { reason } => {
            shell.set_error_status(format!("ghost accept ignored: {reason}"));
        }
    }
}

fn reject_current_ghost_suggestion(shell: &mut JamShellState) {
    if shell.app.reject_current_ghost_suggestion() {
        shell.set_error_status("rejected current ghost suggestion");
    } else {
        shell.set_error_status("ghost reject ignored: no current ghost suggestion");
    }
}

fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn scene_select_unavailable_status(shell: &JamShellState) -> &'static str {
    match shell.app.jam_view.scene.scene_jump_availability {
        SceneJumpAvailabilityView::WaitingForMoreScenes => "scene jump waits for 2 scenes",
        SceneJumpAvailabilityView::Ready | SceneJumpAvailabilityView::Unknown => {
            "no next scene candidate available"
        }
    }
}
