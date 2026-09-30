use super::model::{
    Tr909SourceSupportContextPolicy, Tr909SourceSupportProfilePolicy,
    Tr909SourceSupportReasonPolicy,
};
use crate::{
    ids::SceneId,
    session::SceneState,
    source_graph::{
        EnergyClass, Section, SectionLabelHint, SourceGraph, section_for_projected_scene,
        section_for_transport_bar,
    },
    transport::TransportClockState,
};

#[must_use]
pub fn derive_tr909_source_support_reason(
    source_graph: Option<&SourceGraph>,
    transport: &TransportClockState,
    scene_context: Option<&SceneId>,
    scene_state: &SceneState,
) -> Option<Tr909SourceSupportReasonPolicy> {
    let graph = source_graph?;
    let (current_section, _) =
        tr909_source_support_section(graph, transport, scene_context, scene_state)?;
    let profile = source_support_profile_for_section(current_section);
    should_lift_feral_break_support(graph, profile)
        .then_some(Tr909SourceSupportReasonPolicy::FeralBreakLift)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Tr909SourceSupportPolicy {
    pub(super) profile: Tr909SourceSupportProfilePolicy,
    pub(super) context: Tr909SourceSupportContextPolicy,
}

pub(super) fn derive_tr909_source_support(
    source_graph: Option<&SourceGraph>,
    transport: &TransportClockState,
    scene_context: Option<&SceneId>,
    scene_state: &SceneState,
) -> Option<Tr909SourceSupportPolicy> {
    let graph = source_graph?;
    let (current_section, context) =
        tr909_source_support_section(graph, transport, scene_context, scene_state)?;

    Some(Tr909SourceSupportPolicy {
        profile: source_support_profile_for_graph_section(graph, current_section),
        context,
    })
}

fn tr909_source_support_section<'a>(
    graph: &'a SourceGraph,
    transport: &TransportClockState,
    scene_context: Option<&SceneId>,
    scene_state: &SceneState,
) -> Option<(&'a Section, Tr909SourceSupportContextPolicy)> {
    scene_context
        .and_then(|scene_id| {
            section_for_projected_scene(graph, scene_state, scene_id)
                .map(|section| (section, Tr909SourceSupportContextPolicy::SceneTarget))
        })
        .or_else(|| {
            section_for_transport_bar(graph, transport)
                .map(|section| (section, Tr909SourceSupportContextPolicy::TransportBar))
        })
}

fn source_support_profile_for_graph_section(
    graph: &SourceGraph,
    section: &Section,
) -> Tr909SourceSupportProfilePolicy {
    let profile = source_support_profile_for_section(section);
    if should_lift_feral_break_support(graph, profile) {
        Tr909SourceSupportProfilePolicy::BreakLift
    } else {
        profile
    }
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

fn should_lift_feral_break_support(
    graph: &SourceGraph,
    profile: Tr909SourceSupportProfilePolicy,
) -> bool {
    if profile != Tr909SourceSupportProfilePolicy::SteadyPulse {
        return false;
    }

    graph.has_feral_break_support_evidence()
}
