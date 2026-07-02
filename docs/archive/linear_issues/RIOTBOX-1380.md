# `RIOTBOX-1380` P023: Carry review artifact refs into source-family readiness actions

- Ticket: `RIOTBOX-1380`
- Title: `P023: Carry review artifact refs into source-family readiness actions`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1380/p023-carry-review-artifact-refs-into-source-family-readiness-actions`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1380-p023-carry-review-artifact-refs-into-source-family-readiness`
- Linear branch: `feature/riotbox-1380-p023-carry-review-artifact-refs-into-source-family-readiness`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1344 (https://github.com/marang/riotbox/pull/1344)`
- Merge commit: `8ac5ec415ab937ef31b1bc17e057f55f15382e80`
- Deleted from Linear: `2026-07-02`
- Verification: `python3 -m py_compile scripts/generate_sound_quality_readiness_report.py scripts/sound_quality_readiness_human_review.py; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1380-readiness; python3 scripts/generate_sound_quality_readiness_report.py --validate-report artifacts/audio_qa/local-riotbox-1380-readiness/sound-quality-readiness-report.json; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

The readiness report named concrete source-family review candidates but dropped rendered WAV, metrics, and prompt artifact refs from the underlying review queue.

## What Shipped

- Carried rendered_wav, metrics, and review_prompt refs into readiness queue summaries and matching source-family actions, with validation/mutation coverage.

## Notes

- None
