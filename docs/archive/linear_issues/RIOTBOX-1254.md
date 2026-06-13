# `RIOTBOX-1254` P023: Strengthen destructive gesture contrast from weak-output candidates

- Ticket: `RIOTBOX-1254`
- Title: `P023: Strengthen destructive gesture contrast from weak-output candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1254/p023-strengthen-destructive-gesture-contrast-from-weak-output`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1254-p023-destructive-gesture-contrast`
- Linear branch: `feature/riotbox-1254-p023-strengthen-destructive-gesture-contrast-from-weak`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1228 (https://github.com/marang/riotbox/pull/1228)`
- Merge commit: `3114a6ae84d15130a25992e6af01bd726b11eabc`
- Deleted from Linear: `2026-06-13`
- Verification: `py_compile`, `destructive-variation-professional-smoke`, `professional-output-suite-smoke`, readiness validation, `just audio-qa-ci`, `just ci`, GitHub `rust-ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Destructive gestures were still a weak-output candidate: flat stutter or polite restore could pass broader diagnostics without proving a stage-meaningful cut/stutter/restore contrast.

## What Shipped

- Tightened destructive-variation thresholds, strengthened source-aware tail policy for quieter dropouts/sharper stutters/harder restores, added professional-suite regression checks, and documented diagnostic boundaries.

## Notes

- None
