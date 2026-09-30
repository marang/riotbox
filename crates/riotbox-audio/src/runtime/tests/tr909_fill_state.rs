use crate::runtime::render_tr909_w30_preview::render_tr909_buffer;
use crate::runtime::shared_transport_tr909::RealtimeTr909RenderState;
use crate::runtime::shared_w30_resample_callback::Tr909CallbackState;
use crate::runtime::signal_metrics;
use crate::runtime::tests::signal_test_helpers::test_max_frame_delta;
use crate::runtime::tests::tr909_fill_fixtures::{
    cursor_20_phrase_drive_fill_render, render_legacy_tr909_composite_control,
    render_tr909_in_callback_blocks,
};
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909SourceSupportContext, Tr909SourceSupportProfile, Tr909TakeoverRenderProfile,
};

#[test]
fn phrase_drive_fill_is_sample_exact_across_callback_partitions_and_clean_bar_wraps() {
    let render = RealtimeTr909RenderState {
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..cursor_20_phrase_drive_fill_render()
    };
    let sample_rate = 48_000;
    let channel_count = 2;
    let frames_per_bar = 96_000;
    let frame_count = frames_per_bar * 2;

    let full_block = render_tr909_in_callback_blocks(
        &render,
        sample_rate,
        channel_count,
        frame_count,
        frame_count,
    );
    let callbacks_127 =
        render_tr909_in_callback_blocks(&render, sample_rate, channel_count, frame_count, 127);
    let callbacks_128 =
        render_tr909_in_callback_blocks(&render, sample_rate, channel_count, frame_count, 128);

    assert_eq!(full_block, callbacks_127);
    assert_eq!(full_block, callbacks_128);

    let fresh_second_bar = render_tr909_in_callback_blocks(
        &RealtimeTr909RenderState {
            position_beats: 4.0,
            ..render
        },
        sample_rate,
        channel_count,
        frames_per_bar,
        127,
    );
    assert_eq!(
        &callbacks_127[frames_per_bar * channel_count..],
        fresh_second_bar,
        "a continuous Fill bar restored stale voice tails at the wrap"
    );

    let boundary_sample = frames_per_bar * channel_count;
    let boundary_delta =
        (callbacks_127[boundary_sample] - callbacks_127[boundary_sample - channel_count]).abs();
    assert!(
        boundary_delta < 0.05,
        "continuous Fill bar wrap introduced a click-sized edge: {boundary_delta}"
    );
}

#[test]
fn fill_to_break_reinforce_starts_from_fresh_legacy_state_without_a_tail_edge() {
    let fill = RealtimeTr909RenderState {
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..cursor_20_phrase_drive_fill_render()
    };
    let break_reinforce = RealtimeTr909RenderState {
        mode: Tr909RenderMode::BreakReinforce,
        position_beats: 4.0,
        ..fill
    };
    let sample_rate = 48_000;
    let channel_count = 2;
    let frames_per_bar = 96_000;
    let transition_frames = 4_096;
    let mut state = Tr909CallbackState::default();
    let mut fill_bar = vec![0.0_f32; frames_per_bar * channel_count];
    render_tr909_buffer(&mut fill_bar, sample_rate, channel_count, &fill, &mut state);
    let mut transitioned = vec![0.0_f32; transition_frames * channel_count];
    render_tr909_buffer(
        &mut transitioned,
        sample_rate,
        channel_count,
        &break_reinforce,
        &mut state,
    );

    let mut fresh_state = Tr909CallbackState::default();
    let mut fresh = vec![0.0_f32; transition_frames * channel_count];
    render_tr909_buffer(
        &mut fresh,
        sample_rate,
        channel_count,
        &break_reinforce,
        &mut fresh_state,
    );
    assert_eq!(
        transitioned, fresh,
        "Fill voice or subdivision state leaked into BreakReinforce"
    );

    let boundary_delta = (transitioned[0] - fill_bar[fill_bar.len() - channel_count]).abs();
    assert!(
        boundary_delta < 0.08,
        "Fill-to-BreakReinforce introduced a click-sized edge: {boundary_delta}"
    );
    let mut boundary_window = fill_bar[fill_bar.len() - 512 * channel_count..].to_vec();
    boundary_window.extend_from_slice(&transitioned[..512 * channel_count]);
    assert!(
        test_max_frame_delta(&boundary_window, channel_count) < 0.35,
        "Fill-to-BreakReinforce boundary exceeded the local transient budget"
    );
}

#[test]
fn fill_one_subdivision_transport_jump_clears_in_flight_voices() {
    let render = RealtimeTr909RenderState {
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..cursor_20_phrase_drive_fill_render()
    };
    let sample_rate = 48_000;
    let channel_count = 2;
    let priming_frames = 512;
    let primed_position = priming_frames as f64 * 120.0 / 60.0 / sample_rate as f64;
    let jumped = RealtimeTr909RenderState {
        position_beats: primed_position + 0.125,
        ..render
    };
    let mut state = Tr909CallbackState::default();
    let mut priming = vec![0.0_f32; priming_frames * channel_count];
    render_tr909_buffer(
        &mut priming,
        sample_rate,
        channel_count,
        &render,
        &mut state,
    );
    assert!(
        signal_metrics(&priming).active_samples > 0,
        "priming Fill did not start a voice"
    );

    let mut after_jump = vec![0.0_f32; 1_024 * channel_count];
    render_tr909_buffer(
        &mut after_jump,
        sample_rate,
        channel_count,
        &jumped,
        &mut state,
    );
    let mut fresh_state = Tr909CallbackState::default();
    let mut fresh = vec![0.0_f32; after_jump.len()];
    render_tr909_buffer(
        &mut fresh,
        sample_rate,
        channel_count,
        &jumped,
        &mut fresh_state,
    );
    assert_eq!(
        after_jump, fresh,
        "an exact one-subdivision seek retained a stale Fill tail"
    );
}

#[test]
fn non_fill_modes_remain_sample_exact_with_the_legacy_composite_renderer() {
    let cases = [
        RealtimeTr909RenderState {
            mode: Tr909RenderMode::SourceSupport,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: Some(Tr909SourceSupportProfile::DropDrive),
            source_support_context: Some(Tr909SourceSupportContext::TransportBar),
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            takeover_profile: None,
            drum_bus_level: 0.78,
            slam_enabled: false,
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 8.0,
            source_bar_grid_anchor_position_beats: None,
        },
        RealtimeTr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            takeover_profile: None,
            drum_bus_level: 0.82,
            slam_enabled: true,
            slam_intensity: 0.85,
            is_transport_running: true,
            tempo_bpm: 132.0,
            position_beats: 16.0,
            source_bar_grid_anchor_position_beats: None,
        },
        RealtimeTr909RenderState {
            mode: Tr909RenderMode::Takeover,
            routing: Tr909RenderRouting::DrumBusTakeover,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
            phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
            takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
            drum_bus_level: 0.76,
            slam_enabled: false,
            slam_intensity: 0.45,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 24.0,
            source_bar_grid_anchor_position_beats: None,
        },
    ];

    for render in cases {
        let current = render_tr909_in_callback_blocks(&render, 48_000, 2, 24_000, 24_000);
        let legacy = render_legacy_tr909_composite_control(&render, 48_000, 2, 24_000);
        assert_eq!(
            current, legacy,
            "non-Fill mode {:?} drifted from the established composite sample path",
            render.mode
        );
    }
}
