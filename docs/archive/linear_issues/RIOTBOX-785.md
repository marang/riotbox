# `RIOTBOX-785` Add app observer Source Timing fixture for locked-grid summary path

- Ticket: `RIOTBOX-785`
- Title: `Add app observer Source Timing fixture for locked-grid summary path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-785/add-app-observer-source-timing-fixture-for-locked-grid-summary-path`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-785-app-observer-locked-grid-fixture`
- Linear branch: `feature/riotbox-785-add-app-observer-source-timing-fixture-for-locked-grid`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#780 (https://github.com/marang/riotbox/pull/780)`
- Merge commit: `8d0ee7f25b1f587fb887eca286b56e0abae7745a`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-785-user-session-fixtures.log just user-session-observer-validator-fixtures`; `scripts/run_compact.sh /tmp/riotbox-785-locked-probe-test.log cargo test -p riotbox-app feral_grid_jam_locked_observer_probe_surfaces_locked_source_timing -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-785-fmt.log cargo fmt --check`; `git diff --check`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P012 needed committed app observer proof that the shared Jam Source Timing summary can expose a locked-grid timing snapshot with grid/bar/phrase detail, anchor evidence, and groove evidence.

## What Shipped

- Added a valid locked-grid user-session observer NDJSON fixture and wired it into the observer validator fixture recipe.

## Notes

- No TUI, generated audio, or runtime timing policy behavior changed.
