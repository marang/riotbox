# `RIOTBOX-215` Update recipes for Scene launch suggestion direction

- Ticket: `RIOTBOX-215`
- Title: `Update recipes for Scene launch suggestion direction`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-215/update-recipes-for-scene-launch-suggestion-direction`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-215-launch-direction-recipes`
- Linear branch: `feature/riotbox-215-update-recipes-for-scene-launch-suggestion-direction`
- PR: `#205`
- Merge commit: `bfe721c`
- Labels: `ux`
- Follow-ups: `RIOTBOX-216`

## Why This Ticket Existed

`RIOTBOX-213` made suggested Scene launch gestures more readable by naming the deterministic next target and energy direction when available. The hands-on recipes needed to teach that cue so users can intentionally read the launch path before queuing it.

## What Shipped

- Updated Scene-focused recipes to show `[y] jump <scene> (rise/drop/hold)` as the pre-queue suggestion.
- Distinguished the suggested jump cue from the queued `launch -> ... @ next ...` boundary cue.
- Documented the generic `[y] jump` fallback when the next launch target cannot be inferred.

## Verification

- `git diff --check`
- `rg -n "\[y\] jump <scene>|pre-queue hint|generic \`[y] jump\`" docs/jam_recipes.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only learning-path slice; no runtime behavior, UI implementation, screenshots, or audio fixtures changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
