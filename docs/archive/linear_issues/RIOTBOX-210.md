# `RIOTBOX-210` Add restore energy direction to suggested gesture cue

- Ticket: `RIOTBOX-210`
- Title: `Add restore energy direction to suggested gesture cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-210/add-restore-energy-direction-to-suggested-gesture-cue`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-210-restore-direction-suggested-gesture`
- Linear branch: `feature/riotbox-210-add-restore-energy-direction-to-suggested-gesture-cue`
- PR: `#200`
- Merge commit: `396d81b`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-211`

## Why This Ticket Existed

The periodic Scene Brain TUI seam review found that footer/help showed restore energy direction, but the primary suggested gesture still only said `[Y] restore <scene> now`. The highest-priority Jam action prompt needed the same musical intent as the explanatory surfaces.

## What Shipped

- Added `rise/drop/hold` direction to the primary suggested restore gesture when known.
- Preserved the target-only fallback when restore direction is unavailable.
- Updated focused Jam restore-ready regression coverage.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app renders_jam_shell_with_restore_ready_cue`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- UI-readability follow-up only; no runtime Scene Brain policy, model, persistence, or audio behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
