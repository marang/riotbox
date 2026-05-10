# `RIOTBOX-245` Teach Scene restore TR-909 accent cue in recipes

- Ticket: `RIOTBOX-245`
- Title: `Teach Scene restore TR-909 accent cue in recipes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-245/teach-scene-restore-tr-909-accent-cue-in-recipes`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-245-restore-accent-recipes`
- Linear branch: `feature/riotbox-245-teach-scene-restore-tr-909-accent-cue-in-recipes`
- PR: `#235`
- Merge commit: `4d0e44c`
- Labels: `ux`
- Follow-ups: `RIOTBOX-246`

## Why This Ticket Existed

The Scene Brain recipes already taught launch-time `scene_target` and `accent scene` diagnostics. After restore coupling was protected in focused and fixture-backed regressions, the user-facing recipe needed to explain that the same Log cue can apply after restore without implying a finished transition engine.

## What Shipped

- Updated Recipe 10 in `docs/jam_recipes.md`.
- Added a restore-side TR-909 render check after the restore lands.
- Clarified that `accent scene` after restore refers to the restored Scene target.
- Preserved the diagnostic/support-lift boundary and avoided new recipe breadth.

## Verification

- `git diff --check`
- `rg -n "after restore|restored Scene target|target is the restored Scene" docs/jam_recipes.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only learning-path slice; no runtime behavior, TUI layout, new recipe family, or audio QA harness changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
