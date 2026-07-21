use super::super::{
    AppRuntimeState, SidecarState, SourceAudioStatus,
    runtime_replay_warnings::derive_replay_summary_warnings,
};
use riotbox_audio::{
    mc202::{Mc202RenderMode, Mc202RenderRouting, Mc202RenderState},
    runtime::{AudioRuntimeLifecycle, SourceMonitorAudioRoute},
    tr909::{Tr909RenderMode, Tr909RenderRouting, Tr909RenderState},
    w30::{
        W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState, W30ResampleTapMode,
        W30ResampleTapRouting, W30ResampleTapState,
    },
};
use riotbox_core::{action::SourceMonitorMode, session::SessionFile, style::PerformancePresetId};

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
    if let Some(health) = runtime
        .audio
        .as_ref()
        .filter(|health| health.callback_scratch_overflow_count > 0)
    {
        warnings.push(format!(
            "audio callback scratch overflow: {} buffers silenced",
            health.callback_scratch_overflow_count
        ));
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

    warnings.extend(derive_source_audio_warnings(runtime, session));
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

fn derive_source_audio_warnings(runtime: &AppRuntimeState, session: &SessionFile) -> Vec<String> {
    if matches!(
        session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Riotbox
    ) {
        return Vec::new();
    }

    match &runtime.source_audio.status {
        SourceAudioStatus::Unavailable { path, reason } => {
            vec![format!(
                "source audio unavailable for source monitor: {path} ({reason})"
            )]
        }
        SourceAudioStatus::Loaded {
            sample_rate,
            channel_count,
            frame_count,
            ..
        } if runtime.source_monitor_audio_route == SourceMonitorAudioRoute::SourceUnavailable => {
            vec![format!(
                "source monitor unavailable: source audio format is {sample_rate} Hz, {channel_count} ch, {frame_count} frames"
            )]
        }
        SourceAudioStatus::NotRequested | SourceAudioStatus::Loaded { .. } => Vec::new(),
    }
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

    let preset_owns_patternless_primitive =
        session
            .runtime_state
            .style
            .active_preset
            .is_some_and(|preset| {
                let definition = preset.definition();
                lane.reinforcement_mode == Some(definition.tr909_reinforcement_mode)
                    && matches!(
                        (preset, render.mode),
                        (
                            PerformancePresetId::FeralBreakAlphaV1,
                            Tr909RenderMode::SourceSupport | Tr909RenderMode::Fill
                        ) | (
                            PerformancePresetId::FeralBreakAlphaV2,
                            Tr909RenderMode::BreakReinforce | Tr909RenderMode::Fill
                        )
                    )
            });
    if render.pattern_ref.is_none()
        && !preset_owns_patternless_primitive
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
        let source_plan = session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .as_ref();
        if let Some(plan) = source_plan.filter(|plan| !plan.is_source_derived()) {
            let reason = plan
                .fallback_reason
                .as_deref()
                .unwrap_or("source phrase plan was not source-derived");
            warnings.push(format!(
                "MC-202 source phrase degraded and is not routed to music_bus_bass: {reason}"
            ));
        } else {
            warnings.push(
                "MC-202 source phrase unavailable; primitive fallback is not routed to music_bus_bass"
                    .into(),
            );
        }
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
    if matches!(render.routing, W30PreviewRenderRouting::Silent)
        && render.source_window_preview.is_none()
        && render.pad_playback.is_none()
    {
        warnings.push(
            "W-30 preview unavailable: no source window or artifact-backed pad material".into(),
        );
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

#[cfg(test)]
mod tests {
    use riotbox_audio::tr909::{Tr909RenderMode, Tr909RenderRouting, Tr909RenderState};
    use riotbox_core::{session::SessionFile, style::PerformancePresetId};

    use super::derive_tr909_render_warnings;

    #[test]
    fn typed_feral_v2_primitive_does_not_claim_a_missing_pattern() {
        let mut session = SessionFile::new("diagnostics", "0.1.0", "2026-07-21T00:00:00Z");
        PerformancePresetId::FeralBreakAlphaV2.apply_to_session(&mut session);
        let render = Tr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            routing: Tr909RenderRouting::DrumBusSupport,
            drum_bus_level: 0.8,
            ..Tr909RenderState::default()
        };

        assert!(
            !derive_tr909_render_warnings(&render, &session)
                .iter()
                .any(|warning| warning.contains("pattern_ref"))
        );
    }

    #[test]
    fn unowned_patternless_support_keeps_its_warning() {
        let mut session = SessionFile::new("diagnostics", "0.1.0", "2026-07-21T00:00:00Z");
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(riotbox_core::session::Tr909ReinforcementModeState::BreakReinforce);
        let render = Tr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            routing: Tr909RenderRouting::DrumBusSupport,
            drum_bus_level: 0.8,
            ..Tr909RenderState::default()
        };

        assert!(
            derive_tr909_render_warnings(&render, &session)
                .iter()
                .any(|warning| warning.contains("pattern_ref"))
        );
    }
}
