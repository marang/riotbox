# `RIOTBOX-1567` — Version silent-host lifecycle checks for PipeWire object reuse and Pulse module identity

- Ticket: `RIOTBOX-1567`
- Title: `Version silent-host lifecycle checks for PipeWire object reuse and Pulse module identity`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1567/version-silent-host-lifecycle-checks-for-pipewire-object-reuse-and`
- Project: `P017 | Live Performance Readiness`
- Milestone: `P017 source-free tooling maintenance for RIOTBOX-1041 on the accepted P023 callback path; no issue milestone returned`
- Status: `Done`
- Created: `2026-10-03`
- Started: `2026-10-03`
- Finished: `2026-10-03`
- Branch: `feature/riotbox-1567-silent-host-lifetime-v2`
- Linear branch: `feature/riotbox-1567-version-silent-host-lifecycle-checks-for-pipewire-object`
- Assignee: `Markus`
- Labels: `review-followup`, `benchmark`, `Bug`
- PR: [#1670](https://github.com/marang/riotbox/pull/1670)
- Merge commit: `ab6af50201ffbbd96d3b8e03f143984393c16a7f`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: Historical generated RED/GREEN; 89 silent-host contract fixtures with ResourceWarnings fatal; full source-free local just ci; exact-head native CI 37133323538 passed; no host invocation
- Docs touched: `docs/benchmarks/silent_host_observation_v2.json`, `docs/benchmarks/silent_host_observation_v2.md`, `docs/dev_environment.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1567_silent_host_lifecycle_v2_2026-10-03.md`, `docs/specs/audio_core_spec.md`
- Follow-ups: RIOTBOX-1570 later owned a separately authorized prospective V2 attempt; RIOTBOX-1041 remains open

## Why This Ticket Existed

The consumed RIOTBOX-1566 V1 attempt exposed two P2 tool defects: comparing a recyclable PipeWire ID without object type/serial falsely failed teardown after the original stream disappeared; and cleanup assumed module JSON contained an index field absent in the actual supported schema. The repair had to version the contract without rewriting or retrying failed V1 evidence.

## What Shipped

- V2/RBX-437 was frozen before code changes with SHA256 9b8517f13285e36a6082fb57131602949c89758199d57ccef42a26f4a542bcea. Every existing duration, source exclusion, failed V1 artifact and claim limit remained unchanged.

- Typed admitted Node identity includes object.serial as well as integer ID. Recycled Client or unrelated replacement-Node IDs no longer impersonate the original stream, while true orphan Nodes, foreign/targeting streams, malformed identity, route replacement and deadline failures still reject.

- One cohesive Pulse module adapter replaces the nonexistent JSON-index assumption with exact short-text module ID, type/name and stereo argument checks. Ambiguous/replaced identity rejects; unload occurs once and requires valid module/Node absence inside the shared deadline.

- The old CLI opt-in is disabled before any effects, even for its former explicit flag. Internal generated attempts require explicit temporary owners; no new real attempt owner or host execution authority was created. Watchdog, signal, uncertain-cleanup and publication protections remain intact.

## Historical Verification And Outcomes

- Generated RED/GREEN reproduced the original ID-reuse and module-schema defects plus Client/new-Node reuse, genuine orphan, targeting/foreign stream, corrupt identity, replacement, ambiguity and delayed cleanup controls.

- Review found an inherited P2 that treated malformed snapshot metadata as sink absence; it was reproduced and fixed with positive/negative controls. A stale generated signal-test entry after CLI disablement was also repaired. No finding was deferred to clear this PR.

- All 89 source-free silent-host fixtures passed with ResourceWarnings fatal. Full local just ci passed and the complete 89-test recipe was rerun after the final review correction.

- Independent spec/evidence review reran 75 focused generated tests; adversarial/operations review ran 28 identity and 31 integration tests. Final independent reviews and coordinator self-review retained zero P0-P3 findings.

- Historical coordinator checks confirmed V1 tracked protocol/report and Rust crates had no diff and original result/both cleanup hashes remained unchanged. Archival work did not reopen those ignored artifacts.

- Native CI 37133323538 passed both existing jobs at exact reviewed head 3dde31acd36639809a625d08703a48617271f097 before merge.

## Notes

- This source-free repair did not invoke actual host/audio services, read source audio or perform listening. It changes no Rust driver, DSP, Session/replay or musician-facing sound, and does not qualify host/endurance/physical/musical/release behavior.

- Module query/unload remains sampled and non-atomic. Observed replacement rejects; the contract does not claim protection against malicious inter-command replacement.

- V1 remains FAILED and consumed. Any later actual observation required fresh explicit authorization and its own prospective phase/owner; V2's generated evidence and disabled launch CLI did not grant that authority.

- Historical closeout synchronized clean main/origin/main and removed the exact local/remote branch. The issue was retained in Done, with RIOTBOX-1041 still open.

- Detailed historical evidence: [implementation report](../../reviews/riotbox_1567_silent_host_lifecycle_v2_2026-10-03.md).

- Archive verification on 2026-10-16 read the full current Linear issue, all returned comments (no further page), its linked implementation PR body/files and merge/check metadata. Linear was Done and the exact PR was MERGED with both existing native checks successful. The tests, source/audio exclusions and host observations above are historical evidence, not new executions; archival work did not run product/audio tests, query host audio or open source/ignored evidence artifacts.
