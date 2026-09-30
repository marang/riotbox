use crate::jam_app::projection::w30_material::{
    W30PadPlaybackTransform, build_w30_capture_artifact_playback,
    build_w30_capture_artifact_preview, build_w30_source_window_preview,
};
use crate::jam_app::transport_helpers::trusted_source_timing_bpm;
use riotbox_audio::source_audio::SourceAudioCache;
use riotbox_audio::w30::{
    W30HookArticulationProfile, W30HookArticulationRenderState, W30PreviewRenderMode,
    W30PreviewRenderRouting, W30PreviewRenderState, W30PreviewSourceProfile,
};
use riotbox_core::action::{Action, ActionCommand, ActionParams, ActionStatus};
use riotbox_core::ids::CaptureId;
use riotbox_core::live_performance_policy::{
    LivePerformanceDestructiveIntent, derive_live_performance_policy,
};
use riotbox_core::session::{SessionFile, W30HookArticulationProfileState, W30PreviewModeState};
use riotbox_core::source_graph::SourceGraph;
use riotbox_core::transport::TransportClockState;
use riotbox_core::w30_damage_policy::{
    W30_TRANSIENT_BITE_GATE_STEP_FRACTION, latest_committed_w30_damage_intensity,
};
use std::collections::BTreeMap;

pub(in crate::jam_app) fn build_w30_preview_render_state(
    session: &SessionFile,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
    source_audio_cache: Option<&SourceAudioCache>,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
    require_capture_artifact: bool,
) -> W30PreviewRenderState {
    let w30 = &session.runtime_state.lane_state.w30;
    let has_lane_focus =
        w30.active_bank.is_some() || w30.focused_pad.is_some() || w30.last_capture.is_some();
    if !has_lane_focus {
        return W30PreviewRenderState::default();
    }

    let mode = match w30.preview_mode.unwrap_or(W30PreviewModeState::LiveRecall) {
        W30PreviewModeState::LiveRecall => W30PreviewRenderMode::LiveRecall,
        W30PreviewModeState::RawCaptureAudition => W30PreviewRenderMode::RawCaptureAudition,
        W30PreviewModeState::PromotedAudition => W30PreviewRenderMode::PromotedAudition,
    };
    let last_trigger = last_committed_w30_trigger_action(session);
    let live_policy = source_graph.and_then(|graph| derive_live_performance_policy(session, graph));

    let capture = w30.last_capture.as_ref().and_then(|capture_id| {
        session
            .captures
            .iter()
            .find(|capture| capture.capture_id == *capture_id)
    });
    let last_preview_action =
        last_committed_w30_preview_action(session).map(|action| action.command);
    let source_profile = match mode {
        W30PreviewRenderMode::Idle => None,
        W30PreviewRenderMode::RawCaptureAudition => {
            Some(W30PreviewSourceProfile::RawCaptureAudition)
        }
        W30PreviewRenderMode::PromotedAudition => Some(W30PreviewSourceProfile::PromotedAudition),
        W30PreviewRenderMode::LiveRecall => capture.map(|capture| match last_preview_action {
            Some(ActionCommand::W30BrowseSlicePool) => W30PreviewSourceProfile::SlicePoolBrowse,
            _ if capture.is_pinned => W30PreviewSourceProfile::PinnedRecall,
            _ => W30PreviewSourceProfile::PromotedRecall,
        }),
    };
    let tempo_bpm = trusted_source_timing_bpm(session, source_graph).unwrap_or(0.0);
    let source_window_preview = if !matches!(mode, W30PreviewRenderMode::Idle) {
        capture.and_then(|capture| {
            build_w30_capture_artifact_preview(capture, capture_audio_cache).or_else(|| {
                if require_capture_artifact || capture.audio_identity.is_some() {
                    None
                } else {
                    build_w30_source_window_preview(capture, source_graph, source_audio_cache)
                }
            })
        })
    } else {
        None
    };
    let mut pad_playback = if !matches!(mode, W30PreviewRenderMode::Idle) {
        capture.and_then(|capture| {
            let transform = w30_pad_playback_transform(
                session,
                &capture.capture_id,
                live_policy.as_ref().map(|policy| policy.destructive_intent),
            );
            build_w30_capture_artifact_playback(capture, capture_audio_cache, transform)
        })
    } else {
        None
    };
    let has_preview_material = source_window_preview.is_some() || pad_playback.is_some();
    let hook_articulation = w30.hook_articulation.as_ref().and_then(|articulation| {
        let capture = capture?;
        if capture.capture_id != articulation.capture_id
            || pad_playback.is_none()
            || tempo_bpm <= 0.0
        {
            return None;
        }
        let profile = match articulation.profile {
            W30HookArticulationProfileState::TurnaroundV1 => {
                W30HookArticulationProfile::TurnaroundV1
            }
            W30HookArticulationProfileState::PitchDiveV1 => W30HookArticulationProfile::PitchDiveV1,
            W30HookArticulationProfileState::FilterSlamV1 => {
                W30HookArticulationProfile::FilterSlamV1
            }
        };
        Some(W30HookArticulationRenderState {
            profile,
            started_at_beat: articulation.started_at_beat,
        })
    });
    if let Some(pad_playback) = pad_playback.as_mut() {
        pad_playback.hook_articulation = hook_articulation;
    }
    let routing = if has_preview_material {
        W30PreviewRenderRouting::MusicBusPreview
    } else {
        W30PreviewRenderRouting::Silent
    };
    W30PreviewRenderState {
        mode,
        routing,
        source_profile,
        active_bank_id: w30.active_bank.as_ref().map(ToString::to_string),
        focused_pad_id: w30.focused_pad.as_ref().map(ToString::to_string),
        capture_id: w30.last_capture.as_ref().map(ToString::to_string),
        trigger_revision: last_trigger.map_or(0, |action| action.id.0),
        trigger_velocity: last_trigger
            .and_then(|action| match &action.params {
                ActionParams::Mutation { intensity, .. } => Some(intensity.clamp(0.0, 1.0)),
                _ => None,
            })
            .unwrap_or(0.0),
        source_window_preview,
        pad_playback,
        music_bus_level: if has_preview_material {
            live_policy
                .as_ref()
                .map_or_else(
                    || session.runtime_state.mixer_state.music_level,
                    |policy| policy.w30_music_level,
                )
                .clamp(0.0, 1.0)
        } else {
            0.0
        },
        grit_level: session.runtime_state.macro_state.w30_grit.clamp(0.0, 1.0),
        is_transport_running: transport.is_playing,
        tempo_bpm,
        position_beats: transport.position_beats,
    }
}

