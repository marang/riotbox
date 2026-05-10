# `RIOTBOX-203` Add a source-window formatter unit test

- Ticket: `RIOTBOX-203`
- Title: `Add a source-window formatter unit test`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-203/add-a-source-window-formatter-unit-test`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-203-source-window-formatter-test`
- Linear branch: `feature/riotbox-203-add-a-source-window-formatter-unit-test`
- PR: `#193`
- Merge commit: `b33d386`
- Labels: `Audio`, `benchmark`, `ux`
- Follow-ups: `RIOTBOX-204`

## Why This Ticket Existed

`RIOTBOX-202` centralized TUI source-window formatting behind small helpers, but the intended compact shapes were still only indirectly covered by larger shell snapshots.

## What Shipped

- Added direct unit coverage for the source-window formatter helper output.
- Locked the three current compact shapes: span-only, Capture provenance, and W-30 Log.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app source_window_formatters_keep_surface_shapes_stable`
- `git diff --check main..HEAD`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Unit-test only; no runtime, model, or audio behavior changed.
- This makes formatter intent explicit without relying only on broader snapshot failures.
