# `RIOTBOX-235` Carry TR-909 support context through realtime audio seam

- Ticket: `RIOTBOX-235`
- Title: `Carry TR-909 support context through realtime audio seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-235/carry-tr-909-support-context-through-realtime-audio-seam`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-235-tr909-context-realtime-seam`
- Linear branch: `feature/riotbox-235-carry-tr-909-support-context-through-realtime-audio-seam`
- PR: `#225`
- Merge commit: `a4c6f7f`
- Labels: `benchmark`, `review-followup`
- Follow-ups: `RIOTBOX-236`

## Why This Ticket Existed

The app/TUI layer carried TR-909 `scene_target` / `transport_bar` context, but the private realtime audio render state dropped it at the shared atomic seam. The callback did not need new sound behavior yet, but future audio QA and diagnostics need proof that the typed support context reached the audio side.

## What Shipped

- Added `source_support_context` to the private realtime TR-909 render state.
- Carried the context through `SharedTr909RenderState` update/snapshot with atomic enum mapping.
- Extended focused runtime tests for unset, `scene_target`, and `transport_bar` context.
- Extended the TR-909 audio regression fixture schema and data with context labels.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-audio shared_render_state_tracks_updates -- --nocapture`
- `cargo test -p riotbox-audio fixture_backed_tr909_audio_regressions_hold -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Realtime seam slice only; no audible drum behavior, TUI wording, Scene selection policy, or listening-pack gate changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
