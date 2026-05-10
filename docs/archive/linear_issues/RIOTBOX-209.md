# `RIOTBOX-209` Update recipes for restore energy direction cue

- Ticket: `RIOTBOX-209`
- Title: `Update recipes for restore energy direction cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-209/update-recipes-for-restore-energy-direction-cue`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-209-restore-direction-recipes`
- Linear branch: `feature/riotbox-209-update-recipes-for-restore-energy-direction-cue`
- PR: `#199`
- Merge commit: `06b86c5`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-210`

## Why This Ticket Existed

The periodic Scene Brain TUI seam review found that recipes still documented the pre-direction restore-ready cue even though the UI now renders `rise/drop/hold` when energy data is known.

## What Shipped

- Updated Scene Brain recipe examples to show the optional `rise/drop/hold` segment in restore-ready cues.
- Explained that the direction appears when both current and restore energies are known.
- Documented the fallback to target-only cue wording when energy direction cannot be inferred.

## Verification

- `git diff --check`
- `rg -n "rise/drop/hold|both current and restore energies|falls back" docs/jam_recipes.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only follow-up; no runtime behavior, UI implementation, or broad recipe rewrite changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
