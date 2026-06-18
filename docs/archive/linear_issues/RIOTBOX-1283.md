# `RIOTBOX-1283` Remove fallback selection strategies from scripted professional-output generators

- Ticket: `RIOTBOX-1283`
- Title: `Remove fallback selection strategies from scripted professional-output generators`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1283/remove-fallback-selection-strategies-from-scripted-professional-output`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1283-remove-fallback-selection-strategies`
- Linear branch: `feature/riotbox-1283-remove-fallback-selection-strategies-from-scripted`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1256 (https://github.com/marang/riotbox/pull/1256)`
- Merge commit: `19a4ac1a9206dac7a5a43b72d04a581886986afa`
- Deleted from Linear: `2026-06-18`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py`; `git diff --check`; `just dense-break-performance-pack-smoke`; `just professional-output-suite-smoke`; `just audio-qa-ci`; `just ci`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md`, `docs/benchmarks/professional_output_suite_v1_2026-06-04.md`
- Follow-ups: `None`

## Why This Ticket Existed

Professional-output diagnostics must not let fallback scaffolds look like source-derived Riotbox intelligence.

## What Shipped

- Renamed fallback selection strategies to unavailable/degraded diagnostic states.
- Added dense-break report validation that rejects fallback selection strategies during generation and validation.
- Added Professional Output Suite child metadata and validator gates for fallback_selection_strategy_count.
- Added negative smoke/mutation fixtures and benchmark documentation for the fallback-selection evidence boundary.

## Notes

- None
