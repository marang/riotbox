# `RIOTBOX-220` Align footer Scene jump availability wording

- Ticket: `RIOTBOX-220`
- Title: `Align footer Scene jump availability wording`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-220/align-footer-scene-jump-availability-wording`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-220-footer-scene-jump-availability`
- Linear branch: `feature/riotbox-220-align-footer-scene-jump-availability-wording`
- PR: `#210`
- Merge commit: `c0a275d`
- Labels: `ux`
- Follow-ups: `RIOTBOX-221`

## Why This Ticket Existed

`RIOTBOX-218` clarified the suggested gesture when no queueable Scene jump exists, but the footer still advertised `y scene jump` unconditionally. The footer needed to stop contradicting the primary gesture cue.

## What Shipped

- Rendered footer primary gesture as `y jump waits` when no queueable Scene jump exists yet.
- Kept normal `y scene jump` wording when a next Scene target is available or unknown.
- Extended the single-Scene Jam regression to cover footer wording.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app renders_jam_shell_with_single_scene_jump_waiting_cue`
- `cargo test -p riotbox-app footer_line_styles_define_first_visual_hierarchy`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Footer wording slice only; no Scene selection policy, audio behavior, or broad footer redesign changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
