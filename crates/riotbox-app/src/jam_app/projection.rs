use std::collections::BTreeMap;

use riotbox_audio::{
    mc202::{
        Mc202ContourHint, Mc202HookResponse, Mc202NoteBudget, Mc202PhraseShape, Mc202RenderMode,
        Mc202RenderRouting, Mc202RenderState,
    },
    source_audio::{SourceAudioCache, SourceAudioWindow},
    tr909::{
        Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
        Tr909RenderState, Tr909SourceSupportContext, Tr909SourceSupportProfile,
        Tr909TakeoverRenderProfile,
    },
    w30::{
        W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30_PREVIEW_SAMPLE_WINDOW_LEN,
        W30PadPlaybackSampleWindow, W30PreviewRenderMode, W30PreviewRenderRouting,
        W30PreviewRenderState, W30PreviewSampleWindow, W30PreviewSourceProfile, W30ResampleTapMode,
        W30ResampleTapRouting, W30ResampleTapSourceProfile, W30ResampleTapState,
    },
};
use riotbox_core::{
    action::{Action, ActionCommand, ActionParams, ActionStatus},
    ids::{CaptureId, SceneId},
    session::{
        Mc202PhraseVariantState, SceneMovementDirectionState, SceneMovementLaneIntentState,
        SceneMovementState, SessionFile, W30PreviewModeState,
    },
    source_graph::{
        EnergyClass, Section, SectionLabelHint, SourceGraph, section_for_projected_scene,
        section_for_transport_bar,
    },
    tr909_policy::{
        Tr909PatternAdoptionPolicy, Tr909PhraseVariationPolicy, Tr909RenderModePolicy,
        Tr909RenderRoutingPolicy, Tr909SourceSupportContextPolicy, Tr909SourceSupportProfilePolicy,
        Tr909TakeoverRenderProfilePolicy, derive_tr909_render_policy_with_scene_context,
    },
    transport::TransportClockState,
};

fn audio_tr909_render_mode(mode: Tr909RenderModePolicy) -> Tr909RenderMode {
    match mode {
        Tr909RenderModePolicy::Idle => Tr909RenderMode::Idle,
        Tr909RenderModePolicy::SourceSupport => Tr909RenderMode::SourceSupport,
        Tr909RenderModePolicy::Fill => Tr909RenderMode::Fill,
        Tr909RenderModePolicy::BreakReinforce => Tr909RenderMode::BreakReinforce,
        Tr909RenderModePolicy::Takeover => Tr909RenderMode::Takeover,
    }
}

fn audio_tr909_render_routing(routing: Tr909RenderRoutingPolicy) -> Tr909RenderRouting {
    match routing {
        Tr909RenderRoutingPolicy::SourceOnly => Tr909RenderRouting::SourceOnly,
        Tr909RenderRoutingPolicy::DrumBusSupport => Tr909RenderRouting::DrumBusSupport,
        Tr909RenderRoutingPolicy::DrumBusTakeover => Tr909RenderRouting::DrumBusTakeover,
    }
}

fn audio_tr909_source_support_profile(
    profile: Option<Tr909SourceSupportProfilePolicy>,
) -> Option<Tr909SourceSupportProfile> {
    profile.map(|profile| match profile {
        Tr909SourceSupportProfilePolicy::SteadyPulse => Tr909SourceSupportProfile::SteadyPulse,
        Tr909SourceSupportProfilePolicy::BreakLift => Tr909SourceSupportProfile::BreakLift,
        Tr909SourceSupportProfilePolicy::DropDrive => Tr909SourceSupportProfile::DropDrive,
    })
}

fn audio_tr909_source_support_context(
    context: Option<Tr909SourceSupportContextPolicy>,
) -> Option<Tr909SourceSupportContext> {
    context.map(|context| match context {
        Tr909SourceSupportContextPolicy::SceneTarget => Tr909SourceSupportContext::SceneTarget,
        Tr909SourceSupportContextPolicy::TransportBar => Tr909SourceSupportContext::TransportBar,
    })
}

fn audio_tr909_takeover_profile(
    profile: Option<Tr909TakeoverRenderProfilePolicy>,
) -> Option<Tr909TakeoverRenderProfile> {
    profile.map(|profile| match profile {
        Tr909TakeoverRenderProfilePolicy::ControlledPhrase => {
            Tr909TakeoverRenderProfile::ControlledPhrase
        }
        Tr909TakeoverRenderProfilePolicy::SceneLock => Tr909TakeoverRenderProfile::SceneLock,
    })
}

