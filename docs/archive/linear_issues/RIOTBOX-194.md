# `RIOTBOX-194` Add a source-backed W-30 recipe smoke path for users

- Ticket: `RIOTBOX-194`
- Title: `Add a source-backed W-30 recipe smoke path for users`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-194/add-a-source-backed-w-30-recipe-smoke-path-for-users`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-194-source-backed-w30-recipe`
- Linear branch: `feature/riotbox-194-add-a-source-backed-w-30-recipe-smoke-path-for-users`
- PR: `#184`
- Merge commit: `0c25d12`
- Labels: `Audio`, `ux`
- Follow-ups: `RIOTBOX-195`

## Why This Ticket Existed

The source-backed W-30 preview path had become visible in the TUI, but users still needed one concrete practice flow that explains which keys to press and how to interpret `.../src` versus `.../fallback`.

## What Shipped

- Added `Recipe 11: Check Source-Backed W-30 Reuse` to `docs/jam_recipes.md`.
- Documented a source-backed W-30 audition -> promote -> hit/recall smoke path.
- Explained the expected `audition raw/src`, `audition/src`, `recall/.../src`, and fallback cue meanings.
- Linked Recipe 11 from the README's suggested next moves.

## Verification

- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only slice; no runtime behavior changed.
- The recipe stays honest that Riotbox currently uses bounded preview excerpts, not a full W-30 sampler engine.
