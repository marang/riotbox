# `RIOTBOX-239` Update Jam recipes for TR-909 accent cue

- Ticket: `RIOTBOX-239`
- Title: `Update Jam recipes for TR-909 accent cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-239/update-jam-recipes-for-tr-909-accent-cue`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-239-jam-recipes-tr909-accent`
- Linear branch: `feature/riotbox-239-update-jam-recipes-for-tr-909-accent-cue`
- PR: `#229`
- Merge commit: `710ad8a`
- Labels: `ux`, `benchmark`
- Follow-ups: `RIOTBOX-240`

## Why This Ticket Existed

`RIOTBOX-238` surfaced a compact TR-909 support accent cue in Log/Inspect diagnostics. Recipe 10 already taught `scene_target` and `transport_bar`, so the learning path needed to explain `accent scene` and fallback wording in the same place.

## What Shipped

- Updated `docs/jam_recipes.md` Recipe 10 with `accent scene` and `accent off fallback` wording.
- Kept the explanation honest: the accent is a subtle support lift, not a finished transition engine or arranger.
- Pointed users to the Log render line while practicing Scene jump/restore.

## Verification

- `git diff --check`
- `rg -n "accent scene|accent off fallback|Scene-target accent|source_support \\| accent" docs/jam_recipes.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only learning-path slice; no runtime behavior, broad recipe rewrite, screenshots, or audio QA harness changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
