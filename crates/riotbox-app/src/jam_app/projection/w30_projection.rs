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
    grit_level: f32,
    variation_intensity: f32,
) -> Option<W30ResampleSourceProjection> {
    let cache = capture_audio_cache?.get(&capture.capture_id)?;
    project_resample_source_from_interleaved(
        cache.interleaved_samples(),
        usize::from(cache.channel_count),
        cache.sample_rate,
        grit_level,
        variation_intensity,
    )
}

struct W30ResampleSourceProjection {
    audio: Box<W30ResampleSourceWindow>,
    hard_policy: W30ResampleTapHardPolicy,
    hard_suitability: W30ResampleHardSuitabilityPlan,
    hard_calibration: W30ResampleHardCalibrationPlan,
    hard_trigger_mask: u8,
    hard_slice_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    hard_attack_lengths: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    hard_attack_bite: W30ResampleAttackBitePlan,
    hard_low_impact: W30ResampleLowImpactPlan,
    hard_gesture: W30ResampleHardGesturePlan,
    hard_transient_contrast: f32,
}

pub(super) fn resample_source_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_sample_rate: u32,
) -> Option<Box<W30ResampleSourceWindow>> {
    project_resample_source_from_interleaved(samples, channel_count, source_sample_rate, 0.0, 0.0)
        .map(|projection| projection.audio)
}

fn project_resample_source_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_sample_rate: u32,
    grit_level: f32,
    variation_intensity: f32,
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
    let hard_suitability = derive_w30_resample_hard_suitability(samples, channel_count);
    let (hard_policy, hard_trigger_mask, hard_slice_cursors, hard_transient_contrast) =
        if hard_suitability.status == W30ResampleHardSuitability::Suitable {
            analyze_w30_resample_hard_policy(samples, channel_count, source_sample_rate)
        } else {
            (
                W30ResampleTapHardPolicy::Unavailable,
                0,
                [0; W30_RESAMPLE_HARD_SLICE_COUNT],
                0.0,
            )
        };
    let hard_attack_lengths =
        if hard_suitability.status == W30ResampleHardSuitability::Suitable {
            derive_w30_resample_attack_lengths(
                samples,
                channel_count,
                source_sample_rate,
                sample_count,
                hard_slice_cursors,
            )
        } else {
            [0; W30_RESAMPLE_HARD_SLICE_COUNT]
        };
    let proxy_sample_rate =
        source_sample_rate as f32 * sample_count as f32 / frame_count.max(1) as f32;
    let hard_attack_bite = if hard_policy == W30ResampleTapHardPolicy::SourceTransientChop {
        derive_w30_resample_attack_bite(
            &resample[..sample_count],
            proxy_sample_rate,
            hard_trigger_mask,
            hard_slice_cursors,
            hard_attack_lengths,
        )
    } else {
        W30ResampleAttackBitePlan::default()
    };
    let mut hard_low_impact = if hard_policy == W30ResampleTapHardPolicy::SourceTransientChop {
        derive_w30_resample_low_impact(
            &resample[..sample_count],
            proxy_sample_rate,
            hard_trigger_mask,
            hard_slice_cursors,
            hard_attack_lengths,
        )
    } else {
        W30ResampleLowImpactPlan::default()
    };
    let hard_calibration = derive_w30_resample_hard_calibration(
        &resample[..sample_count],
        proxy_sample_rate,
        hard_policy,
        hard_trigger_mask,
        hard_slice_cursors,
        grit_level,
        variation_intensity,
        &mut hard_low_impact,
    );
    let hard_gesture = if hard_policy == W30ResampleTapHardPolicy::SourceTransientChop {
        derive_w30_resample_hard_gesture(
            &resample[..sample_count],
            proxy_sample_rate,
            hard_trigger_mask,
            hard_slice_cursors,
        )
    } else {
        W30ResampleHardGesturePlan::default()
    };
    let source_frame_count = frame_count.try_into().unwrap_or(u64::MAX);
    let source_revision = resample_source_revision(
        source_sample_rate,
        source_frame_count,
        &resample[..sample_count],
    );
    Some(W30ResampleSourceProjection {
        audio: Box::new(W30ResampleSourceWindow {
            source_revision,
            source_start_frame: 0,
            source_sample_rate,
            source_frame_count,
            sample_count,
            samples: resample,
        }),
        hard_policy,
        hard_suitability,
        hard_calibration,
        hard_trigger_mask,
        hard_slice_cursors,
        hard_attack_lengths,
        hard_attack_bite,
        hard_low_impact,
        hard_gesture,
        hard_transient_contrast,
    })
}

const W30_RESAMPLE_H13_HEAD_SECONDS: f32 = 0.02;
const W30_RESAMPLE_H13_BODY_END_SECONDS: f32 = 0.10;
const W30_RESAMPLE_H13_PICKUP_SECONDS: f32 = 0.12;
const W30_RESAMPLE_H13_BODY_TO_HEAD_TARGET: f32 = 0.95;
const W30_RESAMPLE_H13_PICKUP_MIN_SOURCE_RMS_SHARE: f32 = 0.20;
const W30_RESAMPLE_H13_MIN_WINDOW_RMS: f32 = 1.0e-5;

fn derive_w30_resample_hard_gesture(
    proxy: &[f32],
    proxy_sample_rate: f32,
    trigger_mask: u8,
    onset_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
) -> W30ResampleHardGesturePlan {
    if proxy.len() < 2
        || !proxy_sample_rate.is_finite()
        || proxy_sample_rate <= 0.0
        || trigger_mask == 0
    {
        return W30ResampleHardGesturePlan::default();
    }
    let head_frames = (proxy_sample_rate * W30_RESAMPLE_H13_HEAD_SECONDS)
        .round()
        .max(1.0) as usize;
    let body_end_frames = (proxy_sample_rate * W30_RESAMPLE_H13_BODY_END_SECONDS)
        .round()
        .max((head_frames + 1) as f32) as usize;
    let mut selected: Option<(usize, f32, f32, f32)> = None;
    for (slot, onset_cursor) in onset_cursors.iter().copied().enumerate() {
        if trigger_mask & (1_u8 << slot) == 0 {
            continue;
        }
        let onset = usize::from(onset_cursor).min(proxy.len() - 1);
        let head_end = onset.saturating_add(head_frames).min(proxy.len());
        let body_end = onset.saturating_add(body_end_frames).min(proxy.len());
        if head_end <= onset || body_end <= head_end {
            continue;
        }
        let head_rms = rms(&proxy[onset..head_end]);
        let body_rms = rms(&proxy[head_end..body_end]);
        if head_rms < W30_RESAMPLE_H13_MIN_WINDOW_RMS
            || body_rms < W30_RESAMPLE_H13_MIN_WINDOW_RMS
        {
            continue;
        }
        let impact_score = head_rms + body_rms * W30_RESAMPLE_H13_BODY_TO_HEAD_TARGET;
        if selected.is_none_or(|(_, _, _, score)| impact_score > score) {
            selected = Some((slot, head_rms, body_rms, impact_score));
        }
    }
    let Some((impact_slot, selected_head_rms, selected_body_rms, _)) = selected else {
        return W30ResampleHardGesturePlan::default();
    };
    let target_body_rms = (selected_head_rms * W30_RESAMPLE_H13_BODY_TO_HEAD_TARGET)
        .max(selected_body_rms * W30_RESAMPLE_H13_MIN_BODY_GAIN);
    let body_gain = (target_body_rms / selected_body_rms)
        .clamp(
            W30_RESAMPLE_H13_MIN_BODY_GAIN,
            W30_RESAMPLE_H13_MAX_BODY_GAIN,
        );
    let head_energy = selected_head_rms.powi(2) * W30_RESAMPLE_H13_HEAD_SECONDS;
    let body_energy = selected_body_rms.powi(2)
        * (W30_RESAMPLE_H13_BODY_END_SECONDS - W30_RESAMPLE_H13_HEAD_SECONDS);
    let impact_level_compensation = ((head_energy + body_energy)
        / (head_energy + body_energy * body_gain.powi(2)).max(f32::EPSILON))
    .sqrt()
    .clamp(W30_RESAMPLE_H13_MIN_IMPACT_LEVEL_COMPENSATION, 1.0);
    let pickup_slot =
        (impact_slot + W30_RESAMPLE_HARD_SLICE_COUNT - 1) % W30_RESAMPLE_HARD_SLICE_COUNT;
    let pickup_frames = (proxy_sample_rate * W30_RESAMPLE_H13_PICKUP_SECONDS)
        .round()
        .max(1.0) as usize;
    let impact_onset = usize::from(onset_cursors[impact_slot]).min(proxy.len() - 1);
    let impact_end = impact_onset.saturating_add(pickup_frames).min(proxy.len());
    let impact_pickup_rms = rms(&proxy[impact_onset..impact_end]);
    let pickup_slot_end = ((pickup_slot + 1) * proxy.len() / W30_RESAMPLE_HARD_SLICE_COUNT)
        .min(proxy.len());
    let pickup_slot_start = pickup_slot_end.saturating_sub(pickup_frames);
    let pickup_context_rms = rms(&proxy[pickup_slot_start..pickup_slot_end]);
    let pickup_target_rms =
        pickup_context_rms.max(rms(proxy) * W30_RESAMPLE_H13_PICKUP_MIN_SOURCE_RMS_SHARE);
    let pickup_gain = (pickup_target_rms / impact_pickup_rms.max(f32::EPSILON))
        .clamp(W30_RESAMPLE_H13_MIN_PICKUP_GAIN, 1.0);
    W30ResampleHardGesturePlan {
        recipe: W30ResampleHardGestureRecipe::SourceReverseIntoImpactV1,
        impact_slot: impact_slot as u8,
        pickup_slot: pickup_slot as u8,
        body_gain,
        impact_level_compensation,
        pickup_gain,
        selected_head_rms,
        selected_body_rms,
    }
}

