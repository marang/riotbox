# `RIOTBOX-1051` Add P015 in-app Jam taste/proof help cues

- Ticket: `RIOTBOX-1051`
- Title: `Add P015 in-app Jam taste/proof help cues`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1051/add-p015-in-app-jam-tasteproof-help-cues`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1051-p015-jam-help-taste-proof`
- Linear branch: `feature/riotbox-1051-add-p015-in-app-jam-tasteproof-help-cues`
- Assignee: `Markus`
- Labels: None
- PR: `#1028 (https://github.com/marang/riotbox/pull/1028)`
- Merge commit: `dc8d04353dc29244681fd629558b6f8e64e18331`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo fmt --check: pass`; `cargo test -p riotbox-app renders_help_overlay_with_jam_taste_and_proof_guidance -- --nocapture: pass`; `cargo test -p riotbox-app help_overlay -- --nocapture: pass`; `just p015-jam-taste-recipe-proof: pass`; `just ci: pass`; `GitHub rust-ci on PR #1028: pass`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P015 Jam taste/proof cues needed in-app explanation so musicians can understand cautious, scene-ready, and proof states without leaving Riotbox.

## What Shipped

- Added a compact Jam taste / proof section to the Help overlay.
- Mapped existing Arrangement / Scene readiness and proof state to musician-facing help language.
- Added focused Help overlay snapshot coverage for the taste/proof guidance.

## Notes

- None
