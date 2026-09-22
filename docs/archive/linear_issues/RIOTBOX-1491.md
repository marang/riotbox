# `RIOTBOX-1491` [P3] Measure and remove quadratic Session restore history validation scans

- Ticket: `RIOTBOX-1491`
- Title: `[P3] Measure and remove quadratic Session restore history validation scans`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1491/p3-measure-and-remove-quadratic-session-restore-history-validation`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-05`
- Started: `2026-09-22`
- Finished: `2026-09-22`
- Branch: `feature/riotbox-1491-restore-history-index`
- Linear branch: `feature/riotbox-1491-p3-measure-and-remove-quadratic-session-restore-history`
- Assignee: `Markus`
- Labels: `Core`, `Improvement`, `benchmark`, `review-followup`
- PR: `#1525 (https://github.com/marang/riotbox/pull/1525)`
- Merge commit: `cef503d65e0bccd3de7695047e1271f4d4a8e8d9`
- Deleted from Linear: `2026-09-22`
- Verification: `Full local source-free just ci and GitHub Rust CI passed; solo code/Rust review has zero outstanding findings.`; `16k entry medians: plain 634.360 to 78.203 ms; typed undo 2279.728 to 89.565 ms, informational local measurements.`
- Docs touched: `docs/reviews/riotbox_1491_restore_history_scaling_2026-09-22.md`
- Follow-ups: `RIOTBOX-1412 measures unchanged W-30 snapshot cost independently.`

## Why This Ticket Existed

Long Session histories incurred repeated quadratic restore validation scans.

## What Shipped

- Transient action, commit and trusted-undo indexes preserve validation and replay contracts; semantic validation module and regression/benchmark coverage.

## Notes

- No schema, audio behavior, source access, DAW or human listening change. Original undo normalization finding was already stale.
