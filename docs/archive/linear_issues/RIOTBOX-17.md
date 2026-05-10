# `RIOTBOX-17` Add Jam runtime orchestration above transport and queue

- Ticket: `RIOTBOX-17`
- Title: `Add Jam runtime orchestration above transport and queue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-17/add-jam-runtime-orchestration-above-transport-and-queue`
- Project: `P002 | Core Skeleton`
- Milestone: `Core Skeleton`
- Status: `Done`
- Created: `2026-04-12`
- Finished: `2026-04-12`
- Branch: `lemonsterizoone/riotbox-17-add-jam-runtime-orchestration-above-transport-and-queue`
- PR: `#9`
- Merge commit: `d51d3ec`
- Follow-ups: `RIOTBOX-20`

## Why This Ticket Existed

The app layer needed to connect transport updates, queue commits, and Jam state coherently.

## What Shipped

- Added app-level runtime orchestration for transport refresh and boundary commits.

## Notes

- This established the first real runtime coordinator in `riotbox-app`.
