# `RIOTBOX-1364` P023: Reconcile weak-output priority with current W-30 response evidence

- Ticket: `RIOTBOX-1364`
- Title: `P023: Reconcile weak-output priority with current W-30 response evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1364/p023-reconcile-weak-output-priority-with-current-w-30-response`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1364-p023-reconcile-weak-output-priority-with-current-w30`
- Linear branch: `feature/riotbox-1364-p023-reconcile-weak-output-priority-with-current-w-30`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1328 (https://github.com/marang/riotbox/pull/1328)`
- Merge commit: `b2f5754f85d5f0c7fa46070093ca93092b5a3495`
- Deleted from Linear: `2026-07-01`
- Verification: `python -m py_compile scripts/generate_sound_quality_readiness_report.py; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1364-readiness-final; negative validator probe for missing chop_policy stale reconciliation; just audio-qa-ci; just ci; GitHub rust-ci pass on PR #1328.`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `Current reconciliation now points at bass_movement as the next product-priority category; implement or reconcile that gap from current professional evidence next.`

## Why This Ticket Existed

P023 weak-output routing still reported chop_policy as top priority from stale/static weak fixtures even though current professional-suite W-30 response evidence passed dense, matrix, and tonal gates.

## What Shipped

- Added current_evidence_reconciliation to the sound-quality readiness report so chop_policy is marked stale fixture-only when current W-30 response gates pass, while keeping negative fixtures diagnostic and surfacing the current product top category separately.

## Notes

- None
