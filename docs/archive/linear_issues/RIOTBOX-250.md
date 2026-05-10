# `RIOTBOX-250` Style queued timing rail hierarchy on Jam

- Ticket: `RIOTBOX-250`
- Title: `Style queued timing rail hierarchy on Jam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-250/style-queued-timing-rail-hierarchy-on-jam`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-250-style-queued-timing-rail-hierarchy-on-jam`
- Linear branch: `feature/riotbox-250-style-queued-timing-rail-hierarchy-on-jam`
- PR: `#240`
- Merge commit: `2ca872f`
- Labels: `ux`
- Follow-ups: `RIOTBOX-251`

## Why This Ticket Existed

The Jam screen already had a compact timing rail such as `wait [===>] next bar` or `pulse [===>]`, but it still competed as flat text inside the Next panel. User feedback repeatedly called out that the right musical moment and quantized snap point are hard to see.

## What Shipped

- Rendered the existing queued timing rail as styled spans instead of a flat string.
- Emphasized countdown rails and boundary labels with the current semantic color system.
- Kept beat, bar, and phrase counters as lower-emphasis context.
- Added focused style regressions for generic waits and Scene pulse rails.

## Verification

- `git diff --check main...HEAD`
- `cargo fmt --check`
- `cargo test -p riotbox-app queued_timing_rail -- --nocapture`
- `cargo test -p riotbox-app queued_scene_timing_rail -- --nocapture`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI styling slice only; no scheduler behavior, quantization behavior, new timing widget, broad Jam layout redesign, or audio behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
