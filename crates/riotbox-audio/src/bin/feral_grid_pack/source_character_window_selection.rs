const SOURCE_CHARACTER_MIN_SCORE_LIFT: f32 = 0.0025;
const SOURCE_CHARACTER_MIN_RMS_RETENTION: f32 = 0.98;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
struct SourceCharacterWindowSelection {
    requested_start_seconds: f32,
    requested_duration_seconds: f32,
    selected_start_seconds: f32,
    selected_duration_seconds: f32,
    selected_start_frame: u64,
    selected_frame_count: usize,
    requested_head_score: f32,
    selected_score: f32,
    score_lift: f32,
    scanned_candidate_count: u32,
    reason: &'static str,
}

fn select_source_character_window(
    source: &SourceAudioCache,
    requested: SourceAudioWindow,
    search: SourceAudioWindow,
) -> (SourceAudioWindow, SourceCharacterWindowSelection) {
    let requested_duration_seconds = requested.frame_count as f32 / source.sample_rate as f32;
    let requested_start_seconds = requested.start_frame as f32 / source.sample_rate as f32;
    let candidate_frame_count = requested.frame_count.min(search.frame_count);
    if candidate_frame_count == 0 {
        return (
            requested,
            SourceCharacterWindowSelection {
                requested_start_seconds,
                requested_duration_seconds,
                selected_start_seconds: requested_start_seconds,
                selected_duration_seconds: requested_duration_seconds,
                selected_start_frame: requested.start_frame as u64,
                selected_frame_count: requested.frame_count,
                requested_head_score: 0.0,
                selected_score: 0.0,
                score_lift: 0.0,
                scanned_candidate_count: 0,
                reason: "source_window_empty",
            },
        );
    }

    let search_end = search.start_frame.saturating_add(search.frame_count);
    let max_start = search_end.saturating_sub(candidate_frame_count);
    let requested_start = requested.start_frame.min(max_start);
    let requested_head_score =
        source_character_window_score(source, requested_start, candidate_frame_count);
    let requested_head_rms =
        source_character_window_rms(source, requested_start, candidate_frame_count);
    let hop = (candidate_frame_count / 4).max(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    let mut best_start = requested_start;
    let mut best_score = requested_head_score;
    let mut scanned_candidate_count = 0_u32;

    let mut start = search.start_frame.min(max_start);
    while start <= max_start {
        let score = source_character_window_score(source, start, candidate_frame_count);
        let candidate_rms = source_character_window_rms(source, start, candidate_frame_count);
        scanned_candidate_count = scanned_candidate_count.saturating_add(1);
        if candidate_rms >= requested_head_rms * SOURCE_CHARACTER_MIN_RMS_RETENTION
            && score > best_score
        {
            best_score = score;
            best_start = start;
        }
        if max_start - start < hop {
            break;
        }
        start += hop;
    }

    if max_start != start {
        let score = source_character_window_score(source, max_start, candidate_frame_count);
        let candidate_rms = source_character_window_rms(source, max_start, candidate_frame_count);
        scanned_candidate_count = scanned_candidate_count.saturating_add(1);
        if candidate_rms >= requested_head_rms * SOURCE_CHARACTER_MIN_RMS_RETENTION
            && score > best_score
        {
            best_score = score;
            best_start = max_start;
        }
    }

    let score_lift = best_score - requested_head_score;
    let selected_window = if score_lift >= SOURCE_CHARACTER_MIN_SCORE_LIFT {
        SourceAudioWindow {
            start_frame: best_start,
            frame_count: candidate_frame_count,
        }
    } else {
        SourceAudioWindow {
            start_frame: requested_start,
            frame_count: candidate_frame_count,
        }
    };
    let selected_score = if score_lift >= SOURCE_CHARACTER_MIN_SCORE_LIFT {
        best_score
    } else {
        requested_head_score
    };
    let selected_start_seconds = selected_window.start_frame as f32 / source.sample_rate as f32;
    let selected_duration_seconds = selected_window.frame_count as f32 / source.sample_rate as f32;

    (
        selected_window,
        SourceCharacterWindowSelection {
            requested_start_seconds,
            requested_duration_seconds,
            selected_start_seconds,
            selected_duration_seconds,
            selected_start_frame: selected_window.start_frame as u64,
            selected_frame_count: selected_window.frame_count,
            requested_head_score,
            selected_score,
            score_lift: selected_score - requested_head_score,
            scanned_candidate_count,
            reason: if selected_window.start_frame == requested_start {
                "requested_source_window_kept"
            } else {
                "source_character_window_promoted"
            },
        },
    )
}

fn source_character_window_rms(
    source: &SourceAudioCache,
    start_frame: usize,
    frame_count: usize,
) -> f32 {
    let window = SourceAudioWindow {
        start_frame,
        frame_count,
    };
    let mono = mono_frames(
        source.window_samples(window),
        usize::from(source.channel_count),
    );
    rms(&mono)
}

fn source_character_window_score(
    source: &SourceAudioCache,
    start_frame: usize,
    frame_count: usize,
) -> f32 {
    let window = SourceAudioWindow {
        start_frame,
        frame_count,
    };
    let mono = mono_frames(
        source.window_samples(window),
        usize::from(source.channel_count),
    );
    if mono.is_empty() {
        return 0.0;
    }
    let rms_value = rms(&mono);
    let transient = positive_abs_delta(&mono);
    let peak = peak_abs(&mono);
    let active_floor = (rms_value * 0.35).max(0.001);
    let active_ratio =
        mono.iter().filter(|sample| sample.abs() >= active_floor).count() as f32
            / mono.len() as f32;
    let crest = if rms_value > f32::EPSILON {
        peak / rms_value
    } else {
        0.0
    }
    .min(8.0);

    rms_value * 0.55 + transient * 1.20 + peak * 0.10 + active_ratio * 0.015 + crest * 0.002
}

#[cfg(test)]
mod source_character_window_selection_tests {
    use super::*;

    #[test]
    fn source_character_window_selection_promotes_late_transient_character() {
        let one_second = SAMPLE_RATE as usize;
        let mut samples = Vec::with_capacity(one_second * 2 * usize::from(CHANNEL_COUNT));
        for frame in 0..one_second * 2 {
            let phase = frame as f32 / SAMPLE_RATE as f32;
            let sample = if frame < one_second {
                (phase * 90.0 * std::f32::consts::TAU).sin() * 0.004
            } else {
                let local = frame - one_second;
                let pulse = local % 5_512;
                let transient = if pulse < 192 {
                    0.72 * (1.0 - pulse as f32 / 192.0)
                } else {
                    0.0
                };
                let grit = (phase * 1_900.0 * std::f32::consts::TAU).sin() * 0.065;
                transient + grit
            };
            samples.push(sample);
            samples.push(sample * 0.96);
        }
        let source = SourceAudioCache::from_interleaved_samples(
            "late-character.wav",
            SAMPLE_RATE,
            CHANNEL_COUNT,
            samples,
        )
        .expect("source");
        let requested = source.window_by_seconds(0.0, 1.0);
        let search = source.window_by_seconds(0.0, 2.0);

        let (selected, proof) = select_source_character_window(&source, requested, search);
        let (selected_repeat, proof_repeat) =
            select_source_character_window(&source, requested, search);

        assert_eq!(selected, selected_repeat);
        assert_eq!(proof, proof_repeat);
        assert!(selected.start_frame >= one_second, "{proof:?}");
        assert!(proof.score_lift >= SOURCE_CHARACTER_MIN_SCORE_LIFT);
        assert_eq!(proof.reason, "source_character_window_promoted");
        assert!(proof.selected_score > proof.requested_head_score);
        assert!(proof.scanned_candidate_count >= 2);
    }

    #[test]
    fn source_character_window_selection_rejects_transient_peak_with_weaker_rms_base() {
        let half_second = SAMPLE_RATE as usize / 2;
        let mut samples = Vec::with_capacity(half_second * 4 * usize::from(CHANNEL_COUNT));
        for frame in 0..half_second * 4 {
            let phase = frame as f32 / SAMPLE_RATE as f32;
            let sample = if frame < half_second {
                (phase * 140.0 * std::f32::consts::TAU).sin() * 0.20
            } else if frame >= half_second * 3 {
                let local = frame - half_second * 3;
                let pulse = local % 5_512;
                let transient = if pulse < 96 {
                    0.78 * (1.0 - pulse as f32 / 96.0)
                } else {
                    0.0
                };
                let grit = (phase * 1_700.0 * std::f32::consts::TAU).sin() * 0.045;
                transient + grit
            } else {
                (phase * 110.0 * std::f32::consts::TAU).sin() * 0.12
            };
            samples.push(sample);
            samples.push(sample * 0.96);
        }
        let source = SourceAudioCache::from_interleaved_samples(
            "late-peak-weaker-rms.wav",
            SAMPLE_RATE,
            CHANNEL_COUNT,
            samples,
        )
        .expect("source");
        let requested = source.window_by_seconds(0.0, 0.5);
        let search = source.window_by_seconds(0.0, 2.0);

        let (selected, proof) = select_source_character_window(&source, requested, search);

        assert_eq!(selected.start_frame, requested.start_frame, "{proof:?}");
        assert_eq!(proof.reason, "requested_source_window_kept");
        assert_eq!(proof.score_lift, 0.0);
        assert!(proof.scanned_candidate_count >= 2);
    }
}
