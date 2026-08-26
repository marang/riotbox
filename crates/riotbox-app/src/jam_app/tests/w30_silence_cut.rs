use super::*;

fn silence_cut_state(with_source_audio: bool) -> JamAppState {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.session.runtime_state.lane_state.w30.preview_mode = Some(W30PreviewModeState::LiveRecall);
    if with_source_audio {
        state.capture_audio_cache.insert(
            CaptureId::from("cap-01"),
            source_cache_for_w30_diversity(127.0),
        );
    }
    state.refresh_view();
    state
}

#[test]
fn queue_targets_focused_capture_on_next_bar() {
    let mut state = silence_cut_state(true);
    state.set_transport_playing(true);
    state.refresh_view();

    assert_eq!(
        state.queue_w30_silence_cut(635),
        Some(QueueControlResult::Enqueued)
    );
    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::W30SilenceCut);
    assert_eq!(pending[0].quantization, Quantization::NextBar);
    assert_eq!(
        pending[0].params,
        ActionParams::Mutation {
            intensity: 1.0,
            target_id: Some("cap-01".into()),
        }
    );
    assert_eq!(
        pending[0].explanation.as_deref(),
        Some("silence-cut cap-01 on W-30 pad bank-a/pad-01 after four beats")
    );
}

#[test]
fn queue_refuses_stopped_or_missing_source_state() {
    let mut state = silence_cut_state(false);

    assert_eq!(state.queue_w30_silence_cut(636), None);
    assert!(state.queue.pending_actions().is_empty());
    state.set_transport_playing(true);
    state.refresh_view();
    assert_eq!(state.queue_w30_silence_cut(637), None);
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn commit_persists_projects_and_replays_without_other_lane_changes() {
    let mut state = silence_cut_state(true);
    state.session.runtime_state.macro_state.w30_grit = 0.31;
    state.session.runtime_state.mixer_state.music_level = 0.73;
    state.set_transport_playing(true);
    state.refresh_view();
    let replay_base = state.session.clone();
    let unchanged_mc202 = state.session.runtime_state.lane_state.mc202.clone();
    let unchanged_tr909 = state.session.runtime_state.lane_state.tr909.clone();
    let unchanged_source_monitor = state.session.runtime_state.source_monitor.clone();

    assert_eq!(
        state.queue_w30_silence_cut(636),
        Some(QueueControlResult::Enqueued)
    );
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 84,
            bar_index: 21,
            phrase_index: 5,
            scene_id: Some(SceneId::from("scene-1")),
        },
        736,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.lane_state.mc202,
        unchanged_mc202
    );
    assert_eq!(
        state.session.runtime_state.lane_state.tr909,
        unchanged_tr909
    );
    assert_eq!(
        state.session.runtime_state.source_monitor,
        unchanged_source_monitor
    );
    let articulation = state
        .session
        .runtime_state
        .lane_state
        .w30
        .hook_articulation
        .as_ref()
        .expect("committed silence-cut articulation");
    assert_eq!(
        articulation.profile,
        riotbox_core::session::W30HookArticulationProfileState::SilenceCutV1
    );
    assert_eq!(articulation.capture_id, CaptureId::from("cap-01"));
    assert_eq!(articulation.started_at_beat, 84);
    let projected = state
        .runtime
        .w30_preview
        .pad_playback
        .as_ref()
        .and_then(|pad| pad.hook_articulation)
        .expect("projected silence-cut articulation");
    assert_eq!(
        projected.profile,
        riotbox_audio::w30::W30HookArticulationProfile::SilenceCutV1
    );
    assert_eq!(projected.started_at_beat, 84);
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("silence-cut cap-01 on W-30 pad bank-a/pad-01")
    );

    let serialized = serde_json::to_string(&state.session).expect("serialize silence-cut session");
    let restored: SessionFile =
        serde_json::from_str(&serialized).expect("restore silence-cut session");
    assert_eq!(
        restored.runtime_state.lane_state.w30.hook_articulation,
        state.session.runtime_state.lane_state.w30.hook_articulation
    );

    let plan = riotbox_core::replay::build_committed_replay_plan(&state.session.action_log)
        .expect("build silence-cut replay plan");
    let mut replayed = replay_base;
    replayed.action_log = state.session.action_log.clone();
    riotbox_core::replay::apply_replay_plan_to_session(&mut replayed, &plan)
        .expect("replay silence cut");
    assert_eq!(
        replayed.runtime_state.lane_state.w30.hook_articulation,
        state.session.runtime_state.lane_state.w30.hook_articulation
    );
    assert_eq!(replayed.runtime_state.macro_state.w30_grit, 0.31);
    assert_eq!(replayed.runtime_state.mixer_state.music_level, 0.73);
}
