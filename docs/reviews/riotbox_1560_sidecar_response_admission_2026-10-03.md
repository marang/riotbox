# Sidecar response admission — RIOTBOX-1560

Date: 2026-10-03. Base: `e80cb816e9510086b2e2cb2c46296728d2d6a17d`.
Classification: maintenance/regression of the P023 ingest process boundary.
The concrete risk is unbounded host framing allocation from a fast faulty
analysis peer despite an otherwise bounded I/O deadline.

## Change and contract

RBX-430 and Technology Stack §3.4 freeze an 8 MiB per-response wire limit,
including CR/LF when present. Read sizes respect remaining space; one overrun
probe byte can be read but never appended. UTF-8 and JSON decode only admitted
frames. Geometric growth and the small trailing-frame copy use fallible
reservation. The completed frame's large allocation leaves the client.

Both control and analysis return explicit typed size/allocation errors through
the existing fatal transport path: close pipes, terminate/reap the direct peer,
and require a fresh client for reuse. Ordinary complete analysis errors still
leave a synchronized peer usable. Protocol 0.1, the single absolute deadline,
coalesced frames and final valid EOF without newline remain compatible within
the admitted size. At exactly the limit with no terminator, EOF or one more
byte is still needed; the deadline continues to apply.

The budget is a deliberate resource policy, not a graph-size measurement or
universal-source claim. Legitimate graphs above the limit fail visibly without
truncation. No Action, queue/commit surface, Session/replay model, app facade
state, dependency, worker, configuration or audible mechanism changes.

## Evidence

- RED: the new public-client regression failed because an oversized valid Pong
  was accepted (`accepted an over-limit valid response`).
- GREEN: generated metadata-only peers cover exact and excess LF/CRLF/EOF,
  still-open unterminated excess, control/analysis error context, cleanup and
  unavailable reuse, UTF-8 byte counting and coalesced frames at/near the limit.
- Buffer tests check growth at the cap, refusal to append/reserve an overrun
  byte, trailing-byte preservation and release of completed large capacity.
- Focused Sidecar suite: 32 unit tests and 1 integration test passed, including
  existing deadlines, protocol/error handling and the one-MiB transport control.
- Release profile: all 6 response-admission tests passed.
- Full source-free local `just ci`: passed, including Rust/Python suites,
  synthetic audio/observer/manifest contracts, JSON/docs checks and strict
  all-feature Clippy. Final focused rerun also passed after adding an explicit
  assertion pinning V1 to exactly 8 MiB.

Transient evidence: `/tmp/riotbox-1560-red.log`,
`/tmp/riotbox-1560-green.log`, `/tmp/riotbox-1560-sidecar-tests.log`.
Release evidence: `/tmp/riotbox-1560-release.log`.
Full gate: `/tmp/riotbox-1560-ci.log`; final focused gate:
`/tmp/riotbox-1560-final-focused.log`.

Jev was used only as a bounded semantic check of five evidence statements,
including three deliberately overstated controls. `jev-1.13.0` supported the
two bounded claims and rejected total-process RAM, immediate-at-limit rejection
and real-heap-exhaustion-test claims, matching the coordinator's manual check.
One batched call used 1262 input / 232 output tokens, took 0.35 s and cost an
estimated $0.000053. This is not an independent code review, coverage proof or
product integration. Input and raw judgments are retained transiently under
`/tmp/riotbox-1560.0ypG5h/jev-claims{,-result}.json`.

## Branch review

Two independent reviews retained no P0–P3 findings: Spec and Evidence Auditor,
and Adversarial Implementation Reviewer with Rust/API/performance focus. Both
traced complete framing, client invalidation and downstream error propagation,
checked contract/test claims and independently ran the 6 response-limit and 3
buffer tests. The Rust reviewer additionally ran all 9 deadline regressions.
They inspected the historical RED/GREEN evidence; allocation-failure cleanup
was inspected, not fault-injected. The exact-limit, still-open timeout combination
has adjacent EOF/overrun/deadline tests and code inspection, not a dedicated
regression. Native CI remains the PR gate.

The coordinator's short self-review covered cap arithmetic, fallible tail
preservation, coalesced response ownership, error propagation before App
publication, scope and module growth; no remaining finding was retained.

## Limits and review boundary

The bound is on framing bytes/requested capacity, not total RSS, allocator
overhead, parsed JSON, outgoing requests, Python analysis or concurrent clients.
Global heap exhaustion is not induced in tests. The direct peer is controlled;
process-tree containment and kernel scheduling remain outside the existing
deadline contract. No real source, Holdout, commercial reference, host audio,
DAW, playback or musical qualification is involved.

The transport remains one cohesive small owner; admission tests have their own
ordinary Rust module. Client error mapping stays with the existing client.
No mechanical module split is required. This is the second substantive slice
after the RIOTBOX-1558 current-state architecture checkpoint.
