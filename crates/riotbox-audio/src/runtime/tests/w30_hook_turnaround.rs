fn hook_turnaround_test_render(
    articulation: Option<crate::w30::W30HookArticulationRenderState>,
) -> W30PreviewRenderState {
    let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        let phase = index as f32 / W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as f32;
        let transient = if index % 1_024 < 48 { 0.48 } else { 0.0 };
        *sample = (phase * std::f32::consts::TAU * 5.0).sin() * 0.38
            + (phase * std::f32::consts::TAU * 11.0).sin() * 0.17
            + transient;
    }

    W30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        active_bank_id: Some("bank-a".into()),
        focused_pad_id: Some("pad-01".into()),
        capture_id: Some("cap-hook".into()),
        trigger_revision: 1,
        trigger_velocity: 0.82,
        source_window_preview: None,
        pad_playback: Some(W30PadPlaybackSampleWindow {
            source_start_frame: 0,
            source_end_frame: 96_000,
            source_sample_rate: 48_000,
            playback_frame_count: 96_000,
            sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
            loop_enabled: true,
            playback_rate: 1.0,
            reverse: false,
            gate_step_fraction: 0.0,
            loop_crossfade_sample_count: 128,
            chop_slice_count: W30_PAD_CHOP_SLICE_COUNT,
            chop_slice_starts: [0, 1_024, 2_048, 3_072, 4_096, 5_120, 6_144, 7_168],
            hook_articulation: articulation,
            samples,
        }),
        music_bus_level: 0.72,
        grit_level: 0.28,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 4.0,
    }
}

fn render_hook_turnaround_in_chunks(render: &W30PreviewRenderState, chunk_frames: usize) -> Vec<f32> {
    const TOTAL_FRAMES: usize = 120_000;
    let shared = SharedW30PreviewRenderState::new(render);
    let snapshot = shared.snapshot();
    let mut state = W30PreviewCallbackState::default();
    let mut output = vec![0.0; TOTAL_FRAMES];

    for chunk in output.chunks_mut(chunk_frames) {
        render_w30_preview_buffer(chunk, 48_000, 1, &snapshot, &mut state);
    }

    output
}

fn region_delta_rms(first: &[f32], second: &[f32]) -> f32 {
    let square_sum = first
        .iter()
        .zip(second)
        .map(|(left, right)| {
            let delta = left - right;
            delta * delta
        })
        .sum::<f32>();
    (square_sum / first.len().max(1) as f32).sqrt()
}

#[test]
fn w30_hook_turnaround_changes_only_the_frozen_middle_and_returns_cleanly() {
    const FRAMES_PER_BEAT: usize = 24_000;
    let control = render_hook_turnaround_in_chunks(&hook_turnaround_test_render(None), 128);
    let candidate = render_hook_turnaround_in_chunks(
        &hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
            profile: W30HookArticulationProfile::TurnaroundV1,
            started_at_beat: 4,
        })),
        128,
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
    let render = hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
        profile: W30HookArticulationProfile::TurnaroundV1,
        started_at_beat: 4,
    }));

    assert_eq!(
        render_hook_turnaround_in_chunks(&render, 128),
        render_hook_turnaround_in_chunks(&render, 257),
    );
}