const W30_RESAMPLE_H12_TEXTURE_TARGET_LEVEL_RATIO: f32 = 1.05;
const W30_RESAMPLE_H12_HIT_TARGET_LEVEL_RATIO: f32 = 1.20;
const W30_RESAMPLE_H12_MIN_PREDICTED_BODY_RATIO: f32 = 1.15;
const W30_RESAMPLE_H12_MIN_OUTPUT_GAIN: f32 = 0.25;
const W30_RESAMPLE_H12_MAX_OUTPUT_GAIN: f32 = 1.25;

#[allow(clippy::too_many_arguments)]
fn derive_w30_resample_hard_calibration(
    proxy: &[f32],
    proxy_sample_rate: f32,
    hard_policy: W30ResampleTapHardPolicy,
    trigger_mask: u8,
    onset_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    grit_level: f32,
    variation_intensity: f32,
    low_impact: &mut W30ResampleLowImpactPlan,
) -> W30ResampleHardCalibrationPlan {
    match hard_policy {
        W30ResampleTapHardPolicy::SourceTextureBite => {
            derive_w30_resample_texture_calibration(proxy, grit_level, variation_intensity)
        }
        W30ResampleTapHardPolicy::SourceTransientChop
            if low_impact.recipe == W30ResampleLowImpactRecipe::SourceHitShaperV3 =>
        {
            let plan = derive_w30_resample_hit_shaper_calibration(
                proxy,
                proxy_sample_rate,
                trigger_mask,
                onset_cursors,
            );
            if plan.predicted_level_matched_body_ratio
                < W30_RESAMPLE_H12_MIN_PREDICTED_BODY_RATIO
            {
                low_impact.recipe = W30ResampleLowImpactRecipe::Unavailable;
                W30ResampleHardCalibrationPlan {
                    output_gain: 1.0,
                    ..plan
                }
            } else {
                plan
            }
        }
        W30ResampleTapHardPolicy::Unavailable
        | W30ResampleTapHardPolicy::SourceTransientChop => {
            W30ResampleHardCalibrationPlan::default()
        }
    }
}

fn derive_w30_resample_texture_calibration(
    proxy: &[f32],
    grit_level: f32,
    variation_intensity: f32,
) -> W30ResampleHardCalibrationPlan {
    const MIN_RMS: f32 = 1.0e-6;
    if proxy.is_empty() {
        return W30ResampleHardCalibrationPlan::default();
    }
    let mut base_last = 0.0_f32;
    let mut base_memory = 0.0_f32;
    let mut hard_last = 0.0_f32;
    let mut hard_memory = 0.0_f32;
    let mut base_square_sum = 0.0_f32;
    let mut hard_square_sum = 0.0_f32;
    for sample in proxy {
        let base = w30_resample_source_character_sample(
            *sample,
            grit_level,
            variation_intensity,
            true,
            false,
            &mut base_last,
            &mut base_memory,
        );
        let hard = w30_resample_source_character_sample(
            *sample,
            grit_level,
            variation_intensity,
            true,
            true,
            &mut hard_last,
            &mut hard_memory,
        );
        base_square_sum += base * base;
        hard_square_sum += hard * hard;
    }
    let base_rms = (base_square_sum / proxy.len() as f32).sqrt();
    let hard_rms = (hard_square_sum / proxy.len() as f32).sqrt();
    let raw_ratio = hard_rms / base_rms.max(MIN_RMS);
    let output_gain = (W30_RESAMPLE_H12_TEXTURE_TARGET_LEVEL_RATIO / raw_ratio.max(MIN_RMS))
        .clamp(
            W30_RESAMPLE_H12_MIN_OUTPUT_GAIN,
            W30_RESAMPLE_H12_MAX_OUTPUT_GAIN,
        );
    W30ResampleHardCalibrationPlan {
        predicted_raw_level_ratio: raw_ratio,
        predicted_compensated_level_ratio: raw_ratio * output_gain,
        predicted_level_matched_body_ratio: 0.0,
        output_gain,
        hit_window_compensation_gain: 1.0,
        exact_callback_calibrated: false,
        exact_callback_evaluated: false,
    }
}

fn derive_w30_resample_hit_shaper_calibration(
    proxy: &[f32],
    proxy_sample_rate: f32,
    trigger_mask: u8,
    onset_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
) -> W30ResampleHardCalibrationPlan {
    const MIN_RMS: f32 = 1.0e-6;
    const HIT_WINDOW_SECONDS: f32 = 0.1;
    const BODY_START_SECONDS: f32 = 0.02;
    const HEAD_SECONDS: f32 = 1.0 / 55.0;
    const RELEASE_START_SECONDS: f32 = 0.06;

    if proxy.len() < 2
        || !proxy_sample_rate.is_finite()
        || proxy_sample_rate <= 0.0
        || trigger_mask == 0
    {
        return W30ResampleHardCalibrationPlan::default();
    }
    let recipe = W30ResampleLowImpactRecipe::SourceHitShaperV3;
    let hit_frames = (proxy_sample_rate * HIT_WINDOW_SECONDS).round().max(1.0) as usize;
    let body_start = (proxy_sample_rate * BODY_START_SECONDS).round().max(1.0) as usize;
    let mut dry_all = Vec::new();
    let mut shaped_all = Vec::new();
    let mut dry_body = Vec::new();
    let mut shaped_body = Vec::new();
    for (slot, onset_cursor) in onset_cursors.iter().copied().enumerate() {
        if trigger_mask & (1_u8 << slot) == 0 {
            continue;
        }
        let onset = usize::from(onset_cursor).min(proxy.len() - 1);
        let end = onset.saturating_add(hit_frames).min(proxy.len());
        if end.saturating_sub(onset) <= body_start {
            continue;
        }
        let segment = &proxy[onset..end];
        let equalized = peaking_eq_window(
            segment,
            proxy_sample_rate,
            recipe.body_eq_center_hz(),
            recipe.body_eq_q(),
            recipe.body_eq_gain_db(),
        );
        for (index, (dry, equalized)) in segment.iter().zip(equalized).enumerate() {
            let seconds = index as f32 / proxy_sample_rate;
            let head_mix = 1.0 - smoothstep01(seconds / HEAD_SECONDS);
            let attack_mix = if seconds <= RELEASE_START_SECONDS {
                1.0
            } else {
                1.0
                    - smoothstep01(
                        (seconds - RELEASE_START_SECONDS)
                            / (HIT_WINDOW_SECONDS - RELEASE_START_SECONDS),
                    )
            };
            let body_mix = attack_mix * (1.0 - head_mix);
            let body_hit = *dry + (equalized - *dry) * body_mix;
            let shaped_head = normalized_soft_clip(body_hit, recipe.head_drive());
            let shaped = body_hit + (shaped_head - body_hit) * recipe.head_wet() * head_mix;
            dry_all.push(*dry);
            shaped_all.push(shaped);
            if index >= body_start {
                dry_body.push(*dry);
                shaped_body.push(shaped);
            }
        }
    }
    if dry_all.is_empty() || dry_body.is_empty() {
        return W30ResampleHardCalibrationPlan::default();
    }
    let raw_level_ratio = rms(&shaped_all) / rms(&dry_all).max(MIN_RMS);
    let raw_body_ratio = rms(&shaped_body) / rms(&dry_body).max(MIN_RMS);
    let level_match_gain = (W30_RESAMPLE_H12_HIT_TARGET_LEVEL_RATIO
        / raw_level_ratio.max(MIN_RMS))
    .clamp(
        W30_RESAMPLE_H12_MIN_OUTPUT_GAIN,
        W30_RESAMPLE_H12_MAX_OUTPUT_GAIN,
    );
    W30ResampleHardCalibrationPlan {
        predicted_raw_level_ratio: raw_level_ratio,
        predicted_compensated_level_ratio: raw_level_ratio
            * W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN,
        predicted_level_matched_body_ratio: raw_body_ratio * level_match_gain,
        output_gain: W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN,
        hit_window_compensation_gain: 1.0,
        exact_callback_calibrated: false,
        exact_callback_evaluated: false,
    }
}

