use crate::source_graph::timing_probe_candidates::period_scoring::{
    BeatPeriodScore, period_score_order,
};

#[test]
fn source_timing_probe_period_score_order_is_total_for_close_scores() {
    let weak_close = BeatPeriodScore {
        bpm: 120.0,
        period_seconds: 0.50,
        score: 0.0,
        matched_onset_ratio: 0.90,
        median_distance_ratio: 0.10,
    };
    let middle_close = BeatPeriodScore {
        bpm: 121.0,
        period_seconds: 0.496,
        score: 0.0005,
        matched_onset_ratio: 0.80,
        median_distance_ratio: 0.20,
    };
    let strong_close = BeatPeriodScore {
        bpm: 122.0,
        period_seconds: 0.492,
        score: 0.0015,
        matched_onset_ratio: 0.70,
        median_distance_ratio: 0.30,
    };
    let mut scores = [weak_close, middle_close, strong_close];

    scores.sort_by(|left, right| {
        period_score_order(left, right).then_with(|| left.bpm.total_cmp(&right.bpm))
    });

    assert_eq!(
        scores.iter().map(|score| score.bpm).collect::<Vec<_>>(),
        vec![122.0, 121.0, 120.0]
    );
}

#[test]
fn source_timing_probe_period_score_order_keeps_near_tie_period_preference() {
    let source_period = BeatPeriodScore {
        bpm: 120.0,
        period_seconds: 0.50,
        score: 0.9996,
        matched_onset_ratio: 1.0,
        median_distance_ratio: 0.0,
    };
    let double_time_period = BeatPeriodScore {
        bpm: 240.0,
        period_seconds: 0.25,
        score: 1.0,
        matched_onset_ratio: 1.0,
        median_distance_ratio: 0.50,
    };
    let mut scores = [double_time_period, source_period];

    scores.sort_by(|left, right| {
        period_score_order(left, right).then_with(|| left.bpm.total_cmp(&right.bpm))
    });

    assert_eq!(scores[0].bpm, 120.0);
}
