# `RIOTBOX-230` Couple Scene launch target to TR-909 source-support context

- Ticket: `RIOTBOX-230`
- Title: `Couple Scene launch target to TR-909 source-support context`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-230/couple-scene-launch-target-to-tr-909-source-support-context`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-230-scene-target-audio-coupling`
- Linear branch: `feature/riotbox-230-couple-scene-launch-target-to-tr-909-source-support-context`
- PR: `#220`
- Merge commit: `efcdd9e`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-231`

## Why This Ticket Existed

`RIOTBOX-229` documented that Scene launch state and Log truth were real, but TR-909 `SourceSupport` profile selection still followed the transport-bar Source Graph section. Scene Brain needed the selected target Scene to become audible through the existing support-profile seam.

## What Shipped

- Added Scene-context-aware TR-909 render policy projection.
- Projected `scene-NN-label` ids into sorted Source Graph sections for `SourceSupport` profile selection.
- Preserved the old transport-bar fallback for legacy, unmapped, or out-of-range Scene ids.
- Wired the Jam app render projection to use active/current Scene context.
- Added focused core and app regression coverage.
- Updated the Scene launch audio-coupling review with follow-up status.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-core source_support_profile`
- `cargo test -p riotbox-app committed_scene_select_projects_target_scene_into_tr909_source_support`
- `cargo test -p riotbox-app source_support_render_profile_tracks_current_source_section`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Audio-coupling policy slice only; no new DSP voice, broad audio redesign, persistence model, or second arranger path changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
