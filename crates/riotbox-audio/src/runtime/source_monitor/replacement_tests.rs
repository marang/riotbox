use super::{
    RealtimeSourceMonitorRenderState, SharedSourceMonitorRenderState, SourceMonitorAudioRoute,
    SourceMonitorAudioSource, SourceMonitorCallbackReader, SourceMonitorCallbackState,
    SourceMonitorRenderState, SourceMonitorSharedSnapshot, apply_source_monitor_policy_with_state,
    source_monitor_route,
};
use crate::source_audio::SourceAudioCache;
use crate::test_heap::{HeapCounts, measure};
use riotbox_core::action::SourceMonitorMode;
use std::sync::Arc;
use std::thread;

#[test]
fn replacement_atomically_changes_pcm_and_anchor() {
    let initial = source_state(constant_source("source-a.wav", 0.25));
    let shared = SharedSourceMonitorRenderState::new(&initial);
    let mut reader = shared
        .take_callback_reader()
        .expect("single callback reader");
    let before = render_callback_source(&mut reader, 0.0);

    shared.replace_source_and_controls(&SourceMonitorRenderState {
        source: Some(constant_source("source-b.wav", -0.5)),
        source_anchor_seconds: Some(0.012),
        source_anchor_position_beats: 4.0,
        ..initial
    });

    let after_snapshot = reader.snapshot();
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
    let mut reader = shared
        .take_callback_reader()
        .expect("single callback reader");

    shared.replace_source_and_controls(&SourceMonitorRenderState {
        source: None,
        ..initial
    });
    let snapshot = reader.snapshot();
    let render = snapshot.render_state();
    let output = render_snapshot_source(&render, 0.0);

    assert_eq!(
        source_monitor_route(render.mode, render.source, 1_000, 1),
        SourceMonitorAudioRoute::SourceUnavailable
    );
    assert!(output.iter().all(|sample| sample.abs() < 1.0e-6));
}

#[test]
fn held_callback_snapshot_stays_coherent_and_publication_retention_is_bounded() {
    let initial = source_state(constant_source("source-a.wav", 0.25));
    let mut retained_pcm = vec![Arc::downgrade(&initial.source.as_ref().unwrap().samples)];
    let shared = SharedSourceMonitorRenderState::new(&initial);
    let mut reader = shared
        .take_callback_reader()
        .expect("single callback reader");
    drop(initial);

    {
        let held_snapshot = reader.snapshot();
        for index in 1..=32 {
            let mut next = source_state(constant_source("replacement.wav", index as f32 / 100.0));
            next.mode = if index % 2 == 0 {
                SourceMonitorMode::Blend
            } else {
                SourceMonitorMode::Source
            };
            next.source_anchor_seconds = Some(f64::from(index) / 1_000.0);
            next.source_anchor_position_beats = f64::from(index);
            retained_pcm.push(Arc::downgrade(&next.source.as_ref().unwrap().samples));
            shared.replace_source_and_controls(&next);
            drop(next);

            assert_eq!(snapshot_first_sample(held_snapshot), 0.25);
            let held_render = held_snapshot.render_state();
            assert_eq!(held_render.mode, SourceMonitorMode::Source);
            assert_eq!(held_render.source_anchor_seconds, Some(0.0));
            assert_eq!(held_render.source_anchor_position_beats, 0.0);
            assert!(
                retained_pcm
                    .iter()
                    .filter(|pcm| pcm.strong_count() > 0)
                    .count()
                    <= 3
            );
        }
    }

    let latest = reader.snapshot();
    assert_eq!(snapshot_first_sample(latest), 0.32);
    let render = latest.render_state();
    assert_eq!(render.mode, SourceMonitorMode::Blend);
    assert_eq!(render.source_anchor_seconds, Some(0.032));
    assert_eq!(render.source_anchor_position_beats, 32.0);
}

#[test]
fn callback_reader_can_only_be_taken_once_even_after_it_is_dropped() {
    let shared = SharedSourceMonitorRenderState::new(&SourceMonitorRenderState::default());
    let reader = shared
        .take_callback_reader()
        .expect("first callback reader");
    assert!(shared.take_callback_reader().is_none());
    drop(reader);
    assert!(shared.take_callback_reader().is_none());
}

