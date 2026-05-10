# `RIOTBOX-246` Surface compact TR-909 lift in Scene post-commit guidance

- Ticket: `RIOTBOX-246`
- Title: `Surface compact TR-909 lift in Scene post-commit guidance`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-246/surface-compact-tr-909-lift-in-scene-post-commit-guidance`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-246-scene-postcommit-909-lift`
- Linear branch: `feature/riotbox-246-surface-compact-tr-909-lift-in-scene-post-commit-guidance`
- PR: `#236`
- Merge commit: `bcbde72`
- Labels: `ux`
- Follow-ups: `RIOTBOX-247`

## Why This Ticket Existed

Scene launch and restore can project a Scene target into TR-909 source support and expose `accent scene` in Log/Inspect. The default Jam surface still made the user go to Log to notice that subtle support lift, so the post-commit cue needed a compact musical hint without turning Jam back into diagnostics.

## What Shipped

- Added a conditional `909 lift` suffix to Scene post-commit guidance when TR-909 support accent is `scene`.
- Kept the cue absent when no Scene-driven TR-909 accent is present.
- Extended Scene jump and restore Jam regressions to cover the compact lift cue.
- Added a negative regression for the no-accent fallback path.

## Verification

- `cargo fmt --check`
- `cargo test -p riotbox-app renders_scene_jump_post_commit_guidance -- --nocapture`
- `cargo test -p riotbox-app renders_scene_restore_post_commit_guidance -- --nocapture`
- `cargo test -p riotbox-app omits_scene_post_commit_tr909_lift_without_scene_accent -- --nocapture`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- UI wording/regression slice only; no audio behavior, Scene selection policy, transition engine, or broad Jam layout changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
