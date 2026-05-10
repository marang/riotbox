# `RIOTBOX-232` Document Scene-target TR-909 support context in Jam recipes

- Ticket: `RIOTBOX-232`
- Title: `Document Scene-target TR-909 support context in Jam recipes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-232/document-scene-target-tr-909-support-context-in-jam-recipes`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-232-jam-recipes-scene-target-context`
- Linear branch: `feature/riotbox-232-document-scene-target-tr-909-support-context-in-jam-recipes`
- PR: `#222`
- Merge commit: `659e97c`
- Labels: `ux`
- Follow-ups: `RIOTBOX-233`

## Why This Ticket Existed

`RIOTBOX-230` and `RIOTBOX-231` made Scene-target TR-909 `SourceSupport` coupling real and inspectable, but the current learning path did not tell a user what `scene_target` versus `transport_bar` means or how to observe it while trying Scene jump/restore recipes.

## What Shipped

- Updated Recipe 10 in `docs/jam_recipes.md` with a bounded TR-909 render-context check after a Scene jump lands.
- Explained `scene_target` and `transport_bar` in musician-facing language.
- Kept the docs explicit that this is render-state/profile coupling, not a finished transition engine.

## Verification

- `git diff --check`
- `rg -n "scene_target|transport_bar|render-state diagnostic|Scene-coupled" docs/jam_recipes.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only learning-path slice; no runtime behavior, TUI layout, or audio path changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
