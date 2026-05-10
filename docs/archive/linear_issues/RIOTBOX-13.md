# `RIOTBOX-13` Add app runtime state for audio and sidecar health

- Ticket: `RIOTBOX-13`
- Title: `Add app runtime state for audio and sidecar health`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-13/add-app-runtime-state-for-audio-and-sidecar-health`
- Project: `P002 | Core Skeleton`
- Milestone: `Core Skeleton`
- Status: `Done`
- Created: `2026-04-12`
- Finished: `2026-04-12`
- Branch: `lemonsterizoone/riotbox-13-add-app-runtime-state-for-audio-and-sidecar-health`
- PR: `#7`
- Merge commit: `0a8496f`
- Follow-ups: `RIOTBOX-17`

## Why This Ticket Existed

The app layer needed runtime-facing health state without moving runtime concerns into core contracts.

## What Shipped

- Added app-level audio and sidecar health state plus Jam-facing summaries.

## Notes

- This separated service/runtime state from `riotbox-core`.