fn audio_tr909_pattern_adoption(
    adoption: Option<Tr909PatternAdoptionPolicy>,
) -> Option<Tr909PatternAdoption> {
    adoption.map(|adoption| match adoption {
        Tr909PatternAdoptionPolicy::SupportPulse => Tr909PatternAdoption::SupportPulse,
        Tr909PatternAdoptionPolicy::MainlineDrive => Tr909PatternAdoption::MainlineDrive,
        Tr909PatternAdoptionPolicy::TakeoverGrid => Tr909PatternAdoption::TakeoverGrid,
    })
}

fn audio_tr909_phrase_variation(
    variation: Option<Tr909PhraseVariationPolicy>,
) -> Option<Tr909PhraseVariation> {
    variation.map(|variation| match variation {
        Tr909PhraseVariationPolicy::PhraseAnchor => Tr909PhraseVariation::PhraseAnchor,
        Tr909PhraseVariationPolicy::PhraseLift => Tr909PhraseVariation::PhraseLift,
        Tr909PhraseVariationPolicy::PhraseDrive => Tr909PhraseVariation::PhraseDrive,
        Tr909PhraseVariationPolicy::PhraseRelease => Tr909PhraseVariation::PhraseRelease,
    })
}

pub(super) fn build_tr909_render_state(
    session: &SessionFile,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
) -> Tr909RenderState {
    let tr909 = &session.runtime_state.lane_state.tr909;
    let mixer = &session.runtime_state.mixer_state;
    let tempo_bpm = source_graph
        .and_then(|graph| graph.timing.bpm_estimate)
        .unwrap_or(0.0);
    let scene_context = session
        .runtime_state
        .scene_state
        .active_scene
        .as_ref()
        .or(transport.current_scene.as_ref());
    let policy = derive_tr909_render_policy_with_scene_context(
        tr909,
        transport,
        source_graph,
        scene_context,
    );

    Tr909RenderState {
        mode: audio_tr909_render_mode(policy.mode),
        routing: audio_tr909_render_routing(policy.routing),
        source_support_profile: audio_tr909_source_support_profile(policy.source_support_profile),
        source_support_context: audio_tr909_source_support_context(policy.source_support_context),
        pattern_ref: tr909.pattern_ref.clone(),
        pattern_adoption: audio_tr909_pattern_adoption(policy.pattern_adoption),
        phrase_variation: scene_movement_tr909_variation(session)
            .or_else(|| audio_tr909_phrase_variation(policy.phrase_variation)),
        takeover_profile: audio_tr909_takeover_profile(policy.takeover_profile),
        drum_bus_level: mixer.drum_level.clamp(0.0, 1.0),
        slam_intensity: scene_movement_tr909_slam(session)
            .max(session.runtime_state.macro_state.tr909_slam)
            .clamp(0.0, 1.0),
        is_transport_running: transport.is_playing,
        tempo_bpm,
        position_beats: transport.position_beats,
        current_scene_id: transport.current_scene.as_ref().map(ToString::to_string),
    }
}

fn active_scene_movement(session: &SessionFile) -> Option<&SceneMovementState> {
    let movement = session.runtime_state.scene_state.last_movement.as_ref()?;
    let active_scene = session
        .runtime_state
        .scene_state
        .active_scene
        .as_ref()
        .or(session.runtime_state.transport.current_scene.as_ref())?;
    (movement.to_scene == *active_scene).then_some(movement)
}

fn scene_movement_tr909_variation(session: &SessionFile) -> Option<Tr909PhraseVariation> {
    let movement = active_scene_movement(session)?;
    Some(match movement.tr909_intent {
        SceneMovementLaneIntentState::Drive => Tr909PhraseVariation::PhraseDrive,
        SceneMovementLaneIntentState::Lift => Tr909PhraseVariation::PhraseLift,
        SceneMovementLaneIntentState::Release => Tr909PhraseVariation::PhraseRelease,
        SceneMovementLaneIntentState::Anchor => Tr909PhraseVariation::PhraseAnchor,
    })
}

