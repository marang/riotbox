# `RIOTBOX-1572` — Execute one prospectively bound silent V3 host observation after metadata repair

- Ticket: `RIOTBOX-1572`
- Title: `Execute one prospectively bound silent V3 host observation after metadata repair`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1572/execute-one-prospectively-bound-silent-v3-host-observation-after`
- Project: `P017 | Live Performance Readiness`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-09`
- Started: `2026-10-09`
- Finished: `2026-10-09`
- Branch: `feature/riotbox-1572-silent-host-v3-once`
- Linear branch: `feature/riotbox-1572-execute-one-prospectively-bound-silent-v3-host-observation`
- Assignee: `Markus`
- Labels: `benchmark`
- PR: [#1675](https://github.com/marang/riotbox/pull/1675)
- Merge commit: `18282cf72b3783f4ae445936c2d484467610b4c5`
- Deleted from Linear: `2026-10-16`
- Verification: Full local source-free `just ci` PASS before execution; 140 generated fixtures PASS before and after; exact pre-execution native CI 37980167201 PASS; final native CI 37981989835 PASS; independent evidence reviews and self-review: zero retained findings.
- Docs touched: `docs/benchmarks/silent_host_riotbox_1572_once_2026-10-09.md`, `docs/dev_environment.md`, `docs/reviews/riotbox_1572_silent_host_v3_observation_2026-10-09.md`, `docs/specs/audio_core_spec.md`
- Follow-ups: RIOTBOX-1573 (attribution, subsequently completed); RIOTBOX-1574 (separate V4 diagnostic observation, subsequently completed); RIOTBOX-1041 remains open; original ALSA trigger unresolved.

## Why This Ticket Existed

A fresh, explicitly authorized silent V3 observation was needed after RIOTBOX-1571 repaired multiline host-metadata admission. It tested sampled real CPAL/PipeWire callback lifetime behind the accepted P023 path, not a musical feature or endurance qualification.

## What Shipped

- Prospectively froze one V3 phase, exact owner and executable fence, binding reviewed clean revision, native CI, protocol and two default-feature binary digests before the sole host invocation.

- Retained one consumed FAILED observation: 250-ms preflight passed with 23 callbacks; run 1 completed 60 samples / 60,010 ms / 5,170 callbacks; run 2 reported one buffer underrun/overrun at sample 36 / 36,004 ms and stopped at 36,005 ms. Run 3 never started.

- Verified owned process/module/stream cleanup and unchanged original default, mute and volume; retained a bounded dated report and generated once-only/signal controls. No operator, runtime, threshold or retry-policy change.

## Notes

- Actual invocation was bound to reviewed `a3069c7051d616de4335bbbec3bfb6f85068ee97`; final report head was `4cd88e394bed605ea395011fe107f46ab22a39da`. Pre-execution and final-report CI are distinct gates.

- Existing independent validators passed preflight/run 1 and rejected run 2 for the retained error; 705 historical route snapshots independently satisfied containment/lifetime checks. This is historical review evidence, not a new validation run.

- Historical result SHA256: `9f130973b65ef5e299687bba3bccd03149879ecd9dd61b45684be1f419631b41`; prospective binding: `4587402ebbc94ffdb94fb7a936af1e12a4715a91e55379ea392e614847f0f5b8`; V3 protocol: `a0934097be7b2f160ff698b3d69c1dd7180e86defcdc0974c5e23606297ef85b`.

- Done means the once-only failed observation and evidence closeout were completed, not that three runs, host stability, physical-device use, endurance or music passed. No retry was taken; RIOTBOX-1573 owned source-free attribution.

- Historical closeout recorded synchronized clean main and exact local/remote branch removal. Earlier V1/V2 and this V3 authorization remain consumed.

- This archive preserves historical scope, verification and outcomes only. It grants no new source, host, device, consumed-attempt, playback or trial access.
