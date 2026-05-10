# `RIOTBOX-160` Add a bounded benchmark for restore-ready cue readability

- Ticket: `RIOTBOX-160`
- Title: `Add a bounded benchmark for restore-ready cue readability`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-160/add-a-bounded-benchmark-for-restore-ready-cue-readability`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-160-add-a-bounded-benchmark-for-restore-ready-cue-readability`
- PR: `#150`
- Merge commit: `401b793`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-162`, `RIOTBOX-166`

## Why This Ticket Existed

The queued-scene guidance stack already had explicit baselines, but the restore-ready state was still only covered indirectly through recipes and shell snapshots.

## What Shipped

- Added the first dedicated restore-ready readability baseline under `docs/benchmarks/`.
- Linked that baseline into the benchmark index and the top-level docs index.

## Notes

- This established the repo-local baseline that later wording refinements could compare against.
