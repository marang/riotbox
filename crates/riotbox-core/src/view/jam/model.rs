use crate::session::SessionFile;
use crate::view::jam::capture_actions::CaptureSummaryView;
use crate::view::jam::performer_state::GhostStatusView;
use crate::view::jam::performer_state::LaneSummaryView;
use crate::view::jam::performer_state::MacroStripView;
use crate::view::jam::performer_state::PendingActionView;
use crate::view::jam::performer_state::RecentActionView;
use crate::view::jam::scene_launch::SceneSummaryView;
use crate::view::jam::source_summary::SourceSummaryView;

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

pub(super) trait SessionAccessors {
    fn transport(&self) -> &crate::session::TransportRuntimeState;
}

impl SessionAccessors for SessionFile {
    fn transport(&self) -> &crate::session::TransportRuntimeState {
        &self.runtime_state.transport
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct JamTransportView {
    pub is_playing: bool,
    pub position_beats: f64,
}
