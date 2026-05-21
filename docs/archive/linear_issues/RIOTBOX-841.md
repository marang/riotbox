# `RIOTBOX-841` Show Source Timing downbeat phase in Jam compact timing lines

- Ticket: `RIOTBOX-841`
- Title: `Show Source Timing downbeat phase in Jam compact timing lines`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-841/show-source-timing-downbeat-phase-in-jam-compact-timing-lines`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-841-jam-source-timing-phase-line`
- Linear branch: `feature/riotbox-841-show-source-timing-downbeat-phase-in-jam-compact-timing`
- Assignee: `Markus`
- Labels: `ux`
- PR: `#836 (https://github.com/marang/riotbox/pull/836)`
- Merge commit: `9713d08c82bc7dcb6ee41c6d0faacb29e691aed1`
- Verification: `cargo fmt --check; cargo test -p riotbox-app shell_state_jam_snapshot -- --nocapture; cargo test -p riotbox-app shell_state_inspect_snapshot -- --nocapture; cargo test -p riotbox-app renders_help_overlay -- --nocapture; cargo test -p riotbox-app renders_start_here_help_steps_for_first_run -- --nocapture; just ci; GitHub Rust CI #2047`
- Docs touched: `docs/specs/tui_screen_spec.md; docs/jam_recipes.md`
- Follow-ups: `None`

## Why This Ticket Existed

Jam compact timing showed grid trust and anchors but not the selected source downbeat phase, forcing musicians to open Source or Inspect to understand bar-phase alignment.

## What Shipped

- Rendered the shared Source Timing primary downbeat offset as compact Jam phase chips (p0/p-) and fuller Help/Start Here phase text, with snapshot tests plus Jam recipe and TUI spec updates.

## Notes

- None
