use std::collections::BTreeMap;

use riotbox_audio::runtime::AudioRuntimeHealth;
use riotbox_core::{
    queue::ActionQueue, session::SessionFile, source_graph::SourceGraph,
    transport::CommitBoundaryState, view::jam::JamViewModel,
};

use super::{
    helpers::max_action_id,
    projection::{
        build_mc202_render_state, build_tr909_render_state, build_w30_preview_render_state,
        build_w30_resample_tap_state, normalize_w30_preview_mode,
    },
    runtime_view::JamRuntimeView,
    state::{AppRuntimeState, JamAppState, SidecarState},
    transport_helpers::{normalize_scene_candidates, transport_clock_from_state},
};

impl JamAppState {
    #[must_use]
    pub fn from_parts(
        mut session: SessionFile,
        source_graph: Option<SourceGraph>,
        mut queue: ActionQueue,
    ) -> Self {
        normalize_w30_preview_mode(&mut session);
        normalize_scene_candidates(&mut session, source_graph.as_ref());
        queue.reserve_action_ids_after(max_action_id(&session));
        let transport = transport_clock_from_state(&session, source_graph.as_ref());
        let last_commit_boundary = latest_commit_boundary_from_log(&session);
        let jam_view = JamViewModel::build(&session, &queue, source_graph.as_ref());
        let runtime_view =
            JamRuntimeView::build(&AppRuntimeState::default(), &session, source_graph.as_ref());
        let mut state = Self {
            files: None,
            session,
            source_graph,
            source_audio_cache: None,
            capture_audio_cache: BTreeMap::new(),
            queue,
            runtime: AppRuntimeState {
                transport,
                last_commit_boundary,
                ..AppRuntimeState::default()
            },
            jam_view,
            runtime_view,
        };
        state.refresh_view();
        state
    }

    pub fn refresh_view(&mut self) {
        self.runtime.tr909_render = build_tr909_render_state(
            &self.session,
            &self.runtime.transport,
            self.source_graph.as_ref(),
        );
        self.runtime.mc202_render = build_mc202_render_state(
            &self.session,
            &self.runtime.transport,
            self.source_graph.as_ref(),
        );
        self.runtime.w30_preview = build_w30_preview_render_state(
            &self.session,
            &self.runtime.transport,
            self.source_graph.as_ref(),
            self.source_audio_cache.as_ref(),
            Some(&self.capture_audio_cache),
        );
        self.runtime.w30_resample_tap =
            build_w30_resample_tap_state(&self.session, &self.runtime.transport);
        self.jam_view = JamViewModel::build(&self.session, &self.queue, self.source_graph.as_ref());
        self.runtime_view =
            JamRuntimeView::build(&self.runtime, &self.session, self.source_graph.as_ref());
    }

    pub fn set_audio_health(&mut self, health: AudioRuntimeHealth) {
        self.runtime.audio = Some(health);
        self.runtime_view =
            JamRuntimeView::build(&self.runtime, &self.session, self.source_graph.as_ref());
    }

    pub fn set_sidecar_state(&mut self, state: SidecarState) {
        self.runtime.sidecar = state;
        self.runtime_view =
            JamRuntimeView::build(&self.runtime, &self.session, self.source_graph.as_ref());
    }
}

pub(in crate::jam_app) fn latest_commit_boundary_from_log(
    session: &SessionFile,
) -> Option<CommitBoundaryState> {
    session
        .action_log
        .commit_records
        .last()
        .map(|record| record.boundary.clone())
}