#[test]
fn control_updates_keep_latest_pcm_instead_of_the_recycled_producer_slot() {
    let shared =
        SharedSourceMonitorRenderState::new(&source_state(constant_source("source-a.wav", 0.25)));
    let mut reader = shared
        .take_callback_reader()
        .expect("single callback reader");
    assert_eq!(snapshot_first_sample(reader.snapshot()), 0.25);

    for (index, sample) in [-0.5, 0.75].into_iter().enumerate() {
        shared
            .replace_source_and_controls(&source_state(constant_source("replacement.wav", sample)));
        let controls = SourceMonitorRenderState {
            mode: SourceMonitorMode::Blend,
            source: None,
            source_anchor_seconds: Some(index as f64 + 0.012),
            source_anchor_position_beats: index as f64 * 4.0,
            ..SourceMonitorRenderState::default()
        };
        shared.update_controls(&controls);

        let control_snapshot = shared.snapshot();
        assert_eq!(snapshot_first_sample(&control_snapshot), sample);
        let callback_snapshot = reader.snapshot();
        assert_eq!(callback_snapshot, control_snapshot.as_ref());
        let render = callback_snapshot.render_state();
        assert_eq!(render.mode, controls.mode);
        assert_eq!(render.source_anchor_seconds, controls.source_anchor_seconds);
        assert_eq!(
            render.source_anchor_position_beats,
            controls.source_anchor_position_beats
        );

        shared.update_controls(&controls);
        assert!(Arc::ptr_eq(&control_snapshot, &shared.snapshot()));
    }
}

#[test]
fn old_pcm_is_reclaimed_by_control_publisher_after_reader_advances() {
    let initial = source_state(constant_source("source-a.wav", 0.25));
    let old_pcm = Arc::downgrade(&initial.source.as_ref().unwrap().samples);
    let shared = Arc::new(SharedSourceMonitorRenderState::new(&initial));
    let mut reader = shared
        .take_callback_reader()
        .expect("single callback reader");
    drop(initial);
    assert_eq!(snapshot_first_sample(reader.snapshot()), 0.25);
    shared.replace_source_and_controls(&source_state(constant_source("source-b.wav", -0.5)));

    let (sample, callback_heap) = measure(|| snapshot_first_sample(reader.snapshot()));
    assert_eq!(sample, -0.5);
    assert_eq!(callback_heap, HeapCounts::default());
    assert!(
        old_pcm.strong_count() > 0,
        "reader advance must not reclaim old PCM"
    );

    let control_owner = Arc::clone(&shared);
    let (retained_after_first_write, retained_after_second_write, control_heap) =
        thread::spawn(move || {
            control_owner.update_controls(&SourceMonitorRenderState::control_only(
                SourceMonitorMode::Blend,
            ));
            let retained_after_first_write = old_pcm.strong_count();
            let (_, control_heap) = measure(|| {
                control_owner.update_controls(&SourceMonitorRenderState::control_only(
                    SourceMonitorMode::Riotbox,
                ));
            });
            (
                retained_after_first_write,
                old_pcm.strong_count(),
                control_heap,
            )
        })
        .join()
        .expect("control publisher thread");
    assert!(retained_after_first_write > 0);
    assert_eq!(retained_after_second_write, 0);
    assert!(control_heap.dealloc > 0);

    // The shared control owner outlives the reader, so dropping the consumer
    // cannot become the final owner of the triple-buffer payloads either.
    let (_, teardown_heap) = measure(|| drop(reader));
    assert_eq!(teardown_heap, HeapCounts::default());
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

fn render_callback_source(
    reader: &mut SourceMonitorCallbackReader,
    position_beats: f64,
) -> Vec<f32> {
    let snapshot = reader.snapshot();
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

fn snapshot_first_sample(snapshot: &SourceMonitorSharedSnapshot) -> f32 {
    snapshot
        .render_state()
        .source
        .expect("snapshot source")
        .interleaved_samples()[0]
}
