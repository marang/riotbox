# `RIOTBOX-217` Show next Scene energy in Jam overview

- Ticket: `RIOTBOX-217`
- Title: `Show next Scene energy in Jam overview`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-217/show-next-scene-energy-in-jam-overview`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-217-next-scene-energy-overview`
- Linear branch: `feature/riotbox-217-show-next-scene-energy-in-jam-overview`
- PR: `#207`
- Merge commit: `d10b7ee`
- Labels: `ux`
- Follow-ups: `RIOTBOX-218`

## Why This Ticket Existed

`RIOTBOX-216` projected the next Scene launch target and inferred energy through the Jam view model. The Jam overview still needed to present that data as a musical target instead of only exposing raw scene ids.

## What Shipped

- Rendered Jam overview next Scene as a compact target such as `drop/high`.
- Preserved `none` when no next Scene target is projected.
- Updated scene UI regressions for the compact target/energy cue.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app renders_more_musical_jam_shell_snapshot`
- `cargo test -p riotbox-app renders_jam_shell_with_scene_brain_summary`
- `cargo test -p riotbox-app scene_fixture_backed_shell_regressions_hold`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI readability slice only; no Scene selection policy, audio behavior, persistence model, or broad Jam redesign changed.
- The fast-forward merge means the feature branch tip is also the merge tip on `main`.
