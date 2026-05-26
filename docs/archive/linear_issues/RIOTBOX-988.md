# `RIOTBOX-988` Verify source transport capture state replay and restore fidelity

- Ticket: `RIOTBOX-988`
- Title: `Verify source transport capture state replay and restore fidelity`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-988/verify-source-transport-capture-state-replay-and-restore-fidelity`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-988-verify-source-transport-capture-state-replay-and-restore-fidelity`
- Linear branch: `feature/riotbox-988-verify-source-transport-capture-state-replay-and-restore`
- Assignee: `Markus`
- Labels: `Feature`, `timing`, `ux`
- PR: `#980 (https://github.com/marang/riotbox/pull/980)`
- Merge commit: `7ba7ce1d180128c92ff504bb95a49189364de1a8`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-988-rebased-app-restore.log cargo test -p riotbox-app source_transport_capture_projection_survives_save_restore_with_confirmed_grid -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-988-rebased-core-replay.log cargo test -p riotbox-core plan_executor_replays_source_transport_capture_state_for_restore_projection -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-988-rebased-ci.log just ci`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Prove source transport, monitor, confirmed grid, and capture projection restore/replay deterministically without mutating analyzer evidence.

## What Shipped

- Added app restore coverage for confirmed and unconfirmed manual-confirm source timing paths.
- Added core replay coverage for transport.play, transport.seek, source_monitor.set_mode, source_timing.confirm_grid, and capture.set_length.
- Split tests into semantic source transport modules to keep review cost below file-size soft budgets.

## Notes

- Rebased onto current main after RIOTBOX-987 merged, retargeted PR #980 to main, then reran focused restore/replay tests and just ci.
