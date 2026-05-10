# `RIOTBOX-198` Teach Recipe 11 to verify the Log source-window cue

- Ticket: `RIOTBOX-198`
- Title: `Teach Recipe 11 to verify the Log source-window cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-198/teach-recipe-11-to-verify-the-log-source-window-cue`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-198-recipe-log-source-window-cue`
- Linear branch: `feature/riotbox-198-teach-recipe-11-to-verify-the-log-source-window-cue`
- PR: `#188`
- Merge commit: `2588248`
- Labels: `Audio`, `ux`
- Follow-ups: `RIOTBOX-199`

## Why This Ticket Existed

`RIOTBOX-197` made the W-30 Log lane show a compact source-window span, but Recipe 11 still taught only the `.../src` and `.../fallback` readiness labels.

## What Shipped

- Updated Recipe 11 expected observations with the compact Log `win ...` source-window cue.
- Kept the recipe scoped to bounded source-backed preview behavior and fallback interpretation.

## Verification

- `git diff --check main..HEAD`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only slice; no runtime behavior changed.
- This makes the recipe match the current Log truth screen.
