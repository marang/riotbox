use crate::jam_app::projection::scene_context::{active_scene_movement, scene_context};
use crate::jam_app::projection::source_phrase_render::mc202_source_phrase_render_plan;
use crate::jam_app::transport_helpers::trusted_source_timing_bpm;
use riotbox_audio::mc202::{
    Mc202ContourHint, Mc202HookResponse, Mc202NoteBudget, Mc202PhraseShape, Mc202RenderMode,
    Mc202RenderRouting, Mc202RenderState,
};
use riotbox_core::ids::SceneId;
use riotbox_core::live_performance_policy::{
    LivePerformanceMc202Intent, derive_live_performance_policy,
};
use riotbox_core::session::{
    Mc202PhraseIntentState, Mc202RoleState, SceneMovementLaneIntentState, SceneMovementState,
    SessionFile,
};
use riotbox_core::source_graph::{
    EnergyClass, Section, SectionLabelHint, SourceGraph, section_for_projected_scene,
    section_for_transport_bar,
};
use riotbox_core::transport::TransportClockState;

pub(in crate::jam_app) fn build_mc202_render_state(
    session: &SessionFile,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
) -> Mc202RenderState {
    let tempo_bpm = trusted_source_timing_bpm(session, source_graph).unwrap_or(0.0);
    let silent_render_state = || Mc202RenderState {
        tempo_bpm,
        position_beats: transport.position_beats,
        is_transport_running: transport.is_playing,
        ..Mc202RenderState::default()
    };
    let mc202 = &session.runtime_state.lane_state.mc202;
    let Some(requested_role) = mc202.role else {
        return silent_render_state();
    };
    let live_policy = source_graph.and_then(|graph| derive_live_performance_policy(session, graph));
    let role = match live_policy.as_ref().map(|policy| policy.mc202_intent) {
        Some(LivePerformanceMc202Intent::BassPressure) => Mc202RoleState::Pressure,
        Some(LivePerformanceMc202Intent::Punctuate) => Mc202RoleState::Answer,
        Some(LivePerformanceMc202Intent::Instigate) => Mc202RoleState::Instigator,
        Some(LivePerformanceMc202Intent::StayOut) => return silent_render_state(),
        None => requested_role,
    };

    let (mode, base_phrase_shape) = mc202_render_mode_and_shape(role);
    let phrase_intent = Mc202PhraseIntentState::from_phrase_variant(mc202.phrase_variant);
    let phrase_shape = mc202_phrase_shape_for_intent(phrase_intent).unwrap_or(base_phrase_shape);
    let current_section = mc202_current_section(
        source_graph,
        transport,
        scene_context(session),
        &session.runtime_state.scene_state,
    );
    if mc202_source_plan_has_explicit_section_mismatch(
        mc202.source_phrase_plan.as_ref(),
        source_graph,
        current_section,
    ) {
        // The committed phrase was derived from a different source section. Keep
        // the target scene source-backed by staying out instead of overlaying an
        // untrusted tonal phrase or inventing a fallback.
        return silent_render_state();
    }
    let hook_response =
        mc202_hook_response_for_role_graph_and_section(role, source_graph, current_section);
    let movement = active_scene_movement(session);
    let touch = scene_movement_mc202_touch(
        session
            .runtime_state
            .macro_state
            .mc202_touch
            .clamp(0.0, 1.0),
        movement,
    );
    let contour_hint = scene_movement_mc202_contour(movement)
        .unwrap_or_else(|| mc202_contour_hint(current_section));
    Mc202RenderState {
        mode,
        routing: mc202_routing_for_source_plan(mc202.source_phrase_plan.as_ref()),
        phrase_shape,
        note_budget: mc202
            .source_phrase_plan
            .as_ref()
            .filter(|plan| plan.is_source_derived())
            .map_or_else(
                || mc202_note_budget_for_shape_and_hook_response(phrase_shape, hook_response),
                |plan| mc202_note_budget_from_source_plan(plan.note_budget),
            ),
        contour_hint,
        hook_response,
        source_phrase_plan: mc202_source_phrase_render_plan(mc202.source_phrase_plan.as_ref()),
        touch: live_policy
            .as_ref()
            .map_or(touch, |policy| touch.max(policy.mc202_touch_floor)),
        music_bus_level: live_policy
            .as_ref()
            .map_or_else(
                || session.runtime_state.mixer_state.music_level,
                |policy| policy.mc202_music_level,
            )
            .clamp(0.0, 1.0),
        tempo_bpm,
        position_beats: transport.position_beats,
        is_transport_running: transport.is_playing,
    }
}

fn mc202_render_mode_and_shape(role: Mc202RoleState) -> (Mc202RenderMode, Mc202PhraseShape) {
    match role {
        Mc202RoleState::Leader => (Mc202RenderMode::Leader, Mc202PhraseShape::RootPulse),
        Mc202RoleState::Follower => (Mc202RenderMode::Follower, Mc202PhraseShape::FollowerDrive),
        Mc202RoleState::Answer => (Mc202RenderMode::Answer, Mc202PhraseShape::RootPulse),
        Mc202RoleState::Pressure => (Mc202RenderMode::Pressure, Mc202PhraseShape::PressureCell),
        Mc202RoleState::Instigator => (
            Mc202RenderMode::Instigator,
            Mc202PhraseShape::InstigatorSpike,
        ),
    }
}

