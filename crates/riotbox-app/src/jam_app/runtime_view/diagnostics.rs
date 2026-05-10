use super::super::{
    AppRuntimeState, SidecarState, runtime_replay_warnings::derive_replay_summary_warnings,
};
use riotbox_audio::{
    mc202::{Mc202RenderMode, Mc202RenderRouting, Mc202RenderState},
    runtime::AudioRuntimeLifecycle,
    tr909::{Tr909RenderMode, Tr909RenderRouting, Tr909RenderState},
    w30::{
        W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState, W30ResampleTapMode,
        W30ResampleTapRouting, W30ResampleTapState,
    },
};
use riotbox_core::session::SessionFile;

pub(super) fn derive_runtime_warnings(
    runtime: &AppRuntimeState,
    session: &SessionFile,
) -> Vec<String> {
    let mut warnings = Vec::new();

    if matches!(
        runtime.audio.as_ref().map(|health| health.lifecycle),
        Some(AudioRuntimeLifecycle::Faulted)
    ) {
        warnings.push("audio runtime faulted".into());
    }

    match &runtime.sidecar {
        SidecarState::Unavailable { reason } => {
            warnings.push(format!("sidecar unavailable: {reason}"));
        }
        SidecarState::Degraded { reason } => {
            warnings.push(format!("sidecar degraded: {reason}"));
        }
        SidecarState::Unknown | SidecarState::Ready { .. } => {}
    }

    warnings.extend(derive_tr909_render_warnings(&runtime.tr909_render, session));
    warnings.extend(derive_mc202_render_warnings(&runtime.mc202_render, session));
    warnings.extend(derive_w30_preview_warnings(&runtime.w30_preview, session));
    warnings.extend(derive_w30_resample_tap_warnings(
        &runtime.w30_resample_tap,
        session,
    ));
    warnings.extend(derive_replay_summary_warnings(session));
    warnings
}

fn derive_tr909_render_warnings(render: &Tr909RenderState, session: &SessionFile) -> Vec<String> {
    let mut warnings = Vec::new();
    let lane = &session.runtime_state.lane_state.tr909;

    if matches!(render.mode, Tr909RenderMode::Idle)
        && !matches!(render.routing, Tr909RenderRouting::SourceOnly)
    {
        warnings.push("909 render idle but routing is not source_only".into());
    }

    if matches!(render.mode, Tr909RenderMode::Takeover)
        && !matches!(render.routing, Tr909RenderRouting::DrumBusTakeover)
    {
        warnings.push("909 takeover render is not routed to drum_bus_takeover".into());
    }

    if !matches!(render.mode, Tr909RenderMode::Takeover) && render.takeover_profile.is_some() {
        warnings.push("909 render carries a takeover profile outside takeover mode".into());
    }

    if matches!(render.mode, Tr909RenderMode::SourceSupport)
        && render.source_support_profile.is_none()
    {
        warnings.push("909 source-support render is missing a support profile".into());
    }

    if matches!(render.mode, Tr909RenderMode::SourceSupport)
        && render.source_support_profile.is_some()
        && render.source_support_context.is_none()
    {
        warnings.push("909 source-support render is missing a support context".into());
    }

    if !matches!(render.mode, Tr909RenderMode::SourceSupport)
        && render.source_support_profile.is_some()
    {
        warnings.push("909 render carries a support profile outside source-support mode".into());
    }

    if !matches!(render.mode, Tr909RenderMode::SourceSupport)
        && render.source_support_context.is_some()
    {
        warnings.push("909 render carries a support context outside source-support mode".into());
    }

    if matches!(
        render.routing,
        Tr909RenderRouting::DrumBusSupport | Tr909RenderRouting::DrumBusTakeover
    ) && render.drum_bus_level <= 0.0
    {
        warnings.push("909 render is routed to the drum bus at zero drum level".into());
    }

    if lane.takeover_enabled && !matches!(render.mode, Tr909RenderMode::Takeover) {
        warnings.push("909 lane takeover is committed but render mode is not takeover".into());
    }

    if render.pattern_ref.is_none()
        && (lane.takeover_enabled
            || lane.reinforcement_mode.is_some()
            || lane.slam_enabled
            || render.takeover_profile.is_some())
    {
        warnings.push("909 render has no pattern_ref while musical support is active".into());
    }

    warnings
}

fn derive_mc202_render_warnings(render: &Mc202RenderState, session: &SessionFile) -> Vec<String> {
    if matches!(render.mode, Mc202RenderMode::Idle) {
        return Vec::new();
    }

    let mut warnings = Vec::new();

    if !matches!(render.routing, Mc202RenderRouting::MusicBusBass) {
        warnings.push("MC-202 render is active but not routed to music_bus_bass".into());
    }

    if render.music_bus_level <= 0.0 {
        warnings.push("MC-202 render is routed to the music bus at zero music level".into());
    }

    if session.runtime_state.lane_state.mc202.role.is_none() {
        warnings.push("MC-202 render is active without a committed role".into());
    }

    warnings
}

fn derive_w30_preview_warnings(
    render: &W30PreviewRenderState,
    session: &SessionFile,
) -> Vec<String> {
    if matches!(render.mode, W30PreviewRenderMode::Idle) {
        return Vec::new();
    }

    let mut warnings = Vec::new();

    if matches!(render.routing, W30PreviewRenderRouting::MusicBusPreview)
        && render.music_bus_level <= 0.0
    {
        warnings.push("W-30 preview is routed to the music bus at zero music level".into());
    }

    let has_capture = render.capture_id.as_ref().is_some_and(|capture_id| {
        session
            .captures
            .iter()
            .any(|capture| capture.capture_id.to_string() == *capture_id)
    });
    if !has_capture {
        warnings
            .push("W-30 preview has no committed capture backing the current lane focus".into());
    }

    warnings
}

fn derive_w30_resample_tap_warnings(
    render: &W30ResampleTapState,
    session: &SessionFile,
) -> Vec<String> {
    if matches!(render.mode, W30ResampleTapMode::Idle) {
        return Vec::new();
    }

    let mut warnings = Vec::new();

    if matches!(render.routing, W30ResampleTapRouting::InternalCaptureTap)
        && render.music_bus_level <= 0.0
    {
        warnings.push("W-30 resample tap is prepared at zero music level".into());
    }

    let has_capture = render.source_capture_id.as_ref().is_some_and(|capture_id| {
        session
            .captures
            .iter()
            .any(|capture| capture.capture_id.to_string() == *capture_id)
    });
    if !has_capture {
        warnings.push("W-30 resample tap has no committed capture backing its lineage".into());
    }

    warnings
}
