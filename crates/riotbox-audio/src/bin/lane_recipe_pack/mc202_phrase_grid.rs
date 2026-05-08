const MC202_PHRASE_GRID_RESOLUTION: &str = "sixteenth";
const MC202_PHRASE_LENGTH_STEPS: u32 = 64;
const MC202_PHRASE_LENGTH_BEATS: f64 = 16.0;
const MC202_PHRASE_GRID_MIN_HIT_RATIO: f64 = 0.95;
const MC202_PHRASE_GRID_MAX_ONSET_OFFSET_MS: f64 = 8.0;
const MC202_ONSET_THRESHOLD: f32 = 0.0001;
const MC202_ONSET_WINDOW_FRAMES: usize = 128;

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
struct Mc202PhraseGridTimingMetrics {
    resolution: &'static str,
    phrase_length_steps: u32,
    phrase_length_beats: f64,
    position_beats: f64,
    starts_on_phrase_boundary: bool,
    candidate_onset_count: usize,
    grid_aligned_onset_count: usize,
    hit_ratio: f64,
    max_onset_offset_ms: f64,
    max_allowed_onset_offset_ms: f64,
    passed: bool,
}

fn mc202_phrase_grid_metrics(
    render_pair: &RenderPair,
    candidate_samples: &[f32],
) -> Option<Mc202PhraseGridTimingMetrics> {
    let RenderPair::Mc202 { candidate, .. } = render_pair else {
        return None;
    };

    let onset_frames = detected_onset_frames(candidate_samples, usize::from(CHANNEL_COUNT));
    let step_frames = SAMPLE_RATE as f64 * 60.0 / candidate.tempo_bpm.max(1.0) as f64 / 4.0;
    let allowed_frames = MC202_PHRASE_GRID_MAX_ONSET_OFFSET_MS * SAMPLE_RATE as f64 / 1000.0;
    let offsets = onset_frames
        .iter()
        .map(|frame| nearest_grid_offset_frames(*frame as f64, step_frames))
        .collect::<Vec<_>>();
    let grid_aligned_onset_count = offsets
        .iter()
        .filter(|offset| **offset <= allowed_frames)
        .count();
    let hit_ratio = if onset_frames.is_empty() {
        0.0
    } else {
        grid_aligned_onset_count as f64 / onset_frames.len() as f64
    };
    let max_onset_offset_ms = offsets
        .into_iter()
        .fold(0.0_f64, f64::max)
        * 1000.0
        / SAMPLE_RATE as f64;
    let starts_on_phrase_boundary =
        nearest_grid_offset_beats(candidate.position_beats, MC202_PHRASE_LENGTH_BEATS) <= 0.0001;
    let passed = starts_on_phrase_boundary
        && !onset_frames.is_empty()
        && hit_ratio >= MC202_PHRASE_GRID_MIN_HIT_RATIO
        && max_onset_offset_ms <= MC202_PHRASE_GRID_MAX_ONSET_OFFSET_MS;

    Some(Mc202PhraseGridTimingMetrics {
        resolution: MC202_PHRASE_GRID_RESOLUTION,
        phrase_length_steps: MC202_PHRASE_LENGTH_STEPS,
        phrase_length_beats: MC202_PHRASE_LENGTH_BEATS,
        position_beats: candidate.position_beats,
        starts_on_phrase_boundary,
        candidate_onset_count: onset_frames.len(),
        grid_aligned_onset_count,
        hit_ratio,
        max_onset_offset_ms,
        max_allowed_onset_offset_ms: MC202_PHRASE_GRID_MAX_ONSET_OFFSET_MS,
        passed,
    })
}

fn detected_onset_frames(samples: &[f32], channel_count: usize) -> Vec<usize> {
    if samples.is_empty() || channel_count == 0 {
        return Vec::new();
    }

    let mut onsets = Vec::new();
    let mut was_active = false;
    let samples_per_window = MC202_ONSET_WINDOW_FRAMES * channel_count;
    for (window_index, window) in samples.chunks(samples_per_window).enumerate() {
        let is_active = window
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
            > MC202_ONSET_THRESHOLD;
        if is_active && !was_active {
            onsets.push(window_index * MC202_ONSET_WINDOW_FRAMES);
        }
        was_active = is_active;
    }
    onsets
}

fn nearest_grid_offset_frames(frame: f64, step_frames: f64) -> f64 {
    if step_frames <= 0.0 || !step_frames.is_finite() {
        return f64::INFINITY;
    }

    let nearest_step = (frame / step_frames).round() * step_frames;
    (frame - nearest_step).abs()
}

fn nearest_grid_offset_beats(position_beats: f64, grid_beats: f64) -> f64 {
    if grid_beats <= 0.0 || !grid_beats.is_finite() {
        return f64::INFINITY;
    }

    let nearest_boundary = (position_beats / grid_beats).round() * grid_beats;
    (position_beats - nearest_boundary).abs()
}
