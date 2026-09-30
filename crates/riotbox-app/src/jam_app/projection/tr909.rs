use crate::jam_app::projection::scene_context::active_scene_movement;
use crate::jam_app::transport_helpers::trusted_source_timing_bpm;
use riotbox_audio::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909RenderState, Tr909SourceSupportContext, Tr909SourceSupportProfile,
    Tr909TakeoverRenderProfile,
};
use riotbox_core::live_performance_policy::{
    LivePerformanceTr909Intent, derive_live_performance_policy,
};
use riotbox_core::session::{
    SceneMovementDirectionState, SceneMovementLaneIntentState, SessionFile,
};
use riotbox_core::source_graph::SourceGraph;
use riotbox_core::style::PerformancePresetId;
use riotbox_core::tr909_policy::{
    Tr909PatternAdoptionPolicy, Tr909PhraseVariationPolicy, Tr909RenderModePolicy,
    Tr909RenderRoutingPolicy, Tr909SourceSupportContextPolicy, Tr909SourceSupportProfilePolicy,
    Tr909TakeoverRenderProfilePolicy, derive_tr909_render_policy_with_scene_context,
};
use riotbox_core::transport::TransportClockState;

fn audio_tr909_render_mode(mode: Tr909RenderModePolicy) -> Tr909RenderMode {
    match mode {
        Tr909RenderModePolicy::Idle => Tr909RenderMode::Idle,
        Tr909RenderModePolicy::SourceSupport => Tr909RenderMode::SourceSupport,
        Tr909RenderModePolicy::Fill => Tr909RenderMode::Fill,
        Tr909RenderModePolicy::BreakReinforce => Tr909RenderMode::BreakReinforce,
        Tr909RenderModePolicy::Takeover => Tr909RenderMode::Takeover,
    }
}

fn audio_tr909_render_routing(routing: Tr909RenderRoutingPolicy) -> Tr909RenderRouting {
    match routing {
        Tr909RenderRoutingPolicy::SourceOnly => Tr909RenderRouting::SourceOnly,
        Tr909RenderRoutingPolicy::DrumBusSupport => Tr909RenderRouting::DrumBusSupport,
        Tr909RenderRoutingPolicy::DrumBusTakeover => Tr909RenderRouting::DrumBusTakeover,
    }
}

fn audio_tr909_source_support_profile(
    profile: Option<Tr909SourceSupportProfilePolicy>,
) -> Option<Tr909SourceSupportProfile> {
    profile.map(|profile| match profile {
        Tr909SourceSupportProfilePolicy::SteadyPulse => Tr909SourceSupportProfile::SteadyPulse,
        Tr909SourceSupportProfilePolicy::BreakLift => Tr909SourceSupportProfile::BreakLift,
        Tr909SourceSupportProfilePolicy::DropDrive => Tr909SourceSupportProfile::DropDrive,
    })
}

fn audio_tr909_source_support_context(
    context: Option<Tr909SourceSupportContextPolicy>,
) -> Option<Tr909SourceSupportContext> {
    context.map(|context| match context {
        Tr909SourceSupportContextPolicy::SceneTarget => Tr909SourceSupportContext::SceneTarget,
        Tr909SourceSupportContextPolicy::TransportBar => Tr909SourceSupportContext::TransportBar,
    })
}

fn audio_tr909_takeover_profile(
    profile: Option<Tr909TakeoverRenderProfilePolicy>,
) -> Option<Tr909TakeoverRenderProfile> {
    profile.map(|profile| match profile {
        Tr909TakeoverRenderProfilePolicy::ControlledPhrase => {
            Tr909TakeoverRenderProfile::ControlledPhrase
        }
        Tr909TakeoverRenderProfilePolicy::SceneLock => Tr909TakeoverRenderProfile::SceneLock,
    })
}

fn audio_tr909_pattern_adoption(
    adoption: Option<Tr909PatternAdoptionPolicy>,
) -> Option<Tr909PatternAdoption> {
    adoption.map(|adoption| match adoption {
        Tr909PatternAdoptionPolicy::SupportPulse => Tr909PatternAdoption::SupportPulse,
        Tr909PatternAdoptionPolicy::MainlineDrive => Tr909PatternAdoption::MainlineDrive,
        Tr909PatternAdoptionPolicy::TakeoverGrid => Tr909PatternAdoption::TakeoverGrid,
    })
}

fn audio_tr909_phrase_variation(
    variation: Option<Tr909PhraseVariationPolicy>,
) -> Option<Tr909PhraseVariation> {
    variation.map(|variation| match variation {
        Tr909PhraseVariationPolicy::PhraseAnchor => Tr909PhraseVariation::PhraseAnchor,
        Tr909PhraseVariationPolicy::PhraseLift => Tr909PhraseVariation::PhraseLift,
        Tr909PhraseVariationPolicy::PhraseDrive => Tr909PhraseVariation::PhraseDrive,
        Tr909PhraseVariationPolicy::PhraseRelease => Tr909PhraseVariation::PhraseRelease,
    })
}

