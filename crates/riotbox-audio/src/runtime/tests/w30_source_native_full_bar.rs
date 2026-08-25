use super::*;

const SAMPLE_RATE: u32 = 48_000;
const FRAMES_PER_BAR: usize = 96_000;

#[test]
fn source_native_full_bar_disables_transport_chop_retriggers() {
    let control = render(W30PadPlaybackGrammar::HalfBeatChopV1);
    let candidate = render(W30PadPlaybackGrammar::SourceNativeFullBarV1);

    assert!(should_trigger_w30_step(&control, 1));
    assert!(should_trigger_w30_step(&control, 2));
    assert!((0..16).all(|step| !should_trigger_w30_step(&candidate, step)));
    assert_eq!(w30_chop_slice_cursor(&candidate, 7), 0.0);
}

#[test]
fn source_native_full_bar_loops_one_source_bar_at_original_rate() {
    let window = render(W30PadPlaybackGrammar::SourceNativeFullBarV1).pad_playback;
    let mut state = W30PreviewCallbackState::default();

    for _ in 0..FRAMES_PER_BAR {
        w30_pad_playback_sample(&window, &mut state, SAMPLE_RATE);
    }

    assert!(
        state.source_native_pad_playback_cursor.abs() < 0.5,
        "one source bar ended at unexpected cursor {}",
        state.source_native_pad_playback_cursor
    );
}

#[test]
fn source_native_full_bar_is_callback_partition_invariant_and_audibly_distinct() {
    let candidate = render(W30PadPlaybackGrammar::SourceNativeFullBarV1);
    let candidate_128 = render_in_chunks(&candidate, 128, FRAMES_PER_BAR * 2);
    let candidate_257 = render_in_chunks(&candidate, 257, FRAMES_PER_BAR * 2);
    let control = render_in_chunks(
        &render(W30PadPlaybackGrammar::HalfBeatChopV1),
        128,
        FRAMES_PER_BAR * 2,
    );

    assert_eq!(candidate_128, candidate_257);
    assert!(candidate_128.iter().any(|sample| sample.abs() > 0.001));
    assert_ne!(candidate_128, control);
}

#[test]
fn source_native_full_bar_without_pad_audio_stays_silent() {
    let mut render = render(W30PadPlaybackGrammar::SourceNativeFullBarV1);
    render.pad_playback = RealtimeW30PadPlaybackSampleWindow::default();

    let output = render_in_chunks(&render, 257, 4_096);

    assert!(output.iter().all(|sample| *sample == 0.0));
}

fn render(grammar: W30PadPlaybackGrammar) -> RealtimeW30PreviewRenderState {
    let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        let phase = index as f32 / W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as f32;
        let transient = if index % 2_048 < 48 { 0.38 } else { 0.0 };
        *sample = (phase * std::f32::consts::TAU * 5.0).sin() * 0.31 + transient;
    }

    RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        trigger_revision: 1,
        trigger_velocity: 0.82,
        source_window_preview: RealtimeW30PreviewSampleWindow::default(),
        pad_playback: RealtimeW30PadPlaybackSampleWindow {
            source_start_frame: 0,
            source_end_frame: FRAMES_PER_BAR as u64,
            source_sample_rate: SAMPLE_RATE,
            playback_frame_count: FRAMES_PER_BAR as u64,
            sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
            loop_enabled: true,
            playback_grammar: grammar,
            playback_rate: 1.0,
            reverse: false,
            gate_step_fraction: 0.0,
            loop_crossfade_sample_count: 128,
            chop_slice_count: W30_PAD_CHOP_SLICE_COUNT,
            chop_slice_starts: [0, 2_048, 4_096, 6_144, 8_192, 10_240, 12_288, 14_336],
            hook_articulation_profile: None,
            hook_articulation_started_at_beat: 0,
            samples,
        },
        music_bus_level: 0.58,
        grit_level: 0.64,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    }
}

fn render_in_chunks(
    render: &RealtimeW30PreviewRenderState,
    chunk_frames: usize,
    total_frames: usize,
) -> Vec<f32> {
    let mut state = W30PreviewCallbackState::default();
    let mut output = vec![0.0; total_frames];
    for chunk in output.chunks_mut(chunk_frames) {
        render_w30_preview_buffer(chunk, SAMPLE_RATE, 1, render, &mut state);
    }
    output
}
