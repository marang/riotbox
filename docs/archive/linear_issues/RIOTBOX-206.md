# `RIOTBOX-206` Document low-energy Scene Brain contrast in recipes

- Ticket: `RIOTBOX-206`
- Title: `Document low-energy Scene Brain contrast in recipes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-206/document-low-energy-scene-brain-contrast-in-recipes`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-206-document-low-energy-scene-contrast`
- Linear branch: `feature/riotbox-206-document-low-energy-scene-brain-contrast-in-recipes`
- PR: `#196`
- Merge commit: `70bc4a1`
- Labels: `ux`
- Follow-ups: `RIOTBOX-207`

## Why This Ticket Existed

Scene Brain fixtures now cover both medium/high and low/high contrast, but the recipes still mostly taught the medium/high path. Users needed a short, honest note explaining how to read `break/low` without implying Scene Brain transitions are musically finished.

## What Shipped

- Added a low-energy contrast note to the Scene Brain recipe area.
- Explained `drop/high <> break/low` and `live break/low <> restore drop/high` as current regression/readability vocabulary.
- Kept the existing recipe structure intact without adding a larger tutorial.

## Verification

- `git diff --check`
- `rg -n "Low-energy contrast|drop/high <> break/low|live break/low" docs/jam_recipes.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only slice; no runtime behavior, UI implementation, screenshots, or broad README rewrite changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
