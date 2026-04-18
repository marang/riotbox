# `RIOTBOX-113` Add source-driven scene contrast recipe for Scene Brain

- Ticket: `RIOTBOX-113`
- Title: `Add source-driven scene contrast recipe for Scene Brain`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-113/add-source-driven-scene-contrast-recipe-for-scene-brain`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-113-scene-contrast-recipes`
- Linear branch: `feature/riotbox-113-add-source-driven-scene-contrast-recipe-for-scene-brain`
- Assignee: `Markus`
- Labels: `None`
- PR: `#106`
- Merge commit: `c2e927d5ecb7ad4f092507f79b735d748effc730`
- Deleted from Linear: `Not deleted`
- Verification: `self-reviewed docs-only diff for recipe accuracy and README consistency`, GitHub Actions `Rust CI` run `#300`
- Docs touched: `README.md`, `docs/jam_recipes.md`
- Follow-ups: `RIOTBOX-114`

## Why This Ticket Existed

The first Scene Brain `jump -> restore` recipe explained the recovery loop, but it still treated all example sources as equally good teachers. Riotbox needed one small source-comparison recipe that said honestly which current source is better for timing and which is better for reading scene contrast.

## What Shipped

- added a new Scene Brain recipe that runs the same `jump -> restore` loop on two different example files
- called out `Beat08_128BPM(Full).wav` as the current timing-friendly learning source and `DH_RushArp_120_A.wav` as the clearer scene-contrast learning source
- added a root README pointer so the source-comparison path is discoverable from the main learning section

## Notes

- this slice stayed documentation-only and did not change scene behavior or source analysis
- the recipe is intentionally honest that Scene Brain legibility still depends heavily on source choice today
