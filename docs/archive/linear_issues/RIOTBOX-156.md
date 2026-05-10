# `RIOTBOX-156` Add fixture-backed regressions for shared Scene Brain energy projection

- Ticket: `RIOTBOX-156`
- Title: `Add fixture-backed regressions for shared Scene Brain energy projection`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-156/add-fixture-backed-regressions-for-shared-scene-brain-energy`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-156-scene-energy-fixtures`
- PR: `#146`
- Merge commit: `4cf9ef3`
- Labels: `review-followup`, `Core`
- Follow-ups: `RIOTBOX-157`

## Why This Ticket Existed

After scene energy moved into the shared projection layer, the repo needed fixture-backed coverage instead of relying only on shell-text assertions.

## What Shipped

- Added fixture-backed regressions for the shared Scene Brain energy projection.
- Covered current and restore energy readability through the replay-safe fixture layer.

## Notes

- This hardened the shared contract before the following help, footer, and benchmark readability work.
