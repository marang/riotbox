#[derive(Clone, Copy, Debug, PartialEq)]
struct W30SourceChopProfile {
    source_window_rms: f32,
    selected_rms_before_gain: f32,
    preview_rms: f32,
    preview_peak_abs: f32,
    selected_start_frame: u64,
    selected_frame_count: usize,
    gain: f32,
    reason: &'static str,
}

#[derive(Serialize)]
struct ManifestW30SourceChopProfile {
    source_window_rms: f32,
    selected_rms_before_gain: f32,
    preview_rms: f32,
    preview_peak_abs: f32,
    selected_start_frame: u64,
    selected_frame_count: usize,
    gain: f32,
    reason: &'static str,
}

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
        preview_samples[index] = (sample * gain * fade).clamp(-0.95, 0.95);
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
    let preview_start_frame = preview.source_start_frame;

    Some((
        preview,
        W30SourceChopProfile {
            source_window_rms: rms(&mono),
            selected_rms_before_gain: selected_rms,
            preview_rms,
            preview_peak_abs,
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

fn manifest_w30_source_chop_profile(
    profile: W30SourceChopProfile,
) -> ManifestW30SourceChopProfile {
    ManifestW30SourceChopProfile {
        source_window_rms: profile.source_window_rms,
        selected_rms_before_gain: profile.selected_rms_before_gain,
        preview_rms: profile.preview_rms,
        preview_peak_abs: profile.preview_peak_abs,
        selected_start_frame: profile.selected_start_frame,
        selected_frame_count: profile.selected_frame_count,
        gain: profile.gain,
        reason: profile.reason,
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
    (0.18 / selected_rms).clamp(0.85, 5.0)
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

fn edge_fade(index: usize, sample_count: usize) -> f32 {
    let fade_len = 32.min(sample_count / 2);
    if fade_len == 0 {
        return 1.0;
    }
    let fade_in = ((index + 1) as f32 / fade_len as f32).min(1.0);
    let fade_out = ((sample_count - index) as f32 / fade_len as f32).min(1.0);
    fade_in.min(fade_out)
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
