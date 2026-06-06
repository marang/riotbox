# `RIOTBOX-1232` Route professional-output failures to concrete sound fix categories

- Ticket: `RIOTBOX-1232`
- Title: `Route professional-output failures to concrete sound fix categories`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1232/route-professional-output-failures-to-concrete-sound-fix-categories`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-06`
- Started: `2026-06-06`
- Finished: `2026-06-06`
- Branch: `feature/riotbox-1232-route-professional-output-failures-to-concrete-sound-fix`
- Linear branch: `feature/riotbox-1232-route-professional-output-failures-to-concrete-sound-fix`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1206 (https://github.com/marang/riotbox/pull/1206)`
- Merge commit: `cdd3984413b8a9be3ef7d9992a64ce4e52b17b09`
- Deleted from Linear: `2026-06-06`
- Verification: `python3 -m py_compile scripts/route_weak_output_fixes.py scripts/generate_dense_break_performance_pack.py scripts/generate_edge_source_professional_diagnostics.py scripts/generate_professional_output_suite.py; git diff --check; just weak-output-fix-routing-fixtures artifacts/audio_qa/local-riotbox-1232-routing; just edge-source-professional-diagnostics-smoke artifacts/audio_qa/local-riotbox-1232-edge-smoke; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1232-suite-smoke; just ci; GitHub rust-ci pass`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1233, RIOTBOX-1224`

## Why This Ticket Existed

Weak professional-output failures needed actionable sound-fix routing instead of raw metric names or unknown generic buckets.

## What Shipped

- Mapped rebuild-only source-character loss to source_selection; added matched_known_routing_signal and musician_fix_reason to weak-output routing cases and markdown; added a live generated weak source-character routing proof and an unknown-code negative fixture; documented the routing contract in roadmap, audio QA spec, and RBX-082.

## Notes

- None
