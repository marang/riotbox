//! Existing bounded QA window search/selection over a decoded in-memory cache.
//! No source-file I/O or product source-qualification authority lives here.

use super::args::Args;
use super::grid::Grid;
use super::sample_measurements::{mono_frames, peak_abs, positive_abs_delta, rms};
use riotbox_audio::source_audio::{SourceAudioCache, SourceAudioWindow};
use riotbox_audio::w30::W30_PREVIEW_SAMPLE_WINDOW_LEN;

pub(super) const SOURCE_CHARACTER_MIN_SCORE_LIFT: f32 = 0.0025;

pub(super) const SOURCE_CHARACTER_MIN_RMS_RETENTION: f32 = 0.98;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub(super) struct SourceCharacterWindowSelection {
    requested_start_seconds: f32,
    pub(super) requested_duration_seconds: f32,
    search_start_seconds: f32,
    pub(super) search_duration_seconds: f32,
    pub(super) selected_start_seconds: f32,
    pub(super) selected_duration_seconds: f32,
    selected_start_frame: u64,
    selected_frame_count: usize,
    pub(super) requested_head_score: f32,
    pub(super) selected_score: f32,
    pub(super) score_lift: f32,
    pub(super) requested_head_rms: f32,
    pub(super) selected_rms: f32,
    pub(super) rms_retention_ratio: f32,
    pub(super) min_rms_retention_ratio: f32,
    pub(super) scanned_candidate_count: u32,
    pub(super) reason: &'static str,
}

pub(super) fn select_source_character_window(
    source: &SourceAudioCache,
    requested: SourceAudioWindow,
    search: SourceAudioWindow,
) -> (SourceAudioWindow, SourceCharacterWindowSelection) {
    let requested_duration_seconds = requested.frame_count as f32 / source.sample_rate as f32;
    let requested_start_seconds = requested.start_frame as f32 / source.sample_rate as f32;
    let search_start_seconds = search.start_frame as f32 / source.sample_rate as f32;
    let search_duration_seconds = search.frame_count as f32 / source.sample_rate as f32;
    let candidate_frame_count = requested.frame_count.min(search.frame_count);
    if candidate_frame_count == 0 {
        return (
            requested,
            SourceCharacterWindowSelection {
                requested_start_seconds,
                requested_duration_seconds,
                search_start_seconds,
                search_duration_seconds,
                selected_start_seconds: requested_start_seconds,
                selected_duration_seconds: requested_duration_seconds,
                selected_start_frame: requested.start_frame as u64,
                selected_frame_count: requested.frame_count,
                requested_head_score: 0.0,
                selected_score: 0.0,
                score_lift: 0.0,
                requested_head_rms: 0.0,
                selected_rms: 0.0,
                rms_retention_ratio: 1.0,
                min_rms_retention_ratio: SOURCE_CHARACTER_MIN_RMS_RETENTION,
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
    let selected_rms = source_character_window_rms(
        source,
        selected_window.start_frame,
        selected_window.frame_count,
    );
    let rms_retention_ratio =
        source_character_rms_retention_ratio(selected_rms, requested_head_rms);
    let selected_start_seconds = selected_window.start_frame as f32 / source.sample_rate as f32;
    let selected_duration_seconds = selected_window.frame_count as f32 / source.sample_rate as f32;

    (
        selected_window,
        SourceCharacterWindowSelection {
            requested_start_seconds,
            requested_duration_seconds,
            search_start_seconds,
            search_duration_seconds,
            selected_start_seconds,
            selected_duration_seconds,
            selected_start_frame: selected_window.start_frame as u64,
            selected_frame_count: selected_window.frame_count,
            requested_head_score,
            selected_score,
            score_lift: selected_score - requested_head_score,
            requested_head_rms,
            selected_rms,
            rms_retention_ratio,
            min_rms_retention_ratio: SOURCE_CHARACTER_MIN_RMS_RETENTION,
            scanned_candidate_count,
            reason: if selected_window.start_frame == requested_start {
                "requested_source_window_kept"
            } else {
                "source_character_window_promoted"
            },
        },
    )
}

fn source_character_rms_retention_ratio(selected_rms: f32, requested_head_rms: f32) -> f32 {
    if requested_head_rms <= f32::EPSILON {
        1.0
    } else {
        selected_rms / requested_head_rms
    }
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
    let active_ratio = mono
        .iter()
        .filter(|sample| sample.abs() >= active_floor)
        .count() as f32
        / mono.len() as f32;
    let crest = if rms_value > f32::EPSILON {
        peak / rms_value
    } else {
        0.0
    }
    .min(8.0);

    rms_value * 0.55 + transient * 1.20 + peak * 0.10 + active_ratio * 0.015 + crest * 0.002
}

pub(super) fn source_character_search_window(
    source: &SourceAudioCache,
    args: &Args,
    grid: &Grid,
) -> SourceAudioWindow {
    let available_seconds =
        (source.duration_seconds() - args.source_start_seconds).max(args.source_window_seconds);
    source.window_by_seconds(
        args.source_start_seconds,
        available_seconds.max(grid.duration_seconds()),
    )
}
