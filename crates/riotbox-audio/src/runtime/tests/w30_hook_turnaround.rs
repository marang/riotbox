use crate::runtime::tests::signal_test_helpers::region_delta_rms;
use crate::runtime::tests::w30_gesture_fixtures::{
    hook_turnaround_test_render, render_hook_turnaround_in_chunks,
};
use crate::w30::W30HookArticulationProfile;

#[test]
fn w30_hook_turnaround_changes_only_the_frozen_middle_and_returns_cleanly() {
    const FRAMES_PER_BEAT: usize = 24_000;
    const TOTAL_FRAMES: usize = 120_000;
    let control =
        render_hook_turnaround_in_chunks(&hook_turnaround_test_render(None), 128, TOTAL_FRAMES);
    let candidate = render_hook_turnaround_in_chunks(
        &hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
            profile: W30HookArticulationProfile::TurnaroundV1,
            started_at_beat: 4,
        })),
        128,
        TOTAL_FRAMES,
    );

    assert_eq!(&candidate[..FRAMES_PER_BEAT], &control[..FRAMES_PER_BEAT]);
    assert!(
        region_delta_rms(
            &candidate[FRAMES_PER_BEAT..3 * FRAMES_PER_BEAT],
            &control[FRAMES_PER_BEAT..3 * FRAMES_PER_BEAT],
        ) > 0.05,
        "the two-beat reverse window must be causally distinct"
    );
    assert!(
        region_delta_rms(
            &candidate[3 * FRAMES_PER_BEAT..4 * FRAMES_PER_BEAT],
            &control[3 * FRAMES_PER_BEAT..4 * FRAMES_PER_BEAT],
        ) > 0.03,
        "the one-beat choke window must be causally distinct"
    );
    let return_candidate = &candidate[4 * FRAMES_PER_BEAT..];
    let return_control = &control[4 * FRAMES_PER_BEAT..];
    let first_return_difference = return_candidate
        .iter()
        .zip(return_control)
        .position(|(left, right)| left != right);
    let differing_return_frames = return_candidate
        .iter()
        .zip(return_control)
        .filter(|(left, right)| left != right)
        .count();
    let last_return_difference = return_candidate
        .iter()
        .zip(return_control)
        .rposition(|(left, right)| left != right);
    assert!(
        first_return_difference.is_none(),
        "ordinary source playback must return exactly on beat four; first: {first_return_difference:?}, last: {last_return_difference:?}, count: {differing_return_frames}, delta RMS: {}",
        region_delta_rms(return_candidate, return_control),
    );
    assert!(candidate.iter().all(|sample| sample.is_finite()));
    assert!(candidate.iter().all(|sample| sample.abs() <= 1.0));
}

#[test]
fn w30_hook_turnaround_is_callback_partition_invariant() {
    const TOTAL_FRAMES: usize = 120_000;
    let render = hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
        profile: W30HookArticulationProfile::TurnaroundV1,
        started_at_beat: 4,
    }));

    assert_eq!(
        render_hook_turnaround_in_chunks(&render, 128, TOTAL_FRAMES),
        render_hook_turnaround_in_chunks(&render, 257, TOTAL_FRAMES),
    );
}

#[test]
fn w30_hook_turnaround_returns_on_the_cumulative_boundary_at_non_integer_tempo() {
    const TEMPO_BPM: f32 = 172.26566;
    const SAMPLE_RATE: f64 = 48_000.0;
    const START_BEAT: f64 = 4.0;
    const RETURN_BEAT: f64 = 8.0;
    const BOUNDARY_SNAP_BEATS: f64 = 1.0e-9;

    let beats_per_frame = f64::from(TEMPO_BPM) / 60.0 / SAMPLE_RATE;
    let maximum_frames = (5.0 / beats_per_frame).ceil() as usize;
    let mut position_beats = START_BEAT;
    let return_frame = (0..=maximum_frames)
        .find(|_| {
            let reached = position_beats >= RETURN_BEAT
                || (position_beats - RETURN_BEAT).abs() <= BOUNDARY_SNAP_BEATS;
            position_beats += beats_per_frame;
            reached
        })
        .expect("the four-beat return boundary must be reachable");

    let mut control_render = hook_turnaround_test_render(None);
    control_render.tempo_bpm = TEMPO_BPM;
    let mut candidate_render =
        hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
            profile: W30HookArticulationProfile::TurnaroundV1,
            started_at_beat: 4,
        }));
    candidate_render.tempo_bpm = TEMPO_BPM;
    let control = render_hook_turnaround_in_chunks(&control_render, 128, maximum_frames);
    let candidate = render_hook_turnaround_in_chunks(&candidate_render, 128, maximum_frames);

    assert_ne!(
        candidate[return_frame - 1],
        control[return_frame - 1],
        "the choke window must remain active immediately before the cumulative boundary"
    );
    assert_eq!(
        &candidate[return_frame..],
        &control[return_frame..],
        "ordinary playback must be sample-exact from the cumulative four-beat boundary"
    );
}