pub(in crate::jam_app) fn build_tr909_render_state(
    session: &SessionFile,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
) -> Tr909RenderState {
    let tr909 = &session.runtime_state.lane_state.tr909;
    let mixer = &session.runtime_state.mixer_state;
    let tempo_bpm = trusted_source_timing_bpm(session, source_graph).unwrap_or(0.0);
    let scene_context = session
        .runtime_state
        .scene_state
        .active_scene
        .as_ref()
        .or(transport.current_scene.as_ref());
    let policy = derive_tr909_render_policy_with_scene_context(
        tr909,
        transport,
        source_graph,
        scene_context,
        &session.runtime_state.scene_state,
    );
    let live_policy = source_graph.and_then(|graph| derive_live_performance_policy(session, graph));
    let explicit_held_state_override = matches!(
        policy.mode,
        Tr909RenderModePolicy::Fill | Tr909RenderModePolicy::Takeover
    ) || tr909.slam_enabled
        || scene_movement_tr909_variation(session).is_some()
        || scene_movement_tr909_slam(session) > 0.0;
    if live_policy.as_ref().is_some_and(|policy| {
        policy.tr909_intent == LivePerformanceTr909Intent::StayOut && !explicit_held_state_override
    }) {
        return Tr909RenderState {
            is_transport_running: transport.is_playing,
            tempo_bpm,
            position_beats: transport.position_beats,
            source_bar_grid_anchor_position_beats: live_policy
                .as_ref()
                .and_then(|policy| policy.source_bar_grid_anchor_beat_cursor)
                .map(|cursor| cursor as f64),
            current_scene_id: transport.current_scene.as_ref().map(ToString::to_string),
            ..Tr909RenderState::default()
        };
    }
    let character_pattern_adoption = matches!(
        policy.mode,
        Tr909RenderModePolicy::SourceSupport | Tr909RenderModePolicy::BreakReinforce
    )
    .then(|| {
        live_policy
            .as_ref()
            .and_then(|policy| policy.tr909_pattern_adoption)
    })
    .flatten();
    let character_phrase_variation = matches!(
        policy.mode,
        Tr909RenderModePolicy::SourceSupport | Tr909RenderModePolicy::BreakReinforce
    )
    .then(|| {
        live_policy
            .as_ref()
            .and_then(|policy| policy.tr909_phrase_variation)
    })
    .flatten();

    Tr909RenderState {
        mode: audio_tr909_render_mode(policy.mode),
        routing: audio_tr909_render_routing(policy.routing),
        source_support_profile: audio_tr909_source_support_profile(policy.source_support_profile),
        source_support_context: audio_tr909_source_support_context(policy.source_support_context),
        pattern_ref: tr909.pattern_ref.clone(),
        pattern_adoption: audio_tr909_pattern_adoption(
            character_pattern_adoption.or(policy.pattern_adoption),
        ),
        phrase_variation: scene_movement_tr909_variation(session)
            .or_else(|| preset_tr909_fill_variation(session, policy.mode))
            .or_else(|| audio_tr909_phrase_variation(character_phrase_variation))
            .or_else(|| audio_tr909_phrase_variation(policy.phrase_variation)),
        takeover_profile: audio_tr909_takeover_profile(policy.takeover_profile),
        drum_bus_level: live_policy
            .as_ref()
            .map_or(mixer.drum_level, |policy| {
                mixer.drum_level.max(policy.tr909_drum_level)
            })
            .clamp(0.0, 1.0),
        slam_enabled: tr909.slam_enabled,
        slam_intensity: scene_movement_tr909_slam(session)
            .max(session.runtime_state.macro_state.tr909_slam)
            .max(
                live_policy
                    .as_ref()
                    .map_or(0.0, |policy| policy.tr909_slam_floor),
            )
            .clamp(0.0, 1.0),
        is_transport_running: transport.is_playing,
        tempo_bpm,
        position_beats: transport.position_beats,
        source_bar_grid_anchor_position_beats: live_policy
            .as_ref()
            .and_then(|policy| policy.source_bar_grid_anchor_beat_cursor)
            .map(|cursor| cursor as f64),
        current_scene_id: transport.current_scene.as_ref().map(ToString::to_string),
    }
}

fn preset_tr909_fill_variation(
    session: &SessionFile,
    mode: Tr909RenderModePolicy,
) -> Option<Tr909PhraseVariation> {
    (mode == Tr909RenderModePolicy::Fill
        && session.runtime_state.style.active_preset
            == Some(PerformancePresetId::FeralBreakAlphaV2))
    .then_some(Tr909PhraseVariation::PhraseDriveHardCut)
}

fn scene_movement_tr909_variation(session: &SessionFile) -> Option<Tr909PhraseVariation> {
    let movement = active_scene_movement(session)?;
    Some(match movement.tr909_intent {
        SceneMovementLaneIntentState::Drive => Tr909PhraseVariation::PhraseDrive,
        SceneMovementLaneIntentState::Lift => Tr909PhraseVariation::PhraseLift,
        SceneMovementLaneIntentState::Release => Tr909PhraseVariation::PhraseRelease,
        SceneMovementLaneIntentState::Anchor => Tr909PhraseVariation::PhraseAnchor,
    })
}

fn scene_movement_tr909_slam(session: &SessionFile) -> f32 {
    active_scene_movement(session).map_or(0.0, |movement| {
        let floor = match movement.direction {
            SceneMovementDirectionState::Rise => 0.36,
            SceneMovementDirectionState::Drop => 0.18,
            SceneMovementDirectionState::Hold => 0.08,
        };
        movement.intensity * floor
    })
}
