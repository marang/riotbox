use super::config::{CHANNEL_COUNT, SAMPLE_RATE};
use super::source_character_window_selection::{
    SOURCE_CHARACTER_MIN_RMS_RETENTION, SOURCE_CHARACTER_MIN_SCORE_LIFT,
    select_source_character_window,
};
use riotbox_audio::source_audio::SourceAudioCache;

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
    assert!(proof.rms_retention_ratio >= proof.min_rms_retention_ratio);
    assert!(proof.selected_rms >= proof.requested_head_rms * SOURCE_CHARACTER_MIN_RMS_RETENTION);
    assert!(proof.search_duration_seconds > proof.requested_duration_seconds);
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
    assert_eq!(proof.rms_retention_ratio, 1.0);
    assert!(proof.scanned_candidate_count >= 2);
}
