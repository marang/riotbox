# `RIOTBOX-836` Assert downbeat-offset compatibility in generated Feral-grid observer/audio gate

- Ticket: `RIOTBOX-836`
- Title: `Assert downbeat-offset compatibility in generated Feral-grid observer/audio gate`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-836/assert-downbeat-offset-compatibility-in-generated-feral-grid`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-836-generated-feral-grid-offset-compat`
- Linear branch: `feature/riotbox-836-assert-downbeat-offset-compatibility-in-generated-feral-grid`
- Assignee: `Markus`
- Labels: `benchmark`
- PR: `#831 (https://github.com/marang/riotbox/pull/831)`
- Merge commit: `488e4bb3ca9cd5d1154a0228a7f1a6a5e7919959`
- Verification: `just observer-audio-correlate-generated-feral-grid`; `just ci`; `git diff --check`; GitHub Actions Rust CI #2032
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-834 added downbeat-offset compatibility to observer/audio summaries, but the generated Feral-grid observer/audio gate still only asserted grid-use compatibility across fallback, cautious/manual-confirm, user-override, and locked-grid paths.

## What Shipped

- Updated the generated Feral-grid observer/audio gate to assert observer and manifest downbeat offsets plus `downbeat_offset_compatibility` for cautious/manual-confirm, user-override, fallback, and locked-grid cases, and documented the generated-gate contract in the Source Timing spec.

## Notes

- None
