# `RIOTBOX-155` Promote current and restore scene energy into the shared Jam view contract

- Ticket: `RIOTBOX-155`
- Title: `Promote current and restore scene energy into the shared Jam view contract`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-155/promote-current-and-restore-scene-energy-into-the-shared-jam-view`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-155-shared-scene-energy`
- PR: `#145`
- Merge commit: `86e6ad9`
- Labels: `review-followup`, `TUI`, `Core`
- Follow-ups: `RIOTBOX-156`, `RIOTBOX-157`

## Why This Ticket Existed

The periodic review flagged that Scene Brain energy was still effectively inferred in the presentation layer instead of coming from one shared Jam view contract.

## What Shipped

- Promoted current and restore scene energy into the shared Jam view model.
- Updated the TUI to consume that shared projection rather than guessing the energy labels locally.

## Notes

- This converted a useful display convention into a shared contract that later shell wording could rely on.
