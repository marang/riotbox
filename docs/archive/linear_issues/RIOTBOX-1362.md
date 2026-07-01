# `RIOTBOX-1362` P023: Strengthen W-30 hook response signature

- Ticket: `RIOTBOX-1362`
- Title: `P023: Strengthen W-30 hook response signature`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1362/p023-strengthen-w-30-hook-response-signature`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1362-p023-strengthen-w30-hook-response-signature`
- Linear branch: `feature/riotbox-1362-p023-strengthen-w-30-hook-response-signature`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1326 (https://github.com/marang/riotbox/pull/1326)`
- Merge commit: `64892a0407d06d77a60ea0071dd9819786c194c6`
- Deleted from Linear: `2026-07-01`
- Verification: `py_compile; pro-pressure-source-matrix smoke; professional-source-wav-pack smoke; professional-output-suite smoke; sound-quality-readiness-report smoke; just audio-qa-ci; just ci; GitHub rust-ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Weak-output routing still ranked chop_policy as top production fix after source-offset diversity improved; the remaining gap was transformed hook response signature rather than more W-30 hit count.

## What Shipped

- Added rendered hook/chop response delta/correlation/transient proof across Matrix, Source-WAV, suite, readiness, and Justfile gates; added source-derived shifted/reversed transient response material; trimmed dense W-30 hook layer so snare pressure remains in front; documented RBX-113.

## Notes

- None
