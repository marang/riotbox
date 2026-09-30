use crate::runtime::source_monitor::{
    SharedSourceMonitorRenderState, SourceMonitorAudioSource, SourceMonitorCallbackState,
    SourceMonitorRenderState, apply_source_monitor_policy_with_state,
    render_source_monitor_mix_offline,
};
use crate::runtime::{signal_delta_metrics, signal_metrics};
use crate::source_audio::SourceAudioCache;
use riotbox_core::action::SourceMonitorMode;

#[test]
fn source_monitor_mode_change_uses_callback_persistent_gain_ramp() {
    let source =
        SourceAudioCache::from_interleaved_samples("mode-transition.wav", 1_000, 1, vec![1.0; 64])
            .expect("source cache");
    let source_render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 60.0,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };
    let shared = SharedSourceMonitorRenderState::new(&source_render);
    let mut callback_state = SourceMonitorCallbackState::default();
    let mut source_block = vec![0.0; 4];
    let snapshot = shared.snapshot();
    let mut render = snapshot.render_state();
    render.is_transport_running = true;
    render.tempo_bpm = 60.0;
    apply_source_monitor_policy_with_state(
        &mut source_block,
        1_000,
        1,
        &render,
        &mut callback_state,
    );

    shared.update_controls(&SourceMonitorRenderState {
        mode: SourceMonitorMode::Riotbox,
        ..source_render
    });
    let mut riotbox_block = vec![1.0; 8];
    let snapshot = shared.snapshot();
    let mut render = snapshot.render_state();
    render.is_transport_running = true;
    render.tempo_bpm = 60.0;
    render.position_beats = 0.004;
    apply_source_monitor_policy_with_state(
        &mut riotbox_block,
        1_000,
        1,
        &render,
        &mut callback_state,
    );

    assert!((source_block[3] - 0.88).abs() < 1.0e-6);
    assert!((riotbox_block[0] - 0.904).abs() < 1.0e-6);
    assert!((riotbox_block[4] - 1.0).abs() < 1.0e-6);
    assert!(
        (riotbox_block[0] - source_block[3]).abs() < 0.03,
        "mode switch retained a hard callback-edge jump: source={source_block:?} riotbox={riotbox_block:?}"
    );
}

#[test]
fn source_monitor_anchor_jump_crossfades_from_previous_source_cursor() {
    let mut samples = vec![1.0; 2_000];
    samples[1_000..].fill(-1.0);
    let source =
        SourceAudioCache::from_interleaved_samples("anchor-transition.wav", 1_000, 1, samples)
            .expect("source cache");
    let initial = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 60.0,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };
    let shared = SharedSourceMonitorRenderState::new(&initial);
    let mut callback_state = SourceMonitorCallbackState::default();
    let mut before = vec![0.0; 100];
    let snapshot = shared.snapshot();
    let mut render = snapshot.render_state();
    render.is_transport_running = true;
    render.tempo_bpm = 60.0;
    apply_source_monitor_policy_with_state(&mut before, 1_000, 1, &render, &mut callback_state);

    let mut after = vec![0.0; 8];
    let snapshot = shared.snapshot();
    let mut render = snapshot.render_state();
    render.is_transport_running = true;
    render.tempo_bpm = 60.0;
    render.position_beats = 1.0;
    apply_source_monitor_policy_with_state(&mut after, 1_000, 1, &render, &mut callback_state);

    assert!((before[99] - 0.88).abs() < 1.0e-6);
    assert!((after[0] - 0.88).abs() < 1.0e-6);
    assert!(
        after[4] < -0.87,
        "anchor crossfade did not reach the new cursor: {after:?}"
    );
    let mut transition = vec![before[99]];
    transition.extend_from_slice(&after);
    let max_adjacent_delta = transition
        .windows(2)
        .map(|window| (window[1] - window[0]).abs())
        .fold(0.0_f32, f32::max);
    assert!(
        max_adjacent_delta <= 0.45,
        "anchor crossfade retained a hard edge: max_delta={max_adjacent_delta} after={after:?}"
    );
}

#[test]
fn source_monitor_transport_stop_fades_previous_cursor_without_file_start_blip() {
    let mut samples = vec![1.0; 2_000];
    samples[1_000..].fill(-1.0);
    let source =
        SourceAudioCache::from_interleaved_samples("transport-stop.wav", 1_000, 1, samples)
            .expect("source cache");
    let running = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 60.0,
        position_beats: 1.0,
        ..SourceMonitorRenderState::default()
    };
    let shared = SharedSourceMonitorRenderState::new(&running);
    let mut callback_state = SourceMonitorCallbackState::default();
    let mut before = vec![0.0; 100];
    let snapshot = shared.snapshot();
    let mut render = snapshot.render_state();
    render.is_transport_running = true;
    render.tempo_bpm = 60.0;
    render.position_beats = 1.0;
    apply_source_monitor_policy_with_state(&mut before, 1_000, 1, &render, &mut callback_state);

    let mut stopped = vec![0.0; 8];
    let snapshot = shared.snapshot();
    let mut render = snapshot.render_state();
    render.is_transport_running = false;
    render.tempo_bpm = 60.0;
    render.position_beats = 1.1;
    apply_source_monitor_policy_with_state(&mut stopped, 1_000, 1, &render, &mut callback_state);

    assert!(before[99] < -0.87);
    assert!(
        stopped[..4].iter().all(|sample| *sample < 0.0),
        "stop leaked file-start polarity: {stopped:?}"
    );
    assert!(
        stopped[4..]
            .iter()
            .all(|sample| sample.abs() <= f32::EPSILON)
    );
    assert!(
        (stopped[0] - before[99]).abs() <= 0.18,
        "transport stop retained a hard edge: before={} stopped={stopped:?}",
        before[99]
    );
}

