# `RIOTBOX-1580` — Correct terminal non-Node ID recycling in loaded recorder observation

- Ticket: `RIOTBOX-1580`
- Title: `Correct terminal non-Node ID recycling in loaded recorder observation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1580/correct-terminal-non-node-id-recycling-in-loaded-recorder-observation`
- Project: `P017 | Live Performance Readiness`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-16`
- Started: `2026-10-16`
- Finished: `2026-10-16`
- Branch: `feature/riotbox-1580-terminal-client-reuse`
- Linear branch: `feature/riotbox-1580-correct-terminal-non-node-id-recycling-in-loaded-recorder`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: [#1684](https://github.com/marang/riotbox/pull/1684)
- Merge commit: `3ab397cfb278c047315bba3b3346da61e209f0cc`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: 34 process/Client controls and 43 loader/workload/historical-fence controls PASS; each independent reviewer ran 77 focused executions; full local `just ci` PASS (165 silent/116 loaded controls); exact native PR CI 38858691544 PASS. Post-merge main run 38859355406 had Linux rust-ci PASS but Windows handshake failures; retained separately, not relabeled green.
- Docs touched: `docs/benchmarks/loaded_recorder_observation_v2.json`, `docs/benchmarks/loaded_recorder_observation_v2.md`, `docs/dev_environment.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1580_loaded_client_reuse_v2_2026-10-16.md`
- Follow-ups: RIOTBOX-1581 separately approved prospective V2 observation subsequently completed with bounded technical PASS; post-merge Windows CI gap retained, root cause unproven; RIOTBOX-1041/1036 remain open; original ALSA cause unresolved.

## Why This Ticket Existed

The consumed RIOTBOX-1579 V1 observation exposed a concrete QA false positive: the caller treated terminal global-ID reuse by a non-audio Client as replacement of the admitted audio Node. A narrow versioned correction was needed without silently changing V1 or guessing an xrun fix.

## What Shipped

- Reproduced the exact failure deterministically through the real generated `run_recording()` supervision sequence, then introduced closed LoadedObservationVersion and pinned V2 / RBX-446.

- Only V2 treats a structurally valid Client with a distinct positive uint64 serial at the former Node ID as Node absence. Changed/replaced Nodes, other/unknown/malformed interfaces, invalid serials, unsafe routes and premature/ambiguous absence continue to reject.

- Preserved V1 default constructor/loader/hash, frozen fence/contracts and original V1 rejection; retained one existing operator/workload/supervisor and all completion, route, diagnostic, deadline, cleanup and final proof/Core gates.

- Added generated RED→GREEN, V1-negative, compatibility and edge controls. No Rust/product/recording/DSP/source policy, buffer/rate/priority or shared lifetime/cleanup change.

## Notes

- Exact reviewed head: `e1f7030a9c9c384bffb5bc09394d0ea038657af5`; merge complete tree was identical. Independent Maintainer/Adversarial and Spec/Evidence review, documentation refresh and self-review retained zero findings.

- The RED regression and explicit V1-negative control preserve the distinction between a correction to future V2 observation and the failed/consumed V1 outcome; no old attempt was reopened and no actual host invocation or retry was authorized.

- Post-merge `sidecar-transport-windows` failed three unchanged real-Python Sidecar tests at the existing 10-second Control handshake: stdio ping/graph build, generated-input file analysis and unsupported-source rejection. Twenty-nine other tests passed. Exact PR Linux+Windows CI had passed on the identical tree, and the next preparation Windows job also passed.

- The differing Windows CI outcomes do not establish an observer regression or prove a scheduler/import cause. Windows support remained intentionally deferred; no timeout increase, rerun/greenwashing or Windows implementation was performed.

- Only sandbox JUST_TEMPDIR moved to a fresh writable `/tmp` owner for local CI. Historical main synchronized and merged branches cleaned while preserving RIOTBOX-1581 work.

- Generated fixture success is not actual recording/host, human, musical, endurance or original ALSA-cause proof. Future host observation still needed its own phase, exact bindings and fresh explicit approval.

- This archive preserves historical scope, verification and outcomes only. It grants no new source, host, device, consumed-attempt, playback or trial access.
