use super::{
    COHERENT_SNAPSHOT_READ_ATTEMPTS, SharedW30PreviewRenderState, W30PreviewRenderMode,
    W30PreviewRenderState, W30PreviewSnapshotCache,
};
use std::sync::atomic::Ordering;

#[test]
fn unchanged_revision_borrows_same_payload_without_reading_fields() {
    let shared = SharedW30PreviewRenderState::new(&W30PreviewRenderState::default());
    let mut cache = W30PreviewSnapshotCache::new(&shared);
    let address = cache.state.pad_playback.samples.as_ptr();
    let state = cache.refresh_from(&shared.revision, || panic!("unexpected payload read"));
    assert_eq!(state.pad_playback.samples.as_ptr(), address);
}

#[test]
fn partial_publication_retains_complete_payload_then_adopts_completed_update() {
    let mut state = W30PreviewRenderState::default();
    let shared = SharedW30PreviewRenderState::new(&state);
    let mut cache = W30PreviewSnapshotCache::new(&shared);
    let original = cache.state;
    let original_revision = cache.revision;
    shared.revision.fetch_add(1, Ordering::AcqRel);
    shared
        .music_bus_level_bits
        .store(0.9_f32.to_bits(), Ordering::Relaxed);
    shared.pad_samples[0].store(0.75_f32.to_bits(), Ordering::Relaxed);
    assert_eq!(cache.refresh(&shared), &original);
    assert_eq!(cache.revision, original_revision);
    shared.revision.fetch_add(1, Ordering::Release);

    state.music_bus_level = 0.9;
    shared.update(&state);
    assert_eq!(cache.refresh(&shared), &shared.snapshot());
    assert_ne!(cache.revision, original_revision);
}

#[test]
fn changing_revision_exhausts_bounded_attempts_without_caching_rejected_payload() {
    let shared = SharedW30PreviewRenderState::new(&W30PreviewRenderState::default());
    let mut cache = W30PreviewSnapshotCache::new(&shared);
    let original = cache.state;
    let revision = cache.revision;
    shared.revision.fetch_add(2, Ordering::AcqRel);
    let mut reads = 0;
    let result = cache.refresh_from(&shared.revision, || {
        reads += 1;
        shared.revision.fetch_add(2, Ordering::AcqRel);
        let mut candidate = original;
        candidate.music_bus_level = 0.8;
        candidate.pad_playback.samples.fill(0.6);
        candidate
    });
    assert_eq!(*result, original);
    assert_eq!(reads, COHERENT_SNAPSHOT_READ_ATTEMPTS);
    assert_eq!(cache.revision, revision);
    assert_eq!(cache.refresh(&shared), &shared.snapshot());
    assert_eq!(
        cache.revision,
        Some(shared.revision.load(Ordering::Acquire))
    );
}

#[test]
fn writer_starting_during_read_does_not_publish_candidate() {
    let shared = SharedW30PreviewRenderState::new(&W30PreviewRenderState::default());
    let mut cache = W30PreviewSnapshotCache::new(&shared);
    let original = cache.state;
    let original_revision = cache.revision;
    shared.revision.fetch_add(2, Ordering::AcqRel);
    let mut reads = 0;
    assert_eq!(
        *cache.refresh_from(&shared.revision, || {
            reads += 1;
            shared.revision.fetch_add(1, Ordering::AcqRel);
            let mut candidate = original;
            candidate.trigger_revision = 99;
            candidate
        }),
        original
    );
    assert_eq!(reads, 1);
    assert_eq!(cache.revision, original_revision);
    shared.revision.fetch_add(1, Ordering::Release);
    assert_eq!(cache.refresh(&shared), &shared.snapshot());
}

#[test]
fn startup_during_active_writer_remains_silent_until_first_complete_snapshot() {
    let state = W30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        music_bus_level: 0.7,
        ..W30PreviewRenderState::default()
    };
    let shared = SharedW30PreviewRenderState::new(&state);
    shared.revision.fetch_add(1, Ordering::AcqRel);
    let mut cache = W30PreviewSnapshotCache::new(&shared);
    assert_eq!(cache.revision, None);
    assert_eq!(cache.state.mode, W30PreviewRenderMode::Idle);
    assert_eq!(cache.state.music_bus_level, 0.0);
    shared.revision.fetch_add(1, Ordering::Release);
    assert_eq!(cache.refresh(&shared), &shared.snapshot());
    assert_eq!(cache.state.mode, W30PreviewRenderMode::LiveRecall);
}
