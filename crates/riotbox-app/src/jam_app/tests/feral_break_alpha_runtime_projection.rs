#[test]
fn feral_break_alpha_fill_keeps_its_destructive_phrase_drive_recipe() {
    let graph = sample_graph();
    let mut baseline = JamAppState::from_parts(
        sample_session(&graph),
        Some(graph.clone()),
        ActionQueue::new(),
    );
    let fill_transport = TransportClockState {
        is_playing: true,
        position_beats: 34.0,
        beat_index: 34,
        bar_index: 9,
        phrase_index: 3,
        current_scene: baseline.runtime.transport.current_scene.clone(),
    };
    baseline.session.runtime_state.lane_state.tr909.last_fill_bar = Some(9);
    baseline.update_transport_clock(fill_transport.clone());

    assert_eq!(baseline.runtime.tr909_render.mode, Tr909RenderMode::Fill);
    assert_eq!(
        baseline.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseLift),
        "the odd phrase-cycle control should remain the non-preset baseline"
    );
    assert_eq!(
        baseline.runtime.tr909_render.fill_recipe_id(),
        Some(riotbox_audio::tr909::Tr909FillRecipeId::GenericFillV1)
    );

    let mut v1_session = sample_session(&graph);
    riotbox_core::style::PerformancePresetId::FeralBreakAlphaV1
        .apply_to_session(&mut v1_session);
    v1_session.runtime_state.lane_state.tr909.last_fill_bar = Some(9);
    let mut v1 = JamAppState::from_parts(
        v1_session,
        Some(graph.clone()),
        ActionQueue::new(),
    );
    v1.update_transport_clock(TransportClockState {
        phrase_index: 2,
        ..fill_transport.clone()
    });
    assert_eq!(
        v1.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDrive)
    );
    assert_eq!(
        v1.runtime.tr909_render.fill_recipe_id(),
        Some(riotbox_audio::tr909::Tr909FillRecipeId::PhraseDriveBreakCutStompV1),
        "the historical v1 preset must not inherit the v2 hard-cut recipe"
    );

    let mut preset_session = sample_session(&graph);
    riotbox_core::style::PerformancePresetId::FeralBreakAlphaV2
        .apply_to_session(&mut preset_session);
    preset_session
        .runtime_state
        .lane_state
        .tr909
        .last_fill_bar = Some(9);
    let mut preset =
        JamAppState::from_parts(preset_session, Some(graph), ActionQueue::new());
    preset.update_transport_clock(fill_transport);

    assert_eq!(preset.runtime.tr909_render.mode, Tr909RenderMode::Fill);
    assert_eq!(
        preset.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDriveHardCut)
    );
    assert_eq!(
        preset.runtime.tr909_render.fill_recipe_id(),
        Some(riotbox_audio::tr909::Tr909FillRecipeId::PhraseDriveBreakCutStompV2)
    );
}