fn scene_movement_tr909_slam(session: &SessionFile) -> f32 {
    active_scene_movement(session).map_or(0.0, |movement| {
        let floor = match movement.direction {
            SceneMovementDirectionState::Rise => 0.36,
            SceneMovementDirectionState::Drop => 0.18,
            SceneMovementDirectionState::Hold => 0.08,
        };
        movement.intensity * floor
    })
}

pub(super) fn build_mc202_render_state(
    session: &SessionFile,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
) -> Mc202RenderState {
    let mc202 = &session.runtime_state.lane_state.mc202;
    let Some(role) = mc202.role.as_deref() else {
        return Mc202RenderState::default();
    };

    let (mode, phrase_shape) = match role {
        "leader" => (Mc202RenderMode::Leader, Mc202PhraseShape::RootPulse),
        "answer" => (Mc202RenderMode::Answer, Mc202PhraseShape::AnswerHook),
        "pressure" => (Mc202RenderMode::Pressure, Mc202PhraseShape::PressureCell),
        "instigator" => (
            Mc202RenderMode::Instigator,
            Mc202PhraseShape::InstigatorSpike,
        ),
        "follower" => (Mc202RenderMode::Follower, Mc202PhraseShape::FollowerDrive),
        _ => return Mc202RenderState::default(),
    };
    let phrase_shape = match mc202.phrase_variant {
        Some(Mc202PhraseVariantState::MutatedDrive) => Mc202PhraseShape::MutatedDrive,
        _ => phrase_shape,
    };
    let current_section = mc202_current_section(source_graph, transport, scene_context(session));
    let hook_response =
        mc202_hook_response_for_role_graph_and_section(role, source_graph, current_section);
    let tempo_bpm = source_graph
        .and_then(|graph| graph.timing.bpm_estimate)
        .unwrap_or(0.0);

    let movement = active_scene_movement(session);
    let touch = scene_movement_mc202_touch(
        session
            .runtime_state
            .macro_state
            .mc202_touch
            .clamp(0.0, 1.0),
        movement,
    );
    let contour_hint = scene_movement_mc202_contour(movement)
        .unwrap_or_else(|| mc202_contour_hint(current_section));

    Mc202RenderState {
        mode,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape,
        note_budget: mc202_note_budget_for_shape_and_hook_response(phrase_shape, hook_response),
        contour_hint,
        hook_response,
        touch,
        music_bus_level: session
            .runtime_state
            .mixer_state
            .music_level
            .clamp(0.0, 1.0),
        tempo_bpm,
        position_beats: transport.position_beats,
        is_transport_running: transport.is_playing,
    }
}

fn scene_movement_mc202_contour(movement: Option<&SceneMovementState>) -> Option<Mc202ContourHint> {
    let movement = movement?;
    Some(match movement.mc202_intent {
        SceneMovementLaneIntentState::Lift => Mc202ContourHint::Lift,
        SceneMovementLaneIntentState::Drive => Mc202ContourHint::Drop,
        SceneMovementLaneIntentState::Release => Mc202ContourHint::Hold,
        SceneMovementLaneIntentState::Anchor => Mc202ContourHint::Hold,
    })
}

fn scene_movement_mc202_touch(base_touch: f32, movement: Option<&SceneMovementState>) -> f32 {
    let Some(movement) = movement else {
        return base_touch;
    };

    match movement.mc202_intent {
        SceneMovementLaneIntentState::Lift => base_touch.max(0.74 + movement.intensity * 0.18),
        SceneMovementLaneIntentState::Drive => base_touch.max(0.70 + movement.intensity * 0.14),
        SceneMovementLaneIntentState::Release => base_touch.min(0.62),
        SceneMovementLaneIntentState::Anchor => base_touch.clamp(0.48, 0.72),
    }
}

fn mc202_current_section<'a>(
    source_graph: Option<&'a SourceGraph>,
    transport: &TransportClockState,
    scene_context: Option<&SceneId>,
) -> Option<&'a Section> {
    let graph = source_graph?;
    scene_context
        .and_then(|scene_id| section_for_projected_scene(graph, scene_id))
        .or_else(|| section_for_transport_bar(graph, transport))
}

fn mc202_contour_hint(section: Option<&Section>) -> Mc202ContourHint {
    section
        .map(mc202_contour_hint_for_section)
        .unwrap_or(Mc202ContourHint::Neutral)
}

