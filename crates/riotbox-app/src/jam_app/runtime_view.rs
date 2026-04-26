use riotbox_audio::{
    runtime::AudioRuntimeLifecycle,
    tr909::{Tr909RenderMode, Tr909RenderRouting, Tr909RenderState, Tr909SourceSupportContext},
    w30::{
        W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState,
        W30PreviewSourceProfile, W30ResampleTapMode, W30ResampleTapRouting,
        W30ResampleTapSourceProfile, W30ResampleTapState,
    },
};
use riotbox_core::session::SessionFile;

use super::{AppRuntimeState, SidecarState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JamRuntimeView {
    pub audio_status: String,
    pub audio_callback_count: u64,
    pub audio_last_error: Option<String>,
    pub sidecar_status: String,
    pub sidecar_version: Option<String>,
    pub tr909_render_mode: String,
    pub tr909_render_routing: String,
    pub tr909_render_profile: String,
    pub tr909_render_support_context: String,
    pub tr909_render_support_accent: String,
    pub tr909_render_pattern_ref: Option<String>,
    pub tr909_render_pattern_adoption: String,
    pub tr909_render_phrase_variation: String,
    pub tr909_render_mix_summary: String,
    pub tr909_render_alignment: String,
    pub tr909_render_transport_summary: String,
    pub w30_preview_mode: String,
    pub w30_preview_routing: String,
    pub w30_preview_profile: String,
    pub w30_preview_target_summary: String,
    pub w30_preview_mix_summary: String,
    pub w30_preview_transport_summary: String,
    pub w30_preview_trigger_summary: String,
    pub w30_resample_tap_mode: String,
    pub w30_resample_tap_routing: String,
    pub w30_resample_tap_profile: String,
    pub w30_resample_tap_source_summary: String,
    pub w30_resample_tap_mix_summary: String,
    pub runtime_warnings: Vec<String>,
}

impl JamRuntimeView {
    #[must_use]
    pub fn build(runtime: &AppRuntimeState, session: &SessionFile) -> Self {
        let (audio_status, audio_callback_count, audio_last_error) = match &runtime.audio {
            Some(health) => (
                match health.lifecycle {
                    AudioRuntimeLifecycle::Idle => "idle".into(),
                    AudioRuntimeLifecycle::Running => "running".into(),
                    AudioRuntimeLifecycle::Stopped => "stopped".into(),
                    AudioRuntimeLifecycle::Faulted => "faulted".into(),
                },
                health.callback_count,
                health.last_stream_error.clone(),
            ),
            None => ("unknown".into(), 0, None),
        };

        let (sidecar_status, sidecar_version) = match &runtime.sidecar {
            SidecarState::Unknown => ("unknown".into(), None),
            SidecarState::Ready { version, .. } => ("ready".into(), version.clone()),
            SidecarState::Unavailable { .. } => ("unavailable".into(), None),
            SidecarState::Degraded { .. } => ("degraded".into(), None),
        };

        let mut runtime_warnings = Vec::new();
        if matches!(
            runtime.audio.as_ref().map(|health| health.lifecycle),
            Some(AudioRuntimeLifecycle::Faulted)
        ) {
            runtime_warnings.push("audio runtime faulted".into());
        }
        match &runtime.sidecar {
            SidecarState::Unavailable { reason } => {
                runtime_warnings.push(format!("sidecar unavailable: {reason}"));
            }
            SidecarState::Degraded { reason } => {
                runtime_warnings.push(format!("sidecar degraded: {reason}"));
            }
            SidecarState::Unknown | SidecarState::Ready { .. } => {}
        }

        runtime_warnings.extend(derive_tr909_render_warnings(&runtime.tr909_render, session));
        runtime_warnings.extend(derive_w30_preview_warnings(&runtime.w30_preview, session));
        runtime_warnings.extend(derive_w30_resample_tap_warnings(
            &runtime.w30_resample_tap,
            session,
        ));

        Self {
            audio_status,
            audio_callback_count,
            audio_last_error,
            sidecar_status,
            sidecar_version,
            tr909_render_mode: runtime.tr909_render.mode.label().into(),
            tr909_render_routing: runtime.tr909_render.routing.label().into(),
            tr909_render_profile: tr909_render_profile_label(&runtime.tr909_render).into(),
            tr909_render_support_context: runtime
                .tr909_render
                .source_support_context
                .map_or_else(|| "unset".into(), |context| context.label().into()),
            tr909_render_support_accent: tr909_render_support_accent_label(&runtime.tr909_render)
                .into(),
            tr909_render_pattern_ref: runtime.tr909_render.pattern_ref.clone(),
            tr909_render_pattern_adoption: runtime
                .tr909_render
                .pattern_adoption
                .map_or_else(|| "unset".into(), |pattern| pattern.label().into()),
            tr909_render_phrase_variation: runtime
                .tr909_render
                .phrase_variation
                .map_or_else(|| "unset".into(), |variation| variation.label().into()),
            tr909_render_mix_summary: format!(
                "drum bus {:.2} | slam {:.2}",
                runtime.tr909_render.drum_bus_level, runtime.tr909_render.slam_intensity
            ),
            tr909_render_alignment: tr909_render_alignment_label(&runtime.tr909_render).into(),
            tr909_render_transport_summary: tr909_render_transport_summary(&runtime.tr909_render),
            w30_preview_mode: runtime.w30_preview.mode.label().into(),
            w30_preview_routing: runtime.w30_preview.routing.label().into(),
            w30_preview_profile: w30_preview_profile_label(&runtime.w30_preview).into(),
            w30_preview_target_summary: w30_preview_target_summary(&runtime.w30_preview),
            w30_preview_mix_summary: format!(
                "music bus {:.2} | grit {:.2}",
                runtime.w30_preview.music_bus_level, runtime.w30_preview.grit_level
            ),
            w30_preview_transport_summary: w30_preview_transport_summary(&runtime.w30_preview),
            w30_preview_trigger_summary: w30_preview_trigger_summary(&runtime.w30_preview),
            w30_resample_tap_mode: runtime.w30_resample_tap.mode.label().into(),
            w30_resample_tap_routing: runtime.w30_resample_tap.routing.label().into(),
            w30_resample_tap_profile: w30_resample_tap_profile_label(&runtime.w30_resample_tap)
                .into(),
            w30_resample_tap_source_summary: w30_resample_tap_source_summary(
                &runtime.w30_resample_tap,
            ),
            w30_resample_tap_mix_summary: format!(
                "music bus {:.2} | grit {:.2}",
                runtime.w30_resample_tap.music_bus_level, runtime.w30_resample_tap.grit_level
            ),
            runtime_warnings,
        }
    }
}

fn w30_preview_profile_label(render: &W30PreviewRenderState) -> &'static str {
    render
        .source_profile
        .map_or("unset", W30PreviewSourceProfile::label)
}