const W30_DAMAGE_PITCH_DRAG_DEPTH: f32 = 0.27;

const W30_DAMAGE_PITCH_DRAG_MIN_RATE: f32 = 0.72;

fn w30_pad_playback_transform(
    session: &SessionFile,
    capture_id: &CaptureId,
    destructive_intent: Option<LivePerformanceDestructiveIntent>,
) -> W30PadPlaybackTransform {
    let Some(intensity) = latest_committed_w30_damage_intensity(session, capture_id) else {
        return W30PadPlaybackTransform::default();
    };

    let (playback_rate, gate_step_fraction) = match destructive_intent {
        Some(LivePerformanceDestructiveIntent::TransientBite) => {
            (1.0, W30_TRANSIENT_BITE_GATE_STEP_FRACTION * intensity)
        }
        Some(LivePerformanceDestructiveIntent::PitchDrag) | None => (
            (1.0 - intensity * W30_DAMAGE_PITCH_DRAG_DEPTH)
                .clamp(W30_DAMAGE_PITCH_DRAG_MIN_RATE, 1.0),
            0.0,
        ),
    };
    W30PadPlaybackTransform {
        playback_rate,
        reverse: false,
        gate_step_fraction,
    }
}

pub(in crate::jam_app) fn normalize_w30_preview_mode(session: &mut SessionFile) {
    let preview_mode = last_committed_w30_preview_action(session)
        .map(|action| match action.command {
            ActionCommand::W30AuditionRawCapture => W30PreviewModeState::RawCaptureAudition,
            ActionCommand::W30AuditionPromoted => W30PreviewModeState::PromotedAudition,
            ActionCommand::W30LiveRecall
            | ActionCommand::W30SwapBank
            | ActionCommand::W30BrowseSlicePool
            | ActionCommand::W30StepFocus
            | ActionCommand::W30TriggerPad => W30PreviewModeState::LiveRecall,
            _ => unreachable!("filtered by helper"),
        })
        .unwrap_or(W30PreviewModeState::LiveRecall);

    let w30 = &mut session.runtime_state.lane_state.w30;
    let has_lane_focus =
        w30.active_bank.is_some() || w30.focused_pad.is_some() || w30.last_capture.is_some();
    if !has_lane_focus || w30.preview_mode.is_some() {
        return;
    }

    w30.preview_mode = Some(preview_mode);
}

fn last_committed_w30_preview_action(session: &SessionFile) -> Option<&Action> {
    session.action_log.actions.iter().rev().find(|action| {
        action.status == ActionStatus::Committed
            && matches!(
                action.command,
                ActionCommand::W30LiveRecall
                    | ActionCommand::W30SwapBank
                    | ActionCommand::W30BrowseSlicePool
                    | ActionCommand::W30StepFocus
                    | ActionCommand::W30AuditionRawCapture
                    | ActionCommand::W30AuditionPromoted
                    | ActionCommand::W30TriggerPad
            )
    })
}

fn last_committed_w30_trigger_action(session: &SessionFile) -> Option<&Action> {
    session.action_log.actions.iter().rev().find(|action| {
        action.status == ActionStatus::Committed
            && matches!(
                action.command,
                ActionCommand::W30TriggerPad
                    | ActionCommand::W30AuditionRawCapture
                    | ActionCommand::W30AuditionPromoted
            )
    })
}
