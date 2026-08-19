#[test]
fn w30_filter_slam_uses_the_frozen_eight_beat_curve_and_twenty_ms_return() {
    let shared = SharedW30PreviewRenderState::new(&hook_turnaround_test_render(Some(
        crate::w30::W30HookArticulationRenderState {
            profile: W30HookArticulationProfile::FilterSlamV1,
            started_at_beat: 4,
        },
    )));
    let render = shared.snapshot();

    let start = crate::runtime::w30_filter_slam::w30_filter_slam_frame(
        &render, 4.0, 48_000,
    )
    .expect("filter-slam start frame");
    assert!((start.cutoff_hz - 14_000.0).abs() < 1.0e-9);
    assert!((start.q - 0.707).abs() < 1.0e-9);
    assert_eq!(start.wet_gain, 1.0);

    let first_midpoint = crate::runtime::w30_filter_slam::w30_filter_slam_frame(
        &render, 6.0, 48_000,
    )
    .expect("filter-slam first midpoint frame");
    assert!((first_midpoint.cutoff_hz - (14_000.0_f64 * 1_800.0).sqrt()).abs() < 1.0e-9);
    assert!((first_midpoint.q - 0.7785).abs() < 1.0e-9);

    let deep_close = crate::runtime::w30_filter_slam::w30_filter_slam_frame(
        &render, 10.0, 48_000,
    )
    .expect("filter-slam deep-close boundary");
    assert!((deep_close.cutoff_hz - 280.0).abs() < 1.0e-9);
    assert!((deep_close.q - 1.2).abs() < 1.0e-9);

    let half_return = crate::runtime::w30_filter_slam::w30_filter_slam_frame(
        &render, 11.02, 48_000,
    )
    .expect("filter-slam half-return frame");
    assert!((half_return.wet_gain - 0.5).abs() < 1.0e-6);
    assert!(
        crate::runtime::w30_filter_slam::w30_filter_slam_frame(
            &render, 11.04, 48_000,
        )
        .is_none(),
        "ordinary W-30 control must resume after exactly twenty milliseconds"
    );
    assert!(
        crate::runtime::w30_filter_slam::w30_filter_slam_frame(
            &render, 12.0, 48_000,
        )
        .is_none(),
        "the typed gesture must be complete at beat eight"
    );
}

#[test]
fn w30_filter_slam_changes_only_the_frozen_window_and_returns_sample_exactly() {
    const FRAMES_PER_BEAT: usize = 24_000;
    const CHANNEL_COUNT: usize = 2;
    const TOTAL_FRAMES: usize = 9 * FRAMES_PER_BEAT;
    const RETURN_FRAMES: usize = 7 * FRAMES_PER_BEAT + 960;

    let control = render_filter_slam_in_chunks(
        &hook_turnaround_test_render(None),
        127,
        TOTAL_FRAMES,
        CHANNEL_COUNT,
    );
    let candidate = render_filter_slam_in_chunks(
        &hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
            profile: W30HookArticulationProfile::FilterSlamV1,
            started_at_beat: 4,
        })),
        127,
        TOTAL_FRAMES,
        CHANNEL_COUNT,
    );

    let effect_delta_rms = region_delta_rms(
        &candidate[..RETURN_FRAMES * CHANNEL_COUNT],
        &control[..RETURN_FRAMES * CHANNEL_COUNT],
    );
    assert!(
        effect_delta_rms > 0.001,
        "the frozen close, hold, and return must be causally distinct: {effect_delta_rms}"
    );
    assert_eq!(
        &candidate[RETURN_FRAMES * CHANNEL_COUNT..],
        &control[RETURN_FRAMES * CHANNEL_COUNT..],
        "ordinary W-30 control must be sample-exact after the twenty-ms return"
    );
    assert!(candidate.iter().all(|sample| sample.is_finite()));
    assert!(candidate.iter().all(|sample| sample.abs() <= 1.0));
    assert!(candidate.chunks_exact(CHANNEL_COUNT).all(|frame| frame[0] == frame[1]));
}

#[test]
fn w30_filter_slam_is_callback_partition_invariant() {
    const TOTAL_FRAMES: usize = 9 * 24_000;
    const CHANNEL_COUNT: usize = 2;
    let render = hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
        profile: W30HookArticulationProfile::FilterSlamV1,
        started_at_beat: 4,
    }));

    assert_eq!(
        render_filter_slam_in_chunks(&render, 127, TOTAL_FRAMES, CHANNEL_COUNT),
        render_filter_slam_in_chunks(&render, 257, TOTAL_FRAMES, CHANNEL_COUNT),
    );
}

#[test]
fn w30_filter_slam_processes_every_configured_output_channel() {
    const CHANNEL_COUNT: usize = 33;
    const TOTAL_FRAMES: usize = 24_000;
    let render = hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
        profile: W30HookArticulationProfile::FilterSlamV1,
        started_at_beat: 4,
    }));

    let candidate = render_filter_slam_in_chunks(&render, 127, TOTAL_FRAMES, CHANNEL_COUNT);
    let control = render_filter_slam_in_chunks(
        &hook_turnaround_test_render(None),
        127,
        TOTAL_FRAMES,
        CHANNEL_COUNT,
    );

    assert!(candidate
        .chunks_exact(CHANNEL_COUNT)
        .all(|frame| frame.iter().all(|sample| *sample == frame[0])));
    assert!(region_delta_rms(&candidate, &control) > 0.001);
}

fn render_filter_slam_in_chunks(
    render: &W30PreviewRenderState,
    chunk_frames: usize,
    total_frames: usize,
    channel_count: usize,
) -> Vec<f32> {
    let shared = SharedW30PreviewRenderState::new(render);
    let snapshot = shared.snapshot();
    let mut state =
        W30PreviewCallbackState::with_sample_rate_and_channels(48_000, channel_count);
    let mut output = vec![0.0; total_frames * channel_count];

    for chunk in output.chunks_mut(chunk_frames * channel_count) {
        render_w30_preview_buffer(chunk, 48_000, channel_count, &snapshot, &mut state);
    }

    output
}
