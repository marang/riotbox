//! Exact production requested-transport owner with only atomic primitives changed.
mod control_sync {
    pub(super) use loom::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
    pub(super) fn model(work: impl Fn() + Send + Sync + 'static) {
        super::model(work);
    }
}
#[path = "../src/runtime/control_snapshot.rs"]
mod control_snapshot;
#[path = "../src/runtime/transport_timing.rs"]
mod transport_timing;

use loom::{sync::Arc, thread};
use transport_timing::{
    AudioTransportPositionRequest, AudioTransportPositionUpdate, RealtimeTransportTimingState,
    SharedTransportTimingState,
};

fn reposition(target: f64) -> AudioTransportPositionUpdate {
    AudioTransportPositionUpdate::Reposition(AudioTransportPositionRequest::from_beats(target))
}

const FIRST: RealtimeTransportTimingState = RealtimeTransportTimingState {
    is_transport_running: false,
    tempo_bpm: 132.0,
    position_beats: 0.0,
    position_request_id: 0,
};
const SECOND: RealtimeTransportTimingState = RealtimeTransportTimingState {
    is_transport_running: true,
    tempo_bpm: 144.0,
    position_beats: 8.0,
    position_request_id: 1,
};

fn model(work: impl Fn() + Send + Sync + 'static) {
    let mut builder = loom::model::Builder::new();
    builder.preemption_bound = None;
    builder.max_permutations = None;
    builder.max_duration = None;
    builder.checkpoint_file = None;
    builder.max_branches = 1_000;
    builder.check(work);
}

#[test]
fn actual_transport_never_accepts_mixed_update() {
    model(|| {
        let publication = Arc::new(SharedTransportTimingState::new(false, 132.0, 0.0));
        let writer = {
            let publication = Arc::clone(&publication);
            thread::spawn(move || publication.update(true, 144.0, reposition(8.0)).unwrap())
        };
        let observed = publication.snapshot_or_previous(&FIRST);
        writer.join().unwrap();
        assert!(
            observed == FIRST || observed == SECOND,
            "mixed control state: {observed:?}"
        );
        assert_eq!(publication.try_snapshot(), Some(SECOND));
    });
}

#[test]
fn startup_never_accepts_unchecked_transport_fields() {
    model(|| {
        let publication = Arc::new(SharedTransportTimingState::new(false, 132.0, 0.0));
        let writer = {
            let publication = Arc::clone(&publication);
            thread::spawn(move || publication.update(true, 144.0, reposition(8.0)).unwrap())
        };
        let observed = publication.try_snapshot();
        writer.join().unwrap();
        assert!(
            observed.is_none() || observed == Some(FIRST) || observed == Some(SECOND),
            "mixed startup state: {observed:?}"
        );
    });
}

#[test]
fn an_update_after_a_committed_publication_is_complete_or_previous() {
    model(|| {
        let publication = Arc::new(SharedTransportTimingState::new(false, 0.0, 0.0));
        publication
            .update(false, 132.0, AudioTransportPositionUpdate::Preserve)
            .unwrap();
        assert_eq!(publication.try_snapshot(), Some(FIRST));
        let writer = {
            let publication = Arc::clone(&publication);
            thread::spawn(move || publication.update(true, 144.0, reposition(8.0)).unwrap())
        };
        let observed = publication.snapshot_or_previous(&FIRST);
        writer.join().unwrap();
        assert!(
            observed == FIRST || observed == SECOND,
            "mixed committed update: {observed:?}"
        );
    });
}

#[test]
fn repeated_running_value_does_not_hide_mixed_tempo_and_position() {
    model(|| {
        let publication = Arc::new(SharedTransportTimingState::new(false, 132.0, 0.0));
        let writer = {
            let publication = Arc::clone(&publication);
            thread::spawn(move || publication.update(false, 144.0, reposition(8.0)).unwrap())
        };
        let observed = publication.snapshot_or_previous(&FIRST);
        writer.join().unwrap();
        let second = RealtimeTransportTimingState {
            is_transport_running: false,
            ..SECOND
        };
        assert!(
            observed == FIRST || observed == second,
            "mixed repeated-value update: {observed:?}"
        );
    });
}

#[test]
fn active_update_is_unavailable_and_reuses_previous_without_payload_reads() {
    model(|| {
        use control_snapshot::{
            begin_coherent_snapshot_update, coherent_snapshot_or, finish_coherent_snapshot_update,
            try_coherent_snapshot,
        };
        let revision = control_sync::AtomicU64::new(0);
        begin_coherent_snapshot_update(&revision);
        assert_eq!(
            try_coherent_snapshot(&revision, || panic!("active payload read")),
            None::<u32>
        );
        assert_eq!(
            coherent_snapshot_or(&revision, &17, || panic!("active payload read")),
            17
        );
        finish_coherent_snapshot_update(&revision);
        assert_eq!(try_coherent_snapshot(&revision, || 23), Some(23));
    });
}

#[test]
fn changing_revision_stops_at_three_candidates_and_preserves_previous() {
    model(|| {
        use control_snapshot::{coherent_snapshot_or, try_coherent_snapshot};
        let revision = control_sync::AtomicU64::new(0);
        let reads = std::cell::Cell::new(0);
        let read = || {
            reads.set(reads.get() + 1);
            revision.fetch_add(2, control_sync::Ordering::Release);
            23
        };
        assert_eq!(try_coherent_snapshot(&revision, read), None);
        assert_eq!(reads.get(), 3);
        reads.set(0);
        assert_eq!(coherent_snapshot_or(&revision, &17, read), 17);
        assert_eq!(reads.get(), 3);
    });
}

#[test]
fn serialized_control_read_preserves_the_exact_requested_state() {
    model(|| {
        let publication = SharedTransportTimingState::new(false, 132.0, 0.0);
        assert_eq!(publication.try_snapshot(), Some(FIRST));
        publication.update(true, 144.0, reposition(8.0)).unwrap();
        assert_eq!(publication.try_snapshot(), Some(SECOND));
    });
}
