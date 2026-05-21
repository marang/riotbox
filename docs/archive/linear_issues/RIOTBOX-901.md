# `RIOTBOX-901` Surface Source Timing phrase counts in compact P012 proof summary

- Ticket: `RIOTBOX-901`
- Title: `Surface Source Timing phrase counts in compact P012 proof summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-901/surface-source-timing-phrase-counts-in-compact-p012-proof-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-901-p012-summary-phrase-counts`
- Linear branch: `feature/riotbox-901-surface-source-timing-phrase-counts-in-compact-p012-proof`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#894 (https://github.com/marang/riotbox/pull/894)`
- Merge commit: `e57bd26af72cc8ebfdb41d6ad9143b00e8f3bc24`
- Deleted from Linear: `2026-05-21`
- Verification: `python3 -m py_compile scripts/write_p012_all_lane_proof_summary.py scripts/validate_p012_all_lane_proof_summary.py; just p012-all-lane-source-grid-output-proof; git diff --check; just ci; GitHub Rust CI #2225`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The compact P012 all-lane proof summary hid phrase-count evidence that generated Feral-grid manifests already carried.

## What Shipped

- Added Phrase count and Phrase bars columns to the Recipe 15 source-timing outcomes table.
- Read phrase counts from generated Feral-grid manifest source_timing fields.
- Updated the P012 proof-summary validator to require the phrase-count columns and expected real-source snippets.
- Kept readiness, grid-use, runtime, lane, and audio-output behavior unchanged.

## Notes

- None
