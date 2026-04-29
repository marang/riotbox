#[test]
fn projects_transport_source_and_scene_summary() {
    let fixture = jam_view_fixture();
    let vm = fixture.build_view_model();

    assert!(vm.transport.is_playing);
    assert_eq!(vm.source.loop_candidate_count, 1);
    assert_eq!(vm.source.hook_candidate_count, 1);
    assert_eq!(vm.source.feral_scorecard.readiness, "ready");
    assert_eq!(vm.source.feral_scorecard.break_rebuild_potential, "high");
    assert_eq!(vm.source.feral_scorecard.hook_fragment_count, 1);
    assert_eq!(vm.source.feral_scorecard.break_support_count, 1);
    assert_eq!(vm.source.feral_scorecard.quote_risk_count, 1);
    assert_eq!(vm.source.feral_scorecard.capture_candidate_count, 1);
    assert_eq!(
        vm.source.feral_scorecard.top_reason,
        "use capture before quoting"
    );
    assert_eq!(
        vm.source.feral_scorecard.warnings,
        vec!["quote risk 1".to_string()]
    );
    assert_eq!(vm.scene.scene_count, 1);
    assert_eq!(vm.scene.restore_scene, None);
    assert_eq!(
        vm.scene.scene_jump_availability,
        SceneJumpAvailabilityView::WaitingForMoreScenes
    );
    assert_eq!(vm.scene.active_scene_energy.as_deref(), Some("high"));
    assert_eq!(vm.scene.restore_scene_energy, None);
}

#[test]
fn projects_capture_summary() {
    let fixture = jam_view_fixture();
    let vm = fixture.build_view_model();

    assert_eq!(vm.capture.capture_count, 1);
    assert_eq!(vm.capture.pinned_capture_count, 0);
    assert_eq!(vm.capture.promoted_capture_count, 1);
    assert_eq!(vm.capture.unassigned_capture_count, 0);
    assert_eq!(vm.capture.pending_capture_count, 2);
    assert_eq!(vm.capture.last_capture_id.as_deref(), Some("cap-01"));
    assert_eq!(
        vm.capture.last_capture_target.as_deref(),
        Some("pad bank-a/pad-01")
    );
    assert_eq!(
        vm.capture.last_capture_target_kind,
        Some(CaptureTargetKindView::W30Pad)
    );
    assert_eq!(
        vm.capture.last_capture_handoff_readiness,
        Some(CaptureHandoffReadinessView::Fallback)
    );
    assert_eq!(
        vm.capture.last_promotion_result.as_deref(),
        Some("promoted to pad bank-a/pad-01")
    );
    assert_eq!(
        vm.capture.latest_w30_promoted_capture_label.as_deref(),
        Some("cap-01 -> bank-a/pad-01")
    );
    assert_eq!(
        vm.capture.recent_capture_rows,
        vec!["cap-01 | bank-a/pad-01 | 2 origins"]
    );
    assert_eq!(
        vm.capture.latest_capture_provenance_lines,
        vec![
            "file captures/cap-01.wav",
            "from action manual or unknown",
            "origins asset-a, src-1",
        ]
    );
    assert!(vm.capture.pinned_capture_ids.is_empty());
    assert_eq!(vm.capture.pending_capture_items.len(), 2);
    assert_eq!(vm.capture.pending_capture_items[0].command, "capture.now");
    assert_eq!(vm.capture.pending_capture_items[0].target, "lanew30");
    assert_eq!(
        vm.capture.pending_capture_items[0].explanation.as_deref(),
        Some("capture current break")
    );
    assert_eq!(
        vm.capture.pending_capture_items[1].command,
        "promote.resample"
    );
    assert_eq!(vm.capture.pending_capture_items[1].target, "lanew30");
}

