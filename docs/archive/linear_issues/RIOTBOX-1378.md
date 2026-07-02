# `RIOTBOX-1378` P023: Prioritize source-family review actions after weak-output reconciliation

- Ticket: `RIOTBOX-1378`
- Title: `P023: Prioritize source-family review actions after weak-output reconciliation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1378/p023-prioritize-source-family-review-actions-after-weak-output`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1378-p023-prioritize-source-family-review-actions-after-weak`
- Linear branch: `feature/riotbox-1378-p023-prioritize-source-family-review-actions-after-weak`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1342 (https://github.com/marang/riotbox/pull/1342)`
- Merge commit: `de026746c423247b2e823878025ba572d2431118`
- Deleted from Linear: `2026-07-02`
- Verification: `python3 -m py_compile scripts/generate_sound_quality_readiness_report.py; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1378-readiness-rerun; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

After all weak-output categories were reconciled to stale controls, the readiness report still listed stale weak-output buckets as main next actions.

## What Shipped

- Filtered stale weak-output controls out of the main Next Actions when current top is none, kept source-family review and human/demo coverage as the primary next path, and added validation/mutation coverage.

## Notes

- None
