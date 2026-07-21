# `RIOTBOX-1421` P023: Add an explicit manual source-grid hypothesis for tonal live sources

- Ticket: `RIOTBOX-1421`
- Title: `P023: Add an explicit manual source-grid hypothesis for tonal live sources`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1421/p023-add-an-explicit-manual-source-grid-hypothesis-for-tonal-live`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-07-21`
- Started: `2026-07-21`
- Finished: `2026-07-21`
- Branch: `feature/riotbox-1421-p023-manual-tonal-source-grid`
- Linear branch: `feature/riotbox-1421-p023-add-an-explicit-manual-source-grid-hypothesis-for-tonal`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`
- PR: `#1371 (https://github.com/marang/riotbox/pull/1371)`
- Merge commit: `8734f0843bfa3345c4b7e9e919d32c09989fee79`
- Deleted from Linear: `2026-07-21`
- Verification: `GitHub rust-ci passed; cargo test -p riotbox-app passed; focused Core/App timing tests passed; exact RushArp and dense-break callback diagnostics passed.`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md; docs/specs/source_graph_spec.md; docs/specs/session_file_spec.md; docs/research_decision_log.md (RBX-155)`
- Follow-ups: `RIOTBOX-1404`

## Why This Ticket Existed

The trusted tonal source had no analyzer grid, and BPM alone could not honestly supply downbeat phase for the exact RIOTBOX-1404 live path.

## What Shipped

- Added a typed musician-declared Manual timing hypothesis from explicit BPM and downbeat phase while preserving analyzer evidence.
- Reused source_timing.confirm_grid across queue, commit, Session, replay, observer, revert, and restart instead of creating a shadow timing system.
- Bound hypothesis identity to BPM and phase, exposed manual origin in exact-path evidence, and unblocked the real DH_RushArp callback-path diagnostic.

## Notes

- Contract enabler only: human_verdict remains unverified and no tonal musical-pass claim was made.
