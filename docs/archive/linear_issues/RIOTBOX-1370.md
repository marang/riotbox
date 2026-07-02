# `RIOTBOX-1370` P023: Reconcile destructive priority with source-referenced destructive proof

- Ticket: `RIOTBOX-1370`
- Title: `P023: Reconcile destructive priority with source-referenced destructive proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1370/p023-reconcile-destructive-priority-with-source-referenced-destructive`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1370-p023-reconcile-destructive-priority-with-source-referenced`
- Linear branch: `feature/riotbox-1370-p023-reconcile-destructive-priority-with-source-referenced`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1334 (https://github.com/marang/riotbox/pull/1334)`
- Merge commit: `538857d2610e6ac3cdebf3a4cb3cc39c11b1a760`
- Deleted from Linear: `2026-07-02`
- Verification: `python3 -m py_compile scripts/generate_sound_quality_readiness_report.py`; `git diff --check`; `just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1370-readiness`; `just ci`; GitHub `rust-ci`
- Docs touched: `docs/execution_roadmap.md`; `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 readiness still treated destructive gesture proof as hook-only, so stale flat-stutter controls could remain the apparent top gap even when source-referenced destructive evidence passed.

## What Shipped

- Sound-quality readiness now carries source-referenced destructive transient ratios into JSON and Markdown summaries.
- Destructive reconciliation now uses the same hook-or-source transient contract as the professional destructive validator.
- The readiness smoke now mutates away source-referenced destructive proof and requires validation to fail before stale destructive demotion is accepted.

## Notes

- None
