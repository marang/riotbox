//! Test-only per-thread heap accounting; never linked into production Audio.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HeapCounts {
    pub(crate) alloc: usize,
    pub(crate) alloc_zeroed: usize,
    pub(crate) realloc: usize,
    pub(crate) dealloc: usize,
    pub(crate) requested_bytes: usize,
}

thread_local! {
    // Const/no-Drop TLS does not allocate to initialize the accounting window.
    static WINDOW: Cell<Option<HeapCounts>> = const { Cell::new(None) };
}

enum Operation {
    Alloc,
    AllocZeroed,
    Realloc,
    Dealloc,
}

fn record(operation: Operation, bytes: usize) {
    // A thread already tearing down TLS must still be able to free memory.
    // No formatting, locks, allocations or panics inside allocator methods.
    let _ = WINDOW.try_with(|window| {
        if let Some(mut counts) = window.get() {
            let count = match operation {
                Operation::Alloc => &mut counts.alloc,
                Operation::AllocZeroed => &mut counts.alloc_zeroed,
                Operation::Realloc => &mut counts.realloc,
                Operation::Dealloc => &mut counts.dealloc,
            };
            *count = count.saturating_add(1);
            if !matches!(operation, Operation::Dealloc) {
                counts.requested_bytes = counts.requested_bytes.saturating_add(bytes);
            }
            window.set(Some(counts));
        }
    });
}

struct MeasuredSystem;

// SAFETY: Every operation delegates its pointer/layout contract unchanged to
// System. Recording accesses only const-initialized thread-local Copy counters.
unsafe impl GlobalAlloc for MeasuredSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(Operation::Alloc, layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(Operation::AllocZeroed, layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record(Operation::Realloc, new_size);
        unsafe { System.realloc(pointer, layout, new_size) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record(Operation::Dealloc, layout.size());
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: MeasuredSystem = MeasuredSystem;

struct WindowGuard;

impl Drop for WindowGuard {
    fn drop(&mut self) {
        let _ = WINDOW.try_with(|window| window.set(None));
    }
}

pub(crate) fn measure<T>(work: impl FnOnce() -> T) -> (T, HeapCounts) {
    WINDOW.with(|window| {
        assert!(
            window.get().is_none(),
            "heap accounting windows must not nest"
        );
        window.set(Some(HeapCounts::default()));
    });
    let guard = WindowGuard;
    let result = work();
    let counts = WINDOW.with(|window| window.get().expect("active accounting window"));
    drop(guard); // Returned result teardown and assertions belong outside the window.
    (result, counts)
}

#[test]
fn probe_detects_each_heap_operation_and_resets_after_unwind() {
    use std::alloc::{alloc, alloc_zeroed, dealloc, realloc};
    let small = Layout::from_size_align(16, 8).unwrap();
    let large = Layout::from_size_align(32, 8).unwrap();
    let (_, counts) = measure(|| {
        // SAFETY: Positive controls use matching layouts, valid nonnull pointers,
        // and deallocate each allocation exactly once; no fixture leaks.
        unsafe {
            let pointer = std::hint::black_box(alloc(small));
            if pointer.is_null() {
                std::alloc::handle_alloc_error(small);
            }
            let pointer = std::hint::black_box(realloc(pointer, small, large.size()));
            if pointer.is_null() {
                std::alloc::handle_alloc_error(large);
            }
            dealloc(pointer, large);
            let pointer = std::hint::black_box(alloc_zeroed(small));
            if pointer.is_null() {
                std::alloc::handle_alloc_error(small);
            }
            dealloc(pointer, small);
        }
    });
    assert_eq!(
        counts,
        HeapCounts {
            alloc: 1,
            alloc_zeroed: 1,
            realloc: 1,
            dealloc: 2,
            requested_bytes: 64
        }
    );
    let panic = std::panic::catch_unwind(|| measure(|| panic!("test unwind")));
    assert!(panic.is_err());
    let (_, empty) = measure(|| std::hint::black_box(42));
    assert_eq!(empty, HeapCounts::default());
}
