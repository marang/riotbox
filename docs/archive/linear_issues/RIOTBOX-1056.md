# `RIOTBOX-1056` Add P015 Help overlay height/readability audit

- Ticket: `RIOTBOX-1056`
- Title: `Add P015 Help overlay height/readability audit`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1056/add-p015-help-overlay-heightreadability-audit`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1056-help-overlay-readability`
- Linear branch: `feature/riotbox-1056-add-p015-help-overlay-heightreadability-audit`
- Assignee: `Markus`
- Labels: None
- PR: `#1033 (https://github.com/marang/riotbox/pull/1033)`
- Merge commit: `e373211b8afd3ad09569874a551560f3b22b43fe`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo fmt --check: pass`; `cargo test -p riotbox-app help_overlay_keeps -- --nocapture: pass`; `cargo test -p riotbox-app help_overlay -- --nocapture: pass`; `git diff --check: pass`; `just ci: pass`; `GitHub rust-ci on PR #1033: pass`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P015 Help overlay gained enough contextual lines that dense states could clip important first-run, taste/proof, and primary gesture guidance.

## What Shipped

- Expanded the Help overlay height from 55% to 85%.
- Added snapshot tests proving first-run, taste/proof, primary, and advanced Help sections remain visible.
- Added snapshot coverage for pending Scene timing, Jam taste/proof, and Primary gestures visible together.

## Notes

- None