#[test]
fn projects_lane_pending_summary() {
    let fixture = jam_view_fixture();
    let vm = fixture.build_view_model();

    assert_eq!(vm.lanes.mc202_pending_role.as_deref(), Some("leader"));
    assert!(!vm.lanes.mc202_pending_follower_generation);
    assert!(!vm.lanes.mc202_pending_answer_generation);
    assert_eq!(vm.lanes.mc202_phrase_ref, None);
    assert_eq!(vm.lanes.w30_active_bank.as_deref(), Some("bank-a"));
    assert_eq!(vm.lanes.w30_focused_pad.as_deref(), Some("pad-01"));
    assert_eq!(
        vm.lanes.w30_pending_trigger_target.as_deref(),
        Some("bank-a/pad-03")
    );
    assert_eq!(
        vm.lanes.w30_pending_recall_target.as_deref(),
        Some("bank-a/pad-02")
    );
    assert_eq!(
        vm.lanes.w30_pending_bank_swap_target.as_deref(),
        Some("bank-c/pad-01")
    );
    assert_eq!(
        vm.lanes.w30_pending_slice_pool_target.as_deref(),
        Some("bank-a/pad-04")
    );
    assert_eq!(
        vm.lanes.w30_pending_slice_pool_capture_id.as_deref(),
        Some("cap-02")
    );
    assert_eq!(
        vm.lanes.w30_pending_slice_pool_reason.as_deref(),
        Some("cycle")
    );
    assert_eq!(
        vm.lanes.w30_pending_damage_profile_target.as_deref(),
        Some("bank-d/pad-03")
    );
    assert_eq!(vm.lanes.w30_pending_audition_target, None);
    assert_eq!(
        vm.lanes.w30_pending_focus_step_target.as_deref(),
        Some("bank-c/pad-01")
    );
    assert_eq!(
        vm.lanes.w30_pending_resample_capture_id.as_deref(),
        Some("cap-01")
    );
    assert!(vm.lanes.tr909_takeover_enabled);
    assert_eq!(vm.lanes.tr909_takeover_pending_target, Some(false));
    assert_eq!(vm.lanes.tr909_takeover_pending_profile, None);
    assert_eq!(
        vm.lanes.tr909_takeover_profile,
        Some(Tr909TakeoverProfileState::SceneLockTakeover)
    );
    assert!(vm.lanes.tr909_fill_armed_next_bar);
    assert_eq!(vm.lanes.tr909_last_fill_bar, Some(8));
    assert_eq!(
        vm.lanes.tr909_reinforcement_mode,
        Some(Tr909ReinforcementModeState::Takeover)
    );
}

#[test]
fn projects_pending_actions_and_assist_ghost_summary() {
    let fixture = jam_view_fixture();
    let vm = fixture.build_view_model();

    assert_eq!(vm.pending_actions.len(), 11);
    assert_eq!(vm.ghost.mode, "assist");
    assert_eq!(vm.ghost.suggestion_count, 1);
    assert!(!vm.ghost.is_read_only);
    assert_eq!(vm.ghost.latest_proposal_id.as_deref(), Some("gp-1"));
    assert_eq!(vm.ghost.latest_summary.as_deref(), Some("capture next bar"));
    assert_eq!(vm.ghost.latest_status.as_deref(), Some("suggested"));
    assert_eq!(vm.ghost.safety, "clear");
    assert_eq!(vm.ghost.active_blocker, None);
}

#[test]
fn projects_blocked_watch_suggestion_into_jam_view() {
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-29T16:00:00Z");
    session.ghost_state.mode = GhostMode::Watch;
    session.ghost_state.suggestion_history = vec![GhostSuggestionRecord {
        proposal_id: "ghost-watch-1".into(),
        summary: "capture the source-backed hit".into(),
        accepted: false,
        rejected: false,
    }];
    session
        .runtime_state
        .lock_state
        .locked_object_ids
        .push("ghost.main".into());

    let vm = JamViewModel::build(&session, &ActionQueue::new(), None);

    assert_eq!(vm.ghost.mode, "watch");
    assert_eq!(vm.ghost.suggestion_count, 1);
    assert!(vm.ghost.is_read_only);
    assert!(vm.ghost.is_blocked);
    assert_eq!(vm.ghost.safety, "blocked");
    assert_eq!(vm.ghost.active_blocker.as_deref(), Some("ghost.main"));
    assert_eq!(
        vm.ghost.latest_summary.as_deref(),
        Some("capture the source-backed hit")
    );
    assert_eq!(vm.ghost.latest_status.as_deref(), Some("suggested"));
}

#[test]
fn projects_rejected_ghost_suggestion_into_jam_view() {
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-29T16:05:00Z");
    session.ghost_state.mode = GhostMode::Watch;
    session.ghost_state.suggestion_history = vec![GhostSuggestionRecord {
        proposal_id: "ghost-watch-1".into(),
        summary: "capture the source-backed hit".into(),
        accepted: false,
        rejected: true,
    }];

    let vm = JamViewModel::build(&session, &ActionQueue::new(), None);

    assert_eq!(vm.ghost.latest_status.as_deref(), Some("rejected"));
}
