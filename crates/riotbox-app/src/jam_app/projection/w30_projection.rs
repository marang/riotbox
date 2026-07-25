fn build_w30_capture_artifact_playback(
    capture: &riotbox_core::session::CaptureRef,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
    transform: W30PadPlaybackTransform,
) -> Option<W30PadPlaybackSampleWindow> {
    let cache = capture_audio_cache?.get(&capture.capture_id)?;
    pad_playback_from_interleaved(
        cache.interleaved_samples(),
        usize::from(cache.channel_count),
        cache.sample_rate,
        0,
        cache.frame_count().try_into().unwrap_or(u64::MAX),
        transform,
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

fn build_w30_capture_artifact_resample_source(
    capture: &riotbox_core::session::CaptureRef,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
) -> Option<W30ResampleSourceProjection> {
    let cache = capture_audio_cache?.get(&capture.capture_id)?;
    project_resample_source_from_interleaved(
        cache.interleaved_samples(),
        usize::from(cache.channel_count),
        cache.sample_rate,
    )
}

struct W30ResampleSourceProjection {
    audio: Box<W30ResampleSourceWindow>,
    hard_policy: W30ResampleTapHardPolicy,
    hard_trigger_mask: u8,
    hard_slice_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    hard_transient_contrast: f32,
}

pub(super) fn resample_source_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_sample_rate: u32,
) -> Option<Box<W30ResampleSourceWindow>> {
    project_resample_source_from_interleaved(samples, channel_count, source_sample_rate)
        .map(|projection| projection.audio)
}

fn project_resample_source_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_sample_rate: u32,
) -> Option<W30ResampleSourceProjection> {
    if channel_count == 0
        || source_sample_rate == 0
        || samples.is_empty()
        || !samples.len().is_multiple_of(channel_count)
        || samples.iter().any(|sample| !sample.is_finite())
    {
        return None;
    }
    let frame_count = samples.len() / channel_count;

    let sample_count = frame_count.min(W30_RESAMPLE_SOURCE_WINDOW_LEN);
    let mut resample = [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN];
    for (index, slot) in resample.iter_mut().take(sample_count).enumerate() {
        let frame_index = if sample_count <= 1 {
            0
        } else {
            index * (frame_count - 1) / (sample_count - 1)
        };
        let base = frame_index * channel_count;
        *slot = samples[base..base + channel_count].iter().sum::<f32>() / channel_count as f32;
    }
    let (hard_policy, hard_trigger_mask, hard_slice_cursors, hard_transient_contrast) =
        analyze_w30_resample_hard_policy(samples, channel_count, source_sample_rate);
    Some(W30ResampleSourceProjection {
        audio: Box::new(W30ResampleSourceWindow {
            source_start_frame: 0,
            source_sample_rate,
            source_frame_count: frame_count.try_into().unwrap_or(u64::MAX),
            sample_count,
            samples: resample,
        }),
        hard_policy,
        hard_trigger_mask,
        hard_slice_cursors,
        hard_transient_contrast,
    })
}