#[test]
fn source_monitor_seeked_running_transport_changes_audible_source_excerpt() {
    let sample_rate = 480;
    let channel_count = 2;
    let tempo_bpm = 120.0;
    let frames_per_beat = 240;
    let frames_per_bar = frames_per_beat * 4;
    let source = SourceAudioCache::from_interleaved_samples(
        "source.wav",
        sample_rate,
        channel_count,
        source_with_bar_markers(frames_per_bar),
    )
    .expect("source cache");
    let generated = vec![0.0; 128];
    let before_seek = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };
    let after_seek = SourceMonitorRenderState {
        position_beats: 16.0,
        ..before_seek.clone()
    };

    let before_output =
        render_source_monitor_mix_offline(&generated, sample_rate, channel_count, &before_seek);
    let after_output =
        render_source_monitor_mix_offline(&generated, sample_rate, channel_count, &after_seek);
    let before_metrics = signal_metrics(&before_output);
    let after_metrics = signal_metrics(&after_output);
    let delta_metrics = signal_delta_metrics(&before_output, &after_output);

    assert!(before_seek.is_transport_running);
    assert!(after_seek.is_transport_running);
    assert!(before_metrics.rms > 0.1);
    assert!(after_metrics.rms > 0.1);
    assert!(delta_metrics.rms > 0.3);
    assert_eq!(before_output[0], 0.18 * 0.88);
    assert_eq!(after_output[0], -0.62 * 0.88);
}

#[test]
fn source_monitor_scene_anchor_repositions_source_excerpt_from_commit_boundary() {
    let sample_rate = 480;
    let channel_count = 2;
    let tempo_bpm = 120.0;
    let frames_per_beat = 240;
    let frames_per_bar = frames_per_beat * 4;
    let source = SourceAudioCache::from_interleaved_samples(
        "source.wav",
        sample_rate,
        channel_count,
        source_with_bar_markers(frames_per_bar),
    )
    .expect("source cache");
    let generated = vec![0.0; 128];
    let transport_only = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm,
        position_beats: 16.0,
        source_anchor_seconds: None,
        source_anchor_position_beats: 0.0,
    };
    let scene_anchored = SourceMonitorRenderState {
        source_anchor_seconds: Some(0.0),
        source_anchor_position_beats: 16.0,
        ..transport_only.clone()
    };

    let transport_output =
        render_source_monitor_mix_offline(&generated, sample_rate, channel_count, &transport_only);
    let anchored_output =
        render_source_monitor_mix_offline(&generated, sample_rate, channel_count, &scene_anchored);
    let delta_metrics = signal_delta_metrics(&transport_output, &anchored_output);

    assert_eq!(transport_output[0], -0.62 * 0.88);
    assert_eq!(anchored_output[0], 0.18 * 0.88);
    assert!(delta_metrics.rms > 0.4);
}

#[test]
fn source_monitor_control_update_changes_anchor_without_replacing_source() {
    let source = SourceAudioCache::from_interleaved_samples(
        "source.wav",
        480,
        2,
        source_with_bar_markers(960),
    )
    .expect("source cache");
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 16.0,
        source_anchor_seconds: Some(4.0),
        source_anchor_position_beats: 16.0,
    };
    let shared = SharedSourceMonitorRenderState::new(&render);
    let render_source_ptr = render
        .source
        .as_ref()
        .expect("render source")
        .interleaved_samples()
        .as_ptr();
    let snapshot = shared.snapshot();
    let snapshot_render = snapshot.render_state();
    let snapshot_source = snapshot_render.source.expect("snapshot source");

    assert_eq!(
        snapshot_source.interleaved_samples().as_ptr(),
        render_source_ptr
    );
    assert_eq!(snapshot_render.source_anchor_seconds, Some(4.0));
    assert_eq!(snapshot_render.source_anchor_position_beats, 16.0);

    shared.update_controls(&SourceMonitorRenderState {
        source: None,
        source_anchor_seconds: None,
        source_anchor_position_beats: 0.0,
        ..render
    });
    let cleared_anchor = shared.snapshot();
    let cleared_anchor_render = cleared_anchor.render_state();

    assert!(cleared_anchor_render.source.is_some());
    assert_eq!(cleared_anchor_render.source_anchor_seconds, None);
    assert_eq!(cleared_anchor_render.source_anchor_position_beats, 0.0);
}

fn source_with_bar_markers(frames_per_bar: usize) -> Vec<f32> {
    let bar_levels = [0.18, 0.32, -0.24, 0.46, -0.62];
    let mut samples = Vec::with_capacity(frames_per_bar * bar_levels.len() * 2);
    for level in bar_levels {
        for _ in 0..frames_per_bar {
            samples.push(level);
            samples.push(-level);
        }
    }
    samples
}
