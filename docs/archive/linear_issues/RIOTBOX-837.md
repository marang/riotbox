# `RIOTBOX-837` Surface downbeat offset in Jam/Source timing summary

- Ticket: `RIOTBOX-837`
- Title: `Surface downbeat offset in Jam/Source timing summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-837/surface-downbeat-offset-in-jamsource-timing-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-837-source-timing-downbeat-offset-ui`
- Linear branch: `feature/riotbox-837-surface-downbeat-offset-in-jamsource-timing-summary`
- Assignee: `Markus`
- Labels: `ux`
- PR: `#832 (https://github.com/marang/riotbox/pull/832)`
- Merge commit: `d1c94f2052952aa498c454eeb49dc02cf02fd8bf`
- Verification: `cargo fmt --check`; `cargo test -p riotbox-core source_timing_summary -- --nocapture`; `cargo test -p riotbox-app renders_source_shell_snapshot -- --nocapture`; `cargo test -p riotbox-app user_session_observer_probe -- --nocapture`; `just ci`; GitHub Actions Rust CI #2035.
- Docs touched: `docs/specs/tui_screen_spec.md`
- Follow-ups: `RIOTBOX-838`

## Why This Ticket Existed

P012 downbeat-offset evidence was available in reports and observer/audio QA, but not on the musician-facing Jam/Source timing surfaces.

## What Shipped

- Added primary_downbeat_offset_beats to the shared Jam Source Timing summary, surfaced it on the Source timing panel, and made observer snapshots read the shared summary value.

## Notes

- None
