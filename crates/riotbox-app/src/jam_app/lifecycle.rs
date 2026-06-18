use std::collections::BTreeMap;

use riotbox_audio::runtime::{
    AudioRuntimeHealth, SourceMonitorRenderState, source_monitor_route_for_cache,
    source_monitor_route_for_output,
};
use riotbox_core::{
    queue::ActionQueue,
    session::SessionFile,
    source_graph::{SourceGraph, section_for_projected_scene},
    transport::CommitBoundaryState,
    view::jam::{JamViewModel, source_timing_consumer_readiness},
};

use super::{
    helpers::max_action_id,
    projection::{
        build_mc202_render_state, build_tr909_render_state, build_w30_preview_render_state,
        build_w30_resample_tap_state, normalize_w30_preview_mode,
    },
    runtime_view::JamRuntimeView,
    state::{AppRuntimeState, JamAppState, SidecarState, SourceAudioStatus},
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
        state.reconstruct_mc202_source_phrase_plan_for_cursor(
            state.session.action_log.actions.len(),
        );
        state.refresh_view();
        state
    }

    pub fn refresh_view(&mut self) {
        if let Some(cache) = self.source_audio_cache.as_ref() {
            self.runtime.source_audio.status = SourceAudioStatus::loaded(cache);
        }

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
        self.runtime.source_monitor_audio_route = match self
            .runtime
            .audio
            .as_ref()
            .and_then(|health| health.output.as_ref())
        {
            Some(output) => source_monitor_route_for_output(
                self.session.runtime_state.source_monitor.mode,
                self.source_audio_cache.as_ref(),
                output.sample_rate,
                output.channel_count,
            ),
            None => source_monitor_route_for_cache(
                self.session.runtime_state.source_monitor.mode,
                self.source_audio_cache.as_ref(),
            ),
        }
        .label()
        .into();
        self.jam_view = JamViewModel::build(&self.session, &self.queue, self.source_graph.as_ref());
        self.runtime_view =
            JamRuntimeView::build(&self.runtime, &self.session, self.source_graph.as_ref());
    }

    pub fn set_audio_health(&mut self, health: AudioRuntimeHealth) {
        self.runtime.audio = Some(health);
        self.refresh_view();
    }

    pub fn set_sidecar_state(&mut self, state: SidecarState) {
        self.runtime.sidecar = state;
        self.runtime_view =
            JamRuntimeView::build(&self.runtime, &self.session, self.source_graph.as_ref());
    }

    #[must_use]
    pub fn source_monitor_render_state(&self) -> SourceMonitorRenderState {
        self.source_monitor_render_state_for_cache(self.source_audio_cache.as_ref())
    }

    #[must_use]
    pub fn source_monitor_control_state(&self) -> SourceMonitorRenderState {
        self.source_monitor_render_state_for_cache(None)
    }

    fn source_monitor_render_state_for_cache(
        &self,
        cache: Option<&riotbox_audio::source_audio::SourceAudioCache>,
    ) -> SourceMonitorRenderState {
        let mut render = SourceMonitorRenderState::from_source_cache(
            self.session.runtime_state.source_monitor.mode,
            cache,
        );
        render.is_transport_running = self.runtime.transport.is_playing;
        render.tempo_bpm = self
            .source_graph
            .as_ref()
            .and_then(|graph| graph.timing.bpm_estimate)
            .unwrap_or(self.runtime.tr909_render.tempo_bpm);
        render.position_beats = self.runtime.transport.position_beats;
        if let Some(anchor) = source_monitor_scene_anchor(&self.session, self.source_graph.as_ref())
        {
            render.source_anchor_seconds = Some(anchor.source_start_seconds);
            render.source_anchor_position_beats = anchor.transport_position_beats;
        }
        render
    }
}

struct SourceMonitorSceneAnchor {
    source_start_seconds: f64,
    transport_position_beats: f64,
}

fn source_monitor_scene_anchor(
    session: &SessionFile,
    source_graph: Option<&SourceGraph>,
) -> Option<SourceMonitorSceneAnchor> {
    let graph = source_graph?;
    if !source_timing_consumer_readiness(Some(graph), session).can_use_source_window_grid() {
        return None;
    }

    let movement = session.runtime_state.scene_state.last_movement.as_ref()?;
    let active_scene = session
        .runtime_state
        .scene_state
        .active_scene
        .as_ref()
        .or(session.runtime_state.transport.current_scene.as_ref())?;
    if movement.to_scene != *active_scene {
        return None;
    }

    let section = section_for_projected_scene(graph, &movement.to_scene)?;
    let commit_record = session
        .action_log
        .commit_records
        .iter()
        .find(|record| record.action_id == movement.action_id)?;

    Some(SourceMonitorSceneAnchor {
        source_start_seconds: f64::from(section.start_seconds),
        transport_position_beats: commit_record.boundary.beat_index as f64,
    })
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
