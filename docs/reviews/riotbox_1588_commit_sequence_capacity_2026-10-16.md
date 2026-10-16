# Checked commit-sequence capacity — RIOTBOX-1588

Date: 2026-10-16. Classification: maintenance/regression protecting the P023
performer commit, Undo, restore and recording/export paths; not an audible
mechanism. Comparison: merged main
`2a5fda67c13b0f7318896f9ae806d6a1653af1ab` and this branch's tracked and new
files. RBX-451, Session §8.7, Replay sequencing and Action Lexicon §3 own the
contract. RIOTBOX-1587's separate adjacent P2 is the starting evidence.

## Defect and correction

A small legal imported history can end at `u32::MAX` for one complete commit
boundary. Before this repair, the next normal toggle panicked after queue
mutation; typed Undo additionally consumed its snapshot and changed its target
without recording the Undo marker. The source-free public-loader diagnostic and
independent rebuild are retained in the preceding RIOTBOX-1587 review. They
demonstrate the defect, not correctness, ordinary long-session frequency or
on-disk last-save loss.

Core `ActionLog::prepare_commit_sequences` derives capacity from the maximum
sequence at the exact complete boundary. Zero work succeeds at MAX; MAX−1
admits one final MAX but refuses two actions together. Count conversion and
addition are checked. The publicly exported, privately constructed,
non-cloned, non-serialized sequence range is arithmetic evidence, not an
exclusive reservation or a new durable cursor.
The existing Core queue uses one readiness predicate to count and commit the
whole batch, checking history before any drain, relabel or history write.
Side-effect completion preserves missing/not-ready `Ok(None)` precedence.

App uses Core-assigned absolute sequences and retains the existing incremental
`record one action → run its pipeline` order. Toggle admits prior Immediate
work plus its new action before enqueue/ID allocation. Undo finds its trusted
target, admits its marker, then allocates before normalization or snapshot
consumption. True writer/explicit-BPM compound operations admit capacity before
autoqueue/publication and recheck at their final queue commit. Clock and audio
acknowledgment updates are staged until crossed-batch admission succeeds.
There is no App-local allocator, history renumbering, new action or schema.

Queue-only APIs retain their ActionId-only contract. Existing UI controls that
queue and subsequently attempt a commit keep the legitimately accepted pending
action when only the second step refuses. They show `queued; commit blocked`
and emit no committed observer event. Direct compound toggle/Undo instead
refuse before their first mutation. Observed-timing refusal stops and drops the
live runtime rather than forwarding stale transport/render truth.

Recorder sync, confirmation, acquired-capture and final-Pause errors still stop
the device; an acquired capture is aborted and no successful Pause is invented.
If a finalized capture's writer refuses, its now-unusable owned request is
rejected rather than left pending. Generated tests run the production controller
through its existing runtime-control seam without constructing an audio device.

## Executed evidence

- Genuine Core RED: `/tmp/riotbox-1588-core-sequence-red.log`.
  Final Core package: 524 library +8 binary tests passed, with one existing
  ignored manual benchmark. `/tmp/riotbox-1588-core-final-tests.log`.
  Strict all-target/all-feature Core Clippy also passed:
  `/tmp/riotbox-1588-core-clippy.log`.
- Genuine App public-loader panic RED:
  `/tmp/riotbox-1588-app-sequence-red.log`. The final exact JamApp sequence
  selection passed13 tests, and the neighboring ID selection passed8:
  `/tmp/riotbox-1588-app-queue-rejection-owned.log` and
  `/tmp/riotbox-1588-app-queue-rejection-id.log`.
  Coverage includes repeat refusal without queue/Session/projection changes,
  whole-batch MAX−1, final-slot save/reload/replay, other-boundary continuation,
  typed Undo/snapshot/no-target/dual-exhaustion precedence, staged clock/ack,
  writers before autoqueue/files, and generated recording publication.
- Production recorder-controller selection passed29 tests:
  `/tmp/riotbox-1588-recorder-tests-green.log`. Six new controls cover initial
  Play, acquired-capture sync, final Pause, finalized-writer refusal, a full
  start boundary with a different free final boundary, and the last two final
  slots followed by save/reload/replay.
- Three new CLI controls passed:
  `/tmp/riotbox-1588-cli-sequence-tests.log`. Direct toggle is unchanged on
  refusal; queue-then-commit retains only its real pending action and reports
  the blocked commit; unavailable grid precedence remains distinct.
- Observer19, UI143, source-timing observer7 and DAWproject summary1 passed:
  `/tmp/riotbox-1588-observer-tests.log`, `/tmp/riotbox-1588-ui-tests.log`,
  `/tmp/riotbox-1588-source-timing-observer-tests.log` and
  `/tmp/riotbox-1588-dawproject-summary-test.log`.
- `cargo check --workspace --all-targets --all-features` passed:
  `/tmp/riotbox-1588-all-features-check.log`.
  The final feature-enabled JamApp sequence13 rerun also passed:
  `/tmp/riotbox-1588-app-feature-sequence-tests.log`.

