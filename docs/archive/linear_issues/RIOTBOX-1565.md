# `RIOTBOX-1565` — Fail closed on CPAL probe startup and callback-observation failures

- Ticket: `RIOTBOX-1565`
- Title: `Fail closed on CPAL probe startup and callback-observation failures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1565/fail-closed-on-cpal-probe-startup-and-callback-observation-failures`
- Project: `P017 | Live Performance Readiness`
- Milestone: `P017 maintenance of the accepted P023 callback/host-preflight path; no issue milestone returned`
- Status: `Done`
- Created: `2026-10-03`
- Started: `2026-10-03`
- Finished: `2026-10-03`
- Branch: `feature/riotbox-1565-cpal-probe-fail-closed`
- Linear branch: `feature/riotbox-1565-fail-closed-on-cpal-probe-startup-and-callback-observation`
- Assignee: `Markus`
- Labels: `Bug`
- PR: [#1668](https://github.com/marang/riotbox/pull/1668)
- Merge commit: `07419e47182b5717ff86d41210432f24433ecbe3`
- Deleted from Linear: `2026-10-16`
- Verification: Historical six pure-health tests and one isolated CLI reporting/exit test; complete riotbox-audio tests (333 library tests, one existing ignored manual benchmark); full source-free local just ci; exact-head native CI 37109534578 passed
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1565_cpal_probe_admission_2026-10-03.md`, `docs/specs/audio_core_spec.md`, `docs/spikes/cpal_audio_latency_spike.md`
- Follow-ups: RIOTBOX-1566 separately owned one isolated silent-host observation; RIOTBOX-1041 remains the wider host-soak/endurance anchor

## Why This Ticket Existed

On main 2fe22469 the CPAL diagnostic printed handled setup errors but returned (), yielding exit 0. Its probe projection could label Running/Stopped as Ok without observing callbacks or checking scratch overflow. This was verified control-flow evidence, not an actual failed device or zero-callback observation. Reliable machine-consumable outcomes were a bounded prerequisite for future RIOTBOX-1041 real-session observation.

## What Shipped

- RBX-435 and Audio Core define bounded probe Ok as Running/Stopped with observed callbacks and without runtime stream error, retained error detail or scratch-overflow evidence. Existing RuntimeTelemetry and faulted health remain the single producer.

- The CLI preserves useful diagnostics and returns success only for typed Ok; Failed and NotRun return nonzero. Error precedence and required counters are explicit.

- The existing fixed 250 ms silent source-free probe duration, callback/DSP implementation and snapshot-before-stop lifetime remain unchanged. No flags, host runner, timing thresholds, bypass or general soak framework were introduced.

## Historical Verification And Outcomes

- Executed baseline RED/GREEN demonstrated the generated zero-callback health regression. Six pure-health cases covered lifecycle/callback/error combinations and all five setup errors; one reporting-unit test covered all three typed outcomes and exit statuses.

- Complete riotbox-audio testing passed 333 library tests plus CLI/binary/integration suites, with one pre-existing ignored manual benchmark. No CPAL probe main or audio device was started by these tests.

- Independent adversarial Rust and spec/evidence/product reviewers each ran the six pure-health tests and reporting test, retained no P0-P3 findings and distinguished their focused execution from coordinator full CI.

- Full source-free local just ci, formatting and diff checks passed. Native CI 37109534578 succeeded on exact reviewed head 728790ccaf2b0f31ad473ddd1bde238dd5874fe8 before merge; PR reviews and final self-review had no remaining findings.

## Notes

- Zero callbacks means unproven/failed observation, not evidence the backend is broken. Observed callbacks do not establish continuous health, physical audibility, latency, xrun absence, endurance or a human verdict. Earlier informational spike observations were not upgraded.

- No actual probe/device/runtime/DAW invocation, real source/Holdout/commercial access, source discovery, playback, TUI or Windows expansion occurred. Core/Session/action state and sound were unchanged.

- Historical closeout synchronized main/origin/main while preserving the next issue's worktree; RIOTBOX-1566 separately owned its bounded observation. This issue was retained in Done and RIOTBOX-1041 remained open.

- Detailed historical evidence: [implementation report](../../reviews/riotbox_1565_cpal_probe_admission_2026-10-03.md).

- Archive verification on 2026-10-16 read the full current Linear issue, all returned comments (no further page), its linked implementation PR body/files and merge/check metadata. Linear was Done and the exact PR was MERGED with both existing native checks successful. The tests, source/audio exclusions and host observations above are historical evidence, not new executions; archival work did not run product/audio tests, query host audio or open source/ignored evidence artifacts.