fn peaking_eq_window(
    samples: &[f32],
    sample_rate: f32,
    center_hz: f32,
    q: f32,
    gain_db: f32,
) -> Vec<f32> {
    let omega = std::f32::consts::TAU * center_hz.max(1.0) / sample_rate.max(1.0);
    let alpha = omega.sin() / (2.0 * q.max(0.01));
    let amplitude = 10.0_f32.powf(gain_db / 40.0);
    let a0 = 1.0 + alpha / amplitude;
    let b0 = (1.0 + alpha * amplitude) / a0;
    let b1 = -2.0 * omega.cos() / a0;
    let b2 = (1.0 - alpha * amplitude) / a0;
    let a1 = -2.0 * omega.cos() / a0;
    let a2 = (1.0 - alpha / amplitude) / a0;
    let mut z1 = 0.0_f32;
    let mut z2 = 0.0_f32;
    samples
        .iter()
        .map(|sample| {
            let output = b0 * *sample + z1;
            z1 = b1 * *sample - a1 * output + z2;
            z2 = b2 * *sample - a2 * output;
            output
        })
        .collect()
}

fn smoothstep01(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn normalized_soft_clip(sample: f32, drive: f32) -> f32 {
    let drive = drive.max(1.0);
    (sample * drive).tanh() / drive.tanh().max(f32::EPSILON)
}

fn derive_w30_resample_hard_suitability(
    samples: &[f32],
    channel_count: usize,
) -> W30ResampleHardSuitabilityPlan {
    let frame_count = samples.len() / channel_count;
    let mut square_sum = 0.0_f32;
    let mut active_frame_count = 0_usize;
    for frame in samples.chunks_exact(channel_count) {
        let mono = frame.iter().sum::<f32>() / channel_count as f32;
        square_sum += mono * mono;
        active_frame_count += usize::from(mono.abs() >= W30_RESAMPLE_HARD_ACTIVE_FRAME_FLOOR);
    }
    let source_rms = (square_sum / frame_count as f32).sqrt();
    let active_frame_ratio = active_frame_count as f32 / frame_count as f32;
    let status = if source_rms < W30_RESAMPLE_HARD_MIN_SOURCE_RMS {
        W30ResampleHardSuitability::InsufficientLevel
    } else if active_frame_ratio < W30_RESAMPLE_HARD_MIN_ACTIVE_FRAME_RATIO {
        W30ResampleHardSuitability::InsufficientActivity
    } else {
        W30ResampleHardSuitability::Suitable
    };
    W30ResampleHardSuitabilityPlan {
        status,
        source_rms,
        active_frame_ratio,
    }
}

fn resample_source_revision(
    source_sample_rate: u32,
    source_frame_count: u64,
    samples: &[f32],
) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut revision = FNV_OFFSET_BASIS;
    for byte in source_sample_rate
        .to_le_bytes()
        .into_iter()
        .chain(source_frame_count.to_le_bytes())
        .chain((samples.len() as u64).to_le_bytes())
        .chain(samples.iter().flat_map(|sample| sample.to_bits().to_le_bytes()))
    {
        revision ^= u64::from(byte);
        revision = revision.wrapping_mul(FNV_PRIME);
    }
    revision.max(1)
}

pub(super) fn derive_w30_resample_attack_lengths(
    samples: &[f32],
    channel_count: usize,
    source_sample_rate: u32,
    proxy_sample_count: usize,
    onset_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
) -> [u16; W30_RESAMPLE_HARD_SLICE_COUNT] {
    const ANALYSIS_WINDOW_MILLISECONDS: usize = 1;
    const PEAK_SEARCH_MILLISECONDS: usize = 20;
    const MIN_ATTACK_MILLISECONDS: usize = 4;
    const MAX_ATTACK_MILLISECONDS: usize = 80;
    const PEAK_RELEASE_RATIO: f32 = 0.55;
    const BODY_FLOOR_RATIO: f32 = 1.25;
    const RELEASE_CONFIRM_WINDOWS: usize = 2;

    let frame_count = samples.len() / channel_count;
    if frame_count <= 1 || proxy_sample_count <= 1 {
        return [0; W30_RESAMPLE_HARD_SLICE_COUNT];
    }
    let window_frames =
        ((source_sample_rate as usize * ANALYSIS_WINDOW_MILLISECONDS) / 1_000).max(1);
    let peak_search_windows =
        (PEAK_SEARCH_MILLISECONDS / ANALYSIS_WINDOW_MILLISECONDS).max(1);
    let min_attack_frames =
        ((source_sample_rate as usize * MIN_ATTACK_MILLISECONDS) / 1_000).max(1);
    let max_attack_frames =
        ((source_sample_rate as usize * MAX_ATTACK_MILLISECONDS) / 1_000).max(min_attack_frames);

    std::array::from_fn(|slot| {
        let onset_frame =
            usize::from(onset_cursors[slot]) * (frame_count - 1) / (proxy_sample_count - 1);
        let analysis_end = (onset_frame + max_attack_frames).min(frame_count);
        if analysis_end <= onset_frame {
            return 1;
        }

        let mut envelope = Vec::with_capacity(
            (analysis_end - onset_frame).div_ceil(window_frames),
        );
        for start in (onset_frame..analysis_end).step_by(window_frames) {
            let end = (start + window_frames).min(analysis_end);
            let mut square_sum = 0.0_f32;
            for frame_index in start..end {
                let base = frame_index * channel_count;
                let mono =
                    samples[base..base + channel_count].iter().sum::<f32>() / channel_count as f32;
                square_sum += mono * mono;
            }
            envelope.push((square_sum / (end - start).max(1) as f32).sqrt());
        }
        if envelope.is_empty() {
            return 1;
        }

        let peak_search_end = envelope.len().min(peak_search_windows);
        let peak_index = envelope[..peak_search_end]
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(index, _)| index)
            .unwrap_or(0);
        let peak = envelope[peak_index];
        let tail_start = envelope.len() * 3 / 4;
        let body_floor = envelope[tail_start..].iter().sum::<f32>()
            / envelope.len().saturating_sub(tail_start).max(1) as f32;
        let release_threshold = (peak * PEAK_RELEASE_RATIO).max(body_floor * BODY_FLOOR_RATIO);
        let release_window = (peak_index + 1..envelope.len())
            .find(|index| {
                let end = (*index + RELEASE_CONFIRM_WINDOWS).min(envelope.len());
                end - *index == RELEASE_CONFIRM_WINDOWS
                    && envelope[*index..end]
                        .iter()
                        .all(|value| *value <= release_threshold)
            })
            .unwrap_or(envelope.len());
        let available_attack_frames = analysis_end.saturating_sub(onset_frame).max(1);
        let minimum_attack_frames = min_attack_frames.min(available_attack_frames);
        let attack_frames = (release_window * window_frames)
            .clamp(minimum_attack_frames, available_attack_frames);
        let proxy_length =
            (attack_frames * (proxy_sample_count - 1)).div_ceil(frame_count - 1);
        proxy_length.clamp(1, usize::from(u16::MAX)) as u16
    })
}

pub(super) fn derive_w30_resample_attack_bite(
    proxy: &[f32],
    proxy_sample_rate: f32,
    trigger_mask: u8,
    onset_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    attack_lengths: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
) -> W30ResampleAttackBitePlan {
    const TARGET_NORMALIZED_PEAK: f32 = 0.78;
    const MAX_INPUT_GAIN: f32 = 12.0;
    const MIN_BAND_RMS: f32 = 1.0e-5;
    const MIN_OUTPUT_GAIN: f32 = 0.25;
    const MAX_OUTPUT_GAIN: f32 = 12.0;

    if proxy.len() < 2
        || !proxy_sample_rate.is_finite()
        || proxy_sample_rate <= 0.0
        || trigger_mask == 0
    {
        return W30ResampleAttackBitePlan::default();
    }

    let candidates = [
        W30ResampleAttackBiteBand::LowMid,
        W30ResampleAttackBiteBand::Presence,
    ];
    let mut selected_band = W30ResampleAttackBiteBand::Unavailable;
    let mut selected_score = 0.0_f32;
    let mut selected_attack_samples = Vec::new();
    for band in candidates {
        let Some((low_hz, high_hz)) = band.cutoff_hz() else {
            continue;
        };
        let mut attack_samples = Vec::new();
        let mut body_samples = Vec::new();
        let mut total_attack_energy = 0.0_f32;
        for slot in 0..W30_RESAMPLE_HARD_SLICE_COUNT {
            if trigger_mask & (1_u8 << slot) == 0 {
                continue;
            }
            let onset = usize::from(onset_cursors[slot]).min(proxy.len() - 1);
            let attack_len = usize::from(attack_lengths[slot].max(1));
            let attack_end = onset.saturating_add(attack_len).min(proxy.len());
            if attack_end <= onset {
                continue;
            }
            total_attack_energy += proxy[onset..attack_end]
                .iter()
                .map(|sample| sample * sample)
                .sum::<f32>();
            attack_samples.extend(bandpass_window(
                &proxy[onset..attack_end],
                proxy_sample_rate,
                low_hz,
                high_hz,
            ));
            let body_end = attack_end.saturating_add(attack_len).min(proxy.len());
            if body_end > attack_end {
                body_samples.extend(bandpass_window(
                    &proxy[attack_end..body_end],
                    proxy_sample_rate,
                    low_hz,
                    high_hz,
                ));
            }
        }
        let attack_rms = rms(&attack_samples);
        if attack_rms < MIN_BAND_RMS {
            continue;
        }
        let body_rms = rms(&body_samples).max(MIN_BAND_RMS);
        let total_attack_rms =
            (total_attack_energy / attack_samples.len().max(1) as f32).sqrt();
        let band_share = (attack_rms / total_attack_rms.max(MIN_BAND_RMS)).min(1.0);
        let score = (attack_rms / body_rms).min(8.0) * band_share.sqrt();
        if score > selected_score {
            selected_score = score;
            selected_band = band;
            selected_attack_samples = attack_samples;
        }
    }

    if selected_band == W30ResampleAttackBiteBand::Unavailable {
        return W30ResampleAttackBitePlan::default();
    }
    let peak = selected_attack_samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f32, f32::max);
    if peak <= MIN_BAND_RMS {
        return W30ResampleAttackBitePlan::default();
    }
    let input_gain = (TARGET_NORMALIZED_PEAK / peak).clamp(1.0, MAX_INPUT_GAIN);
    let drive_normalization = W30_RESAMPLE_HARD_BITE_NONLINEAR_DRIVE
        .tanh()
        .max(f32::EPSILON);
    let (low_hz, high_hz) = selected_band
        .cutoff_hz()
        .expect("selected bite band has cutoffs");
    let gesture_band = bandpass_window(proxy, proxy_sample_rate, low_hz, high_hz);
    let gesture_residual = gesture_band
        .iter()
        .map(|dry| {
            let wet = (dry * input_gain * W30_RESAMPLE_HARD_BITE_NONLINEAR_DRIVE).tanh()
                / drive_normalization
                / input_gain;
            wet - dry
        })
        .collect::<Vec<_>>();
    let output_gain = (rms(&gesture_band) / rms(&gesture_residual).max(MIN_BAND_RMS))
        .clamp(MIN_OUTPUT_GAIN, MAX_OUTPUT_GAIN);
    W30ResampleAttackBitePlan {
        band: selected_band,
        input_gain,
        output_gain,
    }
}

