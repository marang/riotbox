//! Bounded single-control-writer publication to the realtime callback.
//!
//! Every participating payload store must be Release and every payload load
//! Acquire. The first acquired commit excludes older payload values. Observing
//! any newer payload synchronizes with its writer's preceding odd revision,
//! so revision coherence prevents the final guard accepting the old commit.
//! The control owner serializes writers; this is not a multiwriter protocol.
use super::control_sync::{AtomicU64, Ordering};

pub(super) const COHERENT_SNAPSHOT_READ_ATTEMPTS: usize = 3;

pub(super) fn begin_coherent_snapshot_update(revision: &AtomicU64) {
    let previous = revision.fetch_add(1, Ordering::AcqRel);
    debug_assert_eq!(previous % 2, 0, "coherent snapshot update overlap");
}

pub(super) fn finish_coherent_snapshot_update(revision: &AtomicU64) {
    let previous = revision.fetch_add(1, Ordering::Release);
    debug_assert_eq!(previous % 2, 1, "coherent snapshot update was not active");
}

pub(super) fn try_coherent_snapshot<T>(revision: &AtomicU64, read: impl Fn() -> T) -> Option<T> {
    for _ in 0..COHERENT_SNAPSHOT_READ_ATTEMPTS {
        let before = revision.load(Ordering::Acquire);
        if !before.is_multiple_of(2) {
            continue;
        }
        let snapshot = read();
        let after = revision.load(Ordering::Acquire);
        if before == after && after.is_multiple_of(2) {
            return Some(snapshot);
        }
    }
    None
}

pub(super) fn coherent_snapshot_or<T: Clone>(
    revision: &AtomicU64,
    previous: &T,
    read: impl Fn() -> T,
) -> T {
    try_coherent_snapshot(revision, read).unwrap_or_else(|| previous.clone())
}
