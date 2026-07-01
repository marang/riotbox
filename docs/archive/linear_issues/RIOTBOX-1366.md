# `RIOTBOX-1366` P023: Reconcile weak-output destructive priority with current gesture evidence

- Ticket: `RIOTBOX-1366`
- Title: `P023: Reconcile weak-output destructive priority with current gesture evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1366/p023-reconcile-weak-output-destructive-priority-with-current-gesture`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1366-p023-reconcile-weak-output-destructive-priority-with-current`
- Linear branch: `feature/riotbox-1366-p023-reconcile-weak-output-destructive-priority-with-current`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1330 (https://github.com/marang/riotbox/pull/1330)`
- Merge commit: `828a1666ec691768fc693b6c91ac978c5f005d10`
- Deleted from Linear: `2026-07-01`
- Verification: `python -m py_compile scripts/generate_sound_quality_readiness_report.py; git diff --check; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1366-readiness; negative validation probe for missing destructive reconciliation; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 readiness still let stale flat-stutter weak fixtures drive destructive_gesture priority even when current destructive gesture evidence passed.

## What Shipped

- Sound-quality readiness now summarizes destructive gesture metrics, marks destructive_gesture stale fixture-only when current dropout/stutter/restore gates pass, validates that reconciliation, and documents the priority rule.

## Notes

- This is diagnostic reconciliation only: it keeps quality_proof=false and automated_musical_approval=false while moving implementation focus to the next current audible product gap.
