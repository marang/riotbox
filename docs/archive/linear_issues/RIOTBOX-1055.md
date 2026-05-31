# `RIOTBOX-1055` Point in-app first-run help at Recipe 16 taste/proof path

- Ticket: `RIOTBOX-1055`
- Title: `Point in-app first-run help at Recipe 16 taste/proof path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1055/point-in-app-first-run-help-at-recipe-16-tasteproof-path`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1055-help-recipe16-pointer`
- Linear branch: `feature/riotbox-1055-point-in-app-first-run-help-at-recipe-16-tasteproof-path`
- Assignee: `Markus`
- Labels: None
- PR: `#1032 (https://github.com/marang/riotbox/pull/1032)`
- Merge commit: `d0fc0e1e0bed79dd4a60b3629bc0d33ad8112b2f`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo fmt --check: pass`; `cargo test -p riotbox-app renders_help_overlay_with_first_run_guidance -- --nocapture: pass`; `cargo test -p riotbox-app help_overlay -- --nocapture: pass`; `git diff --check: pass`; `targeted rg Recipe 16 Help copy check: pass`; `just ci: pass`; `GitHub rust-ci on PR #1032: pass`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Recipe 16 became the canonical P015 taste/proof path, but first-run Help still pointed only to Recipe 2 and Recipe 5 after the first loop.

## What Shipped

- Updated first-run Help follow-up copy to point at Recipe 16 taste/proof.
- Kept Recipe 2/5 as the gesture/source exploration path.
- Updated focused Help overlay test coverage.

## Notes

- None
