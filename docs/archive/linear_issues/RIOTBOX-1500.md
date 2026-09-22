# `RIOTBOX-1500` [P1] Prevent capture WAV clobbering across Sessions sharing a directory

- Ticket: `RIOTBOX-1500`
- Title: `[P1] Prevent capture WAV clobbering across Sessions sharing a directory`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1500/p1-prevent-capture-wav-clobbering-across-sessions-sharing-a-directory`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-22`
- Started: `2026-09-22`
- Finished: `2026-09-22`
- Branch: `feature/riotbox-1500-capture-artifact-isolation`
- Linear branch: `feature/riotbox-1500-p1-prevent-capture-wav-clobbering-across-sessions-sharing-a`
- Assignee: `Markus`
- Labels: `Bug`, `Core`, `review-followup`
- PR: `#1533 (https://github.com/marang/riotbox/pull/1533)`
- Merge commit: `341de137875cde085736d98bfccf7db176866ec0`
- Deleted from Linear: `2026-09-22`
- Verification: `Public regression failed before and passed after. 107 capture-selected tests, full normal-parallel local just ci and GitHub Rust CI passed; tests cover two Sessions, bus prints, legacy paths, failed saves and partial-write cleanup.`
- Docs touched: `docs/specs/session_file_spec.md,docs/research_decision_log.md,docs/reviews/riotbox_1500_capture_artifact_isolation_2026-09-22.md`
- Follow-ups: `None`

## Why This Ticket Existed

Two Sessions in one directory could truncate each others cap-01 WAV; hash verification detected already-lost audio.

## What Shipped

- Exclusive fresh capture WAV allocation shared by source-window and bus-print paths; persist actual locator and identity only after successful writing. Legacy locators remain readable.

## Notes

- RBX-376 records the publication contract and orphan/power-loss limits. No new dependency, hardlink requirement, DSP, real source or human playback. Solo Rust review and self-review: no retained additional findings.
