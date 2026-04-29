#[test]
fn mc202_fixture_backed_committed_state_regressions_hold() {
    let fixtures: Vec<Mc202RegressionFixture> =
        serde_json::from_str(include_str!("../../../tests/fixtures/mc202_regression.json"))
            .expect("parse MC-202 regression fixtures");

    for fixture in fixtures {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.mc202.role = Some(fixture.initial_role.clone());
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        let queue_result = match fixture.action {
            Mc202RegressionAction::SetRole => state.queue_mc202_role_toggle(fixture.requested_at),
            Mc202RegressionAction::GenerateFollower => {
                state.queue_mc202_generate_follower(fixture.requested_at)
            }
            Mc202RegressionAction::GenerateAnswer => {
                state.queue_mc202_generate_answer(fixture.requested_at)
            }
            Mc202RegressionAction::GeneratePressure => {
                state.queue_mc202_generate_pressure(fixture.requested_at)
            }
            Mc202RegressionAction::GenerateInstigator => {
                state.queue_mc202_generate_instigator(fixture.requested_at)
            }
        };
        assert_eq!(
            queue_result,
            QueueControlResult::Enqueued,
            "{} did not enqueue",
            fixture.name
        );

        let committed = state.commit_ready_actions(
            fixture.boundary.into_commit_boundary_state(),
            fixture.committed_at,
        );
        assert_eq!(
            committed.len(),
            1,
            "{} did not commit exactly one action",
            fixture.name
        );

        assert_eq!(
            state.session.runtime_state.lane_state.mc202.role.as_deref(),
            Some(fixture.expected.role.as_str()),
            "{} role drifted",
            fixture.name
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .mc202
                .phrase_ref
                .as_deref(),
            Some(fixture.expected.phrase_ref.as_str()),
            "{} phrase ref drifted",
            fixture.name
        );
        assert_eq!(
            state.session.runtime_state.macro_state.mc202_touch, fixture.expected.touch,
            "{} touch drifted",
            fixture.name
        );
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some(fixture.expected.result_summary.as_str()),
            "{} result summary drifted",
            fixture.name
        );
        assert!(
            state.jam_view.lanes.mc202_pending_role.is_none(),
            "{} left a pending role behind",
            fixture.name
        );
        assert!(
            !state.jam_view.lanes.mc202_pending_follower_generation,
            "{} left a pending follower-generation behind",
            fixture.name
        );
        assert!(
            !state.jam_view.lanes.mc202_pending_answer_generation,
            "{} left a pending answer-generation behind",
            fixture.name
        );
        assert!(
            !state.jam_view.lanes.mc202_pending_pressure_generation,
            "{} left a pending pressure-generation behind",
            fixture.name
        );
        assert!(
            !state.jam_view.lanes.mc202_pending_instigator_generation,
            "{} left a pending instigator-generation behind",
            fixture.name
        );

        let tempdir = tempdir().expect("create MC-202 regression tempdir");
        let session_path = tempdir.path().join(format!("{}.json", fixture.name));
        save_session_json(&session_path, &state.session).expect("save MC-202 regression session");
        let loaded = load_session_json(&session_path).expect("reload MC-202 regression session");

        assert_eq!(
            loaded.runtime_state.lane_state.mc202.role.as_deref(),
            Some(fixture.expected.role.as_str()),
            "{} role did not survive replay roundtrip",
            fixture.name
        );
        assert_eq!(
            loaded.runtime_state.lane_state.mc202.phrase_ref.as_deref(),
            Some(fixture.expected.phrase_ref.as_str()),
            "{} phrase ref did not survive replay roundtrip",
            fixture.name
        );
        assert_eq!(
            loaded.runtime_state.macro_state.mc202_touch, fixture.expected.touch,
            "{} touch did not survive replay roundtrip",
            fixture.name
        );
        assert_eq!(
            loaded
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some(fixture.expected.result_summary.as_str()),
            "{} result summary did not survive replay roundtrip",
            fixture.name
        );
    }
}

