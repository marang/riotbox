# `RIOTBOX-110` Add first scene-restore learning recipe and README pointer

- Ticket: `RIOTBOX-110`
- Title: `Add first scene-restore learning recipe and README pointer`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-110/add-first-scene-restore-learning-recipe-and-readme-pointer`
- Project: `Riotbox MVP Buildout`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-110-scene-restore-recipe`
- Linear branch: `feature/riotbox-110-add-first-scene-restore-learning-recipe-and-readme-pointer`
- Assignee: `Markus`
- Labels: `None`
- PR: `#103`
- Merge commit: `985b32b6fc1e46a5c31941a912ba72c7c2154f92`
- Deleted from Linear: `Not deleted`
- Verification: `self-reviewed docs-only diff for keymap and shell consistency`, `GitHub Actions Rust CI #292`
- Docs touched: `README.md`, `docs/jam_recipes.md`
- Follow-ups: `RIOTBOX-111`, `RIOTBOX-113`

## Why This Ticket Existed

Once the first restore seam was real, visible, and replay-safe, Riotbox still did not teach users how to try it. The product needed one explicit `scene jump -> restore` recipe so `Y restore` felt like a musical recovery move rather than an unexplained advanced key.

## What Shipped

- added `Recipe 8` to the Jam recipe guide for the first bounded Scene Brain `jump -> restore` loop
- explained when `Y restore` becomes meaningful and what to watch in `Jam` versus `Log`
- added a root README pointer so the Scene Brain learning path sits alongside the existing gesture and source-comparison recipes

## Notes

- this slice stayed documentation-only and did not change any scene behavior
- the recipe is intentionally honest that restore only becomes available after a committed prior scene move

