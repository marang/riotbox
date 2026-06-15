# `RIOTBOX-1267` MC-202 musical scoring, rejection, and phrase memory

- Ticket: `RIOTBOX-1267`
- Title: `MC-202 musical scoring, rejection, and phrase memory`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1267/mc-202-musical-scoring-rejection-and-phrase-memory`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-14`
- Started: `2026-06-15`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1267-mc202-musical-scoring-rejection-memory`
- Linear branch: `feature/riotbox-1267-mc-202-musical-scoring-rejection-and-phrase-memory`
- Assignee: `Markus`
- Labels: None
- PR: `#1242 (https://github.com/marang/riotbox/pull/1242)`
- Merge commit: `c1a42edac32a6b7fa001aff662843ea495c1bf99`
- Deleted from Linear: `2026-06-15`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

MC-202 candidate families needed explicit musical selection logic. Without scorecards and phrase-memory evidence, QA could see which family won but not why it won or why static/control material was rejected.

## What Shipped

- Added persisted MC-202 candidate scorecards with low-end impact, source-grid lock, answer contrast, hook avoidance, phrase memory, destructive usefulness, role fit, selected flag, and rejection reason; added phrase-memory distance to the plan; split scoring into a focused module; added tests for deterministic scoring, memory recording, stay-out explanation, and fallback-control rejection.

## Notes

- None
