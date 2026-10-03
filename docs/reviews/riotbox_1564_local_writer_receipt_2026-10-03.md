# Local writer receipt admission — RIOTBOX-1564

Date: 2026-10-03. Comparison: `6f968ea00f489051065c1d6275786859690486d4`
(RIOTBOX-1563, PR #1666) to this branch. P016 maintenance of the existing export
Action/Session evidence boundary under RIOTBOX-1036; no new DAW or audible path.

## Diagnosis and repair

The [preceding current-state checkpoint](riotbox_1563_current_state_cadence_2026-10-03.md)
found a static identity mismatch: queue pins receipt A, but the local writer and
commit select the latest receipt B if one arrives meanwhile. Action Lexicon
requires matching queued action, written proof and Session receipt evidence.

A generated metadata-only RED now confirms the erroneous acceptance: prepare
the existing JSON-package fixture, queue A, append B carrying the same artifact
references, then commit. The baseline unexpectedly succeeds instead of rejecting
the stale selection. The defect is deterministic at this public App seam; a
history bisection, device, DAW or audio reproduction is unnecessary.

RBX-434 and the owning Action Lexicon contract freeze explicit stale rejection
before the production change. The commit reads the pending receipt ID, compares
it with the current latest DAW receipt, and rejects absent/invalid/stale identity
before invoking the writer. The admitted index remains the proof-attachment
target after writing. The synchronous exclusive `&mut JamAppState` call and
writer's shared Session borrow prevent intervening receipt mutation.

The existing writer and its readiness plan intentionally remain latest-receipt
APIs. Rebinding only the final attachment to A would still leave files written
for B; teaching the complete plan/writer stack historical selection would expand
this repair. Rejection instead requires a fresh queue action for the current
receipt. Auto-queue and neighboring proof-only actions are unchanged; the latter
already support attaching their proof to the queued non-latest receipt.

## Verification

- RED: `/tmp/riotbox-1564-red.log`, unexpected success after queue A / append B.
- Initial GREEN: the same regression passes after the pre-write guard.
- Final four-test receipt-admission family passes. It covers stale A→B, missing/
  empty/unknown/wrong-scope queued IDs, removed/scope-changed selected receipt,
  and a positive later-non-DAW-receipt control.
- Rejection compares the complete Session and a snapshot of the generated
  fixture's directories/file bytes. No staging debris, in-place metadata change,
  receipt/gate mutation, committed action or commit record is admitted. Only the
  pending attempt becomes visible rejected queue history, with a requeue message.
- Explicit retry after stale rejection succeeds for B: A remains unchanged;
  Action params, commit record, returned receipt, both proof JSONs and observer
  completion name B. Observer reports the original attempt as failed, not
  completed. The generic musician-facing DAW surface remains disabled.
- Broader `cargo test -p riotbox-app daw_session`: 45 unit and 12 integration
  tests pass, including auto-queue, local-writer, host-import/audible-proof
  targeting and observer/report paths. The four new tests are executed separately
  by their own module filter, not included in that 45-test count.
- Formatting, diff check and full source-free local `just ci` pass (workspace
  tests, Python/contracts, generated-audio smokes and strict Clippy). Native
  exact-head CI remains the merge gate.

Logs: `/tmp/riotbox-1564-red.log`, `/tmp/riotbox-1564-green.log`,
`/tmp/riotbox-1564-receipt-tests.log`, `/tmp/riotbox-1564-daw-tests.log`,
`/tmp/riotbox-1564-ci.log`.

## Review and drift

Independent Adversarial Rust Reviewer retained no P0–P3 findings. Inspected
production/callers/contracts/tests and RED evidence; independently ran the four
final receipt tests using the current compiled binary and base-relative diff
check. Did not run broader DAW tests or full CI. Reviewer log:
`/tmp/riotbox-1564-review-receipt-tests.log`.

Independent Spec and Evidence / Product Pragmatist Reviewer retained no P0–P3
findings. Inspected contract alignment, generated-only fixtures, requeue policy,
claim scope and coordinator RED/GREEN logs; independently ran the four new tests
using the already-built binary and diff check. Broader tests/CI remain coordinator
evidence. The Rust reviewer also confirmed this report's behavior and limits.

Coordinator self-review retains no correctness, scope, typed-state, ownership or
test finding. Full local CI is green; native exact-head CI gates merge.

The production owner grows from 463 to 482 lines and retains cohesive DAW
commit behavior. A new ordinary test child owns receipt-admission regressions
without moving old fixtures or mechanically splitting implementation files.
No additional architectural refactor is justified by this bounded change.

No new ActionCommand or Session/replay state. The existing action's five surfaces
are covered: queue identity, pre-write side-effect admission, unchanged Session
on reject / matching commit on success, queue-history/observer feedback, and
generated regression proof. Replay consumes the unchanged recorded contracts;
failed attempts do not add committed replay evidence. No historical receipt or
proof migration is performed, and no second writer/action model is introduced.

## Limits

The local output is still the developer-only JSON skeleton, not a DAWproject,
host import or audible performance. This is not a broader writer transaction or
filesystem-fault audit. Existing actual DAWproject boundaries and audio writers
are unchanged. The tests consume only generated JSON metadata and projections;
creating a shell projection starts neither a TUI nor an audio runtime.

No real source/Holdout/commercial/capture audio, source-directory discovery,
device/DAW execution, human playback, new DSP/format/dependency, TUI/Windows work,
or musical/source-general/hardness/release claim. The repair closes the retained
architecture finding, not the remaining human/host qualification gaps.
