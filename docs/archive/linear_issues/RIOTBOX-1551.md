# `RIOTBOX-1551` — Add bounded 8/16-bar live master recording windows with versioned duration identity

- Ticket: `RIOTBOX-1551`
- Title: `Add bounded 8/16-bar live master recording windows with versioned duration identity`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1551/add-bounded-816-bar-live-master-recording-windows-with-versioned`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-02`
- Started: `2026-10-02`
- Finished: `2026-10-02`
- Branch: `feature/riotbox-1551-bounded-recording-windows`
- Linear branch: `feature/riotbox-1551-add-bounded-816-bar-live-master-recording-windows-with`
- Assignee: `Markus`
- Labels: `None`
- PR: [#1654](https://github.com/marang/riotbox/pull/1654)
- Merge commit: `654791993a744c5c2b6e1948bdb71e0566ad48fb`
- Deleted from Linear: `2026-10-16`
- Verification: Historical: final source-free `just ci`; Core recording 25/replay 121, CLI 144, Audio capture 18, App window 7; native CI 37038877195 passed. Current archival check: Linear Done and merged PR metadata; no test/audio rerun.
- Docs touched: `README.md`, `docs/execution_roadmap.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1551_bounded_recording_windows_2026-10-02.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_core_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: RIOTBOX-1552 owns legacy two-bar producer/readiness reconciliation; RIOTBOX-1553 owns fresh eight-bar host evidence; RIOTBOX-1555 later owns V3 DAW handoff. RIOTBOX-1036/P016 was broader open scope at implementation closeout; its current status is not reassessed here.

## Why This Ticket Existed

The existing P016 post-limiter operator recorder under RIOTBOX-1036 was limited to two bars. This contract/engineering extension lets an already stable Scene produce an explicitly selected eight- or sixteen-bar take without changing DSP, musical policy or the non-interactive command's inability to accept live performer gestures.

## What Shipped

- Typed duration identity reaches the existing ExportLiveRecording Action, Core/Session, restore/replay, receipt/proof, CLI, observer and read-only readiness. Explicit 8/16 uses separate V3; at this merge default/explicit two bars retained historical V2, and V1/V2 serialized meanings were not broadened.
- Unknown, inconsistent or oversized windows reject before capture/publication. The existing production callback, limiter, next-confirmed-bar start, deterministic frame geometry, callback health/discontinuity rejection, 64 MiB capture allocation ceiling, no-clobber and Session-save rollback remain.
- RBX-422 freezes V3 canonical runtime-f32 geometry and tightly bounded IEEE-f64 accumulated position roundoff, separately from physical duration. Actual endpoints remain evidence; the physical start/half-frame gates are not relaxed.
- Generated tests cover 8/16-bar success/failure, rates, fractional tempos, callback partitions, late failures, restore/replay identity and tampering. DAW admission remained V2-only until its own supported successor.
- The separately authored user-requested README architecture overview was included unchanged in its own commit, covering Source Graph, Action lifecycle, Session ownership and realtime boundaries.

## Notes

- Independent review reproduced two P2 V3 timing families: fractional-f32/operation-order authority and accumulated real callback-clock representation. Canonical reconstruction, preallocation roundtrip rejection and bounded accumulation handling fixed both; the roundoff allowance is capped at 1/1024 frame and one microbeat. Actual production-clock tests cover 64/128/1024/varied partitions, while quarter-frame endpoint displacement still rejects without output.
- A late compatibility challenge found that applying full V3 readiness to legacy V2 would newly reject previously publishable two-bar takes at 121.5/166.5 BPM and 48 kHz. V2 producer spans 42187/57812 nanobeats while its old consumer expects 42188/57813. This PR preserved the original V2 host-audio publication gate and strict V3 readiness; RIOTBOX-1552 separately owns the inherited mismatch.
- All three introduced P2 findings were corrected and independently rechecked; zero introduced findings remained. A separate current-state architecture cadence audit found no retained P0–P3 issue in its sampled recording/export/calibration/realtime scope.
- Final reviewed head `f5ee01a7b862c87b3898c18b7ab07cefa13f9ce5` passed native CI. The closeout reports a synchronized main tree identical to reviewed tree `2726fce2003cc97322d19a7f4d70c991ec5a9634` and removal of exact merged feature branches, preserving the separately active 1552 branch.
- This was generated-only engineering/output-path evidence, not real host, listening, DAW import, source-general, hardness, release or Windows expansion. No real source/capture/Holdout/commercial audio or source discovery was authorized or performed. No new ActionCommand, app-local durable state, recorder or export truth was created.

- Archive-only verification on 2026-10-16 re-read the full current Linear issue and all comments, checked attached GitHub PR bodies/branches/merge commits and successful retained native checks, and used tracked review metadata where cited. All implementation tests, benchmarks, source access, host and human outcomes above are historical evidence, not checks rerun by this cleanup. Linear's attachment reader does not fetch external GitHub URLs; those links were verified directly with `gh pr view`. Metadata dates use the Linear UTC timestamps.
