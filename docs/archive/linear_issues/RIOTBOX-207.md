# `RIOTBOX-207` Surface Scene Brain restore energy direction in ready cue

- Ticket: `RIOTBOX-207`
- Title: `Surface Scene Brain restore energy direction in ready cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-207/surface-scene-brain-restore-energy-direction-in-ready-cue`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-207-restore-energy-direction`
- Linear branch: `feature/riotbox-207-surface-scene-brain-restore-energy-direction-in-ready-cue`
- PR: `#197`
- Merge commit: `2617157`
- Labels: `ux`
- Follow-ups: `RIOTBOX-208`

## Why This Ticket Existed

The Jam screen could show `restore <scene>/<energy> ready`, but users still had to infer whether pressing `Y` would move energy upward, downward, or sideways. The UI already had energy-delta helpers, so the direction could be surfaced without changing Scene Brain runtime policy.

## What Shipped

- Added compact `rise/drop/hold` wording to the restore-ready Jam cue when current and restore energy are both known.
- Preserved the previous restore-ready fallback wording when energy data is incomplete.
- Updated focused Jam and help-overlay regressions for the new `rise` cue.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app renders_jam_shell_with_restore_ready_cue`
- `cargo test -p riotbox-app renders_help_overlay_with_restore_ready_cue`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- UI-readability only; no model, queue, persistence, runtime scene policy, or audio behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