pub(super) fn analyze_w30_resample_hard_policy(
    samples: &[f32],
    channel_count: usize,
    source_sample_rate: u32,
) -> (
    W30ResampleTapHardPolicy,
    u8,
    [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    f32,
) {
    const ENVELOPE_WINDOW_MILLISECONDS: usize = 20;
    const TRANSIENT_CHOP_MIN_RISE_TO_MEAN: f32 = 0.9;
    const MEDIUM_TRANSIENT_RISE_TO_MEAN: f32 = 1.4;
    const STRONG_TRANSIENT_RISE_TO_MEAN: f32 = 2.2;
    const TRIGGER_SLOT_COUNT: usize = 8;
    const LOW_TRANSIENT_TRIGGER_COUNT: usize = 4;
    const MEDIUM_TRANSIENT_TRIGGER_COUNT: usize = 5;
    const STRONG_TRANSIENT_TRIGGER_COUNT: usize = 6;

    let frame_count = samples.len() / channel_count;
    let window_frames =
        ((source_sample_rate as usize * ENVELOPE_WINDOW_MILLISECONDS) / 1_000).max(1);
    let mut envelope = Vec::with_capacity(frame_count.div_ceil(window_frames));
    for start_frame in (0..frame_count).step_by(window_frames) {
        let end_frame = (start_frame + window_frames).min(frame_count);
        let mut square_sum = 0.0_f32;
        for frame_index in start_frame..end_frame {
            let base = frame_index * channel_count;
            let mono =
                samples[base..base + channel_count].iter().sum::<f32>() / channel_count as f32;
            square_sum += mono * mono;
        }
        envelope.push((square_sum / (end_frame - start_frame).max(1) as f32).sqrt());
    }
    let mean_envelope = envelope.iter().sum::<f32>() / envelope.len().max(1) as f32;
    if mean_envelope <= 1.0e-6 {
        return (
            W30ResampleTapHardPolicy::Unavailable,
            0,
            [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            0.0,
        );
    }
    let hard_slice_cursors = std::array::from_fn(|slot_index| {
        select_resample_slot_onset_cursor(
            samples,
            channel_count,
            frame_count,
            sample_count_for_resample_proxy(frame_count),
            source_sample_rate,
            slot_index,
        )
    });
    let strongest_rise = envelope
        .windows(2)
        .map(|window| (window[1] - window[0]).max(0.0))
        .fold(0.0_f32, f32::max);
    let transient_contrast = strongest_rise / mean_envelope;
    if transient_contrast < TRANSIENT_CHOP_MIN_RISE_TO_MEAN {
        return (
            W30ResampleTapHardPolicy::SourceTextureBite,
            0,
            hard_slice_cursors,
            transient_contrast,
        );
    }

    let trigger_count = if transient_contrast >= STRONG_TRANSIENT_RISE_TO_MEAN {
        STRONG_TRANSIENT_TRIGGER_COUNT
    } else if transient_contrast >= MEDIUM_TRANSIENT_RISE_TO_MEAN {
        MEDIUM_TRANSIENT_TRIGGER_COUNT
    } else {
        LOW_TRANSIENT_TRIGGER_COUNT
    };
    let mut slot_scores = [0.0_f32; TRIGGER_SLOT_COUNT];
    for (slot_index, slot_score) in slot_scores.iter_mut().enumerate() {
        let start = slot_index * envelope.len() / TRIGGER_SLOT_COUNT;
        let end = ((slot_index + 1) * envelope.len() / TRIGGER_SLOT_COUNT).max(start + 1);
        let end = end.min(envelope.len());
        let slot = &envelope[start.min(envelope.len())..end];
        let local_peak = slot.iter().copied().fold(0.0_f32, f32::max);
        let local_rise = (start..end)
            .map(|index| {
                let before = index
                    .checked_sub(1)
                    .and_then(|previous| envelope.get(previous))
                    .copied()
                    .unwrap_or(0.0);
                (envelope[index] - before).max(0.0)
            })
            .fold(0.0_f32, f32::max);
        *slot_score = local_rise + local_peak * 0.15;
    }
    let mut ranked_slots = std::array::from_fn::<_, TRIGGER_SLOT_COUNT, _>(|index| index);
    ranked_slots.sort_by(|left, right| {
        slot_scores[*right]
            .total_cmp(&slot_scores[*left])
            .then_with(|| left.cmp(right))
    });
    let mut trigger_mask = 1_u8;
    for slot_index in ranked_slots {
        if trigger_mask.count_ones() as usize >= trigger_count {
            break;
        }
        trigger_mask |= 1_u8 << slot_index;
    }
    (
        W30ResampleTapHardPolicy::SourceTransientChop,
        trigger_mask,
        hard_slice_cursors,
        transient_contrast,
    )
}

fn sample_count_for_resample_proxy(frame_count: usize) -> usize {
    frame_count.min(W30_RESAMPLE_SOURCE_WINDOW_LEN)
}

fn select_resample_slot_onset_cursor(
    samples: &[f32],
    channel_count: usize,
    frame_count: usize,
    proxy_sample_count: usize,
    source_sample_rate: u32,
    slot_index: usize,
) -> u16 {
    const ONSET_WINDOW_MILLISECONDS: usize = 1;
    const ONSET_ANALYSIS_HOPS_PER_WINDOW: usize = 4;

    if frame_count <= 1 || proxy_sample_count <= 1 {
        return 0;
    }
    let slot_start = slot_index * frame_count / W30_RESAMPLE_HARD_SLICE_COUNT;
    let slot_end = ((slot_index + 1) * frame_count / W30_RESAMPLE_HARD_SLICE_COUNT)
        .max(slot_start + 1)
        .min(frame_count);
    let window_frames =
        ((source_sample_rate as usize * ONSET_WINDOW_MILLISECONDS) / 1_000).max(1);
    let hop_frames = (window_frames / ONSET_ANALYSIS_HOPS_PER_WINDOW).max(1);
    let first_onset = slot_start.max(window_frames);
    let last_onset = slot_end.saturating_sub(window_frames);
    if first_onset > last_onset {
        return ((slot_start * (proxy_sample_count - 1)) / (frame_count - 1))
            .try_into()
            .unwrap_or(u16::MAX);
    }

    let mut best_onset = slot_start;
    let mut best_score = f32::NEG_INFINITY;
    for onset in (first_onset..=last_onset).step_by(hop_frames) {
        let mut before = 0.0_f32;
        let mut after = 0.0_f32;
        for frame_index in onset - window_frames..onset {
            let base = frame_index * channel_count;
            let mono =
                samples[base..base + channel_count].iter().sum::<f32>() / channel_count as f32;
            before += mono.abs();
        }
        for frame_index in onset..onset + window_frames {
            let base = frame_index * channel_count;
            let mono =
                samples[base..base + channel_count].iter().sum::<f32>() / channel_count as f32;
            after += mono.abs();
        }
        before /= window_frames as f32;
        after /= window_frames as f32;
        let score = (after - before).max(0.0) + after * 0.05;
        if score > best_score {
            best_score = score;
            best_onset = onset;
        }
    }
    ((best_onset * (proxy_sample_count - 1)) / (frame_count - 1))
        .try_into()
        .unwrap_or(u16::MAX)
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
    source_sample_rate: u32,
    source_start_frame: u64,
    source_end_frame: u64,
    transform: W30PadPlaybackTransform,
) -> Option<W30PadPlaybackSampleWindow> {
    let channel_count = channel_count.max(1);
    let frame_count = samples.len() / channel_count;
    if frame_count == 0 {
        return None;
    }

    let sample_count = frame_count.min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
    let mut playback = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    for (index, slot) in playback.iter_mut().take(sample_count).enumerate() {
        let frame_index = if sample_count <= 1 {
            0
        } else {
            index * (frame_count - 1) / (sample_count - 1)
        };
        let base = frame_index * channel_count;
        let sum: f32 = samples[base..base + channel_count].iter().sum();
        *slot = sum / channel_count as f32;
    }
    let chop_slice_starts = derive_transient_chop_plan(&playback[..sample_count]);

    Some(W30PadPlaybackSampleWindow {
        source_start_frame,
        source_end_frame,
        source_sample_rate,
        playback_frame_count: frame_count.try_into().unwrap_or(u64::MAX),
        sample_count,
        loop_enabled: true,
        playback_rate: transform.playback_rate,
        reverse: transform.reverse,
        gate_step_fraction: transform.gate_step_fraction,
        loop_crossfade_sample_count: sample_count.min(128).min(sample_count / 4),
        chop_slice_count: W30_PAD_CHOP_SLICE_COUNT,
        chop_slice_starts,
        samples: playback,
    })
}

fn derive_transient_chop_plan(samples: &[f32]) -> [u32; W30_PAD_CHOP_SLICE_COUNT] {
    if samples.len() < W30_PAD_CHOP_SLICE_COUNT * 2 {
        return [0; W30_PAD_CHOP_SLICE_COUNT];
    }

    let bin_len = samples.len() / W30_PAD_CHOP_SLICE_COUNT;
    let mut candidates = [(0_usize, 0.0_f32); W30_PAD_CHOP_SLICE_COUNT];
    const ONSET_WINDOW: usize = 64;
    const ONSET_HOP: usize = 32;
    for (bin, candidate) in candidates.iter_mut().enumerate() {
        let start = bin * bin_len;
        let end = if bin + 1 == W30_PAD_CHOP_SLICE_COUNT {
            samples.len()
        } else {
            (bin + 1) * bin_len
        };
        let search_start = (start + ONSET_WINDOW).min(end);
        let search_end = end.saturating_sub(ONSET_WINDOW);
        let mut strongest = (start, 0.0_f32);
        for index in (search_start..search_end).step_by(ONSET_HOP) {
            let before = samples[index - ONSET_WINDOW..index]
                .iter()
                .map(|sample| sample.abs())
                .sum::<f32>()
                / ONSET_WINDOW as f32;
            let after = samples[index..index + ONSET_WINDOW]
                .iter()
                .map(|sample| sample.abs())
                .sum::<f32>()
                / ONSET_WINDOW as f32;
            let onset = (after - before).max(0.0) + after * 0.05;
            if onset > strongest.1 {
                strongest = (index, onset);
            }
        }
        *candidate = strongest;
    }
    candidates.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });

    // A compact sampler riff: the strongest source onset becomes the anchor while
    // three other source-derived attacks supply call/response variation.
    let ranked = [
        candidates[0].0,
        candidates[1].0,
        candidates[2].0,
        candidates[3].0,
    ];
    let order = [0, 1, 0, 2, 0, 3, 1, 2];
    std::array::from_fn(|index| ranked[order[index]].try_into().unwrap_or(u32::MAX))
}

