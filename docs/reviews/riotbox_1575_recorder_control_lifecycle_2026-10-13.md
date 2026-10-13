# RIOTBOX-1575 — recorder control lifecycle

Date: 2026-10-13. Base: `d3e5a28e76f3b8615a841014143cd1ca0344c226`.
Classification: P016 maintenance/regression under RIOTBOX-1036, protecting the
accepted P023 recording journey. This removes a control-path testing blind
spot, not the original ALSA trigger or a musical/reliability qualification.

## Semantic ownership and preserved contracts

The CLI facade retains observer path preflight, Session hydration, actual
device startup and final shell construction. The private `capture_lifecycle`
owns the existing transport confirmation, timing-fed App/Core action commits,
recording plan queueing, callback-capture supervision, stop/finalization and
existing `commit_and_save_live_master_recording` delegation. Production and
tests execute that same owner through real-device and generated-observation
adapters; the test adapter never emulates queue/Session/publication policy.
Monotonic time and action timestamps are dependencies, not hidden product truth.

Session, replay, receipt, artifact transaction and capture admission retain
their existing owners. No public Audio constructor, ActionCommand, dependency,
DSP, renderer, source policy, TUI/DAW surface or persistent JamAppState state was
added. The App facade is borrowed, not copied through controller frames; shell
construction happens after that frame returns. RBX-442 pins this ownership.

Confirmation keeps exact BPM bits, two seconds and 10-ms polling. Capture keeps
arm-distance + frames/rate + two seconds and precedence runtime fault → capture
fault → completion → timeout. Completion still wins at the deadline. Preserve
pre-stop health for commit, verify stopped lifecycle before finish, and retain
the actual callback tap/writer. Only transport started by this CLI is paused.
Historical V1/V2 evidence and V3/V4 duration, geometry, health, proof/receipt and
CLI/observer contracts are unchanged. Duplicate idempotent stops in existing
failure branches are retained rather than combined with a behavior rewrite.

## Findings and repair evidence

- **P2, Rust ownership/performance lens — initial extraction exhausted the
  normal test-thread stack. Fixed.** The exact already-playing lifecycle test
  failed twice; the early-confirmation-abort case also failed without recording
  or publication. GDB showed ordinary transport commit/view-refresh frames,
  not recursive adapter dispatch. Passing/returning the large facade by value
  across extra frames added stack pressure. The owner now borrows it and returns
  only its ephemeral JSON summary. The exact test and full twelve-test loop pass
  on ordinary thread settings; no stack-limit change or heap bypass was used.
  Logs: `/tmp/riotbox-1575-stack-red-{actual,repeat}.log`,
  `/tmp/riotbox-1575-stack-minimized.log`, `/tmp/riotbox-1575-stack-green.log`.
- **P3, Spec/Evidence lens — unconfirmed stop was projected as stopped.
  Fixed.** A generated adapter retained Running after stop; the existing
  lifecycle correctly refused finalization but its blocked JSON still reported
  `runtime_stopped=true`. The exact new assertion failed, then passed after the
  flag was derived from the post-stop health snapshot. Ordinary real-shell stop
  still yields the same true value. This is a defensive failure-projection
  correction, not a demonstrated physical-backend shutdown defect.
  RED: `/tmp/riotbox-1575-stop-projection-red.log`; GREEN: lifecycle log below.

A first incorrectly qualified exact filter selected zero tests and is excluded
from proof. The first queue-rejection fixture also assumed that clearing the
active Scene would survive the existing transport-start normalization; it did
not. The corrected negative case is a genuine occupied destination, whose
foreign bytes stay unchanged. Neither preparation mistake is a product finding.
No debug instrumentation or throwaway source survives.

## Verification and limits

Baseline before edits: 62 source-free `live_master` tests pass
(`/tmp/riotbox-1575-baseline.log`). Twelve new generated lifecycle tests pass in
0.85 seconds (`/tmp/riotbox-1575-lifecycle-tests.log`), and the combined existing
plus new 74-test family passes (`/tmp/riotbox-1575-recording-regressions.log`):

- complete 2/8/16-bar actual queue/publish/save/restore; confirmed delayed
  transport and actual Core commits from timing snapshots;
- CLI-started versus preexisting playing transport, exact-BPM mismatch,
  missing confirmation and confirmation faults;
- queue rejection/collision, existing pending action and arm failure without
  aborting a capture not acquired by this attempt;
- runtime/scratch/capture faults, missing/incomplete progress, exact timeout,
  completion-at-deadline and fault-before-completion precedence;
- verified stop before finish and publication, failed stop/finalization,
  late capture fault despite an earlier clean snapshot;
- Session-save rollback, no final receipt/artifacts on failure and unchanged
  foreign destination/Session targets.

Existing CLI parse/path/observer tests and numerical publisher/callback tests
remain complementary: they cover contracts outside the new control seam.
Generated observations and PCM are test controls, never a real host/callback
recording, source intelligence, listening or release evidence. No real source,
capture, Holdout, commercial reference, ignored previous attempt, audio service,
device, broad QA, DAW or human playback was accessed. No host phase was retried.
The RIOTBOX-1572 error stays unexplained and RIOTBOX-1041 remains open.

Independent Spec and Evidence Auditor review of code and final documentation,
and Adversarial Implementation Reviewer with Rust/API/Maintainer lenses, retain
zero findings. Both independently ran all twelve lifecycle tests; the latter
also passed three observer compatibility tests. The coordinator's short
self-review found no remaining defect. These focused suites overlap and are
not additive coverage counts.

Full local source-free `just ci` passes, including strict all-target/all-feature
Clippy (`/tmp/riotbox-1575-ci.log`). Native CI and normal PR/merge/main/Linear
closeout still gate completion; exact final outcomes and review dispositions
are recorded in Linear and the project journal.
