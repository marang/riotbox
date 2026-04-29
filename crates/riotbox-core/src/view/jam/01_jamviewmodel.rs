use crate::{
    queue::ActionQueue,
    session::{
        Mc202PhraseVariantState, SessionFile, Tr909ReinforcementModeState,
        Tr909TakeoverProfileState,
    },
    source_graph::{
        AssetType, CandidateType, EnergyClass, QualityClass, RelationshipType, Section, SourceGraph,
    },
};

#[cfg(test)]
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub struct JamViewModel {
    pub transport: JamTransportView,
    pub source: SourceSummaryView,
    pub scene: SceneSummaryView,
    pub macros: MacroStripView,
    pub lanes: LaneSummaryView,
    pub capture: CaptureSummaryView,
    pub pending_actions: Vec<PendingActionView>,
    pub recent_actions: Vec<RecentActionView>,
    pub ghost: GhostStatusView,
    pub warnings: Vec<String>,
}

