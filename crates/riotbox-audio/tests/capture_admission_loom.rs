//! Models the actual capture admission source, not a second algorithm.
//! Production uses std atomics through its private admission_sync owner;
//! this integration target substitutes only those primitives with Loom.
mod admission_sync {
    pub(super) use loom::sync::atomic::{AtomicUsize, Ordering};
}

#[path = "../src/runtime/live_master_capture/admission.rs"]
mod admission;

use admission::CaptureAdmission;
use loom::sync::Arc;
use loom::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use loom::thread;

fn model(work: impl Fn() + Send + Sync + 'static) {
    let mut builder = loom::model::Builder::new();
    // No scheduling/permutation pruning, independent of host environment.
    // Exceeding the branch guard fails instead of yielding a partial pass.
    builder.preemption_bound = None;
    builder.max_permutations = None;
    builder.max_duration = None;
    builder.checkpoint_file = None;
    builder.max_branches = 1_000;
    builder.check(work);
}

#[test]
fn close_and_quiescence_exclude_unfinished_admitted_work() {
    model(|| {
        let gate = Arc::new(CaptureAdmission::new());
        let payload = Arc::new(AtomicUsize::new(0));
        let finalized = Arc::new(AtomicBool::new(false));
        let writer = {
            let gate = Arc::clone(&gate);
            let payload = Arc::clone(&payload);
            let finalized = Arc::clone(&finalized);
            thread::spawn(move || {
                let Some(lease) = gate.try_enter() else {
                    return false;
                };
                assert!(!finalized.load(Ordering::SeqCst));
                payload.store(37, Ordering::Relaxed);
                thread::yield_now();
                assert!(!finalized.load(Ordering::SeqCst));
                drop(lease);
                true
            })
        };
        gate.close();
        let observed = if gate.is_quiescent() {
            finalized.store(true, Ordering::SeqCst);
            Some(payload.load(Ordering::Relaxed))
        } else {
            None // Existing nonwaiting CallbackStillActive path.
        };
        let wrote = writer.join().unwrap();
        if wrote && let Some(value) = observed {
            assert_eq!(
                value, 37,
                "quiescence must acquire completed payload writes"
            );
        }
        assert!(gate.is_quiescent());
        assert!(gate.try_enter().is_none(), "closed capture never reopens");
    });
}

#[test]
fn one_closed_capture_cannot_disable_a_new_capture() {
    model(|| {
        let old = Arc::new(CaptureAdmission::new());
        let new = Arc::new(CaptureAdmission::new());
        let writer = {
            let old = Arc::clone(&old);
            let new = Arc::clone(&new);
            thread::spawn(move || {
                let old_lease = old.try_enter();
                let new_lease = new.try_enter().expect("independent new capture");
                drop(old_lease);
                drop(new_lease);
            })
        };
        old.close();
        old.close(); // Repeated abort/closure is idempotent, never a reopen.
        writer.join().unwrap();
        assert!(old.is_quiescent());
        assert!(old.try_enter().is_none());
        assert!(new.try_enter().is_some());
    });
}
