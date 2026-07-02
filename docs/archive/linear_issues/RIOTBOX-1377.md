# `RIOTBOX-1377` P023: Reconcile stale fixture-threshold routing after current destructive proof

- Ticket: `RIOTBOX-1377`
- Title: `P023: Reconcile stale fixture-threshold routing after current destructive proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1377/p023-reconcile-stale-fixture-threshold-routing-after-current`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1377-p023-reconcile-stale-fixture-threshold-routing-after-current`
- Linear branch: `feature/riotbox-1377-p023-reconcile-stale-fixture-threshold-routing-after-current`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1341 (https://github.com/marang/riotbox/pull/1341)`
- Merge commit: `d8666159abc46a309a93440b9c59d0c7dc4bff0b`
- Deleted from Linear: `2026-07-02`
- Verification: `python3 -m py_compile scripts/generate_sound_quality_readiness_report.py; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1377-readiness; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

After ui_cue reconciliation, readiness selected fixture_threshold only because of secondary expected-fail negative-control routing, even though current destructive output proof passed.

## What Shipped

- Added fixture_threshold current-evidence reconciliation, kept primary/unknown threshold cases current, demoted only secondary negative-control routes covered by current destructive proof, and advanced readiness to source-selection/human-demo coverage without quality claims.

## Notes

- None
