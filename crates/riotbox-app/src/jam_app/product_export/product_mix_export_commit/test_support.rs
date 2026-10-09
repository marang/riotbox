//! Scoped filesystem fault checkpoints; no production or persisted state.

use std::cell::RefCell;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::jam_app) enum ProductMixCheckpoint {
    BeforeArtifactSnapshot,
    AfterArtifactSnapshot,
    BeforeArtifactPublication,
}

struct Hook {
    checkpoint: ProductMixCheckpoint,
    callback: Box<dyn FnOnce()>,
}

thread_local! {
    static HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) };
}

pub(in crate::jam_app) fn with_product_mix_checkpoint<T>(
    checkpoint: ProductMixCheckpoint,
    callback: impl FnOnce() + 'static,
    operation: impl FnOnce() -> T,
) -> T {
    struct RestoreHook(Option<Hook>);
    impl Drop for RestoreHook {
        fn drop(&mut self) {
            HOOK.with(|hook| *hook.borrow_mut() = self.0.take());
        }
    }
    let previous = HOOK.with(|hook| {
        hook.replace(Some(Hook {
            checkpoint,
            callback: Box::new(callback),
        }))
    });
    let _restore = RestoreHook(previous);
    operation()
}

pub(super) fn run(checkpoint: ProductMixCheckpoint) {
    let selected = HOOK.with(|hook| {
        let mut hook = hook.borrow_mut();
        if hook
            .as_ref()
            .is_some_and(|hook| hook.checkpoint == checkpoint)
        {
            hook.take()
        } else {
            None
        }
    });
    if let Some(hook) = selected {
        (hook.callback)();
    }
}