fn scene_context(session: &SessionFile) -> Option<&SceneId> {
    session
        .runtime_state
        .transport
        .current_scene
        .as_ref()
        .or(session.runtime_state.scene_state.active_scene.as_ref())
}

fn mc202_contour_hint_for_section(section: &Section) -> Mc202ContourHint {
    match (section.label_hint, section.energy_class) {
        (SectionLabelHint::Build, _) => Mc202ContourHint::Lift,
        (SectionLabelHint::Drop, EnergyClass::High | EnergyClass::Peak)
        | (SectionLabelHint::Chorus, EnergyClass::High | EnergyClass::Peak) => {
            Mc202ContourHint::Drop
        }
        (SectionLabelHint::Break | SectionLabelHint::Intro | SectionLabelHint::Outro, _) => {
            Mc202ContourHint::Hold
        }
        (_, EnergyClass::Low) => Mc202ContourHint::Hold,
        _ => Mc202ContourHint::Neutral,
    }
}

fn mc202_hook_response_for_role_graph_and_section(
    role: &str,
    source_graph: Option<&SourceGraph>,
    section: Option<&Section>,
) -> Mc202HookResponse {
    if !matches!(role, "follower" | "leader") {
        return Mc202HookResponse::Direct;
    }

    let is_hook_like = section.is_some_and(|section| {
        matches!(section.label_hint, SectionLabelHint::Chorus)
            || section
                .tags
                .iter()
                .any(|tag| matches!(tag.as_str(), "hook" | "chorus"))
    });

    if is_hook_like || source_graph.is_some_and(SourceGraph::has_feral_break_support_evidence) {
        Mc202HookResponse::AnswerSpace
    } else {
        Mc202HookResponse::Direct
    }
}

fn mc202_note_budget_for_shape_and_hook_response(
    shape: Mc202PhraseShape,
    hook_response: Mc202HookResponse,
) -> Mc202NoteBudget {
    if hook_response == Mc202HookResponse::AnswerSpace {
        return Mc202NoteBudget::Sparse;
    }

    match shape {
        Mc202PhraseShape::PressureCell => Mc202NoteBudget::Sparse,
        Mc202PhraseShape::InstigatorSpike => Mc202NoteBudget::Push,
        Mc202PhraseShape::MutatedDrive => Mc202NoteBudget::Wide,
        Mc202PhraseShape::RootPulse
        | Mc202PhraseShape::FollowerDrive
        | Mc202PhraseShape::AnswerHook => Mc202NoteBudget::Balanced,
    }
}

pub(super) fn build_w30_preview_render_state(
    session: &SessionFile,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
    source_audio_cache: Option<&SourceAudioCache>,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
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
    let tempo_bpm = source_graph
        .and_then(|graph| graph.timing.bpm_estimate)
        .unwrap_or(0.0);
    let source_window_preview = if !matches!(mode, W30PreviewRenderMode::Idle) {
        capture.and_then(|capture| {
            build_w30_capture_artifact_preview(capture, capture_audio_cache).or_else(|| {
                build_w30_source_window_preview(capture, source_graph, source_audio_cache)
            })
        })
    } else {
        None
    };
    let pad_playback = if !matches!(mode, W30PreviewRenderMode::Idle) {
        capture
            .and_then(|capture| build_w30_capture_artifact_playback(capture, capture_audio_cache))
    } else {
        None
    };

    W30PreviewRenderState {
        mode,
        routing: W30PreviewRenderRouting::MusicBusPreview,
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
        music_bus_level: session
            .runtime_state
            .mixer_state
            .music_level
            .clamp(0.0, 1.0),
        grit_level: session.runtime_state.macro_state.w30_grit.clamp(0.0, 1.0),
        is_transport_running: transport.is_playing,
        tempo_bpm,
        position_beats: transport.position_beats,
    }
}

fn build_w30_capture_artifact_playback(
    capture: &riotbox_core::session::CaptureRef,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
) -> Option<W30PadPlaybackSampleWindow> {
    let cache = capture_audio_cache?.get(&capture.capture_id)?;
    pad_playback_from_interleaved(
        cache.interleaved_samples(),
        usize::from(cache.channel_count),
        0,
        cache.frame_count().try_into().unwrap_or(u64::MAX),
    )
}

