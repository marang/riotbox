# `RIOTBOX-1175` P016: Add live-recording host-audio receipt readiness gate

- Ticket: `RIOTBOX-1175`
- Title: `P016: Add live-recording host-audio receipt readiness gate`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1175/p016-add-live-recording-host-audio-receipt-readiness-gate`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1175-p016-live-recording-host-audio-readiness-gate`
- Linear branch: `feature/riotbox-1175-p016-add-live-recording-host-audio-receipt-readiness-gate`
- Assignee: `Markus`
- Labels: `Core`
- PR: `#1154 (https://github.com/marang/riotbox/pull/1154)`
- Merge commit: `3443dfc16b840eef63da547d1c9840887506f1ef`
- Deleted from Linear: `2026-06-03`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Live-recording receipts had typed host-audio refs but no Core/Session readiness gate to decide whether that evidence was sufficient.

## What Shipped

- Added a live-recording host-audio readiness report with typed status/blockers for wrong scope, unsupported flag, missing evidence, blank host/device, zero duration, callback-gap breaches, and stream errors; kept the gate receipt-only and updated P016 specs.

## Notes

- None
