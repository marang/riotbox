# `RIOTBOX-219` Document disabled Scene jump cue

- Ticket: `RIOTBOX-219`
- Title: `Document disabled Scene jump cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-219/document-disabled-scene-jump-cue`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-219-disabled-scene-jump-docs`
- Linear branch: `feature/riotbox-219-document-disabled-scene-jump-cue`
- PR: `#209`
- Merge commit: `cf5d33b`
- Labels: `ux`
- Follow-ups: `RIOTBOX-220`

## Why This Ticket Existed

`RIOTBOX-218` clarified the suggested Scene jump gesture when no queueable target exists. The TUI contract and hands-on recipes needed to explain that the cue can say `waits for 2 scenes` instead of implying a jump is available.

## What Shipped

- Documented that Scene launch suggestions may say `[y] jump waits for 2 scenes`.
- Updated Jam recipes to distinguish unknown launch target fallback from known unavailable Scene material.
- Kept wording aligned with the implemented `RIOTBOX-218` cue.

## Verification

- `git diff --check`
- `rg -n "waits for 2 scenes|launch waits for more scene" docs/specs/tui_screen_spec.md docs/jam_recipes.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only contract/learning-path slice; no runtime behavior, screenshots, audio behavior, or broad recipe rewrite changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
