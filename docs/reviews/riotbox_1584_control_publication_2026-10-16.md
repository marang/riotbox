# Bounded control publication — RIOTBOX-1584

Date: 2026-10-16. Maintenance/regression on the accepted P023 live-control and
recording path; no new musical behavior or source claim. Implementation base:
`245e09dbee9363008ed862f01e57af58a5261c9c`, following the integrated1583 report
and requested Done cleanup. A separate workflow-only commit16c63b40 records
actual deletion dates and the retained1587 followup; it changes no product code.

## Defects and repair

The1583 checkpoint retained a P2: independent Relaxed payload atomics could
escape equal Acquire revision guards. Both its byte-identical extract and the
subsequent actual-source Loom test accept `(false,132,8)` between initial
`(false,132,0)` and the sole `(true,144,8)` update. This is control→callback
publication, not1582's separate observed callback→App owner or an ALSA incident.

Two coupled assumptions were separately challenged rather than hidden:

- After payload ordering alone is repaired, the old startup reader still
  returns an unchecked mixed tuple after exhausting its attempts.
- A generated compile-time control confirms the public Linux shell was Sync;
  its `&self` update methods did not enforce the assumed single control writer.

The private `control_snapshot` owner now returns a validated value or None
after at most three attempts. Every participating scalar, optional field and
sample element uses Release stores and Acquire loads. The first committed
revision orders its payload before reads; a later payload observation orders
its preceding odd revision before the final guard, preventing acceptance of the
old even generation. Existing revision orderings remain AcqRel/Release/Acquire.
This relies on atomic coherence, synchronized initialization, one writer and
no full revision-cycle ABA, not platform-specific Relaxed behavior.

Actual transport representation/publication lives in private `transport_timing`;
Loom imports those exact production modules and substitutes only atomic
primitives. Other domain owners retain their representations. The W-30 cache
keeps its silent initial fallback, unchanged-revision borrowed-array fast path
and previous complete payload/revision on rejected reads. No callback lock,
allocation, logging, I/O or unbounded retry was introduced.

Processor preparation rejects unavailable required transport/TR-909/MC-202/
resample groups through typed errors before consuming unique callback readers.
It does not fabricate a requested observation or seed fallback from unchecked
fields. Unshared offline preparation and stepped mix plans instead project
through private exclusive mutable borrows, without panic, silence substitution,
duplicate projection logic or reconstructing publications for every block.

The shell's private `PhantomData<Cell<()>>` enforces !Sync. This intentionally
restricts the **public auto-trait contract**; existing method signatures remain.
The marker itself preserves Send, checked on the supported Linux build. The
unique control owner may move where its backend permits; it cannot be shared
for concurrent `&self` updates. No new mutex/poison policy or observer API is
introduced. Separate callback ownership and1582 observed timing stay unchanged.

RBX-449 and AudioCore §11.4 own this decision. All-SC payload stores and
unmodeled fence alternatives were rejected for this slice; the large W-30
arrays motivate avoiding per-sample SC writes, not a measured performance claim.

## Executed evidence

Retained implementation-agent RED logs under `/tmp/riotbox-1584-`:

- `control-actual-red.log`: exact production transport/protocol model rejects
  the original mixed tuple before the payload repair.
- `startup-actual-red.log`: the same actual source still fails with stronger
  payload ordering while unchecked startup fallback remains.
- `control-owner-red.log`: compile-fail ownership control wrongly compiles
  before the marker, proving the missing public single-writer restriction.

Coordinator independently rebuilt/reran the original1583 extract counterexample;
the new actual-source RED/GREEN selections below were run by the implementation
agent. Model proof is not full-runtime or hardware evidence.

| Focus | Result |
| --- | --- |
| Actual-source control Loom: update, startup, committed, repeated fields, active update, three-attempt bound, exclusive projection | 7 PASS |
| Existing observed-timing and capture-admission Loom | 4 PASS |
| Exact production callback / cold heap / generated output / startup rejection | 8 PASS |
| W-30 snapshot cache | 12 PASS; 1 manual benchmark ignored |
| Shared render groups / lifecycle (including Send) | 6 / 10 PASS |
| Offline parity / fixture regressions / stop-output | 5 / 5 / 3 PASS |
| Public !Sync compile-fail doctest | 1 PASS |
| Audio crate all-target/all-feature strict Clippy, diff whitespace | PASS |

Logs: `/tmp/riotbox-1584-{control-final,callback,arrays,cache,shared-render,lifecycle,parity,fixture-regressions,stop-output,control-owner-green,clippy}.log`.
Selections overlap: three full-array/threaded controls overlap other groups;
seven isolated callback child invocations are not additional top-level tests.
W-30 full-array controls are threaded tests, not exhaustive weak-memory models.
No new latency or cost benchmark was run. Exact final log names and independent
review/full local/native gate results are recorded in Linear as verified; no
pending gate is inferred from these targeted passes.

## Scope and remaining gates

No ActionCommand, Core/Session/replay schema, persistent App state, DSP/sound
policy, recording/observer/artifact version, rate/buffer/priority/audio threshold,
dependency, public test constructor or general framework changes. No real
source/capture/Holdout/commercial audio, discovery, ignored attempt, actual
host/service/device/audio launch, playback, DAW or Windows expansion. There is
no musical/human/hardness/source-general/latency/endurance/release qualification.
The original ALSA cause and broader1041/1036 remain open; prior phases stay
consumed. RIOTBOX-1587 separately owns checked ActionId exhaustion.

Independent Maintainer/Adversarial Rust Concurrency review and separate
Spec/Evidence/Product Pragmatist review of `245e09db..1de0831b` retain zero
P0–P3 findings. Both independently run the seven actual-source Loom controls
and !Sync doctest; the maintainer additionally runs cache2 + protocol5,
lifecycle10, shared-render6 and exact callback8 (overlap, not unique totals).
Logs: `/tmp/riotbox-1584-independent-{control-loom,w30-cache,w30-protocol,shell-traits,lifecycle,shared-render,callback}.log`
and `/tmp/riotbox-1584-spec-review-{loom,owner}.log`.
The coordinator's short self-review also retains no additional finding.

Full source-free local CI, exact-head native CI and normal PR/merge/main/Linear
closeout remain gates, recorded in Linear as completed. This report is not an
advance pass for them or a claim that RA alone proves all runtime behavior.
