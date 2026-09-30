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
