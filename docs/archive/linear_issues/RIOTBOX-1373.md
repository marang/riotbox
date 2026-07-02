# `RIOTBOX-1373` P023: Reconcile drum-pressure priority with current TR-909 pressure proof

- Ticket: `RIOTBOX-1373`
- Title: `P023: Reconcile drum-pressure priority with current TR-909 pressure proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1373/p023-reconcile-drum-pressure-priority-with-current-tr-909-pressure`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1373-p023-reconcile-drum-pressure-priority-with-current-tr-909`
- Linear branch: `feature/riotbox-1373-p023-reconcile-drum-pressure-priority-with-current-tr-909`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1337 (https://github.com/marang/riotbox/pull/1337)`
- Merge commit: `93cba0caf2bf7a8e6f2706e2eafab1c280b29894`
- Deleted from Linear: `2026-07-02`
- Verification: `python3 -m py_compile scripts/generate_sound_quality_readiness_report.py; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1373-readiness; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Old weak drum-pressure fixtures still appeared as current priority after current dense and rendered TR-909 proof passed.

## What Shipped

- Readiness now reconciles drum_pressure as stale regression control only when dense snare pressure and rendered TR-909 support/low-band/masking gates pass, with mutation coverage and docs.

## Notes

- None
