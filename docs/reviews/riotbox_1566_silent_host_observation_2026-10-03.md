# Silent virtual-host observation — RIOTBOX-1566

Date: 2026-10-03. **V1 attempt failed at the first run's teardown; no accepted
runs and no three-run pass.** The 60 sampled callback intervals and final
Stopped record were healthy, but recycled PipeWire object identity caused a
false teardown rejection. Runs 2 and 3 were not launched. Separate post-failure
cleanup removed only the owned, inactive test sink. Original failed evidence
is unchanged. Source-free CI passed twice; exact-head native PR CI is pending.

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
- Executed clean implementation revision:
  `cf4f76664a50ba8f054e5e2a9fc19016092c89dd`. Both exact binaries were rebuilt
  from that revision after final source-free CI, before host admission.
- The preserved ignored owner is exactly
  `artifacts/development/riotbox-1566/attempt-01`. Its admission/result records
  bind revision, binaries and protocol. It must not be removed or reused.

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

## Pre-host review findings and disposition

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
Backend/startup/terminal/orphan-node rereview was clear before host execution.
Final integrated signal/containment/publication/log-close review had zero
retained P0–P3 findings at that boundary;
the independent reviewer ran 28 focused integration tests successfully.
The coordinator checked module ownership, scope, claim boundaries and
the lack of new ActionCommand, JamAppState, Session or DSP state. The final
self-review had zero remaining findings after the corrections below. This is
not a claim that the later actual attempt found no defects.
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
the host operator. Full source-free `just ci` passed twice
(`/tmp/riotbox-1566-ci.log`, `/tmp/riotbox-1566-final-ci.log`). The final
integrated independent 28-test check also passed. These generated checks missed
the actual metadata shapes below; green CI does not establish a host pass.
The unchanged 58-test recipe passed again after the documentation update
(`/tmp/riotbox-1566-post-result-contracts.log`). Exact-head native PR CI remains
to be recorded.

## Actual observation

The operator admitted an active local non-container Linux user session and
local PipeWire. Child-local ALSA configuration exposed only the exact stereo
null sink; no physical fallback was available. The owned module was
`536870916`, sink Node `76` / object serial `6223`, named
`riotbox_1566_03a286c85c10435fb7a3f7262e77918d`.

- The existing 250 ms preflight passed: PID `1197987`, 23 callbacks, zero
  reported stream/scratch faults, typed Ok, an observed stereo route and
  verified removal. Total supervised elapsed time was 0.499 seconds.
- Run 1 (PID `1198545`) emitted 62 valid records: started, 60 samples, stopped.
  Intervals were 1000–1003 ms and each advanced by 85–87 callbacks. The final
  sample was at 60010 ms; Stopped at 60012 ms, with 5169 callbacks, zero reported
  stream/scratch faults and no last stream error. Stderr was empty.
- All 441 active-route observations matched the owned stereo sink, with no
  foreign, wrong-target or outgoing-sink link in the retained route evidence.
  Negotiated output was ALSA/default, F32, stereo, 44100 Hz. Maximum gap
  27120 microseconds is diagnostic only, not a latency/xrun acceptance result.
- The supervisor rejected teardown and exited 1. The immutable result is
  `failed`, `runs: []`, `cleanup_verified: false`, with
  `UnverifiedStreamCleanup` / terminal deadline exceeded. It retained the
  containment sink. Runs 2 and 3 were not launched. No threshold, algorithm,
  protocol or V1 implementation was changed in response; no retry occurred.

### Newly established findings

1. **P2 — recycled ID is mistaken for the original stream.**
   `scripts/run_silent_host_probe.py::require_removed` tests an admitted node
   ID against every object, without Node type or `object.serial`. The original
   Node `82` / serial `6297` had already disappeared when PipeWire reused ID
   `82` for a `pw-dump` Client. All 35 teardown snapshots lack the original
   stream; 34 contain a new Client at that ID. The last snapshot lacks ID `82`
   but is beyond the deadline. This is a reproduced false rejection, **not
   evidence of a backend stream leak**. Reviewer provenance: coordinator's
   Performance And Operations lens; independent Spec And Evidence metadata
   audit confirmed the attribution and transcript/hash summaries.
2. **P2 — cleanup assumes an absent Pulse JSON field.**
   `remove_owned_sink` reads `matches[0]["index"]`, but the actual host's
   `pactl --format=json list modules` objects contain only `argument`, `name`,
   `properties`, `usage_counter`. A matching module would raise `KeyError`.
   The attempt did not reach this path because its earlier uncertainty guard
   retained containment. This is separately observed metadata plus static
   control-flow evidence, not another executed attempt failure. A source-free
   mock using that schema also reproduced `KeyError('index')` before any unload
   command; no host service was contacted by the reproduction. Bounded
   `pactl list short modules` exposed the exact module identity for subsequent
   cleanup. Reviewer provenance: Performance And Operations lens.

