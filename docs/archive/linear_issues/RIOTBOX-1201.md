# `RIOTBOX-1201` Add rendered weak-output examples for professional source families

- Ticket: `RIOTBOX-1201`
- Title: `Add rendered weak-output examples for professional source families`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1201/add-rendered-weak-output-examples-for-professional-source-families`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-04`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1201-add-rendered-weak-output-examples-for-professional-source`
- Linear branch: `feature/riotbox-1201-add-rendered-weak-output-examples-for-professional-source`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1179 (https://github.com/marang/riotbox/pull/1179)`
- Merge commit: `c02cb9bec49a5ec8508b39c52f5e77190b4c3043`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/audio_qa_evidence_boundary.py scripts/generate_rendered_weak_professional_outputs.py scripts/generate_professional_output_suite.py; just rendered-weak-professional-output-fixtures artifacts/audio_qa/local-rendered-weak-professional-outputs; just professional-output-suite-smoke artifacts/audio_qa/local-professional-output-suite; just ci; GitHub Rust CI #27004178577 passed`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md; docs/benchmarks/professional_output_suite_v1_2026-06-04.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1202, RIOTBOX-1206, RIOTBOX-1207`

## Why This Ticket Existed

Rendered weak audio and evidence boundaries prevent scripted professional-output diagnostics from being mistaken for product-quality proof.

## What Shipped

- Added shared audio QA evidence-boundary fields across professional-output reports.
- Added rendered weak professional-output WAV diagnostics for flat stutter and weak restore rejection.
- Wired the negative diagnostics into audio-qa-ci and the professional-output suite.
- Updated roadmap, benchmark docs, and decision log to keep scripted audio as diagnostic evidence only.

## Notes

- None