fn bandpass_window(samples: &[f32], sample_rate: f32, low_hz: f32, high_hz: f32) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }
    let nyquist_guard = sample_rate * 0.45;
    let low_hz = low_hz.min(nyquist_guard).max(1.0);
    let high_hz = high_hz.min(nyquist_guard).max(low_hz + 1.0);
    let low_alpha = 1.0 - (-std::f32::consts::TAU * low_hz / sample_rate).exp();
    let high_alpha = 1.0 - (-std::f32::consts::TAU * high_hz / sample_rate).exp();
    let mut low_state = samples[0];
    let mut high_state = samples[0];
    samples
        .iter()
        .map(|sample| {
            low_state += low_alpha * (*sample - low_state);
            high_state += high_alpha * (*sample - high_state);
            high_state - low_state
        })
        .collect()
}

fn rms(samples: &[f32]) -> f32 {
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len().max(1) as f32).sqrt()
}

pub(super) fn derive_w30_resample_low_impact(
    proxy: &[f32],
    proxy_sample_rate: f32,
    trigger_mask: u8,
    onset_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    attack_lengths: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
) -> W30ResampleLowImpactPlan {
    const LOW_HZ: f32 = 45.0;
    const HIGH_HZ: f32 = 180.0;
    const MIN_RMS: f32 = 1.0e-6;

    if proxy.len() < 2
        || !proxy_sample_rate.is_finite()
        || proxy_sample_rate <= HIGH_HZ * 2.0
        || trigger_mask == 0
    {
        return W30ResampleLowImpactPlan::default();
    }

    let mut attack = Vec::new();
    let mut body = Vec::new();
    for slot in 0..W30_RESAMPLE_HARD_SLICE_COUNT {
        if trigger_mask & (1_u8 << slot) == 0 {
            continue;
        }
        let onset = usize::from(onset_cursors[slot]).min(proxy.len() - 1);
        let attack_len = usize::from(attack_lengths[slot].max(1));
        let attack_end = onset.saturating_add(attack_len).min(proxy.len());
        if attack_end <= onset {
            continue;
        }
        attack.extend_from_slice(&proxy[onset..attack_end]);
        let body_end = attack_end.saturating_add(attack_len).min(proxy.len());
        if body_end > attack_end {
            body.extend_from_slice(&proxy[attack_end..body_end]);
        }
    }
    if attack.is_empty() {
        return W30ResampleLowImpactPlan::default();
    }

    let low_attack = bandpass_window(&attack, proxy_sample_rate, LOW_HZ, HIGH_HZ);
    let low_body = bandpass_window(&body, proxy_sample_rate, LOW_HZ, HIGH_HZ);
    let low_attack_rms = rms(&low_attack);
    let full_attack_rms = rms(&attack).max(MIN_RMS);
    let source_rms = rms(proxy).max(MIN_RMS);
    let low_band_attack_share = (low_attack_rms / full_attack_rms).min(1.0);
    let low_band_attack_over_body = low_attack_rms / rms(&low_body).max(MIN_RMS);
    let low_band_attack_over_source = low_attack_rms / source_rms;
    let recipe = if low_band_attack_share >= W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_SHARE
        && low_band_attack_over_body >= W30_RESAMPLE_HIT_SHAPER_MIN_ATTACK_OVER_BODY
        && low_band_attack_over_source >= W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_OVER_SOURCE
    {
        W30ResampleLowImpactRecipe::SourceHitShaperV3
    } else {
        W30ResampleLowImpactRecipe::Unavailable
    };
    W30ResampleLowImpactPlan {
        recipe,
        low_band_attack_share,
        low_band_attack_over_body,
        low_band_attack_over_source,
    }
}

#[cfg(test)]
mod resample_attack_bite_tests {
    use super::*;

    fn attack_proxy(frequency_hz: f32) -> Vec<f32> {
        let sample_rate = 8_000.0;
        (0..800)
            .map(|index| {
                let amplitude = if index < 80 {
                    0.7
                } else if index < 160 {
                    0.08
                } else {
                    0.0
                };
                amplitude
                    * (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate).sin()
            })
            .collect()
    }

    #[test]
    fn attack_bite_selects_the_source_dominant_attack_band() {
        let onsets = [0; W30_RESAMPLE_HARD_SLICE_COUNT];
        let lengths = [80; W30_RESAMPLE_HARD_SLICE_COUNT];

        let low_mid =
            derive_w30_resample_attack_bite(&attack_proxy(500.0), 8_000.0, 1, onsets, lengths);
        let presence =
            derive_w30_resample_attack_bite(&attack_proxy(1_600.0), 8_000.0, 1, onsets, lengths);

        assert_eq!(low_mid.band, W30ResampleAttackBiteBand::LowMid);
        assert_eq!(presence.band, W30ResampleAttackBiteBand::Presence);
        for plan in [low_mid, presence] {
            assert!(plan.input_gain.is_finite() && plan.input_gain >= 1.0);
            assert!(plan.output_gain.is_finite() && plan.output_gain > 0.0);
        }
    }

    #[test]
    fn attack_bite_stays_unavailable_without_a_trigger_or_signal() {
        let onsets = [0; W30_RESAMPLE_HARD_SLICE_COUNT];
        let lengths = [80; W30_RESAMPLE_HARD_SLICE_COUNT];

        assert_eq!(
            derive_w30_resample_attack_bite(&[0.0; 800], 8_000.0, 1, onsets, lengths),
            W30ResampleAttackBitePlan::default()
        );
        assert_eq!(
            derive_w30_resample_attack_bite(&attack_proxy(1_600.0), 8_000.0, 0, onsets, lengths),
            W30ResampleAttackBitePlan::default()
        );
    }

    #[test]
    fn low_impact_requires_a_source_owned_low_transient() {
        let sample_rate = 8_000.0;
        let low_transient = (0..800)
            .map(|index| {
                let amplitude = if index < 160 { 0.8 } else { 0.08 };
                amplitude
                    * (std::f32::consts::TAU * 90.0 * index as f32 / sample_rate).sin()
            })
            .collect::<Vec<_>>();
        let high_transient = (0..800)
            .map(|index| {
                let amplitude = if index < 160 { 0.8 } else { 0.08 };
                amplitude
                    * (std::f32::consts::TAU * 1_200.0 * index as f32 / sample_rate).sin()
            })
            .collect::<Vec<_>>();
        let onsets = [0; W30_RESAMPLE_HARD_SLICE_COUNT];
        let lengths = [160; W30_RESAMPLE_HARD_SLICE_COUNT];

        let low_plan =
            derive_w30_resample_low_impact(&low_transient, sample_rate, 1, onsets, lengths);
        let high_plan =
            derive_w30_resample_low_impact(&high_transient, sample_rate, 1, onsets, lengths);

        assert_eq!(
            low_plan.recipe,
            W30ResampleLowImpactRecipe::SourceHitShaperV3
        );
        assert_eq!(
            high_plan.recipe,
            W30ResampleLowImpactRecipe::Unavailable
        );
        assert!(
            low_plan.low_band_attack_share >= W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_SHARE
        );
        assert!(
            low_plan.low_band_attack_over_body
                >= W30_RESAMPLE_HIT_SHAPER_MIN_ATTACK_OVER_BODY
        );
    }

