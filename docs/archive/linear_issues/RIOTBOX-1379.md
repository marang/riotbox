# `RIOTBOX-1379` P023: Make source-family readiness actions point at concrete review candidates

- Ticket: `RIOTBOX-1379`
- Title: `P023: Make source-family readiness actions point at concrete review candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1379/p023-make-source-family-readiness-actions-point-at-concrete-review`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-02`
- Started: `2026-07-02`
- Finished: `2026-07-02`
- Branch: `feature/riotbox-1379-p023-make-source-family-readiness-actions-point-at-concrete`
- Linear branch: `feature/riotbox-1379-p023-make-source-family-readiness-actions-point-at-concrete`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1343 (https://github.com/marang/riotbox/pull/1343)`
- Merge commit: `fd1aa206a3c0022e09615163aa57226c816b8ff9`
- Deleted from Linear: `2026-07-02`
- Verification: `python3 -m py_compile scripts/generate_sound_quality_readiness_report.py; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1379-readiness; python3 scripts/generate_sound_quality_readiness_report.py --validate-report artifacts/audio_qa/local-riotbox-1379-readiness/sound-quality-readiness-report.json; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

After readiness prioritized source-family review, its main source-selection actions were still generic even when the embedded human-review queue knew concrete candidate ids and review reasons.

## What Shipped

- Linked missing source-family readiness actions to matching review queue candidates with candidate id, priority, demo-worthy reason, not-demo-ready reason, required verdict state, and validation/mutation coverage.

## Notes

- None