fn mc202_routing_for_source_plan(
    source_plan: Option<&riotbox_core::session::Mc202SourcePhrasePlanState>,
) -> Mc202RenderRouting {
    if source_plan.is_some_and(riotbox_core::session::Mc202SourcePhrasePlanState::is_source_derived)
    {
        return Mc202RenderRouting::MusicBusBass;
    }

    Mc202RenderRouting::Silent
}

fn mc202_note_budget_from_source_plan(
    budget: riotbox_core::session::Mc202SourcePhraseNoteBudgetState,
) -> Mc202NoteBudget {
    match budget {
        riotbox_core::session::Mc202SourcePhraseNoteBudgetState::Sparse => Mc202NoteBudget::Sparse,
        riotbox_core::session::Mc202SourcePhraseNoteBudgetState::Balanced => {
            Mc202NoteBudget::Balanced
        }
        riotbox_core::session::Mc202SourcePhraseNoteBudgetState::Push => Mc202NoteBudget::Push,
        riotbox_core::session::Mc202SourcePhraseNoteBudgetState::Wide => Mc202NoteBudget::Wide,
    }
}

fn mc202_phrase_shape_for_intent(intent: Mc202PhraseIntentState) -> Option<Mc202PhraseShape> {
    match intent {
        Mc202PhraseIntentState::Base => None,
        Mc202PhraseIntentState::MutatedDrive => Some(Mc202PhraseShape::MutatedDrive),
    }
}

fn scene_movement_mc202_contour(movement: Option<&SceneMovementState>) -> Option<Mc202ContourHint> {
    let movement = movement?;
    Some(match movement.mc202_intent {
        SceneMovementLaneIntentState::Lift => Mc202ContourHint::Lift,
        SceneMovementLaneIntentState::Drive => Mc202ContourHint::Drop,
        SceneMovementLaneIntentState::Release => Mc202ContourHint::Hold,
        SceneMovementLaneIntentState::Anchor => Mc202ContourHint::Hold,
    })
}

fn scene_movement_mc202_touch(base_touch: f32, movement: Option<&SceneMovementState>) -> f32 {
    let Some(movement) = movement else {
        return base_touch;
    };

    match movement.mc202_intent {
        SceneMovementLaneIntentState::Lift => base_touch.max(0.74 + movement.intensity * 0.18),
        SceneMovementLaneIntentState::Drive => base_touch.max(0.70 + movement.intensity * 0.14),
        SceneMovementLaneIntentState::Release => base_touch.min(0.62),
        SceneMovementLaneIntentState::Anchor => base_touch.clamp(0.48, 0.72),
    }
}

fn mc202_current_section<'a>(
    source_graph: Option<&'a SourceGraph>,
    transport: &TransportClockState,
    scene_context: Option<&SceneId>,
    scene_state: &riotbox_core::session::SceneState,
) -> Option<&'a Section> {
    let graph = source_graph?;
    scene_context
        .and_then(|scene_id| section_for_projected_scene(graph, scene_state, scene_id))
        .or_else(|| section_for_transport_bar(graph, transport))
}

fn mc202_source_plan_has_explicit_section_mismatch(
    plan: Option<&riotbox_core::session::Mc202SourcePhrasePlanState>,
    source_graph: Option<&SourceGraph>,
    current_section: Option<&Section>,
) -> bool {
    let (Some(plan), Some(graph)) = (plan, source_graph) else {
        return false;
    };
    if plan.source_id != graph.source.source_id {
        return true;
    }

    match (plan.source_section_id.as_ref(), current_section) {
        (Some(source_section_id), Some(current_section)) => {
            *source_section_id != current_section.section_id
        }
        (Some(_), None) => true,
        (None, _) => false,
    }
}

fn mc202_contour_hint(section: Option<&Section>) -> Mc202ContourHint {
    section
        .map(mc202_contour_hint_for_section)
        .unwrap_or(Mc202ContourHint::Neutral)
}

fn mc202_contour_hint_for_section(section: &Section) -> Mc202ContourHint {
    match (section.label_hint, section.energy_class) {
        (SectionLabelHint::Build, _) => Mc202ContourHint::Lift,
        (SectionLabelHint::Drop, EnergyClass::High | EnergyClass::Peak)
        | (SectionLabelHint::Chorus, EnergyClass::High | EnergyClass::Peak) => {
            Mc202ContourHint::Drop
        }
        (SectionLabelHint::Break | SectionLabelHint::Intro | SectionLabelHint::Outro, _) => {
            Mc202ContourHint::Hold
        }
        (_, EnergyClass::Low) => Mc202ContourHint::Hold,
        _ => Mc202ContourHint::Neutral,
    }
}

fn mc202_hook_response_for_role_graph_and_section(
    _role: Mc202RoleState,
    _source_graph: Option<&SourceGraph>,
    _section: Option<&Section>,
) -> Mc202HookResponse {
    Mc202HookResponse::Direct
}

fn mc202_note_budget_for_shape_and_hook_response(
    shape: Mc202PhraseShape,
    hook_response: Mc202HookResponse,
) -> Mc202NoteBudget {
    let _ = hook_response;

    match shape {
        Mc202PhraseShape::PressureCell => Mc202NoteBudget::Sparse,
        Mc202PhraseShape::InstigatorSpike => Mc202NoteBudget::Push,
        Mc202PhraseShape::MutatedDrive => Mc202NoteBudget::Wide,
        Mc202PhraseShape::RootPulse | Mc202PhraseShape::FollowerDrive => Mc202NoteBudget::Balanced,
    }
}
