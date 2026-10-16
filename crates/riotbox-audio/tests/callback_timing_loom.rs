//! Models the exact production callback-timing publication with Loom atomics.
mod timing_sync {
    pub(super) use loom::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
}

#[path = "../src/runtime/timing_snapshot.rs"]
mod timing_snapshot;

use loom::{sync::Arc, thread};
use timing_snapshot::{AudioRuntimeTimingSnapshot, CallbackTimingPublication};

const FIRST: AudioRuntimeTimingSnapshot = AudioRuntimeTimingSnapshot {
    is_transport_running: false,
    tempo_bpm: 128.0,
    position_beats: 12.5,
};
const SECOND: AudioRuntimeTimingSnapshot = AudioRuntimeTimingSnapshot {
    is_transport_running: true,
    tempo_bpm: 132.0,
    position_beats: 64.25,
};

fn model(work: impl Fn() + Send + Sync + 'static) {
    let mut builder = loom::model::Builder::new();
    // No scheduling/time/permutation pruning; exceeding the branch guard fails.
    builder.preemption_bound = None;
    builder.max_permutations = None;
    builder.max_duration = None;
    builder.checkpoint_file = None;
    builder.max_branches = 1_000;
    builder.check(work);
}

#[test]
fn first_publication_is_absent_or_a_complete_observation() {
    model(|| {
        let publication = Arc::new(CallbackTimingPublication::new());
        assert_eq!(publication.snapshot_with_position_request(), None);
        let writer = {
            let publication = Arc::clone(&publication);
            thread::spawn(move || publication.publish(FIRST, 1))
        };
        let observed = publication.snapshot_with_position_request();
        assert!(
            observed.is_none() || observed == Some((FIRST, 1)),
            "{observed:?}"
        );
        writer.join().unwrap();
        assert_eq!(
            publication.snapshot_with_position_request(),
            Some((FIRST, 1))
        );
    });
}

#[test]
fn an_update_never_publishes_mixed_transport_tempo_and_position() {
    model(|| {
        let publication = Arc::new(CallbackTimingPublication::new());
        publication.publish(FIRST, 1);
        let writer = {
            let publication = Arc::clone(&publication);
            thread::spawn(move || publication.publish(SECOND, 2))
        };
        let observed = publication.snapshot_with_position_request();
        assert!(
            observed.is_none() || observed == Some((FIRST, 1)) || observed == Some((SECOND, 2)),
            "mixed callback generations: {observed:?}"
        );
        writer.join().unwrap();
        assert_eq!(
            publication.snapshot_with_position_request(),
            Some((SECOND, 2))
        );
    });
}
