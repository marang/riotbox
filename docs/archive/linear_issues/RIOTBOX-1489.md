# `RIOTBOX-1489` [P2] Verify source WAV content identity before marking restored audio Loaded

- Ticket: `RIOTBOX-1489`
- Title: `[P2] Verify source WAV content identity before marking restored audio Loaded`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1489/p2-verify-source-wav-content-identity-before-marking-restored-audio`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-05`
- Started: `2026-09-08`
- Finished: `2026-09-08`
- Branch: `feature/riotbox-1489-restore-source-integrity`
- Linear branch: `feature/riotbox-1489-p2-verify-source-wav-content-identity-before-marking`
- Assignee: `Markus`
- Labels: `Audio`, `Bug`, `Core`, `review-followup`
- PR: `#1511 (https://github.com/marang/riotbox/pull/1511)`
- Merge commit: `4beb4dcf2dcb790d8f1ddedbccdb2a800d06341f`
- Deleted from Linear: `2026-09-08`
- Verification: `Full local and final all-four combined source-free CI, independent reviews and GitHub rust-ci passed.`
- Docs touched: `docs/specs/session_file_spec.md`, `docs/research_decision_log.md` (RBX-370)
- Follow-ups: `RIOTBOX-1491 and TUI remain deferred.`

## Why This Ticket Existed

Restore trusted source paths without verifying WAV content identity.

## What Shipped

- Verified the same decoded full-file byte buffer against Graph and Session source hashes before cache admission.
- Added unavailable-cache and monitor/activation regressions and repaired synthetic fixtures.

## Notes

- RBX-370 freezes identity admission. No real source or playback; no DSP or musical-quality claim.
