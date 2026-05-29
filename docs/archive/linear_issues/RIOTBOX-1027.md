# `RIOTBOX-1027` Add all-lane mix movement proof to representative showcase

- Ticket: `RIOTBOX-1027`
- Title: `Add all-lane mix movement proof to representative showcase`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1027/add-all-lane-mix-movement-proof-to-representative-showcase`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-29`
- Started: `2026-05-29`
- Finished: `2026-05-29`
- Branch: `feature/riotbox-1027-all-lane-mix-movement-proof`
- Linear branch: `feature/riotbox-1027-add-all-lane-mix-movement-proof-to-representative-showcase`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1014 (https://github.com/marang/riotbox/pull/1014)`
- Merge commit: `8218f0105cb7dcd928a872215288d9da2cebb336`
- Deleted from Linear: `2026-05-29`
- Verification: `cargo fmt --check`; `git diff --check`; `cargo test -p riotbox-audio --bin feral_grid_pack`; `just representative-source-showcase-musical-quality-fixtures`; `just audio-qa-ci`; `just ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`, `docs/reviews/p013_all_lane_mix_movement_review_2026-05-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

P013 needed explicit proof that generated-support mix depth is not only non-silence or aggregate balance after W-30 and TR-909 lane-depth slices.

## What Shipped

- Added metrics.all_lane_mix_movement with source-first/support mix delta, correlation, per-lane contribution ratios, generated/W-30 contribution, representative validator gates and fixtures, audio QA spec notes, and branch review notes.

## Notes

- None
