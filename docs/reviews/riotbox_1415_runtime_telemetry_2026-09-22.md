# RIOTBOX-1415: diagnostic poison recovery and dependency ownership

Work class: maintenance/regression. Production base: `ff0a2434`; integrated
archive-only base: `c858a729`.

## Delivered boundary

Runtime telemetry now has its own semantic Rust module rather than sharing
ownership with TR-909 envelope/enum helpers. Render-callback counters and timing
stores are unchanged. Error reporting and full health snapshots recover the
simple poisoned message slot without clearing the poison latch. Public health
reports running-plus-poison as `Faulted` with an explicit diagnostic reason;
stopped stays stopped. The actual stream-error counter is not incremented for
poison. Existing public health fields expose the result to App runtime view,
observer and diagnostic consumers; no string controls a branch decision.

Timing-only snapshots now read atomics directly without locking or cloning the
error text. This is not a claim that full health snapshots are lock-free, that
arbitrary runtime panics are recoverable, or that device behavior was tested.
No Session, replay, capture-lineage or sound-generation state moved into App.

Root `workspace.dependencies` owns the shared serde/serde_json/sha2/tempfile
requirements and serde derive feature. Crate-local libraries remain local;
normal/dev placement is preserved. No new dependency or upgrade.

## Evidence

- Before correction, both injected-poison regressions fail at the original
  `lock().expect()` paths (`/tmp/riotbox-1415-poison-red.log`). The intentional
  caught poisoning panic is separate from the unexpected second panic.
- Four focused tests cover later error replacement, preserved readable message,
  held-lock timing progress, and public health for running/stopped states with
  present/absent prior messages. Fresh telemetry has no poison state. Timing
  regression waits are bounded and release the lock before asserting failure.
- `cargo metadata --locked --no-deps --format-version 1` retains previous
  dependency requirements, features and kinds. `Cargo.lock` is byte-identical;
  Git blob `653af80a45474c291e6ce1e14b9b9e12455aaeb0` before and after.
- Initial normal parallel `just ci` passed (`/tmp/riotbox-1415-ci.log`): 767 App,
  273 Audio, 470 Core and 14 Sidecar library tests, other binaries/doctests,
  synthetic exact RuntimeMix/observer QA, Python contracts, formatting and
  strict Clippy. Two existing informational benchmarks remain ignored.
- Final reviewed normal parallel CI, after adding absent-message coverage:
  passed with the same counts (`/tmp/riotbox-1415-reviewed-ci.log`).
- Textual-include guard and `git diff --check` pass. No textual shard added.

## Sequential solo review

No subagents or independent-review claim; the user's constraint remains active.

- Maintainer/Rust: the semantic telemetry extraction makes lock ownership and
  poison tests local. Existing atomic orderings, render stores and imports used
  by the DSP helpers remain unchanged. No runtime dependency or public struct
  field added; shared Cargo requirements retain their feature ownership.
- Spec/evidence: RBX-380 and the audio-core spec state sticky degradation,
  actual error counts and the non-lock-free limit. This is not evidence of an
  observed realtime callback crash. No hearing/device or musical pass.
- Adversarial/concurrency: poison can be observed before any device error and
  after later replacement. Kept the latch visible instead of clearing it;
  tested the absent-message case too. Timing-under-held-lock test was made
  bounded so a regression fails instead of hanging the suite.
- Product/operations: existing runtime view and observer forward the public
  reason; live-master recording preflight rejects non-running health. There is
  no automatic restart or phantom device error. No source paths are opened by
  this change.

No retained branch-local finding after the test refinements. Host audio,
Bitwig/DAW operation, human listening, real source and holdout access are outside
this maintenance slice.
