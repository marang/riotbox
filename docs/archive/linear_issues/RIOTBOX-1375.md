# `RIOTBOX-1375` P023: Surface unavailable/degraded perform-risk cue before live moves

- Ticket: `RIOTBOX-1375`
- Title: `P023: Surface unavailable/degraded perform-risk cue before live moves`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1375/p023-surface-unavailabledegraded-perform-risk-cue-before-live-moves`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1375-p023-surface-unavailabledegraded-perform-risk-cue-before`
- Linear branch: `feature/riotbox-1375-p023-surface-unavailabledegraded-perform-risk-cue-before`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1339 (https://github.com/marang/riotbox/pull/1339)`
- Merge commit: `70cbcbb9ec39849134d74616aa3d6f7994fd2c49`
- Deleted from Linear: `2026-07-02`
- Verification: `cargo test -p riotbox-app --lib; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/tui_screen_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Readiness made ui_cue current priority; Jam Trust perform-risk still used generic action text before live/bar trust.

## What Shipped

- Jam Trust perform-risk now shows compact bar/live? cue for degraded and unavailable timing while keeping trusted/confirmed behavior unchanged, with snapshot tests and docs.

## Notes

- None