fn w30_preview_target_summary(render: &W30PreviewRenderState) -> String {
    if matches!(render.mode, W30PreviewRenderMode::Idle) {
        return "target unset".into();
    }

    format!(
        "{} / {} | {}",
        render.active_bank_id.as_deref().unwrap_or("bank unset"),
        render.focused_pad_id.as_deref().unwrap_or("pad unset"),
        render.capture_id.as_deref().unwrap_or("capture unset")
    )
}

fn w30_preview_transport_summary(render: &W30PreviewRenderState) -> String {
    if matches!(render.mode, W30PreviewRenderMode::Idle) {
        return "preview idle".into();
    }

    format!(
        "{} @ {:.1} | {:.1} BPM",
        if render.is_transport_running {
            "transport running"
        } else {
            "transport stopped"
        },
        render.position_beats,
        render.tempo_bpm
    )
}

fn w30_preview_trigger_summary(render: &W30PreviewRenderState) -> String {
    if render.trigger_revision == 0 {
        if matches!(render.mode, W30PreviewRenderMode::Idle) {
            return "trigger unset".into();
        }

        return "trigger pending from committed seam".into();
    }

    format!(
        "trigger r{} @ {:.2}",
        render.trigger_revision, render.trigger_velocity
    )
}

fn w30_resample_tap_profile_label(render: &W30ResampleTapState) -> &'static str {
    match render.source_profile {
        None => "unset",
        Some(W30ResampleTapSourceProfile::RawCapture) => "raw_capture",
        Some(W30ResampleTapSourceProfile::PromotedCapture) => "promoted_capture",
        Some(W30ResampleTapSourceProfile::PinnedCapture) => "pinned_capture",
    }
}

fn w30_resample_tap_source_summary(render: &W30ResampleTapState) -> String {
    match render.source_capture_id.as_deref() {
        Some(capture_id) => format!(
            "{capture_id} | gen {} | lineage {}",
            render.generation_depth, render.lineage_capture_count
        ),
        None => format!(
            "source unset | gen {} | lineage {}",
            render.generation_depth, render.lineage_capture_count
        ),
    }
}

fn tr909_render_profile_label(render: &Tr909RenderState) -> &'static str {
    match (render.takeover_profile, render.source_support_profile) {
        (Some(profile), _) => profile.label(),
        (None, Some(profile)) => profile.label(),
        (None, None) => "unset",
    }
}

fn tr909_render_support_accent_label(render: &Tr909RenderState) -> &'static str {
    match (render.mode, render.source_support_context) {
        (Tr909RenderMode::SourceSupport, Some(Tr909SourceSupportContext::SceneTarget)) => "scene",
        (Tr909RenderMode::SourceSupport, Some(Tr909SourceSupportContext::TransportBar)) => {
            "off fallback"
        }
        (Tr909RenderMode::SourceSupport, None) => "off unset",
        _ => "off",
    }
}

fn tr909_render_alignment_label(render: &Tr909RenderState) -> &'static str {
    match render.mode {
        Tr909RenderMode::Idle => "source-only idle",
        Tr909RenderMode::SourceSupport => "support aligned",
        Tr909RenderMode::Fill => "fill aligned",
        Tr909RenderMode::BreakReinforce => "break reinforce aligned",
        Tr909RenderMode::Takeover => "takeover aligned",
    }
}

fn tr909_render_transport_summary(render: &Tr909RenderState) -> String {
    let transport = if render.is_transport_running {
        "running"
    } else {
        "stopped"
    };
    let scene = render.current_scene_id.as_deref().unwrap_or("none");
    format!(
        "{transport} @ {:.1} beats | {:.1} BPM | scene {scene}",
        render.position_beats, render.tempo_bpm
    )
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
