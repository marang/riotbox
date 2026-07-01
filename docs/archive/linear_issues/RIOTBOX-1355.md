# `RIOTBOX-1355` P023: Strengthen source-selection promotion blockers into source-window fixes

- Ticket: `RIOTBOX-1355`
- Title: `P023: Strengthen source-selection promotion blockers into source-window fixes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1355/p023-strengthen-source-selection-promotion-blockers-into-source-window`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1355-p023-strengthen-source-selection-promotion-blockers-into`
- Linear branch: `feature/riotbox-1355-p023-strengthen-source-selection-promotion-blockers-into`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1319 (https://github.com/marang/riotbox/pull/1319)`
- Merge commit: `9a02a3d7514dcb89fa4e9f64b0b351341337853b`
- Deleted from Linear: `2026-07-01`
- Verification: `cargo fmt --check: pass`; `python -m py_compile changed P023 scripts: pass`; `cargo test -p riotbox-audio --bin feral_grid_pack source_character_window_selection -- --nocapture: pass`; `just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1355-dense-pack: pass`; `just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1355-source-wav-pack: pass`; `just edge-source-professional-diagnostics-smoke artifacts/audio_qa/local-riotbox-1355-edge-source: pass`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1355-professional-output-suite-pass: pass`; `just sound-quality-readiness-report-smoke: pass`; `just audio-qa-ci: pass`; `just ci: pass`; `GitHub rust-ci on PR #1319: pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`, `docs/execution_roadmap.md`, `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Source-selection blockers were still diagnostic/reporting artifacts: the professional suite could report source-window selection while scanning only one candidate and promoting nothing.

## What Shipped

- Feral grid source-character window selection now searches available source audio and reports search duration, requested/selected RMS, retention ratio, and score lift.
- Professional-suite and readiness gates now require real searched source-window cases, at least one promotion, and RMS retention for promoted windows.
- Dense/sparse/pad diagnostics can request short source-character windows for active selection, while tonal-hook and bad-timing review cases keep longer context when that preserves the family contract.

## Notes

- Latest professional-suite/readiness summary: 8 searched cases, 7 promoted cases, minimum observed RMS retention 1.0, max score lift 0.035956576.
