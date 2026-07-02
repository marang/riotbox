# `RIOTBOX-1372` P023: Reconcile source-selection priority after non-dense policy coverage

- Ticket: `RIOTBOX-1372`
- Title: `P023: Reconcile source-selection priority after non-dense policy coverage`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1372/p023-reconcile-source-selection-priority-after-non-dense-policy`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1372-p023-reconcile-source-selection-priority-after-non-dense`
- Linear branch: `feature/riotbox-1372-p023-reconcile-source-selection-priority-after-non-dense`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1336 (https://github.com/marang/riotbox/pull/1336)`
- Merge commit: `a3041f3d9d9b18634a942f20ad4328c106a4d57c`
- Deleted from Linear: `2026-07-02`
- Verification: `python3 -m py_compile scripts/generate_sound_quality_readiness_report.py scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py`; `git diff --check`; `just professional-output-suite-smoke`; `just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1372-readiness`; `just ci`; GitHub `rust-ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`; `docs/execution_roadmap.md`; `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 readiness had enough family-coverage context to show source_selection as current, but the professional-output summary only aggregated dense_break policy evidence while existing tonal and sparse source-WAV policy evidence was omitted.

## What Shipped

- Professional-output source-selection policy summaries now aggregate dense-break plus professional source-WAV tonal and sparse policy cases.
- Candidate-count floors now apply only to expanded source-window searches, so verified tonal full-window policy can have one candidate.
- Sound-quality readiness now reconciles source_selection as stale once candidate families are covered by promotion-allowed policy families, and advances the current product gap to drum_pressure.

## Notes

- None
