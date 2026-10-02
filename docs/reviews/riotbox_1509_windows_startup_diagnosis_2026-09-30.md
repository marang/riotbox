# RIOTBOX-1509 — Windows startup diagnosis remains open

Date: 2026-09-30. Maintenance/regression investigation; **no fix claimed**.

## Exact failure and feedback loop

Docs-only archive PR #1560, head `55996da45842082fe27f7bd62163f36c68912308`,
native run `36779055802` attempt 1, Windows job `110104227509`: the actual
`cargo test --locked -p riotbox-sidecar` invocation failed 8/24 unit tests in
22.57 seconds. Eight initial protocol handshakes hit the unchanged 10-second
Control deadline: both analysis-readiness tests, protocol mismatch,
requestless error, bundled ping, synthetic file analysis, unsupported file,
and coalesced frames. Later backpressure, partial-response, EOF, Unicode and
peer-reuse regressions passed. This is a failure of expected fixture readiness,
not evidence that a timeout failed to bound the caller.

The native test command is the red-capable loop: it exercises the original
client/fixtures and exact assertion failures. Its CI relay takes minutes to
provision/build; the actual failing test took seconds. Linux is not a substitute
for the Windows scheduling/pipe environment. No real audio/source was used:
the test named `real_source_file_path` creates its own synthetic WAV.

## Bounded probes

| Native probe | Result |
| --- | --- |
| Same archive head, run `36779055802` attempt 2, job `110107867278` | 24/24 pass, 2.44 s; subprocess test also passes |
| Independent Core-view head `057a2209`, run `36780046907`, job `110107722701` | 24/24 pass, 1.93 s |
| Change only test threads to 24, probe head `00eeb9d1`, run `36780474110`, job `110109159478` | 24/24 pass, 2.00 s; subprocess test also passes |

Linux baseline and 24-thread probe both pass all 24 unit tests plus the
subprocess launch regression. Logs: `/tmp/riotbox-1509-baseline-sidecar.log`
and `/tmp/riotbox-1509-parallel24-linux.log`.