Selections overlap and must not be added as unique repository coverage.
The first recorder selection failed because a new test incorrectly expected
queue-only recording admission to refuse at a full start boundary. The
production contract was retained and a positive different-final-boundary test
replaced that assumption. This intermediate run is not green:
`/tmp/riotbox-1588-recorder-tests.log`.
An intermediate broad App filter also ran the then-red main-owned recorder
case; `/tmp/riotbox-1588-app-queue-rejection.log` is18 pass/1 fail, not a green
App-wide run. The exact owned selections and later recorder result above are
the passing evidence.

## Review findings and disposition

### [P2] Queue-only export rejection incorrectly required sequence capacity

Reviewers: Adversarial Implementation Reviewer (coordinator's integration
review). Location: `product_export/product_mix_export_commit.rs`,
`reject_product_mix_export_request`. An initial new guard blocked rejection
although this helper creates no committed record. It also masked the existing
missing-active-source outcome. Fixed by removing only that guard, retaining
true committing-writer admission, and adding a public MAX control that rejects
explicit and missing-source requests without touching Session, disk or files.
The13-test JamApp selection and8 neighboring ID controls pass as cited above.

### [P2] Finalized take retained a dead pending export after writer refusal

Reviewers: Adversarial Implementation Reviewer (coordinator's integration
review). Location: `cli/live_master_recording/capture_lifecycle.rs`, final
`commit_and_save_live_master_recording` error path. A new early writer refusal
could leave its queued request pending after the controller consumed the
capture. Genuine failing controller regression:
`/tmp/riotbox-1588-recorder-writer-red.log`. Fixed by rejecting that owned plan
on this terminal failure. The regression now proves rejection, unconditional
device stop, no published files/receipt and byte-identical saved Session; all29
controller tests pass.

Independent Torvalds-Inspired Maintainer / Adversarial Implementation Reviewer /
Rust/API specialist review found zero retained P0–P3 issues in the complete
code boundary, including untracked modules. It independently reran Core18,
App/CLI/recorder21 and the all-feature App selection22, all PASS:
`/tmp/riotbox-1588-rust-review-{core-sequences,app-sequences,app-sequences-features}.log`.
The separately independent Spec And Evidence Auditor / Product Pragmatist /
Devil's Advocate And Risk Assessor also found zero retained P0–P3 issues and
independently reran Core18 and App/CLI/recorder21, PASS. Both independently
checked `git diff --check main`, complete-boundary/final-slot controls,
incremental pipeline order, precedence, clock/ack staging, writer admission and
same-controller recorder cleanup. Their code conclusions were blind to each
other and to the two integration findings recorded above; neither is credited
with discovering those findings.
The second review's test logs are `/tmp/riotbox-1588-spec-review-core.log` and
`/tmp/riotbox-1588-spec-review-app.log`. Its subsequent report-only evidence audit
found no material discrepancy; the known exact-log-path typo and public-type /
private-construction wording were corrected. That later confirmation is not
additional independent discovery provenance.

The coordinator's follow-up self-review found no additional P0–P3 issue in
capacity ownership, typed caller migration, mutation/publication ordering,
unchanged assertion preservation, error feedback or stop/abort paths. Existing
large cohesive queue/commit/event-loop owners and the596-line recorder controller
were inspected as module-cost signals, not split into artificial shards.
The six existing UI queue-then-commit controls now share one small boundary/error
helper rather than duplicating admission policy. New arithmetic and exhaustion
tests have semantic child owners; no general error/transaction abstraction or
new durable state was introduced.

Full local source-free `just ci` passed:
`/tmp/riotbox-1588-full-ci.log`, including normal Rust and optional limiter
fixtures, generated audio/observer smokes, access/CI contracts, Decision-search
fixtures, tracked JSON, format/diff checks and strict all-target/all-feature
Clippy. Its implementation candidate is
`1bd37f3bf11dba1d8ba060146fb292c83d71af37`; subsequent changes only record
executed evidence in this report. Exact-head native PR CI is pending. No merge
claim is implied by the local result or zero-findings reviews above.

## Bounds

No new ActionCommand, persistent App state, Session/replay schema, dependency,
DSP/renderer/sound policy, artifact/recording/observer version, source algorithm,
numeric audio gate or public test-only constructor. The broad Rust test diff
migrates fallible consumers while retaining their existing successful behavior
assertions. New exhaustion tests have semantic owners, not textual shards or
a generic transaction/error framework.

No actual source/capture/Holdout/commercial audio reads, source-directory
discovery, old ignored-attempt access, host/service/device observations or audio
runtime launch, playback, DAW or Windows expansion. Generated temporary test
artifacts are not real-source or host evidence. No fresh trial permission,
musical/hardness, latency/endurance, original ALSA-cause or release pass.
RIOTBOX-1041/1036 remain open.
