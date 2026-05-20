# `RIOTBOX-814` Cover MC-202 lane source-grid alignment in listening-manifest fixtures

- Ticket: `RIOTBOX-814`
- Title: `Cover MC-202 lane source-grid alignment in listening-manifest fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-814/cover-mc-202-lane-source-grid-alignment-in-listening-manifest-fixtures`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-814-mc202-manifest-fixtures`
- Linear branch: `feature/riotbox-814-cover-mc-202-lane-source-grid-alignment-in-listening`
- Assignee: `Markus`
- Labels: `benchmark`
- PR: `#809 (https://github.com/marang/riotbox/pull/809)`
- Merge commit: `687025363f9b79085205c55d68c5a5081f724d45`
- Verification: `just listening-manifest-validator-fixtures; git diff --check; python3 -m json.tool on valid and invalid fixtures; just audio-qa-ci; just ci; GitHub Actions Rust CI #1966 passed`
- Docs touched: `None`
- Follow-ups: `None for this slice.`

## Why This Ticket Existed

Add explicit shared listening-manifest fixture coverage for the MC-202 lane source-grid alignment key documented in RIOTBOX-813.

## What Shipped

- Added mc202_source_grid_alignment to the valid lane source-grid fixture, added an invalid MC-202 alignment fixture, and wired it into just listening-manifest-validator-fixtures.

## Notes

- Linear deletion not performed because LINEAR_API_TOKEN is not available in this session.
