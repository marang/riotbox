# `RIOTBOX-1566` — Observe source-free Linux callback continuity through isolated bounded host runs

- Ticket: `RIOTBOX-1566`
- Title: `Observe source-free Linux callback continuity through isolated bounded host runs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1566/observe-source-free-linux-callback-continuity-through-isolated-bounded`
- Project: `P017 | Live Performance Readiness`
- Milestone: `P017 bounded operational evidence under RIOTBOX-1041 on the accepted P023 callback path; no issue milestone returned`
- Status: `Done`
- Created: `2026-10-03`
- Started: `2026-10-03`
- Finished: `2026-10-03`
- Branch: `feature/riotbox-1566-silent-host-continuity`
- Linear branch: `feature/riotbox-1566-observe-source-free-linux-callback-continuity-through`
- Assignee: `Markus`
- Labels: `benchmark`, `Spike`, `Audio`
- PR: [#1669](https://github.com/marang/riotbox/pull/1669)
- Merge commit: `37403fdfcb79fc09dc6965e44dd674a9767de750`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: Historical 11 generated Rust driver tests and 58 Python contract tests; two full source-free local just ci passes; one consumed FAILED V1 real-user-session attempt; exact-head native CI 37114303855 passed
- Docs touched: `docs/benchmarks/silent_host_observation_v1.json`, `docs/benchmarks/silent_host_observation_v1.md`, `docs/dev_environment.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1566_silent_host_observation_2026-10-03.md`, `docs/specs/audio_core_spec.md`
- Follow-ups: RIOTBOX-1567 (two P2 V1 tooling defects, separately repaired under V2); RIOTBOX-1041 remains open; no retry authority survives this completed attempt

## Why This Ticket Existed

The accepted live callback path had only a 250 ms startup diagnostic, not repeated longer-lived observation and teardown. This issue prospectively bounded one source-free Linux-user-session experiment: the existing preflight followed by three separate 60-second idle/default runtime processes on an isolated virtual sink, without physical fallback, source hydration, playback or a new product authority.

## What Shipped

- V1/RBX-436 froze the durations, per-second callback progression, startup/supervision/teardown budgets, stop conditions, exact ownership and source exclusions before execution. Protocol SHA256 ba26bc50db85a6b745738917100689a7ca5ac3c2aa95cc7c699db2ef404358d3 remained unchanged after the result.

- The dev-only driver reused AudioRuntimeShell start/health/stop and pure probe projection. The operator bound exact PID -> PipeWire Client -> Node -> stereo-sink serial routing, no fallback/reconnect/move, no outgoing sink links and a private PipeWire-only ALSA child configuration selected before CPAL open.

- Independent process watchdogs, bounded metadata, common absolute startup/terminal/cleanup budgets, signal handling, uncertain-cleanup propagation and failed publication handling were added and tested with generated processes/metadata. Uncertain process or stream cleanup retains containment rather than silently unloading it.

- One actual attempt completed as FAILED, with zero accepted runs. The preflight and first 60-second run started; runs 2/3 and retries did not. The report preserves the original implementation, failed result/protocol and separately attributable cleanup instead of silently changing acceptance.

## Historical Verification And Outcomes

- Before launch, independent reviews exposed late startup-snapshot admission, missing terminal budget enforcement, routing isolation, orphan-node detection, signal/log-close handling and unsafe-cleanup masking issues. They were reproduced/fixed/rechecked before execution. A user-requested pause preserved uncommitted work without a CPAL process, sink or attempt owner; work resumed only on explicit instruction.

- Historical source-free verification passed 11 Rust driver tests and 58 Python tests, including generated-child supervision/signals, routing adversaries, deadlines, publication and uncertain cleanup. Full local just ci passed twice; the unchanged 58-test recipe was repeated after report changes.

- The actual attempt was bound to clean revision cf4f76664a50ba8f054e5e2a9fc19016092c89dd and exact rebuilt binaries. The 250 ms preflight passed; run 1 yielded 60 advancing intervals, 5,169 callbacks, zero reported faults and final Stopped.

- Run-1 teardown falsely rejected recyclable integer ID 82 after the original stream Node/serial 6297 disappeared and pw-dump Clients reused the ID. All 35 teardown snapshots lacked the original stream; 34 included a recycled Client. Independent metadata review confirmed this was not backend stream-leak evidence. Nevertheless V1 remains failed, consumed and unapproved for reuse.

- A separate actual schema inspection found pactl module JSON lacked the assumed index/id field. That cleanup path had not been reached in the failed attempt; it was the second P2 tool incompatibility, not an additional observed runtime failure. Both P2 findings were explicitly deferred to RIOTBOX-1567 under a versioned successor.

- Containment was initially retained. Separate read-only checks established the original processes, stream and links were gone. The original default output no longer matched, for an unknown reason; no restoration or causal claim was made. A fresh cleanup verified the exact idle owned module/sink, removed only it, and confirmed current defaults/mute/volume were unchanged.

- Original failed result SHA256 bb020e9a008b8e5073e8b84ae77ef2dbbdd4ced128868131587a09e8e8fbac35 and both cleanup records remain historical under ignored artifacts/development/riotbox-1566/attempt-01. These paths/hashes were preserved from ticket/PR evidence, not reopened during archival.

- Post-result independent evidence review corrected one P3 attribution wording issue. Exact-head native CI 37114303855 passed at 43695c5a59d2aa057bef7554024d8bac26e4d21b. The two P2 tool defects stayed explicit; CI did not turn the host attempt into a pass.

## Notes

- Done means the single bounded experiment and its honest evidence/cleanup workflow completed, not host qualification. Zero accepted runs, no three-run pass, no retry and no original-baseline-preservation claim are the authoritative outcome.

- This can supply only sampled silent virtual-endpoint lifecycle/callback evidence. It does not establish physical-device, loaded musical runtime, latency/xrun, acoustic-silence, broad endurance, DAW import, musical, source-general or release readiness; RIOTBOX-1041 was not completed.

- No real source/Development/Holdout/commercial/capture reads, source-directory discovery, human listening, physical fallback, DAW/TUI or Windows expansion occurred. Ephemeral observation state is not Core/Session/replay state.

- Historical closeout synchronized clean main/origin/main and removed exact implementation branches after native CI. The recent Done issue and original failed evidence were retained; separate RIOTBOX-1567 did not authorize another host attempt.

- Detailed historical evidence: [implementation report](../../reviews/riotbox_1566_silent_host_observation_2026-10-03.md).

- Archive verification on 2026-10-16 read the full current Linear issue, all returned comments (no further page), its linked implementation PR body/files and merge/check metadata. Linear was Done and the exact PR was MERGED with both existing native checks successful. The tests, source/audio exclusions and host observations above are historical evidence, not new executions; archival work did not run product/audio tests, query host audio or open source/ignored evidence artifacts.
