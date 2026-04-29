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