Ranked hypotheses were concurrent cold interpreters, cold host/filesystem
startup, and a pattern-specific pipe defect. Increasing concurrency did not
reproduce the failure; it does not establish concurrency as the cause. The
failed and successful same-head retry both used Windows image
`20260925.250.1`; the other successful runs used `20260922.246.2`. Image
variation exists, but image version alone does not explain the failure. All
reported Python versions were 3.12.10. Hosted runner labels are not an exact
image-build pin: [GitHub runner-image policy](https://github.com/actions/runner-images/blob/main/README.md).

## Disposition and review

The failure is sporadic and its triggering condition remains unpinned. The
diagnosis workflow stops causal/fix claims at this boundary: no reliable
minimized red loop exists yet. A passing retry is not a repair. Restore the
original CI command; do not merge the 24-thread experiment as a fix, serialize
tests without evidence, widen production/test timeouts, skip failing tests or
alter the transport. Production code and all original assertions remain intact.

RIOTBOX-1509 remains open for fresh native failure evidence or a controlled
reproducer. Future failure capture should distinguish interpreter launch,
first request write and first response availability rather than log everything.
This investigation report may merge separately without completing the issue.
Sequential solo review found no correctness or scope change in the final
docs-only diff; it is not independent review or root-cause confirmation.

No source, holdout, commercial reference, playback, device, DAW or audio-policy
change. Safe unrelated authorized maintenance/closeout is not blocked.

## 2026-10-02: independent log review and bounded native recurrence probe

Base: `70466cbe412980fcbea582bd1427d748a4618c2c`. RIOTBOX-1509 returned to
In Progress before branch work. Two read-only agents independently examined
historical native evidence and the reproduction/observation seam; the coordinator
verified the primary logs and final round counts. No production fix is claimed.

The [original failed job](https://github.com/marang/riotbox/actions/runs/36779055802/job/110104227509)
contains a separate earlier delay: at `21:23:49.380Z` toolchain setup reports
`timeout reading rustc version`; at `21:23:57.870Z` it reports rustc 1.98.1.
The [same-head retry](https://github.com/marang/riotbox/actions/runs/36779055802/job/110107867278)
has no corresponding version-query timeout. Failed compilation took 1m57s
versus 28.07s on retry. Both restored the same cache key and 86,772,137-byte
payload; extraction took about 25.18s versus 8.45s. These observations establish
delays outside Sidecar too, not a causal explanation of the handshake failures.

Both historical jobs used Windows image `20260925.250.1`, Python 3.12.10 and
rustc 1.98.1. Workers/regions differed. The test log reports four failures around
`21:26:59.179Z`, then four around `21:27:09.249–.310Z`; those are result timestamps,
not per-test start measurements or proof of CPU count/concurrency. Process spawn,
Python readiness, request-write completion and first-response timing were absent.

A bounded GitHub run-history query after the original run, through
`36869250848`, returned 178 subsequent Rust CI runs, all successful on attempt
one. Individual job logs were not all reviewed. The latest main Windows job
passed 24/24 in 1.96s using rustc 1.99.0. Current Sidecar Rust/Python files are
byte-identical to the original failed head; the only Cargo.lock difference is
an unrelated Audio dependency edge. Core has intervening changes, so this is
not a claim of identical whole-workspace source or binaries.

The new [native Windows probe](https://github.com/marang/riotbox/actions/runs/36976860996/job/110742444564)
ran on diagnostic head `9f62771d60049f240d6cc067348750762f24e1b8`, draft PR
[#1648](https://github.com/marang/riotbox/pull/1648). A temporary branch-limited
PowerShell loop repeated exactly `cargo test --locked -p riotbox-sidecar` up to
30 times, stopping on the first nonzero exit. Default concurrency, output capture,
Python environment, all original assertions and timeout policies were preserved.
The existing 15-minute job limit bounded the experiment. No runtime telemetry,
Python startup hooks or deliberately delayed peers were added.

All 30 rounds exited zero: each executed all 24 library tests and the one
unrelated-CWD integration test. The first library run took 1.81s; subsequent
runs took 1.19–1.20s. Whole-command time was 14,326ms initially, then 1,430–1,461ms;
it includes Cargo/build/other targets and is not handshake latency. The worker
used the same image version and Python 3.12.10, with rustc 1.99.0. This is one
fresh worker and 29 later repetitions on it, not 30 independent cold-host trials.
The Windows tests and strict Clippy step passed.

The original-version retry already passed on 1.98.1, so neither this newer
toolchain nor another green pinned run would establish a compiler effect.
The experiment did not produce a high-rate red loop; causal diagnosis and a
production correction remain unsupported. Do not widen timeouts, serialize the
suite or select another transport from these green observations. The temporary
loop is removed from the final PR; its workflow equals the baseline exactly.
The final deliverable is this evidence update. RIOTBOX-1509 remains open/Todo.

For a fresh native failure or a supplied reproducing environment, the next
bounded observation should record monotonic spawn begin/end plus PID, operation
deadline, request-write/flush completion, first response bytes, frame completion
and failure outcome. Prefer fixed in-memory marks emitted after the exchange;
avoid polling logs and request bodies. Spawn return is not interpreter readiness,
and kernel write completion is not child consumption. These are identified
measurement points, not newly shipped instrumentation or a second transport.

Local unchanged baseline passed 24 library tests and the integration test
(`/tmp/riotbox-1509-v2-baseline.log`); CI contract fixtures and diff checks passed.
The independent Spec and Evidence Auditor found no defect in the temporary
loop's branch scope, original command, failure propagation or run budget. The
final report passed independent Spec and Evidence and Product Pragmatist /
Adversarial Implementation reviews with no findings, followed by coordinator
self-review. Full local `just ci` exited zero
(`/tmp/riotbox-1509-v2-ci.log`). Normal PR CI at the final documentation head
still gates merging the evidence update.
