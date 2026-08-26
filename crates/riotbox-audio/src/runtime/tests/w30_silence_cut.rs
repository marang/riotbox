use super::*;

#[test]
fn w30_silence_cut_uses_the_frozen_one_beat_window_and_five_ms_tapers() {
    let shared = SharedW30PreviewRenderState::new(&hook_turnaround_test_render(Some(
        crate::w30::W30HookArticulationRenderState {
            profile: W30HookArticulationProfile::SilenceCutV1,
            started_at_beat: 4,
        },
    )));
    let render = shared.snapshot();

    let fade_start =
        crate::runtime::render_tr909_w30_preview::w30_hook_articulation_frame(&render, 7.99)
            .expect("silence-cut fade-out start");
    assert!((fade_start.terminal_gain - 1.0).abs() < 1.0e-6);
    assert!(!fade_start.silent);

    let fade_midpoint =
        crate::runtime::render_tr909_w30_preview::w30_hook_articulation_frame(&render, 7.995)
            .expect("silence-cut fade-out midpoint");
    assert!((fade_midpoint.terminal_gain - std::f32::consts::FRAC_1_SQRT_2).abs() < 1.0e-5);

    let cut = crate::runtime::render_tr909_w30_preview::w30_hook_articulation_frame(&render, 8.5)
        .expect("silence-cut silent beat");
    assert!(cut.silent);
    assert_eq!(cut.terminal_gain, 0.0);

    let return_midpoint =
        crate::runtime::render_tr909_w30_preview::w30_hook_articulation_frame(&render, 9.005)
            .expect("silence-cut return midpoint");
    assert!(!return_midpoint.silent);
    assert!((return_midpoint.terminal_gain - std::f32::consts::FRAC_1_SQRT_2).abs() < 1.0e-5);

    assert!(
        crate::runtime::render_tr909_w30_preview::w30_hook_articulation_frame(&render, 9.01)
            .is_none(),
        "ordinary W-30 playback must resume after the five-ms return"
    );
}

#[test]
fn w30_silence_cut_changes_only_the_frozen_window_and_returns_sample_exactly() {
    const FRAMES_PER_BEAT: usize = 24_000;
    const FADE_FRAMES: usize = 240;
    const TOTAL_FRAMES: usize = 6 * FRAMES_PER_BEAT;
    let control =
        render_hook_turnaround_in_chunks(&hook_turnaround_test_render(None), 128, TOTAL_FRAMES);
    let candidate = render_hook_turnaround_in_chunks(
        &hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
            profile: W30HookArticulationProfile::SilenceCutV1,
            started_at_beat: 4,
        })),
        128,
        TOTAL_FRAMES,
    );

    let fade_out_start = 4 * FRAMES_PER_BEAT - FADE_FRAMES;
    let cut_start = 4 * FRAMES_PER_BEAT;
    let cut_end = 5 * FRAMES_PER_BEAT;
    let fade_in_end = cut_end + FADE_FRAMES;
    assert_eq!(&candidate[..fade_out_start], &control[..fade_out_start]);
    assert!(
        candidate[cut_start..cut_end]
            .iter()
            .all(|sample| *sample == 0.0)
    );
    assert_eq!(&candidate[fade_in_end..], &control[fade_in_end..]);
    assert!(
        region_delta_rms(
            &candidate[fade_out_start..fade_in_end],
            &control[fade_out_start..fade_in_end]
        ) > 0.01
    );
    assert!(candidate.iter().all(|sample| sample.is_finite()));
    assert!(candidate.iter().all(|sample| sample.abs() <= 1.0));
}

#[test]
fn w30_silence_cut_is_callback_partition_and_restart_invariant() {
    const TOTAL_FRAMES: usize = 6 * 24_000;
    let render = hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
        profile: W30HookArticulationProfile::SilenceCutV1,
        started_at_beat: 4,
    }));
    let first = render_hook_turnaround_in_chunks(&render, 128, TOTAL_FRAMES);

    assert_eq!(
        first,
        render_hook_turnaround_in_chunks(&render, 257, TOTAL_FRAMES)
    );
    assert_eq!(
        first,
        render_hook_turnaround_in_chunks(&render, 128, TOTAL_FRAMES)
    );
}

#[test]
fn w30_silence_cut_missing_material_remains_silent() {
    const TOTAL_FRAMES: usize = 6 * 24_000;
    let mut render = hook_turnaround_test_render(None);
    render.routing = W30PreviewRenderRouting::Silent;
    render.pad_playback = None;
    render.capture_id = None;

    assert!(
        render_hook_turnaround_in_chunks(&render, 128, TOTAL_FRAMES)
            .iter()
            .all(|sample| *sample == 0.0)
    );
}

#[test]
fn w30_silence_cut_qualification_tempos_keep_the_same_exact_boundaries() {
    const SAMPLE_RATE: f64 = 48_000.0;
    const FADE_FRAMES: usize = 240;

    for tempo_bpm in [130.0_f32, 135.110_32, 119.680_88, 120.0] {
        let mut control_render = hook_turnaround_test_render(None);
        control_render.tempo_bpm = tempo_bpm;
        control_render.position_beats = 8.0;
        let mut candidate_render = control_render.clone();
        candidate_render
            .pad_playback
            .as_mut()
            .expect("test W-30 pad")
            .hook_articulation = Some(crate::w30::W30HookArticulationRenderState {
            profile: W30HookArticulationProfile::SilenceCutV1,
            started_at_beat: 8,
        });
        let total_frames = (6.0 * 60.0 / f64::from(tempo_bpm) * SAMPLE_RATE).round() as usize;
        let control = render_hook_turnaround_in_chunks(&control_render, 128, total_frames);
        let candidate = render_hook_turnaround_in_chunks(&candidate_render, 128, total_frames);
        let cut_start = test_frame_offset_at_beat_boundary(8.0, 12.0, tempo_bpm);
        let cut_end = test_frame_offset_at_beat_boundary(8.0, 13.0, tempo_bpm);
        let fade_out_start = cut_start - FADE_FRAMES;
        let fade_in_end = cut_end + FADE_FRAMES;

        assert_eq!(
            &candidate[..fade_out_start],
            &control[..fade_out_start],
            "tempo {tempo_bpm} changed the prefix"
        );
        assert!(
            candidate[cut_start..cut_end]
                .iter()
                .all(|sample| *sample == 0.0),
            "tempo {tempo_bpm} did not produce exact one-beat silence"
        );
        assert_eq!(
            &candidate[fade_in_end..],
            &control[fade_in_end..],
            "tempo {tempo_bpm} did not return sample-exactly"
        );
        assert_eq!(
            candidate,
            render_hook_turnaround_in_chunks(&candidate_render, 257, total_frames),
            "tempo {tempo_bpm} changed across callback partitions"
        );
    }
}

fn test_frame_offset_at_beat_boundary(
    start_position_beats: f64,
    target_position_beats: f64,
    tempo_bpm: f32,
) -> usize {
    const BOUNDARY_SNAP_BEATS: f64 = 1.0e-9;
    let beats_per_frame = f64::from(tempo_bpm) / 60.0 / 48_000.0;
    let maximum_frames =
        ((target_position_beats - start_position_beats) / beats_per_frame).ceil() as usize + 2;
    let mut position_beats = start_position_beats;
    for frame in 0..=maximum_frames {
        if position_beats >= target_position_beats
            || (position_beats - target_position_beats).abs() <= BOUNDARY_SNAP_BEATS
        {
            return frame;
        }
        position_beats += beats_per_frame;
    }
    panic!("test beat boundary was unreachable")
}
