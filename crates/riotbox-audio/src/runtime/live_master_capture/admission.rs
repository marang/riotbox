//! Admission for capture work, independent of publication-slot ownership.
//!
//! The runtime and Loom tests compile this same protocol with their respective
//! atomic primitives. A buffer is closed once and never reopened.

use super::admission_sync::{AtomicUsize, Ordering};

const CLOSED_BIT: usize = 1 << (usize::BITS - 1);
const COUNT_MASK: usize = CLOSED_BIT - 1;
// Abort well before a count could carry into the permanent closed bit.
const MAX_LEASE_COUNT: usize = COUNT_MASK / 2;

pub(super) struct CaptureAdmission {
    state: AtomicUsize,
}

pub(super) struct CaptureAdmissionLease<'a> {
    admission: &'a CaptureAdmission,
}

impl CaptureAdmission {
    pub(super) fn new() -> Self {
        Self {
            state: AtomicUsize::new(0),
        }
    }

    pub(super) fn try_enter(&self) -> Option<CaptureAdmissionLease<'_>> {
        // This RMW is the admission linearization point. An accepted lease
        // precedes close in this atomic's modification order; later entrants
        // observe the closed bit and cannot touch payload or timing.
        let previous = self.state.fetch_add(1, Ordering::SeqCst);
        if previous & COUNT_MASK >= MAX_LEASE_COUNT {
            std::process::abort();
        }
        if previous & CLOSED_BIT != 0 {
            self.state.fetch_sub(1, Ordering::SeqCst);
            return None;
        }
        Some(CaptureAdmissionLease { admission: self })
    }

    pub(super) fn close(&self) {
        self.state.fetch_or(CLOSED_BIT, Ordering::SeqCst);
    }

    /// A zero observation is a finalization fence only after `close`.
    pub(super) fn is_quiescent(&self) -> bool {
        self.state.load(Ordering::SeqCst) & COUNT_MASK == 0
    }
}

impl Drop for CaptureAdmissionLease<'_> {
    fn drop(&mut self) {
        self.admission.state.fetch_sub(1, Ordering::SeqCst);
    }
}
