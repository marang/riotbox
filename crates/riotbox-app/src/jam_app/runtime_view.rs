use super::{
    AppRuntimeState, SidecarState, runtime_replay_warnings::derive_replay_readiness_labels,
};
use diagnostics::derive_runtime_warnings;
use riotbox_audio::runtime::AudioRuntimeLifecycle;
use riotbox_core::{session::SessionFile, source_graph::SourceGraph};
use summaries::{
    mc202_render_transport_summary, tr909_render_alignment_label, tr909_render_profile_label,
    tr909_render_support_accent_label, tr909_render_support_reason_label,
    tr909_render_transport_summary, w30_preview_profile_label, w30_preview_target_summary,
    w30_preview_transport_summary, w30_preview_trigger_summary, w30_resample_tap_profile_label,
    w30_resample_tap_source_summary,
};

mod diagnostics;
mod summaries;

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
    pub tr909_render_support_reason: String,
    pub tr909_render_pattern_ref: Option<String>,
    pub tr909_render_pattern_adoption: String,
    pub tr909_render_phrase_variation: String,
    pub tr909_render_mix_summary: String,
    pub tr909_render_alignment: String,
    pub tr909_render_transport_summary: String,
    pub mc202_render_mode: String,
    pub mc202_render_routing: String,
    pub mc202_render_phrase_shape: String,
    pub mc202_render_mix_summary: String,
    pub mc202_render_transport_summary: String,
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
    pub replay_restore_status: String,
    pub replay_restore_anchor: String,
    pub replay_restore_payload: String,
    pub replay_restore_suffix: String,
    pub replay_restore_unsupported: String,
    pub runtime_warnings: Vec<String>,
}

impl JamRuntimeView {
    #[must_use]
    pub fn build(
        runtime: &AppRuntimeState,
        session: &SessionFile,
        source_graph: Option<&SourceGraph>,
    ) -> Self {
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

        let runtime_warnings = derive_runtime_warnings(runtime, session);
        let replay_readiness = derive_replay_readiness_labels(session);

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
            tr909_render_support_reason: tr909_render_support_reason_label(
                &runtime.tr909_render,
                &runtime.transport,
                session,
                source_graph,
            ),
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
            mc202_render_mode: runtime.mc202_render.mode.label().into(),
            mc202_render_routing: runtime.mc202_render.routing.label().into(),
            mc202_render_phrase_shape: runtime.mc202_render.phrase_shape.label().into(),
            mc202_render_mix_summary: format!(
                "music bus {:.2} | touch {:.2} | budget {} | contour {} | hook {}",
                runtime.mc202_render.music_bus_level,
                runtime.mc202_render.touch,
                runtime.mc202_render.note_budget.label(),
                runtime.mc202_render.contour_hint.label(),
                runtime.mc202_render.hook_response.label()
            ),
            mc202_render_transport_summary: mc202_render_transport_summary(&runtime.mc202_render),
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
            replay_restore_status: replay_readiness.status,
            replay_restore_anchor: replay_readiness.anchor,
            replay_restore_payload: replay_readiness.payload,
            replay_restore_suffix: replay_readiness.suffix,
            replay_restore_unsupported: replay_readiness.unsupported,
            runtime_warnings,
        }
    }
}
