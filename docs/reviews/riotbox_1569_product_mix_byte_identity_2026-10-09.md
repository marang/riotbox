# Product-mix export byte identity — RIOTBOX-1569

Date: 2026-10-09. Maintenance/regression of the existing P016/P023 export
handoff, not a new audible mechanism or DAW qualification. Implementation base:
`ab6af50201ffbbd96d3b8e03f143984393c16a7f`; the separately reviewed checkpoint
landed as PR #1671 at `b70a89d299172d912d1e2f557cba2f26be286b47`.

## Reproduced defect and repair

The [1568 current-state checkpoint](riotbox_1568_current_state_checkpoint_2026-10-09.md)
identified separate proof parsing, proof/artifact hashing and subsequent copies
which reopened mutable handoff paths. A generated public-JamApp regression
changes artifact A to B at a scoped filesystem checkpoint. The old writer
successfully returns/commits a receipt naming A after copying B. This exact
test failed on the baseline and passed after the repair.

`product_export/product_mix_export_commit.rs` now:

- reads proof bytes once; parsing, hash and publication share that buffer;
- streams the artifact into a private temporary descriptor while hashing the
  exact bytes successfully written, rejecting mismatch before destination writes;
- retains/rewinds that descriptor and never reopens the mutable handoff for
  publication;
- hashes final copied bytes through exclusive `create_new` writes before receipt
  construction and Action commit;
- preserves existing empty directories/unrelated files, complete identical-bundle
  idempotency and incomplete/different-bundle rejection;
- cleans only outputs whose exclusive creation succeeded; a failed create never
  grants cleanup ownership of the collision target.

The existing queue rejection, Session receipt/commit and observer projection
remain the authorities. Rejected attempts consume their pending export, create
no successful receipt/Session commit and project failed rather than completed
observer lifecycle. The musician still uses the existing `E` trigger; its
control-thread file I/O stays off the audio callback. Sound generation, timing,
source selection, proof schema and readiness policy are unchanged.

The policy and limits are recorded in Action Lexicon §6.8 and RBX-438. Staging
uses fixed-size copy memory plus temporary disk space, not a whole-WAV buffer or
new hardlink filesystem requirement. The retained proof buffer remains uncapped.
Two-file publication is not crash-atomic; power-loss/fsync, adversarial destination
mutation, aggregate/RSS/disk budgets and deadlines are not established. Ordinary
cleanup can encounter filesystem errors; copy failure never grants success.

## Executed evidence

- Exact baseline RED: one public-method regression failed because the writer
  unexpectedly returned a successful mismatched receipt.
  `/tmp/riotbox-1569-artifact-red-exact.log`.
- Exact repaired GREEN: the same regression passed.
  `/tmp/riotbox-1569-artifact-green.log`.
- Six public-method identity/compatibility tests passed:
  `/tmp/riotbox-1569-identity-tests.log`. They cover mutation before artifact
  snapshot, proof-path mutation after parsing, artifact-path mutation after
  staging, existing/idempotent destinations, incomplete/different bundles and
  late output collision. Positive generated WAVs exceed one copy-buffer chunk.
- Two copy-leaf tests passed: partial-read failure, copied hash mismatch and
  failed-create preservation. `/tmp/riotbox-1569-copy-tests.log`.
- All 23 existing product-export tests passed:
  `/tmp/riotbox-1569-product-tests.log`.
- `git diff --check` and full source-free local `just ci` passed, including final
  strict all-target/all-feature Clippy; `/tmp/riotbox-1569-ci.log`.
  Native PR CI remains a merge gate.

The first short-name `--exact` invocation selected zero tests and is not RED or
GREEN evidence. Only the fully qualified one-test execution above establishes
the regression signal. Test checkpoints are scoped thread-local `cfg(test)`
instrumentation with restoration on unwind, not runtime/Core/Session product
state or a production API. They use no sleeps or scheduling-dependent races.

## Independent review and completion bounds

Two independent read-only branch reviews applied Adversarial Implementation
Reviewer/Rust and Spec And Evidence Auditor/Product Pragmatist lenses. Both
returned zero retained P0–P3 findings after reading full changed functions,
callers, new test/instrumentation modules, owning contracts and RED/GREEN logs.
They ran no additional tests; execution evidence belongs to the coordinator.
The coordinator's short self-review found no additional issue.

The cohesive commit/writer owner remains below the soft size budget; small copy
helpers and separate fault/regression modules have explicit ownership. No
semantic extraction was required, and no mechanical size-only split was made.
No new dependencies, persisted state, Action or replay authority were added.

No real source/Holdout/commercial/capture audio, source-directory discovery,
playback, host-audio services/devices/metadata, DAW or V1 attempt material was
accessed. Generated PCM tests and ordinary byte fixtures are synthetic evidence,
not human listening, musical progress or release qualification. No hearing test
is required for this unchanged-sound integrity repair. Silent-host V2 remains
disabled; future real-host execution needs its separate prospective permission.

Finish native CI, PR merge and synchronized-main/Linear/branch closeout.
At report preparation, the report-only 1568 native CI/merge is complete; local
sync and branch cleanup were deferred to avoid switching sources during the
active build. The safe post-build closeout is tracked in Linear/Git, not a
remaining product behavior requirement.
