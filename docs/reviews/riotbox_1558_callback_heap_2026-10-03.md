# RIOTBOX-1558 — Production callback heap audit

Date: 2026-10-03. Maintenance/regression, third slice of the approved autonomous
batch after RIOTBOX-1556/1557. Base: `170a88a6a24ce97046cbbdf130851e097422b3de`.
Contracts: RBX-427 and its admission-only successor RBX-428; Audio Core
sections 7.2, 11.4 and 11.5. No new audible mechanism or listening verdict.

## Demonstrated defect and repair

The new private processor factory contains the exact render body CPAL invokes;
the device adapter only forwards its buffer. Test preparation is on control,
followed by a fresh worker in an isolated child process. No consumer warmup.

The original stopped, empty-source first callback recorded `alloc=1`,
`requested_bytes=128`; all other heap operations were zero. Its following
callback and an empty accounting window were zero. A diagnostic first source
snapshot reproduced that allocation and made the subsequent callback warm.
The installed ArcSwap debt registration allocates on its consuming thread;
control warming is not a cold-worker repair. Diagnostic branches were removed.

Source Monitor and capture now use preallocated borrowed SPSC publication.
Serialized control writers own authoritative latest payloads separately from
recycled producer slots. Unique callback readers only borrow immutable slot
payloads; refresh neither clones nor drops their Arcs. Producer publication
reclaims old payloads on control, with delayed fixed-slot retention. The actual
mix, transport clock, source policy, limiter, capture tap and conversion order
are unchanged. CPAL preparation refuses duplicate consumers rather than using
an allocating/locking fallback. ALSA stream destruction joins its worker before
the production shell releases publisher owners; tests keep those owners alive
through worker/reader teardown.

Capture activity is no longer inferred from reference counts. RBX-428's single
atomic word holds a permanently closed bit and in-flight count. Its registration
RMW linearizes acceptance against closure; borrowed lease release follows all
sample/timing work. Finish retains the completeness precheck, then closes before
quiescence/detach; active work preserves nonwaiting `CallbackStillActive`.
Abort closes before detach, and stale slots cannot resume writes. No new runtime
unsafe protocol, CAS/spin loop, product state or Session/replay authority.

## Technical evidence

Test-only TLS allocator accounting covers `alloc`, `alloc_zeroed`, `realloc`,
`dealloc` and requested bytes. It forwards System contracts unchanged, uses
const-initialized nonallocating counters and resets on unwind. Measurement spans
the complete processor invocation and local drops. Output/fixture allocation,
logging/assertions, channel synchronization, control publication and whole
processor/worker teardown are outside it. Positive-control pointer black boxes
prevent release optimization from deleting unused test allocations.

- Cold stopped empty F32 regression: first and following callback heap counts
  are all zero after the repair, without warmup.
- Six fresh-process prepared-source/active-capture cases: F32/I16/U16 normal
  and first-call scratch overflow; every measured invocation is heap-zero.
  Generated PCM is nonzero, finite and bounded; captured float samples match
  device output within only format quantization. Overflow is silent and records
  the expected scratch fault without resizing.
- An additional fresh-process F32 mix case prepares active TR-909, MC-202,
  W-30 and resample lane states with generated-only data, retaining the same
  first/warm heap counters and captured/device float sample identity. It is
  composite technical output, not individual-lane or musical qualification.
- Actual worker/control lifecycle: source replacement/control retention,
  complete finish/detach, abort/new capture, armed wait/aligned copy, changed/
  rejected scalar and W-30 reads, reuse, source unavailability and transport stop.
  Stop keeps the existing 5 ms/240-frame fade; zero follows its endpoint.
  Running missing-source silence is tested independently of stopped transport.
- Bounded two-thread stress: 512 actual callbacks and 64 control publication/
  capture-abort cycles. This samples concurrent execution, not all schedules.
- Source reader tests demonstrate coherent held payloads, authoritative latest
  PCM, bounded slot retention, unique acquisition and actual control-side
  reclamation with weak ownership and deallocation counters. Capture reader
  tests distinguish ordinary Arc ownership from held admission leases, reject
  stale sample/timing writes and preserve incomplete finish.
