# `RIOTBOX-1365` P023: Reconcile weak-output bass priority with current pressure evidence

- Ticket: `RIOTBOX-1365`
- Title: `P023: Reconcile weak-output bass priority with current pressure evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1365/p023-reconcile-weak-output-bass-priority-with-current-pressure`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1365-p023-reconcile-weak-output-bass-priority-with-current`
- Linear branch: `feature/riotbox-1365-p023-reconcile-weak-output-bass-priority-with-current`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1329 (https://github.com/marang/riotbox/pull/1329)`
- Merge commit: `7554277334290175b6f321566285207b03aae7db`
- Deleted from Linear: `2026-07-01`
- Verification: `python -m py_compile scripts/generate_sound_quality_readiness_report.py; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1365-readiness; negative validator probe for missing bass_movement stale reconciliation; just audio-qa-ci; just ci; GitHub rust-ci pass on PR #1329.`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `Current reconciliation now points at destructive_gesture as the next product-priority category; implement or reconcile that gap from current professional evidence next.`

## Why This Ticket Existed

After W-30 stale fixture reconciliation, weak-output priority moved to bass_movement even though current professional-suite sparse movement and pressure gates already passed.

## What Shipped

- Extended current_evidence_reconciliation to bass_movement so stale sparse-bass weak fixtures remain negative controls while readiness advances the current product top category to destructive_gesture when current sparse pressure evidence passes.

## Notes

- None
