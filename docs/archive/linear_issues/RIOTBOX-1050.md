# `RIOTBOX-1050` Add P015 perform-first Jam taste recipe proof

- Ticket: `RIOTBOX-1050`
- Title: `Add P015 perform-first Jam taste recipe proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1050/add-p015-perform-first-jam-taste-recipe-proof`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1050-p015-jam-taste-recipe-proof`
- Linear branch: `feature/riotbox-1050-add-p015-perform-first-jam-taste-recipe-proof`
- Assignee: `Markus`
- Labels: None
- PR: `#1027 (https://github.com/marang/riotbox/pull/1027)`
- Merge commit: `5cbfa7c7d3d600384174af35bd497f545a90dc3b`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo fmt --check: pass`; `cargo test -p riotbox-app p015_recipe -- --nocapture: pass`; `just p015-jam-taste-recipe-proof: pass`; `just ci: pass`; `GitHub rust-ci on PR #1027: pass`
- Docs touched: `docs/jam_recipes.md`
- Follow-ups: `None`

## Why This Ticket Existed

P015 needed an executable musician path for Jam taste/proof cues so the compact perform surface, inspect details, and existing output proof stay aligned.

## What Shipped

- Added Recipe 16 for the perform-first Jam taste/proof read from a real source with observer validation.
- Added p015_recipe UI tests for compact perform cues, inspect-owned proof details, and locked-grid scene-ready taste language.
- Added just p015-jam-taste-recipe-proof, reusing the P014 scene-movement observer/audio gate for landed movement proof.

## Notes

- None
