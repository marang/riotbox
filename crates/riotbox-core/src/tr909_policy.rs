use crate::{
    ids::SceneId,
    session::{Tr909LaneState, Tr909ReinforcementModeState, Tr909TakeoverProfileState},
    source_graph::{
        EnergyClass, Section, SectionLabelHint, SourceGraph, section_for_projected_scene,
        section_for_transport_bar,
    },
    transport::TransportClockState,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909RenderModePolicy {
    Idle,
    SourceSupport,
    Fill,
    BreakReinforce,
    Takeover,
}

impl Tr909RenderModePolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::SourceSupport => "source_support",
            Self::Fill => "fill",
            Self::BreakReinforce => "break_reinforce",
            Self::Takeover => "takeover",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909RenderRoutingPolicy {
    SourceOnly,
    DrumBusSupport,
    DrumBusTakeover,
}

impl Tr909RenderRoutingPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SourceOnly => "source_only",
            Self::DrumBusSupport => "drum_bus_support",
            Self::DrumBusTakeover => "drum_bus_takeover",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909SourceSupportProfilePolicy {
    SteadyPulse,
    BreakLift,
    DropDrive,
}

impl Tr909SourceSupportProfilePolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SteadyPulse => "steady_pulse",
            Self::BreakLift => "break_lift",
            Self::DropDrive => "drop_drive",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909SourceSupportContextPolicy {
    SceneTarget,
    TransportBar,
}

impl Tr909SourceSupportContextPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SceneTarget => "scene_target",
            Self::TransportBar => "transport_bar",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909TakeoverRenderProfilePolicy {
    ControlledPhrase,
    SceneLock,
}

impl Tr909TakeoverRenderProfilePolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ControlledPhrase => "controlled_phrase",
            Self::SceneLock => "scene_lock",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909PatternAdoptionPolicy {
    SupportPulse,
    MainlineDrive,
    TakeoverGrid,
}

impl Tr909PatternAdoptionPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SupportPulse => "support_pulse",
            Self::MainlineDrive => "mainline_drive",
            Self::TakeoverGrid => "takeover_grid",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909PhraseVariationPolicy {
    PhraseAnchor,
    PhraseLift,
    PhraseDrive,
    PhraseRelease,
}

impl Tr909PhraseVariationPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PhraseAnchor => "phrase_anchor",
            Self::PhraseLift => "phrase_lift",
            Self::PhraseDrive => "phrase_drive",
            Self::PhraseRelease => "phrase_release",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Tr909RenderPolicyProjection {
    pub mode: Tr909RenderModePolicy,
    pub routing: Tr909RenderRoutingPolicy,
    pub source_support_profile: Option<Tr909SourceSupportProfilePolicy>,
    pub source_support_context: Option<Tr909SourceSupportContextPolicy>,
    pub takeover_profile: Option<Tr909TakeoverRenderProfilePolicy>,
    pub pattern_adoption: Option<Tr909PatternAdoptionPolicy>,
    pub phrase_variation: Option<Tr909PhraseVariationPolicy>,
}

#[must_use]
pub fn derive_tr909_render_policy(
    tr909: &Tr909LaneState,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
) -> Tr909RenderPolicyProjection {
    derive_tr909_render_policy_with_scene_context(tr909, transport, source_graph, None)
}

#[must_use]
pub fn derive_tr909_render_policy_with_scene_context(
    tr909: &Tr909LaneState,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
    scene_context: Option<&SceneId>,
) -> Tr909RenderPolicyProjection {
    let mode = if tr909.takeover_enabled {
        Tr909RenderModePolicy::Takeover
    } else {
        match tr909.reinforcement_mode {
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
        .then(|| derive_tr909_source_support(source_graph, transport, scene_context))
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Tr909SourceSupportPolicy {
    profile: Tr909SourceSupportProfilePolicy,
    context: Tr909SourceSupportContextPolicy,
}

fn derive_tr909_source_support(
    source_graph: Option<&SourceGraph>,
    transport: &TransportClockState,
    scene_context: Option<&SceneId>,
) -> Option<Tr909SourceSupportPolicy> {
    let graph = source_graph?;
    let (current_section, context) = scene_context
        .and_then(|scene_id| {
            section_for_projected_scene(graph, scene_id)
                .map(|section| (section, Tr909SourceSupportContextPolicy::SceneTarget))
        })
        .or_else(|| {
            section_for_transport_bar(graph, transport)
                .map(|section| (section, Tr909SourceSupportContextPolicy::TransportBar))
        })?;

    Some(Tr909SourceSupportPolicy {
        profile: source_support_profile_for_section(current_section),
        context,
    })
}

fn source_support_profile_for_section(section: &Section) -> Tr909SourceSupportProfilePolicy {
    match (section.label_hint, section.energy_class) {
        (SectionLabelHint::Break | SectionLabelHint::Build, _) => {
            Tr909SourceSupportProfilePolicy::BreakLift
        }
        (
            SectionLabelHint::Drop | SectionLabelHint::Chorus,
            EnergyClass::High | EnergyClass::Peak,
        ) => Tr909SourceSupportProfilePolicy::DropDrive,
        _ => Tr909SourceSupportProfilePolicy::SteadyPulse,
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

fn derive_tr909_pattern_adoption(
    mode: Tr909RenderModePolicy,
    pattern_ref: Option<&str>,
    source_support_profile: Option<Tr909SourceSupportProfilePolicy>,
    takeover_profile: Option<Tr909TakeoverRenderProfilePolicy>,
) -> Option<Tr909PatternAdoptionPolicy> {
    if matches!(mode, Tr909RenderModePolicy::Idle) {
        return None;
    }

    if matches!(mode, Tr909RenderModePolicy::Takeover)
        || matches!(
            takeover_profile,
            Some(Tr909TakeoverRenderProfilePolicy::ControlledPhrase)
        )
    {
        return Some(Tr909PatternAdoptionPolicy::TakeoverGrid);
    }

    let pattern_ref = pattern_ref.map(str::to_ascii_lowercase);
    if pattern_ref
        .as_deref()
        .is_some_and(|pattern| pattern.contains("takeover"))
    {
        return Some(Tr909PatternAdoptionPolicy::TakeoverGrid);
    }

    if pattern_ref
        .as_deref()
        .is_some_and(|pattern| pattern.contains("main") || pattern.contains("drop"))
        || matches!(
            source_support_profile,
            Some(Tr909SourceSupportProfilePolicy::DropDrive)
        )
        || matches!(
            mode,
            Tr909RenderModePolicy::Fill | Tr909RenderModePolicy::BreakReinforce
        )
    {
        return Some(Tr909PatternAdoptionPolicy::MainlineDrive);
    }

    Some(Tr909PatternAdoptionPolicy::SupportPulse)
}

fn derive_tr909_phrase_variation(
    mode: Tr909RenderModePolicy,
    transport: &TransportClockState,
    pattern_ref: Option<&str>,
    source_support_profile: Option<Tr909SourceSupportProfilePolicy>,
    takeover_profile: Option<Tr909TakeoverRenderProfilePolicy>,
) -> Option<Tr909PhraseVariationPolicy> {
    if matches!(mode, Tr909RenderModePolicy::Idle) {
        return None;
    }

    let pattern_ref = pattern_ref.map(str::to_ascii_lowercase);
    if pattern_ref
        .as_deref()
        .is_some_and(|pattern| pattern.contains("release"))
    {
        return Some(Tr909PhraseVariationPolicy::PhraseRelease);
    }

    let phrase_cycle = transport.phrase_index % 4;
    let variation = match mode {
        Tr909RenderModePolicy::Takeover => match takeover_profile {
            Some(Tr909TakeoverRenderProfilePolicy::ControlledPhrase) | None => match phrase_cycle {
                0 => Tr909PhraseVariationPolicy::PhraseAnchor,
                1 => Tr909PhraseVariationPolicy::PhraseLift,
                2 => Tr909PhraseVariationPolicy::PhraseDrive,
                _ => Tr909PhraseVariationPolicy::PhraseRelease,
            },
            Some(Tr909TakeoverRenderProfilePolicy::SceneLock) => match phrase_cycle % 2 {
                0 => Tr909PhraseVariationPolicy::PhraseDrive,
                _ => Tr909PhraseVariationPolicy::PhraseAnchor,
            },
        },
        Tr909RenderModePolicy::Fill | Tr909RenderModePolicy::BreakReinforce => {
            match phrase_cycle % 2 {
                0 => Tr909PhraseVariationPolicy::PhraseDrive,
                _ => Tr909PhraseVariationPolicy::PhraseLift,
            }
        }
        Tr909RenderModePolicy::SourceSupport => match source_support_profile {
            Some(Tr909SourceSupportProfilePolicy::SteadyPulse) | None => match phrase_cycle % 2 {
                0 => Tr909PhraseVariationPolicy::PhraseAnchor,
                _ => Tr909PhraseVariationPolicy::PhraseLift,
            },
            Some(Tr909SourceSupportProfilePolicy::BreakLift) => match phrase_cycle % 2 {
                0 => Tr909PhraseVariationPolicy::PhraseLift,
                _ => Tr909PhraseVariationPolicy::PhraseDrive,
            },
            Some(Tr909SourceSupportProfilePolicy::DropDrive) => match phrase_cycle % 2 {
                0 => Tr909PhraseVariationPolicy::PhraseDrive,
                _ => Tr909PhraseVariationPolicy::PhraseLift,
            },
        },
        Tr909RenderModePolicy::Idle => Tr909PhraseVariationPolicy::PhraseAnchor,
    };

    Some(variation)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::{
        ids::{SceneId, SectionId, SourceId},
        session::{Tr909LaneState, Tr909ReinforcementModeState, Tr909TakeoverProfileState},
        source_graph::{DecodeProfile, GraphProvenance, Section, SourceDescriptor, SourceGraph},
        transport::TransportClockState,
    };

    use super::*;

    #[derive(Debug, Deserialize)]
    struct RenderProjectionFixture {
        name: String,
        transport_position_beats: f64,
        #[serde(default)]
        scene_context: Option<String>,
        reinforcement_mode: Tr909ReinforcementModeState,
        takeover_enabled: bool,
        takeover_profile: Option<Tr909TakeoverProfileState>,
        pattern_ref: Option<String>,
        expected_mode: String,
        expected_routing: String,
        expected_pattern_adoption: Option<String>,
        expected_phrase_variation: Option<String>,
        expected_source_support_profile: Option<String>,
        expected_source_support_context: Option<String>,
        expected_takeover_profile: Option<String>,
    }

    fn sample_graph() -> SourceGraph {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "audio/test.wav".into(),
                content_hash: "graph-1".into(),
                duration_seconds: 64.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into(), "section".into()],
                generated_at: "2026-04-18T00:00:00Z".into(),
                source_hash: "graph-1".into(),
                analysis_seed: 7,
                run_notes: Some("tr909-policy-fixture".into()),
            },
        );
        graph.sections.push(Section {
            section_id: SectionId::from("section-drop"),
            label_hint: crate::source_graph::SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: crate::source_graph::EnergyClass::High,
            confidence: 0.9,
            tags: vec!["drop".into()],
        });
        graph.sections.push(Section {
            section_id: SectionId::from("section-break"),
            label_hint: crate::source_graph::SectionLabelHint::Break,
            start_seconds: 16.0,
            end_seconds: 32.0,
            bar_start: 9,
            bar_end: 16,
            energy_class: crate::source_graph::EnergyClass::Medium,
            confidence: 0.85,
            tags: vec!["break".into()],
        });
        graph
    }

    fn transport_state(position_beats: f64) -> TransportClockState {
        let beat_index = position_beats.floor() as u64;
        let bar_index = beat_index / 4;
        let phrase_index = bar_index / 8;
        TransportClockState {
            is_playing: true,
            position_beats,
            beat_index,
            bar_index,
            phrase_index,
            current_scene: Some(SceneId::from("scene-1")),
        }
    }

    #[test]
    fn fixture_backed_render_policy_projection_holds() {
        let fixtures: Vec<RenderProjectionFixture> = serde_json::from_str(include_str!(
            "../../riotbox-app/tests/fixtures/tr909_committed_render_projection.json"
        ))
        .expect("parse committed render projection fixture");

        let graph = sample_graph();
        for fixture in fixtures {
            let transport = transport_state(fixture.transport_position_beats);
            let scene_context = fixture.scene_context.as_deref().map(SceneId::from);
            let policy = derive_tr909_render_policy_with_scene_context(
                &Tr909LaneState {
                    pattern_ref: fixture.pattern_ref.clone(),
                    takeover_enabled: fixture.takeover_enabled,
                    takeover_profile: fixture.takeover_profile,
                    slam_enabled: false,
                    fill_armed_next_bar: false,
                    last_fill_bar: None,
                    reinforcement_mode: Some(fixture.reinforcement_mode),
                },
                &transport,
                Some(&graph),
                scene_context.as_ref(),
            );

            assert_eq!(
                policy.mode.label(),
                fixture.expected_mode,
                "{} mode",
                fixture.name
            );
            assert_eq!(
                policy.routing.label(),
                fixture.expected_routing,
                "{} routing",
                fixture.name
            );
            assert_eq!(
                policy
                    .pattern_adoption
                    .map(|value| value.label().to_string()),
                fixture.expected_pattern_adoption,
                "{} pattern adoption",
                fixture.name
            );
            assert_eq!(
                policy
                    .phrase_variation
                    .map(|value| value.label().to_string()),
                fixture.expected_phrase_variation,
                "{} phrase variation",
                fixture.name
            );
            assert_eq!(
                policy
                    .source_support_profile
                    .map(|value| value.label().to_string()),
                fixture.expected_source_support_profile,
                "{} support profile",
                fixture.name
            );
            assert_eq!(
                policy
                    .source_support_context
                    .map(|value| value.label().to_string()),
                fixture.expected_source_support_context,
                "{} support context",
                fixture.name
            );
            assert_eq!(
                policy
                    .takeover_profile
                    .map(|value| value.label().to_string()),
                fixture.expected_takeover_profile,
                "{} takeover profile",
                fixture.name
            );
        }
    }

    #[test]
    fn source_support_profile_can_follow_projected_scene_context() {
        let graph = sample_graph();
        let transport = transport_state(4.0);
        let policy = derive_tr909_render_policy_with_scene_context(
            &Tr909LaneState {
                pattern_ref: Some("support-scene-02-break".into()),
                takeover_enabled: false,
                takeover_profile: None,
                slam_enabled: false,
                fill_armed_next_bar: false,
                last_fill_bar: None,
                reinforcement_mode: Some(Tr909ReinforcementModeState::SourceSupport),
            },
            &transport,
            Some(&graph),
            Some(&SceneId::from("scene-02-break")),
        );

        assert_eq!(transport.bar_index, 1);
        assert_eq!(
            policy.source_support_profile,
            Some(Tr909SourceSupportProfilePolicy::BreakLift)
        );
        assert_eq!(
            policy.source_support_context,
            Some(Tr909SourceSupportContextPolicy::SceneTarget)
        );
        assert_eq!(
            policy.pattern_adoption,
            Some(Tr909PatternAdoptionPolicy::SupportPulse)
        );
    }

    #[test]
    fn source_support_profile_falls_back_to_transport_for_unmapped_scene_context() {
        let graph = sample_graph();
        let transport = transport_state(4.0);
        let policy = derive_tr909_render_policy_with_scene_context(
            &Tr909LaneState {
                pattern_ref: Some("support-legacy-scene".into()),
                takeover_enabled: false,
                takeover_profile: None,
                slam_enabled: false,
                fill_armed_next_bar: false,
                last_fill_bar: None,
                reinforcement_mode: Some(Tr909ReinforcementModeState::SourceSupport),
            },
            &transport,
            Some(&graph),
            Some(&SceneId::from("scene-1")),
        );

        assert_eq!(
            policy.source_support_profile,
            Some(Tr909SourceSupportProfilePolicy::DropDrive)
        );
        assert_eq!(
            policy.source_support_context,
            Some(Tr909SourceSupportContextPolicy::TransportBar)
        );
    }
}
