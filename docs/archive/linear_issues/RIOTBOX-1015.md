# `RIOTBOX-1015` P012: Restore MC-202 source-grid proof in Recipe 15

- Ticket: `RIOTBOX-1015`
- Title: `P012: Restore MC-202 source-grid proof in Recipe 15`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1015/p012-restore-mc-202-source-grid-proof-in-recipe-15`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-27`
- Started: `2026-05-27`
- Finished: `2026-05-27`
- Branch: `feature/riotbox-1015-p012-restore-mc-202-source-grid-proof-in-recipe-15`
- Linear branch: `feature/riotbox-1015-p012-restore-mc-202-source-grid-proof-in-recipe-15`
- Assignee: `Markus`
- Labels: `Audio`, `timing`
- PR: `#998 (https://github.com/marang/riotbox/pull/998)`
- Merge commit: `f00e845476a79b05682b6f20ce32bf198d330c8b`
- Deleted from Linear: `2026-05-27`
- Verification: `cargo test -p riotbox-audio --bin feral_grid_pack; just beat03-auto-feral-grid-proof local-beat03-feral-grid-auto-proof; just p012-all-lane-source-grid-output-proof; cargo fmt --check; just ci; just audio-qa-ci; just syncopated-source-showcase-smoke; just full-grid-export-reproducibility-smoke; GitHub Rust CI run 26521743640 success`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Recipe 15 strict all-lane source-grid proof failed because MC-202 Feral-grid output was still a silent compatibility stem.

## What Shipped

- Restored a primitive-renderer MC-202 source-grid proof output, updated manifest/report/validator expectations, and documented that this remains primitive support rather than source-derived phrase planning.

## Notes

- Beat03 proof after the change reported MC-202 stem RMS 0.002386326 and mc202_source_grid_alignment.hit_ratio 1.0.