    #[test]
    fn sustained_low_material_does_not_claim_kick_impact() {
        let sample_rate = 8_000.0;
        let sustained = (0..800)
            .map(|index| {
                0.6 * (std::f32::consts::TAU * 90.0 * index as f32 / sample_rate).sin()
            })
            .collect::<Vec<_>>();
        let plan = derive_w30_resample_low_impact(
            &sustained,
            sample_rate,
            1,
            [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            [160; W30_RESAMPLE_HARD_SLICE_COUNT],
        );

        assert_eq!(plan.recipe, W30ResampleLowImpactRecipe::Unavailable);
    }

    #[test]
    fn hard_suitability_rejects_quiet_source_before_policy_selection() {
        let samples = vec![0.02_f32; 8_000];
        let projection =
            project_resample_source_from_interleaved(&samples, 1, 8_000, 1.0, 1.0)
                .expect("projection");

        assert_eq!(
            projection.hard_suitability.status,
            W30ResampleHardSuitability::InsufficientLevel
        );
        assert_eq!(projection.hard_policy, W30ResampleTapHardPolicy::Unavailable);
        assert_eq!(projection.hard_trigger_mask, 0);
        assert_eq!(
            projection.hard_low_impact,
            W30ResampleLowImpactPlan::default()
        );
    }

    #[test]
    fn exact_hit_calibration_uses_the_callback_and_reuses_matching_evidence() {
        let source_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/test_audio/examples/Beat03_130BPM(Full).wav");
        let cache = SourceAudioCache::load_pcm_wav(source_path).expect("load Beat03 fixture");
        let capture_frames =
            (cache.sample_rate as f32 * 4.0 * 60.0 / 130.284_94).round() as usize;
        let capture_sample_count =
            capture_frames.saturating_mul(usize::from(cache.channel_count));
        let projection = project_resample_source_from_interleaved(
            &cache.interleaved_samples()[..capture_sample_count],
            usize::from(cache.channel_count),
            cache.sample_rate,
            0.82,
            0.82,
        )
        .expect("project Beat03 fixture");
        assert_eq!(
            projection.hard_policy,
            W30ResampleTapHardPolicy::SourceTransientChop
        );
        let hard_low_impact = W30ResampleLowImpactPlan {
            recipe: W30ResampleLowImpactRecipe::SourceHitShaperV3,
            low_band_attack_share: 0.8,
            low_band_attack_over_body: 2.0,
            low_band_attack_over_source: 1.0,
        };
        let mut state = W30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            availability: W30ResampleTapAvailability::SourceAudioReady,
            source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
            source_capture_id: Some("cap-calibration-test".into()),
            source_audio: Some(projection.audio),
            lineage_capture_count: 1,
            generation_depth: 1,
            variation: W30ResampleTapVariation::HardDamage,
            variation_revision: 7,
            variation_intensity: 0.82,
            hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
            hard_suitability: projection.hard_suitability,
            hard_calibration: W30ResampleHardCalibrationPlan {
                output_gain: W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN,
                ..W30ResampleHardCalibrationPlan::default()
            },
            hard_trigger_mask: projection.hard_trigger_mask,
            hard_slice_cursors: projection.hard_slice_cursors,
            hard_attack_lengths: projection.hard_attack_lengths,
            hard_attack_bite: projection.hard_attack_bite,
            hard_low_impact,
            hard_gesture: projection.hard_gesture,
            hard_transient_contrast: projection.hard_transient_contrast,
            music_bus_level: 0.64,
            grit_level: 0.82,
            is_transport_running: true,
            tempo_bpm: 130.0,
            position_beats: 0.0,
        };
        let uncalibrated = state.clone();

        calibrate_w30_hit_shaper_exact_callback(&mut state, None);

        assert!(state.hard_calibration.exact_callback_calibrated);
        assert!(
            state.hard_calibration.output_gain
                <= W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN
        );
        assert!(
            state
                .hard_calibration
                .predicted_compensated_level_ratio
                <= W30_EXACT_HIT_MAX_LEVEL_RATIO
        );
        assert!(
            state
                .hard_calibration
                .predicted_level_matched_body_ratio
                >= W30_EXACT_HIT_MIN_BODY_RATIO
        );

        let mut repeated = uncalibrated;
        calibrate_w30_hit_shaper_exact_callback(&mut repeated, Some(&state));
        assert_eq!(repeated.hard_calibration, state.hard_calibration);
    }

    #[test]
    fn exact_hit_calibration_reuses_a_matching_evaluated_rejection() {
        let mut current = W30ResampleTapState {
            variation: W30ResampleTapVariation::HardDamage,
            hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
            hard_low_impact: W30ResampleLowImpactPlan {
                recipe: W30ResampleLowImpactRecipe::SourceHitShaperV3,
                ..W30ResampleLowImpactPlan::default()
            },
            source_audio: Some(Box::new(W30ResampleSourceWindow {
                source_revision: 91,
                source_start_frame: 0,
                source_sample_rate: 48_000,
                source_frame_count: 1,
                sample_count: 1,
                samples: [0.0; riotbox_audio::w30::W30_RESAMPLE_SOURCE_WINDOW_LEN],
            })),
            tempo_bpm: 120.0,
            ..W30ResampleTapState::default()
        };
        let mut previous = current.clone();
        previous.hard_calibration.output_gain = 0.37;
        previous.hard_calibration.exact_callback_evaluated = true;
        previous.hard_calibration.exact_callback_calibrated = false;

        calibrate_w30_hit_shaper_exact_callback(&mut current, Some(&previous));

        assert_eq!(current.hard_calibration, previous.hard_calibration);
    }

    #[test]
    fn exact_hit_calibration_measures_every_rendered_cycle() {
        let sample_rate = 100;
        let tempo_bpm = 60.0;
        let step_frames = 50;
        let frame_count =
            step_frames * W30_RESAMPLE_HARD_SLICE_COUNT * W30_EXACT_HIT_CALIBRATION_CYCLES;
        let low = vec![0.0; frame_count];
        let high = vec![1.0; frame_count];

        let exact_energy = w30_exact_hit_window_energy(
            &high, 1, sample_rate, tempo_bpm, 0b0000_0001, 0.0, 0.01,
        );
        let curve = w30_exact_hit_window_energy_curve(
            &low,
            &high,
            1,
            sample_rate,
            tempo_bpm,
            0b0000_0001,
            W30ExactHitWindow {
                start_seconds: 0.0,
                end_seconds: 0.01,
            },
        );

        assert_eq!(exact_energy, W30_EXACT_HIT_CALIBRATION_CYCLES as f64);
        assert_eq!(curve.at(1.0), W30_EXACT_HIT_CALIBRATION_CYCLES as f64);
    }

    #[test]
    fn hard_suitability_rejects_sparse_source_before_policy_selection() {
        let mut samples = vec![0.0_f32; 8_000];
        samples[..4_000].fill(0.1);
        let projection =
            project_resample_source_from_interleaved(&samples, 1, 8_000, 1.0, 1.0)
                .expect("projection");

        assert_eq!(
            projection.hard_suitability.status,
            W30ResampleHardSuitability::InsufficientActivity
        );
        assert_eq!(projection.hard_policy, W30ResampleTapHardPolicy::Unavailable);
        assert_eq!(projection.hard_trigger_mask, 0);
    }

    #[test]
    fn hard_suitability_allows_active_source_to_select_a_policy() {
        let samples = (0..8_000)
            .map(|index| {
                0.1 * (std::f32::consts::TAU * 220.0 * index as f32 / 8_000.0).sin()
            })
            .collect::<Vec<_>>();
        let projection =
            project_resample_source_from_interleaved(&samples, 1, 8_000, 1.0, 1.0)
                .expect("projection");

        assert_eq!(
            projection.hard_suitability.status,
            W30ResampleHardSuitability::Suitable
        );
        assert_ne!(projection.hard_policy, W30ResampleTapHardPolicy::Unavailable);
    }

    #[test]
    fn texture_calibration_is_source_relative_instead_of_a_fixed_gain() {
        let quiet = (0..8_000)
            .map(|index| {
                0.05 * (std::f32::consts::TAU * 220.0 * index as f32 / 8_000.0).sin()
            })
            .collect::<Vec<_>>();
        let loud = quiet.iter().map(|sample| sample * 8.0).collect::<Vec<_>>();

        let quiet_plan = derive_w30_resample_texture_calibration(&quiet, 1.0, 0.82);
        let loud_plan = derive_w30_resample_texture_calibration(&loud, 1.0, 0.82);

        assert_ne!(quiet_plan.output_gain, loud_plan.output_gain);
        for plan in [quiet_plan, loud_plan] {
            assert!(plan.output_gain.is_finite());
            assert!(
                (plan.predicted_compensated_level_ratio
                    - W30_RESAMPLE_H12_TEXTURE_TARGET_LEVEL_RATIO)
                    .abs()
                    < 0.001
            );
        }
    }

