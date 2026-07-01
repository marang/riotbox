# `RIOTBOX-1368` P023: Make source-selection priority actionable after stale controls reconcile

- Ticket: `RIOTBOX-1368`
- Title: `P023: Make source-selection priority actionable after stale controls reconcile`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1368/p023-make-source-selection-priority-actionable-after-stale-controls`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1368-p023-make-source-selection-priority-actionable-after-stale`
- Linear branch: `feature/riotbox-1368-p023-make-source-selection-priority-actionable-after-stale`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1332 (https://github.com/marang/riotbox/pull/1332)`
- Merge commit: `69df0c9b7cd89e40dcfa5b4fd55ef7ede79e58fa`
- Deleted from Linear: `2026-07-01`
- Verification: `python -m py_compile scripts/generate_sound_quality_readiness_report.py; git diff --check; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1368-readiness; negative generic source-selection priority probe; negative missing source-selection evidence probe; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

After stale weak-output controls reconciled, source_selection became the current P023 product priority but readiness only exposed a generic source-window/source-character follow-up.

## What Shipped

- Sound-quality readiness now emits source_selection_priority with cases, primary cases, source families, artifact refs, demotion and review action linkage, concrete software next step, musician unavailable/degraded action, validator gates, and Markdown output.

## Notes

- The detail remains diagnostic: quality_proof=false and automated_musical_approval=false, with edge sources unavailable/degraded for promotion until timing, texture, and human verdict are trusted.
