#[derive(Clone, Copy, Debug, PartialEq)]
struct W30SourceChopProfile {
    source_window_rms: f32,
    selected_rms_before_gain: f32,
    preview_rms: f32,
    preview_peak_abs: f32,
    body_rms: f32,
    tail_rms: f32,
    tail_to_body_rms_ratio: f32,
    selected_start_frame: u64,
    selected_frame_count: usize,
    gain: f32,
    reason: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct W30SourceLoopClosureProof {
    passed: bool,
    selected_frame_count: usize,
    preview_rms: f32,
    edge_delta_abs: f32,
    max_allowed_edge_delta_abs: f32,
    edge_abs_max: f32,
    max_allowed_edge_abs: f32,
    source_contains_selection: bool,
    reason: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct W30SourceTriggerVariationProof {
    applied: bool,
    grid_subdivision: u32,
    trigger_count: u32,
    beat_anchor_trigger_count: u32,
    offbeat_trigger_count: u32,
    skipped_beat_anchor_count: u32,
    distinct_bar_pattern_count: usize,
    max_quantized_offset_ms: f32,
    max_allowed_quantized_offset_ms: f32,
    reason: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct W30SourceTriggerEvent {
    beat_position: f32,
    velocity: f32,
    source_offset_samples: usize,
}

#[derive(Serialize)]
struct ManifestW30SourceChopProfile {
    source_window_rms: f32,
    selected_rms_before_gain: f32,
    preview_rms: f32,
    preview_peak_abs: f32,
    body_rms: f32,
    tail_rms: f32,
    tail_to_body_rms_ratio: f32,
    selected_start_frame: u64,
    selected_frame_count: usize,
    gain: f32,
    reason: &'static str,
}

#[derive(Serialize)]
struct ManifestW30SourceLoopClosureProof {
    passed: bool,
    selected_frame_count: usize,
    preview_rms: f32,
    edge_delta_abs: f32,
    max_allowed_edge_delta_abs: f32,
    edge_abs_max: f32,
    max_allowed_edge_abs: f32,
    source_contains_selection: bool,
    reason: &'static str,
}

#[derive(Serialize)]
struct ManifestW30SourceTriggerVariationProof {
    applied: bool,
    grid_subdivision: u32,
    trigger_count: u32,
    beat_anchor_trigger_count: u32,
    offbeat_trigger_count: u32,
    skipped_beat_anchor_count: u32,
    distinct_bar_pattern_count: usize,
    max_quantized_offset_ms: f32,
    max_allowed_quantized_offset_ms: f32,
    reason: &'static str,
}

const W30_SOURCE_LOOP_CLOSURE_MAX_EDGE_DELTA_ABS: f32 = 0.060;
const W30_SOURCE_LOOP_CLOSURE_MAX_EDGE_ABS: f32 = 0.040;
const W30_SOURCE_TRIGGER_GRID_SUBDIVISION: u32 = 2;
const W30_SOURCE_TRIGGER_MAX_QUANTIZED_OFFSET_MS: f32 = 0.01;

fn source_chop_preview_from_interleaved(
    samples: &[f32],
    channel_count: usize,
    source_start_frame: u64,
    source_end_frame: u64,
) -> Option<(W30PreviewSampleWindow, W30SourceChopProfile)> {
    let mono = mono_frames(samples, channel_count);
    if mono.is_empty() {
        return None;
    }

    let selected_frame_count = mono.len().min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    let selected_start = select_articulate_segment(&mono, selected_frame_count);
    let selected_end = selected_start + selected_frame_count;
    let selected = &mono[selected_start..selected_end];
    let selected_rms = rms(selected);
    let gain = source_chop_gain(selected_rms);

    let mut preview_samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    for (index, sample) in selected.iter().enumerate() {
        let fade = edge_fade(index, selected_frame_count);
        let articulation = chop_articulation_envelope(index, selected_frame_count);
        let previous = if index == 0 { *sample } else { selected[index - 1] };
        let transient = (*sample - previous) * 0.28;
        preview_samples[index] = ((*sample + transient) * gain * fade * articulation)
            .clamp(-0.95, 0.95);
    }

    let preview = W30PreviewSampleWindow {
        source_start_frame: source_start_frame + selected_start as u64,
        source_end_frame: (source_start_frame + selected_end as u64).min(source_end_frame),
        sample_count: selected_frame_count,
        samples: preview_samples,
    };
    let preview_slice = &preview.samples[..preview.sample_count];
    let preview_rms = rms(preview_slice);
    let preview_peak_abs = peak_abs(preview_slice);
    let (body_rms, tail_rms, tail_to_body_rms_ratio) = chop_articulation_metrics(preview_slice);
    let preview_start_frame = preview.source_start_frame;

    Some((
        preview,
        W30SourceChopProfile {
            source_window_rms: rms(&mono),
            selected_rms_before_gain: selected_rms,
            preview_rms,
            preview_peak_abs,
            body_rms,
            tail_rms,
            tail_to_body_rms_ratio,
            selected_start_frame: preview_start_frame,
            selected_frame_count,
            gain,
            reason: if selected_start == 0 {
                "source_window_head"
            } else {
                "source_articulate_segment"
            },
        },
    ))
}

fn w30_source_loop_closure_proof(
    preview: &W30PreviewSampleWindow,
    profile: W30SourceChopProfile,
) -> W30SourceLoopClosureProof {
    let source_contains_selection = preview.source_end_frame >= preview.source_start_frame
        && preview.source_end_frame.saturating_sub(preview.source_start_frame)
            == preview.sample_count as u64
        && preview.sample_count == profile.selected_frame_count;
    let preview_samples = &preview.samples[..preview.sample_count];
    let (edge_delta_abs, edge_abs_max) = edge_closure_metrics(preview_samples);
    let passed = profile.preview_rms > MIN_SIGNAL_RMS
        && profile.selected_frame_count > 0
        && source_contains_selection
        && edge_delta_abs <= W30_SOURCE_LOOP_CLOSURE_MAX_EDGE_DELTA_ABS
        && edge_abs_max <= W30_SOURCE_LOOP_CLOSURE_MAX_EDGE_ABS;

    W30SourceLoopClosureProof {
        passed,
        selected_frame_count: profile.selected_frame_count,
        preview_rms: profile.preview_rms,
        edge_delta_abs,
        max_allowed_edge_delta_abs: W30_SOURCE_LOOP_CLOSURE_MAX_EDGE_DELTA_ABS,
        edge_abs_max,
        max_allowed_edge_abs: W30_SOURCE_LOOP_CLOSURE_MAX_EDGE_ABS,
        source_contains_selection,
        reason: if passed {
            "source_chop_edges_faded_and_repeat_safe"
        } else {
            "source_chop_loop_closure_out_of_budget"
        },
    }
}

fn manifest_w30_source_chop_profile(
    profile: W30SourceChopProfile,
) -> ManifestW30SourceChopProfile {
    ManifestW30SourceChopProfile {
        source_window_rms: profile.source_window_rms,
        selected_rms_before_gain: profile.selected_rms_before_gain,
        preview_rms: profile.preview_rms,
        preview_peak_abs: profile.preview_peak_abs,
        body_rms: profile.body_rms,
        tail_rms: profile.tail_rms,
        tail_to_body_rms_ratio: profile.tail_to_body_rms_ratio,
        selected_start_frame: profile.selected_start_frame,
        selected_frame_count: profile.selected_frame_count,
        gain: profile.gain,
        reason: profile.reason,
    }
}

fn manifest_w30_source_loop_closure_proof(
    proof: W30SourceLoopClosureProof,
) -> ManifestW30SourceLoopClosureProof {
    ManifestW30SourceLoopClosureProof {
        passed: proof.passed,
        selected_frame_count: proof.selected_frame_count,
        preview_rms: proof.preview_rms,
        edge_delta_abs: proof.edge_delta_abs,
        max_allowed_edge_delta_abs: proof.max_allowed_edge_delta_abs,
        edge_abs_max: proof.edge_abs_max,
        max_allowed_edge_abs: proof.max_allowed_edge_abs,
        source_contains_selection: proof.source_contains_selection,
        reason: proof.reason,
    }
}

fn manifest_w30_source_trigger_variation_proof(
    proof: W30SourceTriggerVariationProof,
) -> ManifestW30SourceTriggerVariationProof {
    ManifestW30SourceTriggerVariationProof {
        applied: proof.applied,
        grid_subdivision: proof.grid_subdivision,
        trigger_count: proof.trigger_count,
        beat_anchor_trigger_count: proof.beat_anchor_trigger_count,
        offbeat_trigger_count: proof.offbeat_trigger_count,
        skipped_beat_anchor_count: proof.skipped_beat_anchor_count,
        distinct_bar_pattern_count: proof.distinct_bar_pattern_count,
        max_quantized_offset_ms: proof.max_quantized_offset_ms,
        max_allowed_quantized_offset_ms: proof.max_allowed_quantized_offset_ms,
        reason: proof.reason,
    }
}

fn w30_source_trigger_events_with_slice_plan(
    grid: &Grid,
    slice_plan: &W30SourceSliceChoicePlan,
) -> Vec<W30SourceTriggerEvent> {
    let mut events = Vec::with_capacity(grid.total_beats as usize + grid.bars as usize);

    for bar in 0..grid.bars {
        let bar_start = bar.saturating_mul(grid.beats_per_bar) as f32;
        let pattern = bar % 4;
        let positions: &[(f32, f32, usize)] = match pattern {
            0 => &[(0.0, 0.94, 0), (1.0, 0.78, 1), (2.0, 0.88, 2), (3.0, 0.72, 3)],
            1 => &[(0.0, 0.96, 0), (0.5, 0.66, 4), (2.0, 0.84, 2), (3.0, 0.76, 5)],
            2 => &[(0.0, 0.92, 1), (1.0, 0.74, 3), (2.5, 0.72, 5), (3.0, 0.86, 0)],
            _ => &[(0.0, 0.98, 0), (1.5, 0.70, 4), (2.0, 0.82, 2), (3.5, 0.68, 6)],
        };

        for (beat_offset, velocity, source_stride) in positions {
            events.push(W30SourceTriggerEvent {
                beat_position: bar_start + beat_offset,
                velocity: *velocity,
                source_offset_samples: slice_plan.offset_for_stride(*source_stride),
            });
        }
    }

    events
}

fn w30_source_trigger_variation_proof(
    grid: &Grid,
    events: &[W30SourceTriggerEvent],
) -> W30SourceTriggerVariationProof {
    let beat_anchor_trigger_count = events
        .iter()
        .filter(|event| is_beat_anchor(event.beat_position))
        .count() as u32;
    let offbeat_trigger_count = events.len() as u32 - beat_anchor_trigger_count;
    let skipped_beat_anchor_count = grid
        .total_beats
        .saturating_sub(beat_anchor_trigger_count.min(grid.total_beats));
    let distinct_bar_pattern_count = grid.bars.min(4) as usize;
    let max_quantized_offset_ms = events
        .iter()
        .map(|event| quantized_offset_ms(event.beat_position, grid.bpm))
        .fold(0.0_f32, f32::max);
    let applied = offbeat_trigger_count > 0
        && distinct_bar_pattern_count > 1
        && max_quantized_offset_ms <= W30_SOURCE_TRIGGER_MAX_QUANTIZED_OFFSET_MS;

    W30SourceTriggerVariationProof {
        applied,
        grid_subdivision: W30_SOURCE_TRIGGER_GRID_SUBDIVISION,
        trigger_count: events.len() as u32,
        beat_anchor_trigger_count,
        offbeat_trigger_count,
        skipped_beat_anchor_count,
        distinct_bar_pattern_count,
        max_quantized_offset_ms,
        max_allowed_quantized_offset_ms: W30_SOURCE_TRIGGER_MAX_QUANTIZED_OFFSET_MS,
        reason: if applied {
            "source_grid_locked_trigger_variation"
        } else {
            "source_trigger_variation_not_applied"
        },
    }
}

fn mono_frames(samples: &[f32], channel_count: usize) -> Vec<f32> {
    let channel_count = channel_count.max(1);
    samples
        .chunks_exact(channel_count)
        .map(|frame| frame.iter().sum::<f32>() / channel_count as f32)
        .collect()
}

fn select_articulate_segment(mono: &[f32], window_len: usize) -> usize {
    if mono.len() <= window_len {
        return 0;
    }

    let hop = (window_len / 4).max(1);
    let mut best_start = 0;
    let mut best_score = f32::MIN;
    let final_start = mono.len() - window_len;
    for start in (0..=mono.len() - window_len).step_by(hop) {
        let score = articulate_segment_score(&mono[start..start + window_len]);
        if score > best_score {
            best_score = score;
            best_start = start;
        }
    }
    let final_score = articulate_segment_score(&mono[final_start..final_start + window_len]);
    if final_score > best_score {
        best_start = final_start;
    }
    best_start
}

fn articulate_segment_score(samples: &[f32]) -> f32 {
    rms(samples) + positive_abs_delta(samples) * 0.35 + peak_abs(samples) * 0.05
}

fn source_chop_gain(selected_rms: f32) -> f32 {
    if selected_rms <= f32::EPSILON {
        return 1.0;
    }
    (0.18 / selected_rms).clamp(0.85, 1.70)
}

fn positive_abs_delta(samples: &[f32]) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for pair in samples.windows(2) {
        total += (pair[1].abs() - pair[0].abs()).max(0.0);
    }
    total / (samples.len() - 1) as f32
}

fn edge_closure_metrics(samples: &[f32]) -> (f32, f32) {
    let Some(first) = samples.first() else {
        return (0.0, 0.0);
    };
    let last = samples.last().copied().unwrap_or(*first);
    let edge_delta_abs = (first - last).abs();
    let edge_abs_max = first.abs().max(last.abs());
    (edge_delta_abs, edge_abs_max)
}

fn edge_fade(index: usize, sample_count: usize) -> f32 {
    let fade_len = 32.min(sample_count / 2);
    if fade_len == 0 {
        return 1.0;
    }
    let fade_in = ((index + 1) as f32 / fade_len as f32).min(1.0);
    let fade_out = ((sample_count - index) as f32 / fade_len as f32).min(1.0);
    fade_in.min(fade_out)
}

fn chop_articulation_envelope(index: usize, sample_count: usize) -> f32 {
    if sample_count <= 1 {
        return 1.0;
    }
    let position = index as f32 / (sample_count - 1) as f32;
    let attack = (position / 0.035).clamp(0.0, 1.0);
    let body = if position <= 0.58 {
        1.0
    } else {
        ((1.0 - position) / 0.42).clamp(0.0, 1.0).powf(0.72)
    };
    attack * (0.22 + body * 0.78)
}

fn chop_articulation_metrics(samples: &[f32]) -> (f32, f32, f32) {
    if samples.len() < 8 {
        return (0.0, 0.0, 0.0);
    }
    let body_start = (samples.len() / 8).min(samples.len() - 1);
    let body_end = (samples.len() / 4).max(body_start + 1).min(samples.len());
    let tail_start = (samples.len() * 3 / 4).min(samples.len() - 1);
    let tail_end = (samples.len() * 15 / 16).max(tail_start + 1).min(samples.len());
    let body_rms = rms(&samples[body_start..body_end]);
    let tail_rms = rms(&samples[tail_start..tail_end]);
    (body_rms, tail_rms, tail_rms / body_rms.max(f32::EPSILON))
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
}

fn peak_abs(samples: &[f32]) -> f32 {
    samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f32, f32::max)
}

fn is_beat_anchor(beat_position: f32) -> bool {
    (beat_position - beat_position.round()).abs() <= f32::EPSILON
}

fn quantized_offset_ms(beat_position: f32, bpm: f32) -> f32 {
    let subdivision = W30_SOURCE_TRIGGER_GRID_SUBDIVISION as f32;
    let quantized = (beat_position * subdivision).round() / subdivision;
    (beat_position - quantized).abs() * 60_000.0 / bpm.max(f32::EPSILON)
}
