# `RIOTBOX-257` Promote Jam timing rail above landed history

- Ticket: `RIOTBOX-257`
- Title: `Promote Jam timing rail above landed history`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-257/promote-jam-timing-rail-above-landed-history`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-257-promote-jam-timing-rail-above-landed-history`
- Linear branch: `feature/riotbox-257-promote-jam-timing-rail-above-landed-history`
- PR: `#247`
- Merge commit: `e1e11c5`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-258`

## Why This Ticket Existed

`RIOTBOX-256` found that the primary Jam `Next` panel still rendered latest landed history before the queued timing rail. That weakened the intended queue -> timing -> landed hierarchy because the timing rail is the imminent musical event while the landed line is history.

## What Shipped

- Moved the primary Jam `Next` panel timing rail above latest landed history when a rail is present.
- Preserved the no-queue fallback order so landed history still appears before status text.
- Added a focused regression for the `Next` panel line order.

## Verification

- `cargo fmt --check`
- `cargo test -p riotbox-app next_panel_promotes_timing_rail_above_landed_history -- --nocapture`
- `cargo test -p riotbox-app renders_more_musical_jam_shell_snapshot -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI hierarchy slice only; no scheduler, quantization, audio output, new timing widget, or broad Jam layout changed.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.
