use riotbox_audio::{
    mc202::{Mc202RenderMode, Mc202RenderState},
    tr909::{Tr909RenderMode, Tr909RenderState, Tr909SourceSupportContext},
    w30::{
        W30PreviewRenderMode, W30PreviewRenderState, W30PreviewSourceProfile,
        W30ResampleTapSourceProfile, W30ResampleTapState,
    },
};
use riotbox_core::{
    session::SessionFile, source_graph::SourceGraph,
    tr909_policy::derive_tr909_source_support_reason,
};

pub(super) fn w30_preview_profile_label(render: &W30PreviewRenderState) -> &'static str {
    render
        .source_profile
        .map_or("unset", W30PreviewSourceProfile::label)
}

pub(super) fn w30_preview_target_summary(render: &W30PreviewRenderState) -> String {
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

pub(super) fn w30_preview_transport_summary(render: &W30PreviewRenderState) -> String {
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

pub(super) fn w30_preview_trigger_summary(render: &W30PreviewRenderState) -> String {
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

pub(super) fn w30_resample_tap_profile_label(render: &W30ResampleTapState) -> &'static str {
    match render.source_profile {
        None => "unset",
        Some(W30ResampleTapSourceProfile::RawCapture) => "raw_capture",
        Some(W30ResampleTapSourceProfile::PromotedCapture) => "promoted_capture",
        Some(W30ResampleTapSourceProfile::PinnedCapture) => "pinned_capture",
    }
}

pub(super) fn w30_resample_tap_source_summary(render: &W30ResampleTapState) -> String {
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

pub(super) fn tr909_render_profile_label(render: &Tr909RenderState) -> &'static str {
    match (render.takeover_profile, render.source_support_profile) {
        (Some(profile), _) => profile.label(),
        (None, Some(profile)) => profile.label(),
        (None, None) => "unset",
    }
}

pub(super) fn tr909_render_support_accent_label(render: &Tr909RenderState) -> &'static str {
    match (render.mode, render.source_support_context) {
        (Tr909RenderMode::SourceSupport, Some(Tr909SourceSupportContext::SceneTarget)) => "scene",
        (Tr909RenderMode::SourceSupport, Some(Tr909SourceSupportContext::TransportBar)) => {
            "off fallback"
        }
        (Tr909RenderMode::SourceSupport, None) => "off unset",
        _ => "off",
    }
}

pub(super) fn tr909_render_support_reason_label(
    render: &Tr909RenderState,
    transport: &riotbox_core::transport::TransportClockState,
    session: &SessionFile,
    source_graph: Option<&SourceGraph>,
) -> String {
    if !matches!(render.mode, Tr909RenderMode::SourceSupport) {
        return "unset".into();
    }
    if render.source_support_profile.is_none() || source_graph.is_none() {
        return "unset".into();
    }

    let scene_context = session
        .runtime_state
        .scene_state
        .active_scene
        .as_ref()
        .or(session.runtime_state.transport.current_scene.as_ref());

    derive_tr909_source_support_reason(source_graph, transport, scene_context)
        .map_or_else(|| "section".into(), |reason| reason.cue_label().into())
}

pub(super) fn tr909_render_alignment_label(render: &Tr909RenderState) -> &'static str {
    match render.mode {
        Tr909RenderMode::Idle => "source-only idle",
        Tr909RenderMode::SourceSupport => "support aligned",
        Tr909RenderMode::Fill => "fill aligned",
        Tr909RenderMode::BreakReinforce => "break reinforce aligned",
        Tr909RenderMode::Takeover => "takeover aligned",
    }
}

pub(super) fn tr909_render_transport_summary(render: &Tr909RenderState) -> String {
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

pub(super) fn mc202_render_transport_summary(render: &Mc202RenderState) -> String {
    if matches!(render.mode, Mc202RenderMode::Idle) {
        return "bass idle".into();
    }

    format!(
        "{} @ {:.1} beats | {:.1} BPM",
        if render.is_transport_running {
            "running"
        } else {
            "stopped"
        },
        render.position_beats,
        render.tempo_bpm
    )
}
