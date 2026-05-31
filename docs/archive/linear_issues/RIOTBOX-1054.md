# `RIOTBOX-1054` Align Recipe 16 scene-jump step with timing trust

- Ticket: `RIOTBOX-1054`
- Title: `Align Recipe 16 scene-jump step with timing trust`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1054/align-recipe-16-scene-jump-step-with-timing-trust`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1054-recipe16-timing-trust`
- Linear branch: `feature/riotbox-1054-align-recipe-16-scene-jump-step-with-timing-trust`
- Assignee: `Markus`
- Labels: None
- PR: `#1031 (https://github.com/marang/riotbox/pull/1031)`
- Merge commit: `0654e384e0ac0e0915694530ad4be2eacbf73f90`
- Deleted from Linear: `2026-05-31`
- Verification: `git diff --check: pass`; `targeted rg Recipe 16 timing-trust language check: pass`; `just ci: pass`; `GitHub rust-ci on PR #1031: pass`
- Docs touched: `docs/jam_recipes.md`, `docs/reviews/p015_jam_productization_review_2026-05-31.md`
- Follow-ups: `None`

## Why This Ticket Existed

P015 broad review found Recipe 16 still promoted scene jump too broadly after first-run UI copy began respecting timing trust.

## What Shipped

- Updated Recipe 16 to use y first only for scene-ready taste.
- Steered cautious, sketch, and unknown taste states toward g or f before trusting scene movement.
- Recorded the P015 Jam productization broad review and closure path under docs/reviews.

## Notes

- None
