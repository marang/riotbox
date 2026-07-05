# `RIOTBOX-1381` P023: Render review artifact refs in readiness markdown

- Ticket: `RIOTBOX-1381`
- Title: `P023: Render review artifact refs in readiness markdown`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1381/p023-render-review-artifact-refs-in-readiness-markdown`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1381-p023-render-review-artifact-refs-in-readiness-markdown`
- Linear branch: `feature/riotbox-1381-p023-render-review-artifact-refs-in-readiness-markdown`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1345 (https://github.com/marang/riotbox/pull/1345)`
- Merge commit: `3b1fae028c9ca03f19e817ee0343d4f24e3e2d83`
- Deleted from Linear: `2026-07-05`
- Verification: `python3 -m py_compile scripts/generate_sound_quality_readiness_report.py; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1381-readiness; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

The readiness JSON carried review artifact refs, but the Markdown report still hid rendered WAV, metrics, and review prompt paths from the human-readable review worklist.

## What Shipped

- Rendered source-family review artifact refs in Next Actions and Human Review Queue Markdown sections, with smoke coverage for bad-timing WAV and prompt refs.

## Notes

- None
