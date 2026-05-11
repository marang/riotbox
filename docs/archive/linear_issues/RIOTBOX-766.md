# `RIOTBOX-766` Add Jam-visible Source Timing observer recipe

- Ticket: `RIOTBOX-766`
- Title: `Add Jam-visible Source Timing observer recipe`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-766/add-jam-visible-source-timing-observer-recipe`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-766-add-jam-visible-source-timing-observer-recipe`
- Linear branch: `feature/riotbox-766-add-jam-visible-source-timing-observer-recipe`
- Assignee: `Markus`
- Labels: `timing`, `ux`
- PR: `#760 (https://github.com/marang/riotbox/pull/760)`
- Merge commit: `88a21fd2622824b9142d8ce1fcc9ba3dd2c96b2f`
- Deleted from Linear: `2026-05-11`
- Verification: `just user-session-observer-validator-fixtures; git diff --check; GitHub Actions Rust CI passed on PR #760`
- Docs touched: `README.md`, `docs/jam_recipes.md`
- Follow-ups: `None`

## Why This Ticket Existed

Users and agents needed a concrete way to record and validate what the Jam timing rail showed during a confusing TUI run without relying on terminal visual memory.

## What Shipped

- Added Recipe 1b to docs/jam_recipes.md for launching with --observer, validating the user-session observer NDJSON, and inspecting source_timing fields; linked README debugging guidance to that recipe.

## Notes

- None
