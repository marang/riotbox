# `RIOTBOX-130` Add one bounded jump-restore learning recipe after the scene readability batch

- Ticket: `RIOTBOX-130`
- Title: `Add one bounded jump-restore learning recipe after the scene readability batch`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-130/add-one-bounded-jump-restore-learning-recipe-after-the-scene`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-130-scene-jump-restore-recipe`
- Linear branch: `feature/riotbox-130-add-one-bounded-jump-restore-learning-recipe-after-the-scene`
- Assignee: `Markus`
- Labels: `None`
- PR: `#123`
- Merge commit: `34846ff6268b5e24264887eea713584daebda193`
- Deleted from Linear: `Not deleted`
- Verification: `git diff --check`, branch diff review, GitHub Actions `Rust CI` run `#337`
- Docs touched: `README.md`, `docs/jam_recipes.md`
- Follow-ups: `RIOTBOX-131`, `RIOTBOX-133`

## Why This Ticket Existed

After the recent Scene Brain readability work, Riotbox had better timing and contrast cues but still no single focused practice flow that taught how to use them together. The user still had to infer the jump/restore seam from several separate docs and shell surfaces.

## What Shipped

- added a dedicated cue-reading recipe for the current `scene jump -> restore` learning loop
- taught the reader how to read `launch ->`, `pulse`, `live <> restore`, and `trail` together
- linked the new recipe from the main README learning-path bullets

## Notes

- this was a docs/product-learning slice only; it did not change Scene Brain behavior
- the recipe stayed bounded and honest about the current shell state instead of pretending Scene Brain is already a finished visual instrument