#[test]
fn w30_fixture_backed_committed_state_regressions_hold() {
    let fixtures: Vec<W30RegressionFixture> =
        serde_json::from_str(include_str!("../../../tests/fixtures/w30_regression.json"))
            .expect("parse W-30 regression fixtures");

    for fixture in fixtures {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.captures[0].assigned_target =
            fixture.capture_assigned.then(|| CaptureTarget::W30Pad {
                bank_id: BankId::from(fixture.capture_bank.clone()),
                pad_id: PadId::from(fixture.capture_pad.clone()),
            });
        session.captures[0].is_pinned = fixture.capture_pinned;
        session.captures[0].source_window =
            fixture
                .source_window
                .as_ref()
                .map(|source_window| CaptureSourceWindow {
                    source_id: SourceId::from(source_window.source_id.clone()),
                    start_seconds: source_window.start_seconds,
                    end_seconds: source_window.end_seconds,
                    start_frame: source_window.start_frame,
                    end_frame: source_window.end_frame,
                });
        for extra in &fixture.extra_captures {
            session.captures.push(CaptureRef {
                capture_id: CaptureId::from(extra.capture_id.clone()),
                capture_type: CaptureType::Pad,
                source_origin_refs: vec!["fixture-extra".into()],
                source_window: None,
                lineage_capture_refs: Vec::new(),
                resample_generation_depth: 0,
                created_from_action: None,
                storage_path: format!("captures/{}.wav", extra.capture_id),
                assigned_target: Some(CaptureTarget::W30Pad {
                    bank_id: BankId::from(extra.bank.clone()),
                    pad_id: PadId::from(extra.pad.clone()),
                }),
                is_pinned: extra.pinned,
                notes: extra.notes.clone(),
            });
        }
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from(
            fixture
                .initial_active_bank
                .clone()
                .unwrap_or_else(|| fixture.capture_bank.clone()),
        ));
        session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from(
            fixture
                .initial_focused_pad
                .clone()
                .unwrap_or_else(|| fixture.capture_pad.clone()),
        ));
        session.runtime_state.lane_state.w30.last_capture =
            fixture.initial_last_capture.clone().map(CaptureId::from);
        session.runtime_state.lane_state.w30.preview_mode = fixture
            .initial_preview_mode
            .as_deref()
            .map(w30_preview_mode_state);
        session.runtime_state.macro_state.w30_grit = fixture.initial_w30_grit.unwrap_or(0.0);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        let queue_result = match fixture.action {
            W30RegressionAction::LiveRecall => state.queue_w30_live_recall(fixture.requested_at),
            W30RegressionAction::RawCaptureAudition => {
                state.queue_w30_audition(fixture.requested_at)
            }
            W30RegressionAction::PromotedAudition => {
                state.queue_w30_promoted_audition(fixture.requested_at)
            }
            W30RegressionAction::TriggerPad => state.queue_w30_trigger_pad(fixture.requested_at),
            W30RegressionAction::SwapBank => state.queue_w30_swap_bank(fixture.requested_at),
            W30RegressionAction::ApplyDamageProfile => {
                state.queue_w30_apply_damage_profile(fixture.requested_at)
            }
            W30RegressionAction::LoopFreeze => state.queue_w30_loop_freeze(fixture.requested_at),
            W30RegressionAction::BrowseSlicePool => {
                state.queue_w30_browse_slice_pool(fixture.requested_at)
            }
        };
        assert_eq!(
            queue_result,
            Some(QueueControlResult::Enqueued),
            "{} did not enqueue",
            fixture.name
        );

        let committed = state.commit_ready_actions(
            fixture.boundary.into_commit_boundary_state(),
            fixture.committed_at,
        );
        assert_eq!(
            committed.len(),
            1,
            "{} did not commit exactly one action",
            fixture.name
        );
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .map(|action| action.command),
            Some(expected_w30_command(fixture.action)),
            "{} command drifted",
            fixture.name
        );

        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .w30
                .active_bank
                .as_ref()
                .map(ToString::to_string),
            Some(fixture.expected.active_bank.clone()),
            "{} bank drifted",
            fixture.name
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .w30
                .focused_pad
                .as_ref()
                .map(ToString::to_string),
            Some(fixture.expected.focused_pad.clone()),
            "{} pad drifted",
            fixture.name
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .w30
                .last_capture
                .as_ref()
                .map(ToString::to_string),
            Some(fixture.expected.last_capture.clone()),
            "{} last capture drifted",
            fixture.name
        );
        assert_eq!(
            state.session.runtime_state.macro_state.w30_grit, fixture.expected.w30_grit,
            "{} grit drifted",
            fixture.name
        );
        if let Some(expected) = fixture.expected.preview_mode.as_deref() {
            assert_eq!(
                state.runtime.w30_preview.mode.label(),
                expected,
                "{} preview mode drifted",
                fixture.name
            );
        }
        if let Some(expected) = fixture.expected.preview_routing.as_deref() {
            assert_eq!(
                state.runtime.w30_preview.routing.label(),
                expected,
                "{} preview routing drifted",
                fixture.name
            );
        }
        if let Some(expected) = fixture.expected.preview_profile.as_deref() {
            assert_eq!(
                state
                    .runtime
                    .w30_preview
                    .source_profile
                    .map(|profile| profile.label()),
                Some(expected),
                "{} preview profile drifted",
                fixture.name
            );
        }
        if let Some(expected) = fixture.expected.preview_capture.as_deref() {
            assert_eq!(
                state.runtime.w30_preview.capture_id.as_deref(),
                Some(expected),
                "{} preview capture drifted",
                fixture.name
            );
        }
        if let Some(expected) = fixture.expected.preview_music_bus_level {
            assert!(
                (state.runtime.w30_preview.music_bus_level - expected).abs() < f32::EPSILON,
                "{} preview music bus drifted",
                fixture.name
            );
        }
        if let Some(expected) = fixture.expected.preview_grit_level {
            assert!(
                (state.runtime.w30_preview.grit_level - expected).abs() < f32::EPSILON,
                "{} preview grit drifted",
                fixture.name
            );
        }
        if let Some(expected) = fixture.expected.preview_transport_running {
            assert_eq!(
                state.runtime.w30_preview.is_transport_running, expected,
                "{} preview transport-running drifted",
                fixture.name
            );
        }
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some(fixture.expected.result_summary.as_str()),
            "{} result summary drifted",
            fixture.name
        );
    }
}

#[test]
fn committed_tr909_slam_action_updates_lane_state_and_macro_intensity() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert!(state.queue_tr909_slam_toggle(300));

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Beat,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert!(state.session.runtime_state.lane_state.tr909.slam_enabled);
    assert!(state.session.runtime_state.macro_state.tr909_slam >= 0.85);
    assert!(state.jam_view.lanes.tr909_slam_enabled);
    assert!(state.jam_view.macros.tr909_slam >= 0.85);
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("enabled TR-909 slam at 0.85")
    );
}

