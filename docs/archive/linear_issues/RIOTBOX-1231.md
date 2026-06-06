# `RIOTBOX-1231` Add real weak-WAV source-character survival fixture

- Ticket: `RIOTBOX-1231`
- Title: `Add real weak-WAV source-character survival fixture`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1231/add-real-weak-wav-source-character-survival-fixture`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-06`
- Started: `2026-06-06`
- Finished: `2026-06-06`
- Branch: `feature/riotbox-1231-add-real-weak-wav-source-character-survival-fixture`
- Linear branch: `feature/riotbox-1231-add-real-weak-wav-source-character-survival-fixture`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1205 (https://github.com/marang/riotbox/pull/1205)`
- Merge commit: `a79950ffdc2657a22613c02beb24a25d6f8e134c`
- Deleted from Linear: `2026-06-06`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py; git diff --check; just dense-break-weak-source-character-fixture-smoke artifacts/audio_qa/local-riotbox-1231-weak-source-character; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1231-dense-smoke; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1231-suite-smoke; just ci; GitHub rust-ci pass`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1232, RIOTBOX-1233, RIOTBOX-1224`

## Why This Ticket Existed

Catches fallback-like rebuild-only audio with a real rendered weak WAV, not only JSON mutation.

## What Shipped

- Added --weak-source-character-fixture and --validate-weak-source-character-report to the dense-break performance generator; added just dense-break-weak-source-character-fixture-smoke; documented the negative diagnostic boundary in roadmap, audio QA spec, and RBX-081.

## Notes

- None
