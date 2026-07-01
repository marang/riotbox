# `RIOTBOX-1367` P023: Reconcile weak-output mix-bus priority with current support evidence

- Ticket: `RIOTBOX-1367`
- Title: `P023: Reconcile weak-output mix-bus priority with current support evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1367/p023-reconcile-weak-output-mix-bus-priority-with-current-support`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1367-p023-reconcile-weak-output-mix-bus-priority-with-current`
- Linear branch: `feature/riotbox-1367-p023-reconcile-weak-output-mix-bus-priority-with-current`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1331 (https://github.com/marang/riotbox/pull/1331)`
- Merge commit: `dd736e793a08f4252dc5766b7a57f6a7c18f9be0`
- Deleted from Linear: `2026-07-01`
- Verification: `python -m py_compile scripts/generate_sound_quality_readiness_report.py; git diff --check; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1367-readiness; negative validation probe for missing mix-bus reconciliation; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

After stale chop, bass, and destructive controls were reconciled, P023 readiness surfaced mix_bus as the current product top category even though current professional-suite mix evidence passed.

## What Shipped

- Sound-quality readiness now carries mix masking headroom, synchronizes the mix support floor with the professional-suite gate, marks mix_bus stale fixture-only when current mix gates pass, validates that reconciliation, and documents the priority rule.

## Notes

- This remains diagnostic reconciliation only: old source-masked and support-buried fixtures stay as negative controls, while quality_proof and automated_musical_approval remain false.
