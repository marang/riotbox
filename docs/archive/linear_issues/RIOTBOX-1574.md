# `RIOTBOX-1574` — Add bounded ALSA xrun measurements and execute one isolated silent diagnostic observation

- Ticket: `RIOTBOX-1574`
- Title: `Add bounded ALSA xrun measurements and execute one isolated silent diagnostic observation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1574/add-bounded-alsa-xrun-measurements-and-execute-one-isolated-silent`
- Project: `P017 | Live Performance Readiness`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-09`
- Started: `2026-10-09`
- Finished: `2026-10-09`
- Branch: `feature/riotbox-1574-instrumented-silent-host`
- Linear branch: `feature/riotbox-1574-add-bounded-alsa-xrun-measurements-and-execute-one-isolated`
- Assignee: `Markus`
- Labels: `Bug`
- PR: [#1677](https://github.com/marang/riotbox/pull/1677)
- Merge commit: `d3e5a28e76f3b8615a841014143cd1ca0344c226`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: 165 generated source-free host controls PASS; full corrected and final local `just ci` PASS; executed-revision native CI 37990118126 PASS; final-head native CI 37992074080 PASS; both final evidence reviews/self-review: zero retained findings.
- Docs touched: `docs/benchmarks/silent_host_observation_v4.json`, `docs/benchmarks/silent_host_observation_v4.md`, `docs/dev_environment.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1574_alsa_diagnostics_2026-10-09.md`, `docs/specs/audio_core_spec.md`
- Follow-ups: RIOTBOX-1041 remains open; original RIOTBOX-1572 ALSA cause unresolved. Later RIOTBOX-1575..1581 provide separate controller/load/observer evidence, not retroactive requalification.

## Why This Ticket Existed

RIOTBOX-1573 established normal reporting and stop semantics but could not identify RIOTBOX-1572's detecting ALSA operation or negotiated geometry. A minimal bounded diagnostic seam and separately approved observation addressed that evidence gap without guessing a buffer fix.

## What Shipped

- Froze diagnostic V4 / RBX-441 while preserving V3 safety gates and budgets; added a QA-only ALSA ABI interposer loaded solely into the supervised child, with prefaulted atomic counters and unchanged argument/return/errno forwarding.

- Read diagnostic evidence only after verified process-group cleanup; reject missing coverage, unsupported identity/ABI and every measured negative return rather than masking the runtime error.

- Completed exactly one fresh user-approved isolated silent invocation: 250-ms preflight plus three 60-second runs all PASS with zero reported stream errors/overflows and zero measured ALSA negative/EPIPE returns across PCM lifetimes.

- Independently checked historical route/PID/measurement identities, original default/mute/volume and owned cleanup; committed the dated bounded outcome. Authorization was consumed, with no retry.

## Notes

- Execution used clean `dc6bf5d3a02e5d628762b513de9410e542d53199` after exact CI and three-binary binding, with no intervening build/tracked edit. Final report head: `22604f5107a7e68f6b7c57bdc524f3c2ca14ea83`.

- Measured applied plugin geometry was stereo 44,100 Hz, buffer 1,024 frames / period 512 frames. This is plugin geometry, not a latency, physical-device or performance qualification.

- Independent findings were reproduced RED and fixed: non-writing PCM errors could evade rejection, and a fork child could register a PCM in its parent's mapping. Generated ABI controls used fake providers and opened no device.

- The original RIOTBOX-1572 failure remains unexplained and unchanged; a successful short instrumented observation does not erase it or establish root-cause repair, endurance, musical readiness or release readiness.

- Result-bound native metadata was retained; historical `/proc` absence is explicitly coordinator observation, not reconstructed raw evidence. Exact local/remote branch cleanup and synchronized clean main were recorded.

- The user requested a pause after this ticket; no next issue started at that historical closeout.

- This archive preserves historical scope, verification and outcomes only. It grants no new source, host, device, consumed-attempt, playback or trial access.