    #[test]
    fn hit_calibration_reports_level_matched_body_evidence() {
        let sample_rate = 8_000.0;
        let source_hit = (0..800)
            .map(|index| {
                0.5 * (std::f32::consts::TAU * 90.0 * index as f32 / sample_rate).sin()
            })
            .collect::<Vec<_>>();

        let plan = derive_w30_resample_hit_shaper_calibration(
            &source_hit,
            sample_rate,
            1,
            [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        );

        assert!(plan.predicted_raw_level_ratio.is_finite());
        assert!(plan.predicted_level_matched_body_ratio.is_finite());
        assert_eq!(
            plan.output_gain,
            W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN
        );
    }

    #[test]
    fn h13_gesture_selects_source_impact_and_preceding_pickup_slot() {
        let sample_rate = 8_000.0;
        let mut proxy = vec![0.02_f32; 8_000];
        let onsets = std::array::from_fn(|slot| (slot * 1_000) as u16);
        for sample in &mut proxy[3_000..3_160] {
            *sample = 0.8;
        }
        for sample in &mut proxy[3_160..3_800] {
            *sample = 0.18;
        }

        let plan =
            derive_w30_resample_hard_gesture(&proxy, sample_rate, 0b0000_1001, onsets);

        assert_eq!(
            plan.recipe,
            W30ResampleHardGestureRecipe::SourceReverseIntoImpactV1
        );
        assert_eq!(plan.impact_slot, 3);
        assert_eq!(plan.pickup_slot, 2);
        assert!(
            (W30_RESAMPLE_H13_MIN_BODY_GAIN..=W30_RESAMPLE_H13_MAX_BODY_GAIN)
                .contains(&plan.body_gain)
        );
        assert!(
            (W30_RESAMPLE_H13_MIN_IMPACT_LEVEL_COMPENSATION..=1.0)
                .contains(&plan.impact_level_compensation)
        );
        assert!((W30_RESAMPLE_H13_MIN_PICKUP_GAIN..=1.0).contains(&plan.pickup_gain));
        assert!(plan.selected_head_rms > plan.selected_body_rms);
    }

    #[test]
    fn h13_gesture_choice_changes_with_source_evidence() {
        let sample_rate = 8_000.0;
        let onsets = std::array::from_fn(|slot| (slot * 1_000) as u16);
        let mut first = vec![0.02_f32; 8_000];
        first[1_000..1_160].fill(0.7);
        first[1_160..1_800].fill(0.16);
        let mut second = vec![0.02_f32; 8_000];
        second[5_000..5_160].fill(0.7);
        second[5_160..5_800].fill(0.16);

        let first_plan =
            derive_w30_resample_hard_gesture(&first, sample_rate, 0b0010_0010, onsets);
        let second_plan =
            derive_w30_resample_hard_gesture(&second, sample_rate, 0b0010_0010, onsets);

        assert_eq!(first_plan.impact_slot, 1);
        assert_eq!(second_plan.impact_slot, 5);
        assert_ne!(first_plan.pickup_slot, second_plan.pickup_slot);
    }

    #[test]
    fn h13_gesture_stays_unavailable_without_source_trigger_evidence() {
        assert_eq!(
            derive_w30_resample_hard_gesture(
                &[0.2; 800],
                8_000.0,
                0,
                [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            ),
            W30ResampleHardGesturePlan::default()
        );
    }
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
    if transient_contrast < W30_RESAMPLE_TRANSIENT_CHOP_MIN_RISE_TO_MEAN {
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

const W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE: u32 = 48_000;
const W30_EXACT_HIT_CALIBRATION_CHANNELS: u16 = 2;
const W30_EXACT_HIT_TARGET_LEVEL_RATIO: f32 = 1.20;
const W30_EXACT_HIT_MAX_LEVEL_RATIO: f32 = 1.30;
// Calibration prevents head collapse while the stricter product-path render
// remains authoritative at 1.15. Keeping these distinct lets the exact
// callback solve whole-path level instead of pinning every source to the
// loudest gain that barely satisfies the estimator.
const W30_EXACT_HIT_MIN_HEAD_RATIO: f32 = 1.10;
const W30_EXACT_HIT_MIN_BODY_RATIO: f32 = 1.15;
const W30_EXACT_HIT_MIN_OUTPUT_GAIN: f32 = 0.25;
const W30_EXACT_HIT_SEARCH_STEPS: usize = 10;
const W30_EXACT_HIT_REFINEMENT_STEPS: usize = 4;
const W30_EXACT_HIT_CALIBRATION_CYCLES: usize = 4;

#[derive(Clone, Copy)]
struct W30ExactHitMetrics {
    level_ratio: f32,
    head_ratio: f32,
    body_ratio: f32,
}

fn w30_hit_calibration_inputs_match(
    state: &W30ResampleTapState,
    previous: &W30ResampleTapState,
) -> bool {
    state
        .source_audio
        .as_ref()
        .zip(previous.source_audio.as_ref())
        .is_some_and(|(current, prior)| current.source_revision == prior.source_revision)
        && state.tempo_bpm.to_bits() == previous.tempo_bpm.to_bits()
        && state.variation == previous.variation
        && state.variation_intensity.to_bits() == previous.variation_intensity.to_bits()
        && state.grit_level.to_bits() == previous.grit_level.to_bits()
        && state.hard_policy == previous.hard_policy
        && state.hard_trigger_mask == previous.hard_trigger_mask
        && state.hard_slice_cursors == previous.hard_slice_cursors
        && state.hard_attack_lengths == previous.hard_attack_lengths
        && state.hard_attack_bite == previous.hard_attack_bite
        && state.hard_low_impact == previous.hard_low_impact
        && state.hard_gesture == previous.hard_gesture
}

fn w30_exact_hit_window_energy(
    samples: &[f32],
    channel_count: usize,
    sample_rate: u32,
    tempo_bpm: f32,
    trigger_mask: u8,
    window_start_seconds: f32,
    window_end_seconds: f32,
) -> f64 {
    if channel_count == 0
        || sample_rate == 0
        || !tempo_bpm.is_finite()
        || tempo_bpm <= 0.0
        || trigger_mask == 0
    {
        return 0.0;
    }
    let frame_count = samples.len() / channel_count;
    let step_frames = (sample_rate as f64 * 30.0 / f64::from(tempo_bpm))
        .round()
        .max(1.0) as usize;
    let window_start = (sample_rate as f32 * window_start_seconds)
        .round()
        .max(0.0) as usize;
    let window_end = (sample_rate as f32 * window_end_seconds)
        .round()
        .max(1.0) as usize;
    let mut energy = 0.0_f64;
    let step_count = frame_count.div_ceil(step_frames);
    for step in 0..step_count {
        let slot = step % W30_RESAMPLE_HARD_SLICE_COUNT;
        if trigger_mask & (1_u8 << slot) == 0 {
            continue;
        }
        let start_frame = step
            .saturating_mul(step_frames)
            .saturating_add(window_start)
            .min(frame_count);
        let end_frame = step
            .saturating_mul(step_frames)
            .saturating_add(window_end)
            .min(frame_count);
        for sample in &samples[start_frame * channel_count..end_frame * channel_count] {
            energy += f64::from(*sample) * f64::from(*sample);
        }
    }
    energy
}

fn w30_exact_hit_bandpass_interleaved(
    samples: &[f32],
    channel_count: usize,
    sample_rate: u32,
    low_hz: f32,
    high_hz: f32,
) -> Vec<f32> {
    if channel_count == 0 || sample_rate == 0 {
        return Vec::new();
    }
    let sample_rate = sample_rate as f32;
    let low_alpha = 1.0 - (-std::f32::consts::TAU * low_hz / sample_rate).exp();
    let high_alpha = 1.0 - (-std::f32::consts::TAU * high_hz / sample_rate).exp();
    let mut low_state = vec![0.0_f32; channel_count];
    let mut high_state = vec![0.0_f32; channel_count];
    let mut initialized = vec![false; channel_count];
    let mut filtered = Vec::with_capacity(samples.len());
    for frame in samples.chunks_exact(channel_count) {
        for (channel, sample) in frame.iter().enumerate() {
            if !initialized[channel] {
                low_state[channel] = *sample;
                high_state[channel] = *sample;
                initialized[channel] = true;
            }
            low_state[channel] += low_alpha * (*sample - low_state[channel]);
            high_state[channel] += high_alpha * (*sample - high_state[channel]);
            filtered.push(high_state[channel] - low_state[channel]);
        }
    }
    filtered
}

#[derive(Clone, Copy)]
struct W30ExactEnergyCurve {
    low_energy: f64,
    low_delta_inner_product: f64,
    delta_energy: f64,
}

impl W30ExactEnergyCurve {
    fn from_samples(low: &[f32], high: &[f32]) -> Self {
        let mut curve = Self {
            low_energy: 0.0,
            low_delta_inner_product: 0.0,
            delta_energy: 0.0,
        };
        for (low, high) in low.iter().zip(high) {
            let low = f64::from(*low);
            let delta = f64::from(*high) - low;
            curve.low_energy += low * low;
            curve.low_delta_inner_product += low * delta;
            curve.delta_energy += delta * delta;
        }
        curve
    }

    fn at(self, interpolation: f64) -> f64 {
        self.low_energy
            + 2.0 * interpolation * self.low_delta_inner_product
            + interpolation * interpolation * self.delta_energy
    }
}

#[derive(Clone, Copy)]
struct W30ExactHitWindow {
    start_seconds: f32,
    end_seconds: f32,
}

fn w30_exact_hit_window_energy_curve(
    low: &[f32],
    high: &[f32],
    channel_count: usize,
    sample_rate: u32,
    tempo_bpm: f32,
    trigger_mask: u8,
    window: W30ExactHitWindow,
) -> W30ExactEnergyCurve {
    if channel_count == 0
        || sample_rate == 0
        || !tempo_bpm.is_finite()
        || tempo_bpm <= 0.0
        || trigger_mask == 0
    {
        return W30ExactEnergyCurve {
            low_energy: 0.0,
            low_delta_inner_product: 0.0,
            delta_energy: 0.0,
        };
    }
    let frame_count = low.len().min(high.len()) / channel_count;
    let step_frames = (sample_rate as f64 * 30.0 / f64::from(tempo_bpm))
        .round()
        .max(1.0) as usize;
    let window_start = (sample_rate as f32 * window.start_seconds)
        .round()
        .max(0.0) as usize;
    let window_end = (sample_rate as f32 * window.end_seconds)
        .round()
        .max(1.0) as usize;
    let mut selected_low = Vec::new();
    let mut selected_high = Vec::new();
    let step_count = frame_count.div_ceil(step_frames);
    for step in 0..step_count {
        let slot = step % W30_RESAMPLE_HARD_SLICE_COUNT;
        if trigger_mask & (1_u8 << slot) == 0 {
            continue;
        }
        let start = step
            .saturating_mul(step_frames)
            .saturating_add(window_start)
            .min(frame_count)
            * channel_count;
        let end = step
            .saturating_mul(step_frames)
            .saturating_add(window_end)
            .min(frame_count)
            * channel_count;
        selected_low.extend_from_slice(&low[start..end]);
        selected_high.extend_from_slice(&high[start..end]);
    }
    W30ExactEnergyCurve::from_samples(&selected_low, &selected_high)
}

struct W30ExactHitMetricModel {
    minimum_gain: f32,
    maximum_gain: f32,
    base_level_energy: f64,
    base_head_energy: f64,
    base_body_energy: f64,
    hard_level_curve: W30ExactEnergyCurve,
    hard_head_curve: W30ExactEnergyCurve,
    hard_body_curve: W30ExactEnergyCurve,
}

impl W30ExactHitMetricModel {
    fn metrics(&self, output_gain: f32) -> W30ExactHitMetrics {
        const MIN_ENERGY: f64 = 1.0e-12;
        let interpolation = f64::from(
            ((output_gain - self.minimum_gain)
                / (self.maximum_gain - self.minimum_gain).max(f32::EPSILON))
            .clamp(0.0, 1.0),
        );
        let ratio = |hard_energy: f64, base_energy: f64| {
            (hard_energy / base_energy.max(MIN_ENERGY)).sqrt() as f32
        };
        W30ExactHitMetrics {
            level_ratio: ratio(
                self.hard_level_curve.at(interpolation),
                self.base_level_energy,
            ),
            head_ratio: ratio(
                self.hard_head_curve.at(interpolation),
                self.base_head_energy,
            ),
            body_ratio: ratio(
                self.hard_body_curve.at(interpolation),
                self.base_body_energy,
            ),
        }
    }
}

fn w30_exact_hit_metric_model(
    base: &[f32],
    hard_low: &[f32],
    hard_high: &[f32],
    state: &W30ResampleTapState,
) -> W30ExactHitMetricModel {
    let channel_count = usize::from(W30_EXACT_HIT_CALIBRATION_CHANNELS);
    let total_energy = |samples: &[f32]| {
        samples
            .iter()
            .map(|sample| f64::from(*sample) * f64::from(*sample))
            .sum::<f64>()
    };
    let (head_low_hz, head_high_hz) = state
        .hard_low_impact
        .recipe
        .presence_cutoff_hz()
        .unwrap_or((900.0, 3_600.0));
    let (body_low_hz, body_high_hz) = state
        .hard_low_impact
        .recipe
        .cutoff_hz()
        .unwrap_or((45.0, 180.0));
    let filtered_base_head = w30_exact_hit_bandpass_interleaved(
        base,
        channel_count,
        W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
        head_low_hz,
        head_high_hz,
    );
    let filtered_hard_low_head = w30_exact_hit_bandpass_interleaved(
        hard_low,
        channel_count,
        W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
        head_low_hz,
        head_high_hz,
    );
    let filtered_hard_high_head = w30_exact_hit_bandpass_interleaved(
        hard_high,
        channel_count,
        W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
        head_low_hz,
        head_high_hz,
    );
    let filtered_base_body = w30_exact_hit_bandpass_interleaved(
        base,
        channel_count,
        W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
        body_low_hz,
        body_high_hz,
    );
    let filtered_hard_low_body = w30_exact_hit_bandpass_interleaved(
        hard_low,
        channel_count,
        W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
        body_low_hz,
        body_high_hz,
    );
    let filtered_hard_high_body = w30_exact_hit_bandpass_interleaved(
        hard_high,
        channel_count,
        W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
        body_low_hz,
        body_high_hz,
    );
    let base_head_energy = w30_exact_hit_window_energy(
        &filtered_base_head,
        channel_count,
        W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
        state.tempo_bpm,
        state.hard_trigger_mask,
        0.0,
        0.02,
    );
    let base_body_energy = w30_exact_hit_window_energy(
        &filtered_base_body,
        channel_count,
        W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
        state.tempo_bpm,
        state.hard_trigger_mask,
        0.02,
        0.10,
    );
    W30ExactHitMetricModel {
        minimum_gain: W30_EXACT_HIT_MIN_OUTPUT_GAIN,
        maximum_gain: W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN,
        base_level_energy: total_energy(base),
        base_head_energy,
        base_body_energy,
        hard_level_curve: W30ExactEnergyCurve::from_samples(hard_low, hard_high),
        hard_head_curve: w30_exact_hit_window_energy_curve(
            &filtered_hard_low_head,
            &filtered_hard_high_head,
            channel_count,
            W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
            state.tempo_bpm,
            state.hard_trigger_mask,
            W30ExactHitWindow {
                start_seconds: 0.0,
                end_seconds: 0.02,
            },
        ),
        hard_body_curve: w30_exact_hit_window_energy_curve(
            &filtered_hard_low_body,
            &filtered_hard_high_body,
            channel_count,
            W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
            state.tempo_bpm,
            state.hard_trigger_mask,
            W30ExactHitWindow {
                start_seconds: 0.02,
                end_seconds: 0.10,
            },
        ),
    }
}

fn w30_exact_hit_direct_metrics(
    base: &[f32],
    hard: &[f32],
    state: &W30ResampleTapState,
) -> W30ExactHitMetrics {
    w30_exact_hit_metric_model(base, hard, hard, state)
        .metrics(W30_EXACT_HIT_MIN_OUTPUT_GAIN)
}

fn w30_exact_hit_render(
    state: &W30ResampleTapState,
    output_gain: f32,
    frame_count: usize,
) -> Vec<f32> {
    let mut hard = state.clone();
    hard.hard_calibration.output_gain = output_gain;
    hard.hard_calibration.hit_window_compensation_gain =
        (W30_RESAMPLE_HIT_SHAPER_PRESERVED_OUTPUT_GAIN / output_gain.max(f32::EPSILON))
            .clamp(1.0, W30_RESAMPLE_HIT_SHAPER_MAX_WINDOW_COMPENSATION_GAIN);
    render_w30_resample_tap_offline(
        &hard,
        W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
        W30_EXACT_HIT_CALIBRATION_CHANNELS,
        frame_count,
    )
}

fn calibrate_w30_hit_shaper_exact_callback(
    state: &mut W30ResampleTapState,
    previous: Option<&W30ResampleTapState>,
) {
    if state.variation != W30ResampleTapVariation::HardDamage
        || state.hard_policy != W30ResampleTapHardPolicy::SourceTransientChop
        || state.hard_low_impact.recipe != W30ResampleLowImpactRecipe::SourceHitShaperV3
        || state.source_audio.is_none()
        || !state.tempo_bpm.is_finite()
        || state.tempo_bpm <= 0.0
    {
        return;
    }
    if let Some(previous) = previous.filter(|previous| {
        previous.hard_calibration.exact_callback_evaluated
            && w30_hit_calibration_inputs_match(state, previous)
    }) {
        state.hard_calibration = previous.hard_calibration;
        return;
    }
    state.hard_calibration.exact_callback_evaluated = true;

    let step_frames = (W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE as f64 * 30.0
        / f64::from(state.tempo_bpm))
    .round()
    .max(1.0) as usize;
    let frame_count = step_frames
        .saturating_mul(W30_RESAMPLE_HARD_SLICE_COUNT)
        .saturating_mul(W30_EXACT_HIT_CALIBRATION_CYCLES);
    let mut base = state.clone();
    base.variation = W30ResampleTapVariation::Base;
    base.hard_calibration = W30ResampleHardCalibrationPlan::default();
    base.hard_gesture = W30ResampleHardGesturePlan::default();
    base.position_beats = 0.0;
    base.is_transport_running = true;
    let mut calibration_state = state.clone();
    calibration_state.position_beats = 0.0;
    calibration_state.is_transport_running = true;
    let base_audio = render_w30_resample_tap_offline(
        &base,
        W30_EXACT_HIT_CALIBRATION_SAMPLE_RATE,
        W30_EXACT_HIT_CALIBRATION_CHANNELS,
        frame_count,
    );
    if base_audio.iter().all(|sample| sample.abs() <= f32::EPSILON) {
        return;
    }

    let schema_audio = w30_exact_hit_render(
        &calibration_state,
        W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN,
        frame_count,
    );
    let low_audio = w30_exact_hit_render(
        &calibration_state,
        W30_EXACT_HIT_MIN_OUTPUT_GAIN,
        frame_count,
    );
    let metric_model =
        w30_exact_hit_metric_model(&base_audio, &low_audio, &schema_audio, &calibration_state);
    let schema_metrics = metric_model.metrics(W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN);
    let mut selected_gain = W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN;

    if schema_metrics.level_ratio > W30_EXACT_HIT_TARGET_LEVEL_RATIO {
        let mut low = W30_EXACT_HIT_MIN_OUTPUT_GAIN;
        let mut high = W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN;
        let low_metrics = metric_model.metrics(low);
        if low_metrics.level_ratio <= W30_EXACT_HIT_TARGET_LEVEL_RATIO {
            for _ in 0..W30_EXACT_HIT_SEARCH_STEPS {
                let midpoint = (low + high) * 0.5;
                let midpoint_metrics = metric_model.metrics(midpoint);
                if midpoint_metrics.level_ratio <= W30_EXACT_HIT_TARGET_LEVEL_RATIO {
                    low = midpoint;
                } else {
                    high = midpoint;
                }
            }
            selected_gain = low;
        } else {
            selected_gain = low;
        }
    }

    let selected_audio = w30_exact_hit_render(&calibration_state, selected_gain, frame_count);
    let mut selected_metrics =
        w30_exact_hit_direct_metrics(&base_audio, &selected_audio, &calibration_state);
    if selected_metrics.level_ratio > W30_EXACT_HIT_TARGET_LEVEL_RATIO {
        let low_metrics =
            w30_exact_hit_direct_metrics(&base_audio, &low_audio, &calibration_state);
        if low_metrics.level_ratio > W30_EXACT_HIT_MAX_LEVEL_RATIO {
            return;
        }
        if low_metrics.level_ratio <= W30_EXACT_HIT_TARGET_LEVEL_RATIO {
            let mut lower_gain = W30_EXACT_HIT_MIN_OUTPUT_GAIN;
            let mut lower_metrics = low_metrics;
            let mut upper_gain = selected_gain;
            for _ in 0..W30_EXACT_HIT_REFINEMENT_STEPS {
                let candidate_gain = (lower_gain + upper_gain) * 0.5;
                let candidate_audio =
                    w30_exact_hit_render(&calibration_state, candidate_gain, frame_count);
                let candidate_metrics =
                    w30_exact_hit_direct_metrics(&base_audio, &candidate_audio, &calibration_state);
                if candidate_metrics.level_ratio <= W30_EXACT_HIT_TARGET_LEVEL_RATIO {
                    lower_gain = candidate_gain;
                    lower_metrics = candidate_metrics;
                } else {
                    upper_gain = candidate_gain;
                }
            }
            selected_gain = lower_gain;
            selected_metrics = lower_metrics;
        } else {
            selected_gain = W30_EXACT_HIT_MIN_OUTPUT_GAIN;
            selected_metrics = low_metrics;
        }
    }
    if selected_metrics.head_ratio < W30_EXACT_HIT_MIN_HEAD_RATIO
        || selected_metrics.body_ratio < W30_EXACT_HIT_MIN_BODY_RATIO
    {
        let schema_direct_metrics =
            w30_exact_hit_direct_metrics(&base_audio, &schema_audio, &calibration_state);
        if schema_direct_metrics.head_ratio < W30_EXACT_HIT_MIN_HEAD_RATIO
            || schema_direct_metrics.body_ratio < W30_EXACT_HIT_MIN_BODY_RATIO
        {
            return;
        }
        let mut lower_gain = selected_gain;
        let mut upper_gain = W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN;
        let mut upper_metrics = schema_direct_metrics;
        for _ in 0..W30_EXACT_HIT_REFINEMENT_STEPS {
            let candidate_gain = (lower_gain + upper_gain) * 0.5;
            let candidate_audio =
                w30_exact_hit_render(&calibration_state, candidate_gain, frame_count);
            let candidate_metrics =
                w30_exact_hit_direct_metrics(&base_audio, &candidate_audio, &calibration_state);
            if candidate_metrics.head_ratio >= W30_EXACT_HIT_MIN_HEAD_RATIO
                && candidate_metrics.body_ratio >= W30_EXACT_HIT_MIN_BODY_RATIO
            {
                upper_gain = candidate_gain;
                upper_metrics = candidate_metrics;
            } else {
                lower_gain = candidate_gain;
            }
        }
        selected_gain = upper_gain;
        selected_metrics = upper_metrics;
    }
    if selected_metrics.level_ratio > W30_EXACT_HIT_MAX_LEVEL_RATIO {
        return;
    }

    state.hard_calibration.predicted_raw_level_ratio = schema_metrics.level_ratio
        / W30_RESAMPLE_HIT_SHAPER_SCHEMA_OUTPUT_GAIN.max(f32::EPSILON);
    state
        .hard_calibration
        .predicted_compensated_level_ratio = selected_metrics.level_ratio;
    state
        .hard_calibration
        .predicted_level_matched_body_ratio = selected_metrics.body_ratio;
    state.hard_calibration.output_gain = selected_gain;
    state.hard_calibration.hit_window_compensation_gain =
        (W30_RESAMPLE_HIT_SHAPER_PRESERVED_OUTPUT_GAIN / selected_gain.max(f32::EPSILON))
            .clamp(1.0, W30_RESAMPLE_HIT_SHAPER_MAX_WINDOW_COMPENSATION_GAIN);
    state.hard_calibration.exact_callback_calibrated = true;
}

pub(super) fn build_w30_resample_tap_state(
    session: &SessionFile,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
    capture_audio_cache: Option<&BTreeMap<CaptureId, SourceAudioCache>>,
    previous: Option<&W30ResampleTapState>,
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
    let (variation, variation_revision, variation_intensity) =
        w30_resample_tap_variation(session, capture);
    let grit_level = session.runtime_state.macro_state.w30_grit.clamp(0.0, 1.0);
    let source_projection = build_w30_capture_artifact_resample_source(
        capture,
        capture_audio_cache,
        grit_level,
        variation_intensity,
    );
    let (
        source_audio,
        hard_policy,
        hard_suitability,
        hard_calibration,
        hard_trigger_mask,
        hard_slice_cursors,
        hard_attack_lengths,
        hard_attack_bite,
        hard_low_impact,
        hard_gesture,
        hard_transient_contrast,
    ) =
        match source_projection {
            Some(projection) => (
                Some(projection.audio),
                projection.hard_policy,
                projection.hard_suitability,
                projection.hard_calibration,
                projection.hard_trigger_mask,
                projection.hard_slice_cursors,
                projection.hard_attack_lengths,
                projection.hard_attack_bite,
                projection.hard_low_impact,
                projection.hard_gesture,
                projection.hard_transient_contrast,
            ),
            None => (
                None,
                W30ResampleTapHardPolicy::Unavailable,
                W30ResampleHardSuitabilityPlan::default(),
                W30ResampleHardCalibrationPlan::default(),
                0,
                [0; W30_RESAMPLE_HARD_SLICE_COUNT],
                [0; W30_RESAMPLE_HARD_SLICE_COUNT],
                W30ResampleAttackBitePlan::default(),
                W30ResampleLowImpactPlan::default(),
                W30ResampleHardGesturePlan::default(),
                0.0,
            ),
        };
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

    let mut state = W30ResampleTapState {
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
        hard_suitability,
        hard_calibration,
        hard_trigger_mask,
        hard_slice_cursors,
        hard_attack_lengths,
        hard_attack_bite,
        hard_low_impact,
        hard_gesture,
        hard_transient_contrast,
        music_bus_level: session
            .runtime_state
            .mixer_state
            .music_level
            .clamp(0.0, 1.0),
        grit_level,
        is_transport_running: transport.is_playing,
        tempo_bpm: trusted_source_timing_bpm(session, source_graph).unwrap_or(0.0),
        position_beats: transport.position_beats,
    };
    calibrate_w30_hit_shaper_exact_callback(&mut state, previous);
    state
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
