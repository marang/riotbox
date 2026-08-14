# `RIOTBOX-1083` P016: Attach confirmed timing grid refs to export artifacts

- Ticket: `RIOTBOX-1083`
- Title: `P016: Attach confirmed timing grid refs to export artifacts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1083/p016-attach-confirmed-timing-grid-refs-to-export-artifacts`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-02`
- Started: `Unknown`
- Finished: `2026-06-02`
- Branch: `feature/riotbox-p016-export-timing-grid-ref`
- Linear branch: `feature/riotbox-1083-p016-attach-confirmed-timing-grid-refs-to-export-artifacts`
- Assignee: `Unassigned`
- Labels: None
- PR: `#1063 (https://github.com/marang/riotbox/pull/1063)`
- Merge commit: `fe196318791e7752ba7b99c0825850e9cb77f334`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1063; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Shipped in PR #1063.

What changed:

* Product-mix artifact-set entries now attach `timing_grid_ref` when `runtime_state.source_timing.confirmed_grid` exists.
* The reference preserves source id, optional hypothesis id, confirming action, and confirmation timestamp.
* Older artifact entries default missing timing-grid evidence to absent.

Why it matters:

Software can retain the confirmed timing basis for an export, and musicians can see that a saved mix came from a trusted grid when one was confirmed.

## What Shipped

- Closed the bounded scope: P016: Attach confirmed timing grid refs to export artifacts.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.
