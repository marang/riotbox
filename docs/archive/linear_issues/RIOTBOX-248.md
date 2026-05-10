# `RIOTBOX-248` Style Scene post-commit Jam cue hierarchy

- Ticket: `RIOTBOX-248`
- Title: `Style Scene post-commit Jam cue hierarchy`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-248/style-scene-post-commit-jam-cue-hierarchy`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-248-style-scene-post-commit-jam-cue-hierarchy`
- Linear branch: `feature/riotbox-248-style-scene-post-commit-jam-cue-hierarchy`
- PR: `#238`
- Merge commit: `05eb3b9`
- Labels: `ux`
- Follow-ups: `RIOTBOX-249`

## Why This Ticket Existed

`RIOTBOX-246` and `RIOTBOX-247` made the compact `909 lift` post-commit cue visible and documented, but the Jam post-commit line still read as flat text. The TUI feedback and spec both call for stronger semantic hierarchy so the player can distinguish live scene, restore target, support lift, and next action without parsing everything as equal diagnostics.

## What Shipped

- Rendered the Scene post-commit Jam cue as styled spans instead of one flat string.
- Emphasized the current Scene result, restore target, compact `909 lift` hint, and next action keys with the current semantic color system.
- Preserved the existing visible snapshot text.
- Added focused style coverage for the post-commit cue hierarchy.

## Verification

- `cargo fmt --check`
- `cargo test -p riotbox-app post_commit -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI styling slice only; no runtime behavior, audio behavior, new keybindings, or broad Jam layout redesign changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
