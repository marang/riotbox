use super::*;

#[test]
fn replacement_atomically_changes_pcm_and_anchor() {
    let initial = source_state(constant_source("source-a.wav", 0.25));
    let shared = SharedSourceMonitorRenderState::new(&initial);
    let before = render_shared_source(&shared, 0.0);

    shared.replace_source_and_controls(&SourceMonitorRenderState {
        source: Some(constant_source("source-b.wav", -0.5)),
        source_anchor_seconds: Some(0.012),
        source_anchor_position_beats: 4.0,
        ..initial
    });

    let after_snapshot = shared.snapshot();
    let after_render = after_snapshot.render_state();
    let after = render_snapshot_source(&after_render, 4.0);

    assert!(
        before
            .iter()
            .all(|sample| (*sample - 0.25 * 0.88).abs() < 1.0e-6)
    );
    assert!(
        after
            .iter()
            .all(|sample| (*sample + 0.5 * 0.88).abs() < 1.0e-6)
    );
    assert_eq!(after_render.source_anchor_seconds, Some(0.012));
    assert_eq!(after_render.source_anchor_position_beats, 4.0);
}

#[test]
fn explicit_missing_replacement_becomes_unavailable_without_fallback() {
    let initial = source_state(constant_source("source.wav", 0.25));
    let shared = SharedSourceMonitorRenderState::new(&initial);

    shared.replace_source_and_controls(&SourceMonitorRenderState {
        source: None,
        ..initial
    });
    let snapshot = shared.snapshot();
    let render = snapshot.render_state();
    let output = render_snapshot_source(&render, 0.0);

    assert_eq!(
        source_monitor_route(render.mode, render.source, 1_000, 1),
        SourceMonitorAudioRoute::SourceUnavailable
    );
    assert!(output.iter().all(|sample| sample.abs() < 1.0e-6));
}

#[test]
fn replaced_pcm_stays_control_owned_until_callback_guard_is_released() {
    let initial = source_state(constant_source("source-a.wav", 0.25));
    let shared = SharedSourceMonitorRenderState::new(&initial);
    let callback_snapshot = shared.snapshot();

    shared.replace_source_and_controls(&SourceMonitorRenderState {
        source: Some(constant_source("source-b.wav", -0.5)),
        ..initial.clone()
    });

    assert_eq!(shared.retired_snapshot_count(), 1);
    assert_eq!(snapshot_first_sample(&callback_snapshot), 0.25);

    drop(callback_snapshot);
    shared.update_controls(&SourceMonitorRenderState {
        mode: SourceMonitorMode::Blend,
        source: None,
        ..initial
    });

    assert_eq!(shared.retired_snapshot_count(), 0);
    assert_eq!(snapshot_first_sample(&shared.snapshot()), -0.5);
}

fn constant_source(path: &str, sample: f32) -> SourceMonitorAudioSource {
    let cache = SourceAudioCache::from_interleaved_samples(path, 1_000, 1, vec![sample; 32])
        .expect("constant source cache");
    SourceMonitorAudioSource::from_cache(&cache)
}

fn source_state(source: SourceMonitorAudioSource) -> SourceMonitorRenderState {
    SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(source),
        is_transport_running: true,
        tempo_bpm: 60.0,
        position_beats: 0.0,
        source_anchor_seconds: Some(0.0),
        source_anchor_position_beats: 0.0,
    }
}

fn render_shared_source(shared: &SharedSourceMonitorRenderState, position_beats: f64) -> Vec<f32> {
    let snapshot = shared.snapshot();
    render_snapshot_source(&snapshot.render_state(), position_beats)
}

fn render_snapshot_source(
    snapshot: &RealtimeSourceMonitorRenderState<'_>,
    position_beats: f64,
) -> Vec<f32> {
    let mut render = snapshot.clone();
    render.is_transport_running = true;
    render.tempo_bpm = 60.0;
    render.position_beats = position_beats;
    let mut output = vec![0.0; 8];
    apply_source_monitor_policy_with_state(
        &mut output,
        1_000,
        1,
        &render,
        &mut SourceMonitorCallbackState::default(),
    );
    output
}

fn snapshot_first_sample(snapshot: &Guard<Arc<SourceMonitorSharedSnapshot>>) -> f32 {
    snapshot
        .render_state()
        .source
        .expect("snapshot source")
        .interleaved_samples()[0]
}
