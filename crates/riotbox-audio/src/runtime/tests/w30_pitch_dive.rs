#[test]
fn w30_pitch_dive_preserves_eight_beats_then_dives_and_ends_in_silence() {
    const FRAMES_PER_BEAT: usize = 24_000;
    const TOTAL_FRAMES: usize = 13 * FRAMES_PER_BEAT;
    const DIVE_START_FRAME: usize = 8 * FRAMES_PER_BEAT;
    const SILENCE_START_FRAME: usize = 12 * FRAMES_PER_BEAT;

    let control =
        render_hook_turnaround_in_chunks(&hook_turnaround_test_render(None), 128, TOTAL_FRAMES);
    let candidate = render_hook_turnaround_in_chunks(
        &hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
            profile: W30HookArticulationProfile::PitchDiveV1,
            started_at_beat: 4,
        })),
        128,
        TOTAL_FRAMES,
    );

    assert_eq!(
        &candidate[..DIVE_START_FRAME],
        &control[..DIVE_START_FRAME],
        "the frozen control must remain sample-exact for the first eight beats"
    );
    assert!(
        region_delta_rms(
            &candidate[DIVE_START_FRAME..SILENCE_START_FRAME],
            &control[DIVE_START_FRAME..SILENCE_START_FRAME],
        ) > 0.05,
        "the four-beat pitch dive must be causally distinct"
    );
    assert!(
        candidate[SILENCE_START_FRAME..]
            .iter()
            .all(|sample| *sample == 0.0),
        "the pitch dive must enter explicit silence at the terminal boundary"
    );
    assert!(candidate.iter().all(|sample| sample.is_finite()));
    assert!(candidate.iter().all(|sample| sample.abs() <= 1.0));
}

#[test]
fn w30_pitch_dive_uses_the_frozen_rate_curve_and_terminal_fade() {
    let shared = SharedW30PreviewRenderState::new(&hook_turnaround_test_render(Some(
        crate::w30::W30HookArticulationRenderState {
            profile: W30HookArticulationProfile::PitchDiveV1,
            started_at_beat: 4,
        },
    )));
    let render = shared.snapshot();

    assert!(
        crate::runtime::render_tr909_w30_preview::w30_hook_articulation_frame(&render, 11.999)
            .is_none()
    );

    let start = crate::runtime::render_tr909_w30_preview::w30_hook_articulation_frame(
        &render, 12.0,
    )
    .expect("pitch-dive start frame");
    assert_eq!(start.playback_rate_multiplier, 1.0);
    assert_eq!(start.terminal_gain, 1.0);
    assert!(start.continuous_cursor);
    assert!(!start.silent);

    let midpoint = crate::runtime::render_tr909_w30_preview::w30_hook_articulation_frame(
        &render, 14.0,
    )
    .expect("pitch-dive midpoint frame");
    assert!((midpoint.playback_rate_multiplier - 0.35_f32.sqrt()).abs() < 1.0e-6);
    assert_eq!(midpoint.terminal_gain, 1.0);

    let fade = crate::runtime::render_tr909_w30_preview::w30_hook_articulation_frame(
        &render, 15.925,
    )
    .expect("terminal fade frame");
    assert!((fade.terminal_gain - 0.5).abs() < 1.0e-5);

    let terminal = crate::runtime::render_tr909_w30_preview::w30_hook_articulation_frame(
        &render, 16.0,
    )
    .expect("terminal silent frame");
    assert_eq!(terminal.playback_rate_multiplier, 0.35);
    assert_eq!(terminal.terminal_gain, 0.0);
    assert!(terminal.continuous_cursor);
    assert!(terminal.silent);
}

#[test]
fn w30_pitch_dive_is_callback_partition_invariant() {
    const TOTAL_FRAMES: usize = 13 * 24_000;
    let render = hook_turnaround_test_render(Some(crate::w30::W30HookArticulationRenderState {
        profile: W30HookArticulationProfile::PitchDiveV1,
        started_at_beat: 4,
    }));

    assert_eq!(
        render_hook_turnaround_in_chunks(&render, 128, TOTAL_FRAMES),
        render_hook_turnaround_in_chunks(&render, 257, TOTAL_FRAMES),
    );
}
