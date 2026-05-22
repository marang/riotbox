# `RIOTBOX-962` Show observer beat/bar counts in P012 proof summary

- Ticket: `RIOTBOX-962`
- Title: `Show observer beat/bar counts in P012 proof summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-962/show-observer-beatbar-counts-in-p012-proof-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-962-show-observer-beatbar-counts-in-p012-proof-summary`
- Linear branch: `feature/riotbox-962-show-observer-beatbar-counts-in-p012-proof-summary`
- Assignee: `Markus`
- Labels: None
- PR: `#955 (https://github.com/marang/riotbox/pull/955)`
- Merge commit: `73d31d385cf3dbd6a161151a09353b174f7b038a`
- Deleted from Linear: `2026-05-22`
- Verification: `python3 -m py_compile scripts/write_p012_all_lane_proof_summary.py scripts/validate_p012_all_lane_proof_summary.py; just p012-all-lane-proof-summary; git diff --check; just ci; GitHub Actions Rust CI run 26305327042 success`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-963 continues Jam-facing compact timing count visibility`

## Why This Ticket Existed

Expose observer-side beat/bar evidence in the compact P012 all-lane proof summary.

## What Shipped

- Added Observer beats/bars columns from control_path.observer_source_timing and validator snippets for the generated summary.

## Notes

- None
