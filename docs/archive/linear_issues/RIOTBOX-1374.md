# `RIOTBOX-1374` P023: Make current UI-cue priority actionable in readiness

- Ticket: `RIOTBOX-1374`
- Title: `P023: Make current UI-cue priority actionable in readiness`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1374/p023-make-current-ui-cue-priority-actionable-in-readiness`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1374-p023-make-current-ui-cue-priority-actionable-in-readiness`
- Linear branch: `feature/riotbox-1374-p023-make-current-ui-cue-priority-actionable-in-readiness`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1338 (https://github.com/marang/riotbox/pull/1338)`
- Merge commit: `e175cab3e43c2f5e0816a0e601748854dad186e5`
- Deleted from Linear: `2026-07-02`
- Verification: `python3 -m py_compile scripts/generate_sound_quality_readiness_report.py; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1374-readiness; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Readiness reported ui_cue as the current product gap, but the category was too generic to drive a concrete musician-facing unavailable/degraded cue slice.

## What Shipped

- Readiness now exposes ui_cue_priority with cases, source families, artifact refs, cue surface, cue reasons, software next step, musician action, required player cues, validation, mutation coverage, and docs.

## Notes

- None
