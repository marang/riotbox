# `RIOTBOX-1255` P023: Strengthen mix-bus clarity from weak-output candidates

- Ticket: `RIOTBOX-1255`
- Title: `P023: Strengthen mix-bus clarity from weak-output candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1255/p023-strengthen-mix-bus-clarity-from-weak-output-candidates`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1255-p023-mix-bus-clarity`
- Linear branch: `feature/riotbox-1255-p023-strengthen-mix-bus-clarity-from-weak-output-candidates`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1229 (https://github.com/marang/riotbox/pull/1229)`
- Merge commit: `13bdcc790ae52c66cecb74330ee43282eb72298d`
- Deleted from Linear: `2026-06-13`
- Verification: `py_compile`, `cargo fmt --check`, `cargo test -p riotbox-audio --bin feral_grid_pack -- --nocapture`, `professional-output-suite-smoke`, `sound-quality-readiness-report-smoke`, `just audio-qa-ci`, `just ci`, GitHub `rust-ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Weak-output routing still flagged mix_bus because generated support could mask source character or blur the strongest audible element even when output existed.

## What Shipped

- Tightened feral grid source-first and generated-support mix-balance gates, made the professional suite contract reject stale loose thresholds, added readiness max-support masking validation, added a negative suite fixture, and documented the diagnostic boundary.

## Notes

- None
