# `RIOTBOX-1490` [P2] Reject overflowing WAV block alignment without panics in decoder and writer

- Ticket: `RIOTBOX-1490`
- Title: `[P2] Reject overflowing WAV block alignment without panics in decoder and writer`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1490/p2-reject-overflowing-wav-block-alignment-without-panics-in-decoder`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-05`
- Started: `2026-09-08`
- Finished: `2026-09-08`
- Branch: `feature/riotbox-1490-wav-format-validation`
- Linear branch: `feature/riotbox-1490-p2-reject-overflowing-wav-block-alignment-without-panics-in`
- Assignee: `Markus`
- Labels: `Audio`, `Bug`, `review-followup`
- PR: `#1509 (https://github.com/marang/riotbox/pull/1509)`
- Merge commit: `da9e994bffcd6fcfe5264299fda0ebecaa6d1276`
- Deleted from Linear: `2026-09-08`
- Verification: `14 source-audio tests passed in debug and release; full just ci, independent Rust review and GitHub rust-ci passed.`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-1491 remains deferred; no TUI work.`

## Why This Ticket Existed

Malformed PCM channel metadata caused overflow or modulo-by-zero panics and invalid writer output.

## What Shipped

- Checked WAV frame alignment and zero sample-rate validation return errors instead of panics.
- Added synthetic debug/release boundary and valid-format roundtrip regressions.

## Notes

- None
