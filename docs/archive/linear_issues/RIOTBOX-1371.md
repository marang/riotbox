# `RIOTBOX-1371` P023: Gate source-selection priority on policy family coverage

- Ticket: `RIOTBOX-1371`
- Title: `P023: Gate source-selection priority on policy family coverage`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1371/p023-gate-source-selection-priority-on-policy-family-coverage`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1371-p023-gate-source-selection-priority-on-policy-family`
- Linear branch: `feature/riotbox-1371-p023-gate-source-selection-priority-on-policy-family`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1335 (https://github.com/marang/riotbox/pull/1335)`
- Merge commit: `49c5a1805879f4fbc0cf573066450746843c6da2`
- Deleted from Linear: `2026-07-02`
- Verification: `python3 -m py_compile scripts/generate_sound_quality_readiness_report.py scripts/generate_professional_output_suite.py`; `git diff --check`; `just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1371-readiness`; `just professional-output-suite-smoke`; `just ci`; GitHub `rust-ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`; `docs/execution_roadmap.md`; `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 readiness showed source_selection as the current product gap while policy evidence only covered dense_break even though the active source-selection candidate also named tonal_hook.

## What Shipped

- Professional-output source-selection policy summaries now expose covered and promotion-allowed source families.
- Sound-quality readiness now surfaces covered and uncovered source-selection policy families and marks the current priority as source_selection_policy_family_gap.
- The readiness smoke now mutates away uncovered family evidence and requires validation to fail.

## Notes

- None
