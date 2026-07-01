# `RIOTBOX-1363` P023: Strengthen source-selection promotion and demotion evidence

- Ticket: `RIOTBOX-1363`
- Title: `P023: Strengthen source-selection promotion and demotion evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1363/p023-strengthen-source-selection-promotion-and-demotion-evidence`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1363-p023-strengthen-source-selection-promotion-and-demotion`
- Linear branch: `feature/riotbox-1363-p023-strengthen-source-selection-promotion-and-demotion`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1327 (https://github.com/marang/riotbox/pull/1327)`
- Merge commit: `13bd4161f0c2ca8046ce5d825200c3e75a11216a`
- Deleted from Linear: `2026-07-01`
- Verification: `python -m py_compile touched scripts; edge-source diagnostics smoke; professional-output-suite smoke; sound-quality-readiness-report smoke; just audio-qa-ci; just ci; GitHub rust-ci pass on PR #1327.`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `Continue P023 source-selection work toward trusted source-derived promotion only after timing confidence, texture suitability, human verdict, and audible output support it.`

## Why This Ticket Existed

P023 source-selection risk needed actionable promotion/demotion evidence so bad-timing and pad/noise edge material stays blocked from demo promotion for concrete reasons instead of a vague source-selection flag.

## What Shipped

- Added per-case source-selection demotion reasons and required review actions to edge-source diagnostics, lifted reason counts/actions through professional-suite and sound-quality readiness, and gated those fields in validators while keeping edge evidence diagnostic-only and quality_proof false.

## Notes

- None
