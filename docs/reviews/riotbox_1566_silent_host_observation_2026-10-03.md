# Silent virtual-host observation — RIOTBOX-1566

Date: 2026-10-03. Status: implementation, generated tests and independent review
complete; final source-free CI pending. **No CPAL launch, null-sink creation
or actual observation has occurred yet.** The user resumed the previously
paused, preserved worktree. This report supersedes its temporary pause handoff.

## Scope and identity

Maintenance / observation for partial P017 / RIOTBOX-1041 evidence on the
accepted P023 callback path, not a new musical mechanism. The
[V1 protocol](../benchmarks/silent_host_observation_v1.md) permits one existing
250 ms admission probe followed by three separate 60-second silent-default
processes. All lanes remain idle, transport stopped and sources absent.

- Branch: `feature/riotbox-1566-silent-host-continuity`.
- Base: `728790ccaf2b0f31ad473ddd1bde238dd5874fe8`, already integrated by
  PR #1668 / merge `07419e47182b5717ff86d41210432f24433ecbe3`.
- RBX-436 and the numeric protocol were written before any host observation.
- Protocol SHA-256, unchanged throughout implementation:
  `ba26bc50db85a6b745738917100689a7ca5ac3c2aa95cc7c699db2ef404358d3`.
- The future ignored owner is exactly
  `artifacts/development/riotbox-1566/attempt-01`. It does not exist yet.
  Its admission/result records must bind revision, binaries and protocol.

## Implementation and boundaries

The Rust dev driver reuses `AudioRuntimeShell` and the pure RBX-435
`AudioProbeSummary::from_health` projection. Its private observation seam tests
60 one-second wait requests using a generated host/clock, callback progress,
raw elapsed/health reporting and explicit stop after observation/output failure.
Default runtime/DSP behavior is unchanged. There are no source/action setters.

Python owners separate transcript validation, effective PipeWire attribution,
Linux process supervision, scoped interruption and the child-local backend
environment. The fixed operator composes these seams; they create no
Core/Session/replay state or substitute telemetry authority. A private
PipeWire-only ALSA config precedes CPAL opening; effective PID→Client→Node→
stereo-link evidence is still mandatory afterward. Original defaults, mute,
volume and unrelated streams are never changed deliberately.

GNU `timeout` bounds the trusted driver independently. The group leader is
kept unreaped until owned-group signaling is finished. Failed or uncertain
process/stream removal retains the containment sink with exact diagnostic
identity. Signals unwind through cleanup; no failed attempt can be retried.
This is not cgroup containment against deliberate child escape, SIGKILL/host
loss recovery, crash-durable artifact storage or a metadata-memory bound.

## Review findings and disposition

The independent Adversarial Implementation Reviewer identified these findings;
behavioral reproductions used generated fixtures where noted:

- P1: unconstrained ALSA `default` could open hardware before route checks.
  Repaired with a standalone child-local PipeWire-only config and sanitized,
  coherent local endpoints. Evidence is static inspection plus primary-source
  loading/property verification, not a reproduced physical/PCM open.
- P2: late first-route observation and terminal drain could bypass their
  five-second budgets. Pre-admission checks and one terminal deadline now
  cover stop, process exit and stream removal. Generated-clock RED/GREEN
  includes a positive 61-second finish and a rejected 66-second finish.
  Late stopped transcripts also fail independently.
- P2: a Node could outlive its Client and evade teardown. Removal now checks
  the admitted node/target as well as PID attribution; a generated orphaned
  node regression fails closed.
- P2: TERM/HUP could skip cleanup; interruption during publication could leave
  success. Scoped signals, latched interruption and same-owner result
  correction cover observation, cleanup and publication.
- P3: pending signal delivery during installation could leak handlers.
  Transactional rollback restores original handlers and masks; generated
  INT/TERM/HUP entry regressions exercise the failure.
- P2: an enclosing route-log close error could mask typed uncertain cleanup
  and bypass containment retention. A generated reproduction established this;
  the explicit log owner now preserves the primary exception and attaches close
  diagnostics. Generated group/stream uncertainty and ordinary-close controls
  pass; the independent reproduction now retains typed cleanup uncertainty.

The independent Spec And Evidence Auditor found and reproduced:

- P2: sink disappearance could pass after its total cleanup budget. Deadline
  enforcement now includes queries/unload and precedes absence acceptance;
  generated delayed-metadata RED/GREEN proves the repair.
- P2: failed stream removal did not reach outer containment cleanup. A typed
  unverified-stream failure preserves PID/node/cause notes and blocks sink
  removal. Generated attempt-boundary tests verify retained containment,
  failed evidence and no replacement launch.
- P2: the initial publication repair missed an interruption during file close.
  The Spec And Evidence pass reproduced this remaining boundary; publication
  now checks after flush/close and corrects only its newly created file after
  verifying filesystem identity. A dedicated close-interrupt regression passes.

Independent Rust review found no remaining issue and ran all 11 driver tests.
Backend/startup/terminal/orphan-node rereview is clear. Final integrated
signal/containment/publication/log-close review has zero retained P0–P3 findings;
the independent reviewer ran 28 focused integration tests successfully.
The coordinator checked module ownership, scope, claim boundaries and
the lack of new ActionCommand, JamAppState, Session or DSP state. The final
self-review has zero remaining findings after the corrections below.
In that Spec And Evidence self-review, metadata overlapping the final sample
reproduced another possible deadline extension (RED). A conservative prelaunch
plus final-sample elapsed anchor now prevents it; generated late-removal and
inside-budget controls cover the repair without changing any numeric limit.

## Reproducible source-free checks

Commands (none opens CPAL, creates a sink or reads real source audio):

```sh
cargo test -p riotbox-audio --bin silent_host_probe
cargo test -p riotbox-audio --lib probe::
just silent-host-contract-fixtures
just ci
```

The driver has 11 tests; process supervision has 14. The complete Python
recipe covers 58 generated test functions spanning transcripts, routing,
deadlines, environment, supervision, actual signals, attempt containment and
publication. Missing-module RED is recorded as such, not as a behavioral
reproduction. A generated route-loss fixture was repaired to count exposed
routes rather than pending startup snapshots; otherwise scheduling could
prevent the intended post-admission fault from being reached.
The native Linux workflow includes the new source-free recipe and never runs
the host operator. The first full source-free `just ci` passed
(`/tmp/riotbox-1566-ci.log`); final delta checks/native CI remain to be recorded.

## Actual observation

Not started. After final independent review and source-free checks, commit the
implementation, build both exact binaries from that revision and execute the
single frozen attempt only if admission succeeds. The first failure ends it
without tuning, retries or physical fallback. Retain metadata and report
residual cleanup precisely.

No source/Holdout/commercial/capture audio or source directory discovery,
human playback, DAW or physical-output qualification is part of this work.
Even a successful future result is only sampled silent virtual-endpoint
callback/lifetime evidence. It cannot establish loaded musical playback,
continuous activity between samples, physical/acoustic behavior, latency,
xrun absence, general endurance, musical/human taste, source-general quality,
release readiness or RIOTBOX-1041 completion.
