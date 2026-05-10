# `RIOTBOX-169` Add a bounded benchmark for the post-landed scene cue with energy labels

- Ticket: `RIOTBOX-169`
- Title: `Add a bounded benchmark for the post-landed scene cue with energy labels`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-169/add-a-bounded-benchmark-for-the-post-landed-scene-cue-with-energy`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-169-post-landed-scene-energy-benchmark`
- PR: `#159`
- Merge commit: `d09b5b0`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-170`

## Why This Ticket Existed

Once the post-landed Scene cue started carrying compact `scene/energy` labels, the repo needed a stable benchmark note so future cue tightening does not silently drop that richer context.

## What Shipped

- Added `docs/benchmarks/scene_post_landed_energy_cue_baseline_2026-04-25.md`.
- Linked the new baseline from the benchmark index and top-level docs index.

## Notes

- This stayed docs-only and captured the current `Jam` cue as a manual readability baseline.
