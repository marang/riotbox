# `RIOTBOX-1308` P023: Strengthen source-window character selection before output promotion

- Ticket: `RIOTBOX-1308`
- Title: `P023: Strengthen source-window character selection before output promotion`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1308/p023-strengthen-source-window-character-selection-before-output`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1308-p023-strengthen-source-window-character-selection-before-output-promotion`
- Linear branch: `feature/riotbox-1308-p023-strengthen-source-window-character-selection-before`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1282 (https://github.com/marang/riotbox/pull/1282)`
- Merge commit: `d916b1ae`
- Deleted from Linear: `2026-06-29`
- Verification: `just ci; GitHub rust-ci; professional suite source_character_window_selection result pass with case_count 10 and promoted_case_count 0; generated listening manifest regression fixed`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `Continue P023 queued follow-ups: RIOTBOX-1309 drum pressure, RIOTBOX-1310 fixture thresholds, RIOTBOX-1311 source/timing UI cue.`

## Why This Ticket Existed

P023 weak-output routing flagged source_selection: Riotbox needed an explicit source-window character decision before mix and gesture polish.

## What Shipped

- Added Feral source-character window selection over real source windows, routed W-30/TR-909/MC-202 through the selected window, recorded requested-vs-selected metrics in reports/manifests/professional-suite/readiness output, added promotion and RMS-retention regressions, and documented the boundary in the roadmap.

## Notes

- Current real-corpus professional suite keeps promoted_case_count 0; promoted behavior is proven by regression fixtures and remains diagnostic with human_verdict unverified, not quality proof.