fn build_w30_capture_artifact_preview(
    capture: &riotbox_core::session::CaptureRef,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
) -> Option<W30PreviewSampleWindow> {
    let cache = capture_audio_cache?.get(&capture.capture_id)?;
    source_preview_from_interleaved(
        cache.interleaved_samples(),
        usize::from(cache.channel_count),
        0,
        cache.frame_count().try_into().unwrap_or(u64::MAX),
    )
}

fn build_w30_source_window_preview(
    capture: &riotbox_core::session::CaptureRef,
    source_graph: Option<&SourceGraph>,
    source_audio_cache: Option<&SourceAudioCache>,
) -> Option<W30PreviewSampleWindow> {
    let source_window = capture.source_window.as_ref()?;
    let graph = source_graph?;
    if source_window.source_id != graph.source.source_id {
        return None;
    }

    let cache = source_audio_cache?;
    let start_frame = usize::try_from(source_window.start_frame).unwrap_or(usize::MAX);
    let end_frame = usize::try_from(source_window.end_frame).unwrap_or(usize::MAX);
    let frame_count = end_frame.saturating_sub(start_frame);
    let window = SourceAudioWindow {
        start_frame,
        frame_count,
    };
    let samples = cache.window_samples(window);
    source_preview_from_interleaved(
        samples,
        usize::from(cache.channel_count),
        source_window.start_frame,
        source_window.end_frame,
    )
}

fn source_preview_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_start_frame: u64,
    source_end_frame: u64,
) -> Option<W30PreviewSampleWindow> {
    let channel_count = channel_count.max(1);
    let frame_count = samples.len() / channel_count;
    if frame_count == 0 {
        return None;
    }

    let sample_count = frame_count.min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    let stride = (frame_count / sample_count).max(1);
    let mut preview = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];

    for (index, slot) in preview.iter_mut().take(sample_count).enumerate() {
        let frame_index = (index * stride).min(frame_count - 1);
        let base = frame_index * channel_count;
        let sum: f32 = samples[base..base + channel_count].iter().sum();
        *slot = sum / channel_count as f32;
    }

    Some(W30PreviewSampleWindow {
        source_start_frame,
        source_end_frame,
        sample_count,
        samples: preview,
    })
}

fn pad_playback_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_start_frame: u64,
    source_end_frame: u64,
) -> Option<W30PadPlaybackSampleWindow> {
    let channel_count = channel_count.max(1);
    let frame_count = samples.len() / channel_count;
    if frame_count == 0 {
        return None;
    }

    let sample_count = frame_count.min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
    let mut playback = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    for (index, slot) in playback.iter_mut().take(sample_count).enumerate() {
        let base = index * channel_count;
        let sum: f32 = samples[base..base + channel_count].iter().sum();
        *slot = sum / channel_count as f32;
    }

    Some(W30PadPlaybackSampleWindow {
        source_start_frame,
        source_end_frame,
        sample_count,
        loop_enabled: true,
        samples: playback,
    })
}

pub(super) fn build_w30_resample_tap_state(
    session: &SessionFile,
    transport: &TransportClockState,
) -> W30ResampleTapState {
    let w30 = &session.runtime_state.lane_state.w30;
    let Some(capture) = w30.last_capture.as_ref().and_then(|capture_id| {
        session
            .captures
            .iter()
            .find(|capture| capture.capture_id == *capture_id)
    }) else {
        return W30ResampleTapState::default();
    };

    let source_profile = if capture.is_pinned {
        Some(W30ResampleTapSourceProfile::PinnedCapture)
    } else if capture.assigned_target.is_some() {
        Some(W30ResampleTapSourceProfile::PromotedCapture)
    } else {
        Some(W30ResampleTapSourceProfile::RawCapture)
    };

    W30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile,
        source_capture_id: Some(capture.capture_id.to_string()),
        lineage_capture_count: capture
            .lineage_capture_refs
            .len()
            .try_into()
            .unwrap_or(u8::MAX),
        generation_depth: capture.resample_generation_depth,
        music_bus_level: session
            .runtime_state
            .mixer_state
            .music_level
            .clamp(0.0, 1.0),
        grit_level: session.runtime_state.macro_state.w30_grit.clamp(0.0, 1.0),
        is_transport_running: transport.is_playing,
    }
}

pub(super) fn normalize_w30_preview_mode(session: &mut SessionFile) {
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
            && matches!(action.command, ActionCommand::W30TriggerPad)
    })
}