Both findings are **deferred to an explicitly versioned successor**, tracked
in [RIOTBOX-1567](https://linear.app/riotbox/issue/RIOTBOX-1567).
V1 is consumed experimental evidence, not an approved reusable host operator.
Changing its acceptance after the result would violate its frozen contract.
The successor needs typed Node/serial lifetime identity, supported exact module
identity parsing, generated reuse/orphan/schema/mismatch regressions, a new
Decision and independent review. Neither that ticket nor this report authorizes
another host attempt. Post-result self-review retains exactly these two P2
tooling findings; no product/DSP change or musical progress is claimed.

### Post-failure cleanup, separate from qualification

The coordinator's read-only checks established that both launched PIDs, the original stream
serial, attributed Clients, targeting streams and all links to/from the owned
sink were absent. The first standalone cleanup precheck made **no mutation**:
the system default had changed from HDMI at 65% to analog stereo at 45%, and
the original HDMI sink was no longer present. The cause/actor of this change
is unknown; the original-baseline preservation condition is not established.
No original default or volume was restored blindly. The first follow-up JSON
records a bare `AssertionError`; its exact failed assertion/no-mutation point
comes from coordinator inspection of the cleanup script and command result,
not from that JSON alone. The HDMI sink's absence and module JSON field shape
were also coordinator-observed terminal metadata, not independently established
by the cleanup records.

A separate, bounded cleanup captured the current state, verified the exact
owned module ID/name/arguments and sink Node/serial again, and established that
it was neither the default nor in use. It unloaded only module `536870916`,
verified module and sink absence, and verified current default/mute/volume
unchanged. No CPAL/audio runtime or playback was launched. This cleanup succeeded, but does
not change the failed attempt's result or its original cleanup field. Both
cleanup follow-up records and all original artifacts are retained.

An independent post-result Spec And Evidence review verified the failed-attempt
narrative and both cleanup record hashes/identity chain. Its P3 correction
narrowed an overly broad "no process launched" statement to no audio runtime
or playback; metadata command processes necessarily ran. Coordinator-only
observations are identified above. The final documentation self-review retains
the two disclosed P2 tooling defects, with no remaining documentation finding.

### Exact evidence identity

SHA-256 values below refer to the ignored owner unless a binary is named:

| Artifact | SHA-256 |
| --- | --- |
| `target/debug/cpal_spike` | `19f433c0e3aa1efe6179956ae003e6c071cb4e001966bbff3231cb5e8d919c54` |
| `target/debug/silent_host_probe` | `3f0742db8f3cdc9ada17d1f719dd108aa3bebbe2db69c1ac5a26a09f3fe09cb4` |
| `admission.json` | `468fed5ddda00d97578f29d45eac6e3352d71e43e4d7c2f23540b6bd1fd53ce8` |
| `result.json` | `bb020e9a008b8e5073e8b84ae77ef2dbbdd4ced128868131587a09e8e8fbac35` |
| `pipewire-only-alsa.conf` | `149aae9fc06993eb389cd5b23e119fcfb59163cd428371056aa3198c7bd321a6` |
| `preflight.stdout.log` | `c0844b365847c0eb42b8e5de140b12f74af9c97a9ca3ad75ac0b0c8ff7b828df` |
| `preflight.routes.ndjson.gz` | `c916a05e7ccb4e1f7d5391d23164684fc68d4e74a1aa83219fa8e9b1acbd1869` |
| `run-1.stdout.ndjson` | `ab7bdb8cdd79c1b3c8e366ffc783370bb696325ddc8da3e810f519288083232a` |
| `run-1.routes.ndjson.gz` | `9c6e64e708bd4564d1a3fa03e27345d016796a30aa249e80b92f3f8d20201f6c` |
| `cleanup-followup.json` | `570bfce9b671cb90eb1ab5f33dc1f8aebb163614f7941ac3a1ae4c9f0acdeae7` |
| `cleanup-current-state-followup.json` | `9e98cf880debaf2cb57d00e087708b540172f6a7731852dbf941575d619eed56` |

No source/Holdout/commercial/capture audio or source directory discovery,
human playback, DAW or physical-output qualification is part of this work.
The healthy intervals are only a partial sampled silent virtual-endpoint
observation inside a failed lifecycle attempt. They cannot establish three-run
continuity, loaded musical playback,
continuous activity between samples, physical/acoustic behavior, latency,
xrun absence, general endurance, musical/human taste, source-general quality,
release readiness or RIOTBOX-1041 completion.
