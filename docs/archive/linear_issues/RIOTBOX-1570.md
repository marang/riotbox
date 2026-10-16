# `RIOTBOX-1570` — Execute one prospectively bound silent virtual-host observation with V2 checks

- Ticket: `RIOTBOX-1570`
- Title: `Execute one prospectively bound silent virtual-host observation with V2 checks`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1570/execute-one-prospectively-bound-silent-virtual-host-observation-with`
- Project: `P017 | Live Performance Readiness`
- Milestone: `P017 one-shot prospective V2 operational observation for RIOTBOX-1041; no issue milestone returned`
- Status: `Done`
- Created: `2026-10-09`
- Started: `2026-10-09`
- Finished: `2026-10-09`
- Branch: `feature/riotbox-1570-silent-host-v2-once`
- Linear branch: `feature/riotbox-1570-execute-one-prospectively-bound-silent-virtual-host`
- Assignee: `Markus`
- Labels: `benchmark`
- PR: [#1673](https://github.com/marang/riotbox/pull/1673)
- Merge commit: `1a7b9a4ff004ea6fa2f25f3c8657b480817e59fd`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: Historical six exact-command generated tests and 95 host fixtures; full source-free local just ci; pre-execution native CI 37972797712 passed; one consumed FAILED real-user-session admission; final exact-head native CI 37974619234 passed
- Docs touched: `docs/benchmarks/silent_host_riotbox_1570_once_2026-10-09.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1570_silent_host_v2_observation_2026-10-09.md`
- Follow-ups: RIOTBOX-1571 (source-free multiline module metadata repair), separately completed; RIOTBOX-1572 later owns a distinct prospectively authorized V3 observation; RIOTBOX-1041 remains open

## Why This Ticket Existed

After the export repair, the user explicitly authorized preparation and exactly one silent V2 host test: isolated virtual output, unchanged 250 ms preflight plus 3 x 60-second budgets, no source/DAW/physical output and no rewriting failed V1. Its purpose was actual CPAL/PipeWire callback progress and clean process/stream teardown only, not musical, soundcard, loaded-runtime, endurance or release qualification. V2's source-free repair had not itself authorized a host call.

## What Shipped

- RBX-439 and an exact one-shot Markdown invocation bound a fresh ignored phase/attempt owner before any real host metadata. The phase reused execute_attempt with OperatorInterrupts rather than introducing a new launcher or enabling the legacy CLI.

- The clean reviewed Git revision, rebuilt default-feature cpal_spike/silent_host_probe hashes and frozen V2 protocol were compared and prospectively recorded. Authorization was consumed before the sole invocation, including admission failures before attempt-owner creation; no retry or alternative owner was permitted.

- The sole actual invocation failed during metadata admission before sink load, CPAL preflight or runtime launch. Honest negative evidence, unchanged host settings/owned-sink absence and exact ignored result identity were documented; the parser repair was separated into RIOTBOX-1571.

- V1 failure, frozen V2 JSON/source_free_validation_only, unchanged budgets, disabled legacy launch path, exact routing/no-fallback containment and independent watchdog/cleanup rules were preserved.

## Historical Verification And Outcomes

- Six generated tests exercised the exact documented invocation with the operator mocked, including identity rejection and consumption on early admission failure. They made no real host-adapter calls. All 95 generated silent-host fixtures and full source-free local just ci passed.

- Two independent command/contract reviews, coordinator self-review and pre-execution native CI 37972797712 passed before the one call on clean reviewed revision 7201fe63001344dc1003df4d46f7c0d21c529991.

- Historical prospective binary SHA256 values were cpal_spike 58f6103c3f34ae3837a0b059c209052c36fa5398ded027a4b959dcba67993fc4 and silent_host_probe 59de339f9488a93958f240c26fea6456bce9e9f1ec872696d75efcd7a7fe778f. V2 protocol SHA256 was 9b8517f13285e36a6082fb57131602949c89758199d57ccef42a26f4a542bcea. Archival work preserved these recorded identities without rebuilding or reopening ignored artifacts.

- The invocation was consumed at 2026-10-09 18:30:48.886 UTC and failed admission at 18:30:49.113 UTC. Active local non-root UID 1000 Linux/PipeWire 1.6.9 context was established, then _read_modules raised EvidenceError: malformed short module columns because ordinary native libpipewire-module-rt arguments contained embedded newlines.

- No null sink, CPAL preflight, 60-second runtime or driver transcript was created. Thus no callback observation, validation or successful runtime result exists for this phase. Prospective/admission/result Git/binary/protocol identities matched.

- The operator recorded cleanup_verified=true; separate read-only checks confirmed unchanged default/mute/volume and exact owned-sink absence. Captured post-execution metadata reproduced the parser error through a mocked adapter without a second host call.

- Result SHA256 cc0a5e5f5eb93f83913193f4618d7efc8892becc89555a475ab24afb40aef80b identifies the retained negative result. Final report/evidence review and a repeated 95-fixture run passed; native CI 37974619234 passed on evidence-update head 2f21731b337f752d0fffcc17e2ac5644c07f9b04 before merge.

## Notes

- Done means the single expressly authorized attempt, negative result and verified unchanged host state were fully recorded. It is not a callback/device/endurance/release pass. Authorization is consumed and cannot be revived by changing metadata code, phase owner or result labels.

- No source/Holdout/commercial/capture audio, source-directory discovery, listening, physical/default output, DAW/TUI or Windows expansion occurred. Sampled virtual evidence would not qualify physical hardware, loaded runtime, latency/xruns, musical hardness or broad endurance even if successful.

- Historical closeout synchronized clean main/origin/main, removed exact feature branches and retained the issue in Done. RIOTBOX-1571 was source-free repair only; any next actual attempt required new explicit authorization and a separate prospective binding.

- Detailed historical evidence: [implementation report](../../reviews/riotbox_1570_silent_host_v2_observation_2026-10-09.md).

- Archive verification on 2026-10-16 read the full current Linear issue, all returned comments (no further page), its linked implementation PR body/files and merge/check metadata. Linear was Done and the exact PR was MERGED with both existing native checks successful. The tests, source/audio exclusions and host observations above are historical evidence, not new executions; archival work did not run product/audio tests, query host audio or open source/ignored evidence artifacts.
