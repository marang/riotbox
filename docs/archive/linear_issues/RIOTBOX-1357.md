# `RIOTBOX-1357` P023: Strengthen destructive gestures from weak-output routing

- Ticket: `RIOTBOX-1357`
- Title: `P023: Strengthen destructive gestures from weak-output routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1357/p023-strengthen-destructive-gestures-from-weak-output-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1357-p023-strengthen-destructive-gestures-from-weak-output`
- Linear branch: `feature/riotbox-1357-p023-strengthen-destructive-gestures-from-weak-output`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1321 (https://github.com/marang/riotbox/pull/1321)`
- Merge commit: `15d59a93b8f08011e5669e73a32d8b8cf3229ff9`
- Deleted from Linear: `2026-07-01`
- Verification: `python -m py_compile; just destructive-variation-professional-smoke; just pro-pressure-source-matrix-smoke; just professional-output-suite-smoke; just sound-quality-readiness-report-smoke; just weak-output-fix-routing-fixtures; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Weak-output routing still identified destructive_gesture as a production fix; dense cuts and restores needed tighter output proof without punishing sparse pressure sections.

## What Shipped

- Strengthened source-backed dense dropout/stutter and restore attack shaping, added family-aware restore/pressure floor reporting, tightened destructive validator and professional-suite gates, and documented the diagnostic quality boundary.

## Notes

- None
