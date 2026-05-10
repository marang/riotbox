# `RIOTBOX-252` Style Scene pending intent line on Jam

- Ticket: `RIOTBOX-252`
- Title: `Style Scene pending intent line on Jam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-252/style-scene-pending-intent-line-on-jam`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-252-style-scene-pending-intent-line-on-jam`
- Linear branch: `feature/riotbox-252-style-scene-pending-intent-line-on-jam`
- PR: `#242`
- Merge commit: `059e4ff`
- Labels: `ux`
- Follow-ups: `RIOTBOX-253`

## Why This Ticket Existed

After the post-commit cue and timing rail gained visual hierarchy, the Scene pending intent line still read as flat text such as `launch -> scene-02-drop @ next bar | energy rise`. This line is one of the first places a player checks after pressing `y` or `Y`.

## What Shipped

- Rendered the existing Scene pending line as styled spans when a Scene transition is pending.
- Emphasized pending verb, target scene, boundary, and energy direction with the current semantic color system.
- Preserved the visible monochrome text.
- Added focused UI style coverage while keeping existing pending Scene snapshots passing.

## Verification

- `git diff --check main...HEAD`
- `cargo fmt --check`
- `cargo test -p riotbox-app scene_pending_line -- --nocapture`
- `cargo test -p riotbox-app pending_scene -- --nocapture`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI styling slice only; no Scene selection policy, scheduler behavior, quantization behavior, broad Jam layout redesign, or audio behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
