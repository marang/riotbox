# `RIOTBOX-977` Make Source Map trust row confirmation-aware

- Ticket: `RIOTBOX-977`
- Title: `Make Source Map trust row confirmation-aware`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-977/make-source-map-trust-row-confirmation-aware`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-977-make-source-map-trust-row-confirmation-aware`
- Linear branch: `feature/riotbox-977-make-source-map-trust-row-confirmation-aware`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `ux`
- PR: `#968 (https://github.com/marang/riotbox/pull/968)`
- Merge commit: `ec1eeaa744ec38ad6707566ece372834bbdbd59e`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-977-rebased-source-map.log cargo test -p riotbox-core source_map -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-977-rebased-source-shell.log cargo test -p riotbox-app shell_state_source -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-977-rebased-ci.log just ci`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/source_timing_intelligence_spec.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Show musician-confirmed source timing trust directly in the Source Map without changing analyzer confidence or Source Graph evidence.

## What Shipped

- Derived Source Map grid trust from matching runtime_state.source_timing.confirmed_grid.
- Rendered grid confirmed and allowed bar-grid Source Map mode for matching confirmed grids.
- Kept mismatched or absent confirmation in the existing fallback/needs-confirm path with core and Source render coverage.

## Notes

- Rebased onto current main after RIOTBOX-976 merged, retargeted PR #968 to main, then reran local focused tests and just ci.
