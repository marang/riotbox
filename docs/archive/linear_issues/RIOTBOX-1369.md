# `RIOTBOX-1369` P023: Apply source-selection priority to source-window policy

- Ticket: `RIOTBOX-1369`
- Title: `P023: Apply source-selection priority to source-window policy`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1369/p023-apply-source-selection-priority-to-source-window-policy`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1369-p023-apply-source-selection-priority-to-source-window-policy`
- Linear branch: `feature/riotbox-1369-p023-apply-source-selection-priority-to-source-window-policy`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1333 (https://github.com/marang/riotbox/pull/1333)`
- Merge commit: `c94132b2f113a75897b08cfbafb3ad01f78a0c1d`
- Deleted from Linear: `2026-07-02`
- Verification: `just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1369-dense; just professional-output-suite-smoke; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1369-readiness; just agent-musical-review-pack-smoke; just destructive-variation-professional-smoke; scripts/run_compact.sh /tmp/riotbox-1369-ci.log just ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 source_selection was the current product risk after stale weak-output controls reconciled; Riotbox needed an applied source-window policy, not only readiness prose.

## What Shipped

- Applied family-specific source_selection_policy to dense product source windows, lifted it through professional suite/readiness, kept edge sources demoted, and updated destructive/agent review to use source-character/source-referenced proof when selected hooks raise the hook baseline.

## Notes

- Diagnostics remain quality_proof=false and human_verdict=unverified; this proves policy execution and output gates, not human musical approval.
