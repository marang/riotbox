# `RIOTBOX-170` Add a follow-up benchmark for the complete Scene Brain cue ladder

- Ticket: `RIOTBOX-170`
- Title: `Add a follow-up benchmark for the complete Scene Brain cue ladder`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-170/add-a-follow-up-benchmark-for-the-complete-scene-brain-cue-ladder`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-170-scene-cue-ladder-benchmark`
- PR: `#160`
- Merge commit: `7da98e7`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-171`

## Why This Ticket Existed

Scene Brain had separate baselines for queued guidance, restore readiness, restore contrast, and post-landed cues, but no single benchmark that walked the full reading path.

## What Shipped

- Added `docs/benchmarks/scene_cue_ladder_baseline_2026-04-25.md`.
- Linked the new cue-ladder baseline from the benchmark index and top-level docs index.

## Notes

- This stayed docs-only and captured the current `queued -> landed -> ready -> restore -> landed` path as one coherent readability baseline.
