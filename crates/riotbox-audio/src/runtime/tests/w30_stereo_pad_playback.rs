fn stereo_pad_test_render(
    left: &[f32],
    right: &[f32],
    grit_level: f32,
) -> (
    RealtimeW30PreviewRenderState,
    Box<[f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN]>,
) {
    assert_eq!(left.len(), right.len());
    assert!(left.len() <= W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);

    let mut mono_samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    let mut stereo_side_samples: Box<[f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN]> = vec![
        0.0;
        W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN
    ]
    .into_boxed_slice()
    .try_into()
    .expect("fixed stereo side window");
    for index in 0..left.len() {
        mono_samples[index] = (left[index] + right[index]) / 2.0;
        stereo_side_samples[index] = (left[index] - right[index]) / 2.0;
    }

    let render = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        trigger_revision: 1,
        trigger_velocity: 0.8,
        source_window_preview: RealtimeW30PreviewSampleWindow::default(),
        pad_playback: RealtimeW30PadPlaybackSampleWindow {
            source_start_frame: 0,
            source_end_frame: left.len() as u64,
            source_sample_rate: 48_000,
            playback_frame_count: left.len() as u64,
            sample_count: left.len(),
            loop_enabled: true,
            playback_rate: 1.0,
            reverse: false,
            gate_step_fraction: 0.0,
            loop_crossfade_sample_count: 0,
            chop_slice_count: 0,
            chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
            hook_articulation_profile: None,
            hook_articulation_started_at_beat: 0,
            samples: mono_samples,
        },
        music_bus_level: 0.58,
        grit_level,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    (render, stereo_side_samples)
}

fn synthetic_pad_channels() -> ([f32; 1_024], [f32; 1_024]) {
    let left = std::array::from_fn(|index| {
        let phase = index as f32 / 1_024.0;
        (phase * std::f32::consts::TAU * 7.0).sin() * 0.42
            + if index % 256 < 12 { 0.28 } else { 0.0 }
    });
    let right = std::array::from_fn(|index| {
        let phase = index as f32 / 1_024.0;
        (phase * std::f32::consts::TAU * 11.0).sin() * 0.31
            - if index % 256 < 12 { 0.19 } else { 0.0 }
    });
    (left, right)
}

fn render_stereo_pad(
    render: &RealtimeW30PreviewRenderState,
    stereo_side_samples: Option<&[f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN]>,
    frame_count: usize,
    chunk_frames: Option<usize>,
) -> Vec<f32> {
    let mut output = vec![0.0; frame_count * 2];
    let mut state = W30PreviewCallbackState::with_sample_rate_and_channels(48_000, 2);
    if let Some(chunk_frames) = chunk_frames {
        for chunk in output.chunks_mut(chunk_frames * 2) {
            render_w30_preview_buffer_with_stereo_side(
                chunk,
                48_000,
                2,
                render,
                stereo_side_samples,
                &mut state,
            );
        }
    } else {
        render_w30_preview_buffer_with_stereo_side(
            &mut output,
            48_000,
            2,
            render,
            stereo_side_samples,
            &mut state,
        );
    }
    output
}

#[test]
fn stereo_pad_is_sample_exact_for_mono_input_and_keeps_mono_control_unchanged() {
    let mono = std::array::from_fn::<_, 1_024, _>(|index| {
        let phase = index as f32 / 1_024.0;
        (phase * std::f32::consts::TAU * 5.0).sin() * 0.4
    });
    let (control, stereo_side_samples) = stereo_pad_test_render(&mono, &mono, 0.64);

    let control_output = render_stereo_pad(&control, None, 2_048, None);
    let candidate_output =
        render_stereo_pad(&control, Some(stereo_side_samples.as_ref()), 2_048, None);

    assert_eq!(candidate_output, control_output);
    assert!(control_output.chunks_exact(2).all(|frame| frame[0] == frame[1]));
}

#[test]
fn stereo_pad_does_not_collapse_antiphase_or_share_character_history() {
    let left = std::array::from_fn::<_, 1_024, _>(|index| {
        let phase = index as f32 / 1_024.0;
        (phase * std::f32::consts::TAU * 9.0).sin() * 0.5
    });
    let right = left.map(|sample| -sample);
    let (control, stereo_side_samples) = stereo_pad_test_render(&left, &right, 0.0);

    let control_output = render_stereo_pad(&control, None, 2_048, None);
    let candidate_output =
        render_stereo_pad(&control, Some(stereo_side_samples.as_ref()), 2_048, None);
    assert!(control_output.iter().all(|sample| *sample == 0.0));
    let side_rms = (candidate_output
        .chunks_exact(2)
        .map(|frame| ((frame[0] - frame[1]) / 2.0).powi(2))
        .sum::<f32>()
        / 2_048.0)
        .sqrt();
    assert!(side_rms > 0.001, "anti-phase side RMS was {side_rms}");
    assert!(
        candidate_output
            .chunks_exact(2)
            .all(|frame| (frame[0] + frame[1]).abs() <= 1.0e-6)
    );

    let silent_right = [0.0; 1_024];
    let (characterized, characterized_side) =
        stereo_pad_test_render(&left, &silent_right, 0.64);
    let characterized_output = render_stereo_pad(
        &characterized,
        Some(characterized_side.as_ref()),
        2_048,
        None,
    );
    assert!(
        characterized_output
            .chunks_exact(2)
            .all(|frame| frame[1] == 0.0),
        "left-channel character history leaked into the silent right channel"
    );
}

#[test]
fn stereo_pad_is_partition_invariant_restartable_and_silent_without_source() {
    let (left, right) = synthetic_pad_channels();
    let (render, stereo_side_samples) = stereo_pad_test_render(&left, &right, 0.64);

    let side = Some(stereo_side_samples.as_ref());
    let contiguous = render_stereo_pad(&render, side, 4_096, None);
    let chunks_128 = render_stereo_pad(&render, side, 4_096, Some(128));
    let chunks_257 = render_stereo_pad(&render, side, 4_096, Some(257));
    let restarted = render_stereo_pad(&render, side, 4_096, None);
    assert_eq!(contiguous, chunks_128);
    assert_eq!(contiguous, chunks_257);
    assert_eq!(contiguous, restarted);

    let unavailable = RealtimeW30PreviewRenderState {
        pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
        ..render
    };
    let silent = render_stereo_pad(&unavailable, side, 512, None);
    assert!(silent.iter().all(|sample| *sample == 0.0));
}
