# `RIOTBOX-165` Refresh restore-state contrast benchmark for current wake-up wording

- Ticket: `RIOTBOX-165`
- Title: `Refresh restore-state contrast benchmark for current wake-up wording`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-165/refresh-restore-state-contrast-benchmark-for-current-wake-up-wording`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-165-refresh-restore-state-contrast-benchmark-for-current-wake-up`
- PR: `#155`
- Merge commit: `926112e`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-166`, `RIOTBOX-167`

## Why This Ticket Existed

The new restore-state contrast benchmark had already landed, but it still quoted the previous wake-up wording and no longer matched the shipped shell.

## What Shipped

- Refreshed the restore-state contrast benchmark to use the current `waits for one landed jump` copy.
- Tightened the benchmark wording so the wake-up side read as blocked and provisional rather than merely generic.

## Notes

- This preserved the benchmark as a valid comparison point for the current restore state machine.
