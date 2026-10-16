# `RIOTBOX-1582` — Preserve restored transport until valid coherent callback timing is observed

- Ticket: `RIOTBOX-1582`
- Title: `Preserve restored transport until valid coherent callback timing is observed`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1582/preserve-restored-transport-until-valid-coherent-callback-timing-is`
- Project: `P017 | Live Performance Readiness`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-16`
- Started: `2026-10-16`
- Finished: `2026-10-16`
- Branch: `feature/riotbox-1582-preserve-restored-transport-until-valid-coherent-callback`
- Linear branch: `feature/riotbox-1582-preserve-restored-transport-until-valid-coherent-callback`
- Assignee: `Markus`
- Labels: `Audio`, `Bug`
- PR: [#1687](https://github.com/marang/riotbox/pull/1687)
- Merge commit: `418a6a1dc342203e9844434b39f59b347ae255bc`
- Deleted from Linear: `2026-10-16`
- Verification: Actual App RED→GREEN; focused App/audio/recorder/queue/callback controls and actual-source Loom models PASS (revision-removal mutation fails); full local source-free `just ci` PASS; exact final-head native 38872241510 and post-merge main 38872949552 PASS; independent initial/delta Rust reviews and self-review: zero findings.
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1582_observed_callback_timing_2026-10-16.md`, `docs/specs/audio_core_spec.md`
- Follow-ups: RIOTBOX-1583 integrated current-state checkpoint; RIOTBOX-1041/1036 remain open; no new host/endurance/latency/musical/release/original ALSA-cause claim.

## Why This Ticket Existed

Before any output callback, fresh telemetry exposed false/zero timing as observed progress. The real paused-restore consumer could overwrite a saved nonzero Session cursor and echo zero into audio control, risking the accepted P023 restore/play/recording path.

## What Shipped

- Reproduced deterministic device-free paused cursor loss from 12.5 to zero through the actual App timing consumer, then exposed optional observed timing across existing runtime/App/recorder seams.

- Unavailable timing now preserves Session/runtime cursor, pending Play/Pause acknowledgment, queue and commit state; genuine observed beat zero remains valid, with no requested-as-observed substitution.

- Added one private bounded single-writer coherent callback publication owner: all-sequentially-consistent odd/even revision and payload operations, at most three reader attempts, no unchecked fallback, locks, heap work, I/O or unbounded loops.

- Preserved recorder health checks, two-second confirmation deadline, ten-millisecond waits and all recording/receipt/replay/DSP contracts; updated Audio Core and RBX-448 with semantic tests and a dated report.

- Proved generated actual callback handoff, active PCM on Play and post-Pause ramp silence, including seven heap-free callbacks, without opening a device.

## Notes

- Exact reviewed head: `720ed9fae9bf68bd951e398362336a75492f0ecf`; merge `418a6a1dc342203e9844434b39f59b347ae255bc` had identical reviewed tree `8692f39bf97d2eecd1c70d31e8db11c2a675fe88`. Post-merge run 38872949552 was independently checked successful at this exact merge during archival metadata verification.

- Focused historical selections: App admission 3, Audio lifecycle 9, diagnostic-lock 4, callback heap 5, exact callback timing/output 1, recorder/controller 20, capture/audition 17, MC-202 timing 13, recording App 21. Selections overlap and are not summed as unique coverage.

- Two Loom models compiled the actual production publication source with atomic primitives substituted. Removing final revision validation produced a real mixed false/128/64.25 generation; restored validation passed. No mutation remained.

- Actual generated callback proof covered unavailable timing before invocation, paused restored 12.5, 64-frame/48-kHz/120-BPM progress, active generated PCM and Pause silence, with no device. This proves an exact callback/output seam, not actual CPAL occurrence or host qualification.

- The local Audio unit pass preceded a final test-only callback addition; that exact new test was rerun by root and both independent reviewers, and native full CI covered exact final head. Full local CI included 165 silent-host/140 loaded-recorder controls, generated audio/contracts/JSON and strict Clippy with `/tmp` JUST_TEMPDIR only.

- Historical closeout recorded clean main equal to origin/main; exact feature-branch cleanup followed. No new action, persistent App state, Session schema, observer version, sound policy or second transport.

- No host/device/service metadata or runtime launch, real source/capture/Holdout/commercial access, ignored attempt, DAW, playback or Windows implementation. RIOTBOX-1581 stays consumed/unchanged and cannot qualify this new repair.

- This archive preserves historical scope, verification and outcomes only. It grants no new source, host, device, consumed-attempt, playback or trial access.