- Two Loom models compile the canonical admission source against modeled
  atomics. They retain unfinished-work exclusion, completed-payload visibility,
  rejection cleanup, permanent closure and independent replacement-capture
  assertions. No permutation/preemption/duration pruning; 1000-branch guard
  fails rather than producing a partial pass.

The two-atomic predecessor failed that unchanged model. Installed Loom 0.7.2
documents weaker SeqCst modeling, confirmed independently from its source; this
was not a demonstrated all-SC hardware race. The single-word successor avoids
that dependency and passes the unchanged assertions.
[Loom limitations](https://github.com/tokio-rs/loom/blob/v0.7.2/README.md),
[upstream discussion](https://github.com/tokio-rs/loom/issues/180).

Local logs: cold RED `/tmp/riotbox-1558-cold-clean-red.log`, cold GREEN
`/tmp/riotbox-1558-cold-green.log`, original model RED
`/tmp/riotbox-1558-admission-loom-red.log`, model GREEN
`/tmp/riotbox-1558-admission-loom-green.log`. Final release Audio unit suite:
`/tmp/riotbox-1558-audio-release-sealed.log`, 327 passed, 1 existing ignored.
Full source-free local `just ci` passed: `/tmp/riotbox-1558-ci-sealed.log`.
The final active-mix level correction is also executed in the final release
suite and targeted Debug callback suite
(`/tmp/riotbox-1558-final-callback-debug.log`). Exact-head native PR CI, merge
and synchronized-main/Linear closeout are recorded on the PR and project journal;
they remain merge gates, not assertions inferred from these local checks.

## Review disposition

Independent Rust/concurrency review found no production correctness issue.
Independent source/factory/spec review excluded its author's capture changes.
Root separately applies Maintainer, Product Pragmatist and Risk Assessor lenses.

Findings fixed before PR:

- P2, Spec/Evidence and API compatibility: a new test incorrectly required
  immediate stop silence despite the unchanged 240-frame ramp. Corrected the
  test's endpoint; production DSP was not adjusted.
- P3, Spec/Evidence: missing-source silence was initially checked only while
  stopped. Restarted transport before the independent missing-source check.
- P3, Test evidence: total health assertions could mistake a legitimate >100 ms
  test-process scheduling pause for semantic failure. Gap telemetry remains
  recorded; heap/identity tests assert non-gap protocol faults separately and
  do not claim device deadlines.
- Release positive control exposed an optimized-away unused zeroed allocation;
  pointer black boxes preserve actual operations and the strict counts pass.
- P3, Spec/Evidence: the additional active-mix fixture initially inherited a
  zero drum-bus level. Set its explicit nonzero level so TR-909 is not silently
  muted; the output assertion remains composite, not per-lane qualification.

Independent final rechecks closed the listed findings with no remaining
P0–P3 in their branch scopes; full local CI passed and native CI gates closeout.
The broader cadence audit is in
`riotbox_1558_current_state_cadence_2026-10-03.md`; its pre-existing ingest
identity finding is separately tracked in RIOTBOX-1559, not fixed here.

## Boundaries and residual risk

Maintained upstream triple_buffer 9.0.0 is unchanged MPL-2.0 code, MSRV 1.86;
its licensing obligations are separate from Riotbox's source-available license.
Loom is dev-only. Fixed slots/retired owners can delay large PCM reclamation
until publication or shutdown; observer Arcs extend it further. No aggregate
RAM, whole-worker heap-free, universal callback deadline or dependency-internal
model proof is claimed.

No real source/capture/Development/Holdout/commercial access or discovery,
device/runtime/DAW launch, playback, musical/human/hardness/source-general or
release qualification. Output proof here is synthetic execution of the actual
production processor, not physical backend/device endurance evidence. Public
Actions, Core/Session schemas, timing geometry, existing sound policy and README
remain unchanged; Windows support was not expanded.