#[cfg(test)]
mod transient_chop_plan_tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn chop_plan_repeats_the_strongest_source_onset_as_a_riff_anchor() {
        let mut samples = vec![0.0; 8_192];
        for (index, amplitude) in [(700, 0.4), (1_500, 0.7), (3_000, 1.0), (6_700, 0.6)] {
            samples[index] = amplitude;
        }

        let plan = derive_transient_chop_plan(&samples);

        assert_eq!(plan[0], plan[2]);
        assert_eq!(plan[0], plan[4]);
        assert!(plan[0].abs_diff(3_000) <= 64);
        assert!(plan.iter().copied().collect::<BTreeSet<_>>().len() >= 4);
    }

    #[test]
    fn chop_plan_changes_when_source_transients_move() {
        let mut first = vec![0.0; 8_192];
        let mut second = vec![0.0; 8_192];
        for index in [400, 1_600, 3_200, 5_900] {
            first[index] = 0.8;
        }
        for index in [800, 2_300, 4_700, 7_200] {
            second[index] = 0.8;
        }

        assert_ne!(
            derive_transient_chop_plan(&first),
            derive_transient_chop_plan(&second)
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct W30PadPlaybackTransform {
    playback_rate: f32,
    reverse: bool,
    gate_step_fraction: f32,
}

const W30_DAMAGE_PITCH_DRAG_DEPTH: f32 = 0.27;
const W30_DAMAGE_PITCH_DRAG_MIN_RATE: f32 = 0.72;
/// Retain only the source-derived attack portion of each grid retrigger. This keeps
/// percussion already present in a sparse source from drifting between the TR-909 hits.
const W30_DAMAGE_TRANSIENT_BITE_GATE_STEP_FRACTION: f32 = 0.44;

impl Default for W30PadPlaybackTransform {
    fn default() -> Self {
        Self {
            playback_rate: 1.0,
            reverse: false,
            gate_step_fraction: 0.0,
        }
    }
}

fn w30_pad_playback_transform(
    session: &SessionFile,
    destructive_intent: Option<LivePerformanceDestructiveIntent>,
) -> W30PadPlaybackTransform {
    let Some(intensity) = last_committed_w30_damage_action(session).and_then(|action| {
        if let ActionParams::Mutation { intensity, .. } = action.params {
            Some(intensity.clamp(0.0, 1.0))
        } else {
            None
        }
    }) else {
        return W30PadPlaybackTransform::default();
    };

    let (playback_rate, gate_step_fraction) = match destructive_intent {
        Some(LivePerformanceDestructiveIntent::TransientBite) => {
            (1.0, W30_DAMAGE_TRANSIENT_BITE_GATE_STEP_FRACTION * intensity)
        }
        Some(LivePerformanceDestructiveIntent::PitchDrag) | None => {
            (
                (1.0 - intensity * W30_DAMAGE_PITCH_DRAG_DEPTH)
                    .clamp(W30_DAMAGE_PITCH_DRAG_MIN_RATE, 1.0),
                0.0,
            )
        }
    };
    W30PadPlaybackTransform {
        playback_rate,
        reverse: false,
        gate_step_fraction,
    }
}

pub(super) fn build_w30_resample_tap_state(
    session: &SessionFile,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
) -> W30ResampleTapState {
    let w30 = &session.runtime_state.lane_state.w30;
    let focused_capture = w30.last_capture.as_ref().and_then(|capture_id| {
        session
            .captures
            .iter()
            .find(|capture| capture.capture_id == *capture_id)
    });
    let Some(capture) = focused_capture
        .filter(|capture| capture.capture_type == riotbox_core::session::CaptureType::Resample)
        .or_else(|| {
            let focused_capture_id = w30.last_capture.as_ref()?;
            session.captures.iter().rev().find(|capture| {
                capture.capture_type == riotbox_core::session::CaptureType::Resample
                    && !capture.lineage_capture_refs.is_empty()
                    && capture.resample_generation_depth > 0
                    && capture
                        .lineage_capture_refs
                        .contains(focused_capture_id)
            })
        })
    else {
        return W30ResampleTapState::default();
    };
    if capture.capture_type != riotbox_core::session::CaptureType::Resample
        || capture.lineage_capture_refs.is_empty()
        || capture.resample_generation_depth == 0
    {
        return W30ResampleTapState::default();
    }

    let source_profile = if capture.is_pinned {
        Some(W30ResampleTapSourceProfile::PinnedCapture)
    } else if capture.assigned_target.is_some() {
        Some(W30ResampleTapSourceProfile::PromotedCapture)
    } else {
        Some(W30ResampleTapSourceProfile::RawCapture)
    };
    let source_projection =
        build_w30_capture_artifact_resample_source(capture, capture_audio_cache);
    let (
        source_audio,
        hard_policy,
        hard_trigger_mask,
        hard_slice_cursors,
        hard_transient_contrast,
    ) =
        match source_projection {
            Some(projection) => (
                Some(projection.audio),
                projection.hard_policy,
                projection.hard_trigger_mask,
                projection.hard_slice_cursors,
                projection.hard_transient_contrast,
            ),
            None => (
                None,
                W30ResampleTapHardPolicy::Unavailable,
                0,
                [0; W30_RESAMPLE_HARD_SLICE_COUNT],
                0.0,
            ),
        };
    let (variation, variation_revision, variation_intensity) =
        w30_resample_tap_variation(session, capture);
    let (availability, routing) = if source_audio.is_some() {
        (
            W30ResampleTapAvailability::SourceAudioReady,
            W30ResampleTapRouting::InternalCaptureTap,
        )
    } else {
        (
            W30ResampleTapAvailability::SourceAudioUnavailable,
            W30ResampleTapRouting::Silent,
        )
    };

    W30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing,
        availability,
        source_profile,
        source_capture_id: Some(capture.capture_id.to_string()),
        source_audio,
        lineage_capture_count: capture
            .lineage_capture_refs
            .len()
            .try_into()
            .unwrap_or(u8::MAX),
        generation_depth: capture.resample_generation_depth,
        variation,
        variation_revision,
        variation_intensity,
        hard_policy,
        hard_trigger_mask,
        hard_slice_cursors,
        hard_transient_contrast,
        music_bus_level: session
            .runtime_state
            .mixer_state
            .music_level
            .clamp(0.0, 1.0),
        grit_level: session.runtime_state.macro_state.w30_grit.clamp(0.0, 1.0),
        is_transport_running: transport.is_playing,
        tempo_bpm: trusted_source_timing_bpm(session, source_graph).unwrap_or(0.0),
        position_beats: transport.position_beats,
    }
}

fn w30_resample_tap_variation(
    session: &SessionFile,
    capture: &riotbox_core::session::CaptureRef,
) -> (W30ResampleTapVariation, u64, f32) {
    let Some(created_from_action) = capture.created_from_action else {
        return (W30ResampleTapVariation::Base, 0, 0.0);
    };
    let Some(created_index) = session
        .action_log
        .actions
        .iter()
        .position(|action| action.id == created_from_action)
    else {
        return (W30ResampleTapVariation::Base, 0, 0.0);
    };

    let targets_capture_lineage = |target_id: &str| {
        capture.capture_id.as_str() == target_id
            || capture
                .lineage_capture_refs
                .iter()
                .any(|capture_id| capture_id.as_str() == target_id)
    };
    session
        .action_log
        .actions
        .iter()
        .enumerate()
        .skip(created_index + 1)
        .rev()
        .find_map(|(index, action)| {
            if action.status != ActionStatus::Committed
                || action.command != ActionCommand::W30ApplyDamageProfile
            {
                return None;
            }
            let ActionParams::Mutation {
                intensity,
                target_id: Some(target_id),
            } = &action.params
            else {
                return None;
            };
            targets_capture_lineage(target_id).then(|| {
                (
                    W30ResampleTapVariation::HardDamage,
                    (index + 1).try_into().unwrap_or(u64::MAX),
                    intensity.clamp(0.0, 1.0),
                )
            })
        })
        .unwrap_or((W30ResampleTapVariation::Base, 0, 0.0))
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
            && matches!(
                action.command,
                ActionCommand::W30TriggerPad
                    | ActionCommand::W30AuditionRawCapture
                    | ActionCommand::W30AuditionPromoted
            )
    })
}

fn last_committed_w30_damage_action(session: &SessionFile) -> Option<&Action> {
    session.action_log.actions.iter().rev().find(|action| {
        action.status == ActionStatus::Committed
            && matches!(action.command, ActionCommand::W30ApplyDamageProfile)
    })
}
