# `RIOTBOX-236` Add bounded Scene-target TR-909 support accent

- Ticket: `RIOTBOX-236`
- Title: `Add bounded Scene-target TR-909 support accent`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-236/add-bounded-scene-target-tr-909-support-accent`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-236-scene-target-tr909-accent`
- Linear branch: `feature/riotbox-236-add-bounded-scene-target-tr-909-support-accent`
- PR: `#226`
- Merge commit: `11f8147`
- Labels: `benchmark`, `review-followup`
- Follow-ups: `RIOTBOX-237`

## Why This Ticket Existed

Scene Brain carried `scene_target` / `transport_bar` through app diagnostics and the realtime audio seam. The next smallest audible step was to let `scene_target` produce a subtle support accent so a landed Scene target can be felt musically without creating a separate transition engine.

## What Shipped

- Added a bounded `scene_target` context boost to TR-909 source-support trigger envelope.
- Added a bounded `scene_target` context gain multiplier on the existing drum-bus support path.
- Kept `transport_bar` fallback behavior unchanged.
- Added a focused audio regression comparing same-profile `scene_target` and `transport_bar` renders.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-audio scene_target_context_adds_bounded_support_accent -- --nocapture`
- `cargo test -p riotbox-audio fixture_backed_tr909_audio_regressions_hold -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Audio-producing slice; verified through current unit/buffer regression coverage.
- No formal listening pack or real-session manual listening gate was run for this slice.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
