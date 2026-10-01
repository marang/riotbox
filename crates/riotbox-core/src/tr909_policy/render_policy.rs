use super::model::{
    Tr909RenderModePolicy, Tr909RenderPolicyProjection, Tr909RenderRoutingPolicy,
    Tr909TakeoverRenderProfilePolicy,
};
use super::pattern_variation::{derive_tr909_pattern_adoption, derive_tr909_phrase_variation};
use super::source_support::derive_tr909_source_support;
use crate::{
    ids::SceneId,
    session::{SceneState, Tr909LaneState, Tr909ReinforcementModeState, Tr909TakeoverProfileState},
    source_graph::SourceGraph,
    transport::TransportClockState,
};

#[must_use]
pub fn derive_tr909_render_policy(
    tr909: &Tr909LaneState,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
) -> Tr909RenderPolicyProjection {
    derive_tr909_render_policy_with_scene_context(
        tr909,
        transport,
        source_graph,
        None,
        &SceneState::default(),
    )
}

#[must_use]
pub fn derive_tr909_render_policy_with_scene_context(
    tr909: &Tr909LaneState,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
    scene_context: Option<&SceneId>,
    scene_state: &SceneState,
) -> Tr909RenderPolicyProjection {
    let fill_is_active = tr909.last_fill_bar == Some(transport.bar_index);
    let mode = if tr909.takeover_enabled {
        Tr909RenderModePolicy::Takeover
    } else if fill_is_active {
        Tr909RenderModePolicy::Fill
    } else {
        match tr909.reinforcement_mode {
            // Legacy sessions may contain the old persistent `Fills` state. New
            // fill-next actions retain the underlying typed mode and use
            // `last_fill_bar` as their one-bar performance window.
            Some(Tr909ReinforcementModeState::Fills) => Tr909RenderModePolicy::Fill,
            Some(Tr909ReinforcementModeState::BreakReinforce) => {
                Tr909RenderModePolicy::BreakReinforce
            }
            Some(Tr909ReinforcementModeState::Takeover) => Tr909RenderModePolicy::Takeover,
            Some(Tr909ReinforcementModeState::SourceSupport) => {
                Tr909RenderModePolicy::SourceSupport
            }
            None if tr909.pattern_ref.is_some() || tr909.slam_enabled => {
                Tr909RenderModePolicy::SourceSupport
            }
            None => Tr909RenderModePolicy::Idle,
        }
    };

    let routing = match mode {
        Tr909RenderModePolicy::Idle => Tr909RenderRoutingPolicy::SourceOnly,
        Tr909RenderModePolicy::SourceSupport
        | Tr909RenderModePolicy::Fill
        | Tr909RenderModePolicy::BreakReinforce => Tr909RenderRoutingPolicy::DrumBusSupport,
        Tr909RenderModePolicy::Takeover => Tr909RenderRoutingPolicy::DrumBusTakeover,
    };

    let source_support = matches!(mode, Tr909RenderModePolicy::SourceSupport)
        .then(|| derive_tr909_source_support(source_graph, transport, scene_context, scene_state))
        .flatten();
    let source_support_profile = source_support.map(|support| support.profile);
    let source_support_context = source_support.map(|support| support.context);
    let takeover_profile = derive_tr909_takeover_render_profile(tr909);
    let pattern_adoption = derive_tr909_pattern_adoption(
        mode,
        tr909.pattern_ref.as_deref(),
        source_support_profile,
        takeover_profile,
    );
    let phrase_variation = derive_tr909_phrase_variation(
        mode,
        transport,
        tr909.pattern_ref.as_deref(),
        source_support_profile,
        takeover_profile,
    );

    Tr909RenderPolicyProjection {
        mode,
        routing,
        source_support_profile,
        source_support_context,
        takeover_profile,
        pattern_adoption,
        phrase_variation,
    }
}

fn derive_tr909_takeover_render_profile(
    tr909: &Tr909LaneState,
) -> Option<Tr909TakeoverRenderProfilePolicy> {
    if !tr909.takeover_enabled {
        return None;
    }

    match tr909.takeover_profile {
        Some(Tr909TakeoverProfileState::ControlledPhraseTakeover) => {
            Some(Tr909TakeoverRenderProfilePolicy::ControlledPhrase)
        }
        Some(Tr909TakeoverProfileState::SceneLockTakeover) | None => {
            Some(Tr909TakeoverRenderProfilePolicy::SceneLock)
        }
    }
}
