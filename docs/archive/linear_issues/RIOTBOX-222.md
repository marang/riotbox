# `RIOTBOX-222` Clarify missing next Scene in overview

- Ticket: `RIOTBOX-222`
- Title: `Clarify missing next Scene in overview`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-222/clarify-missing-next-scene-in-overview`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-222-overview-next-scene-waits`
- Linear branch: `feature/riotbox-222-clarify-missing-next-scene-in-overview`
- PR: `#212`
- Merge commit: `e9d0eb3`
- Labels: `ux`
- Follow-ups: `RIOTBOX-223`

## Why This Ticket Existed

When no queueable Scene jump exists because there is too little scene material, Suggested Gestures, Footer, and Help already explained that the jump waits. The Jam overview still said only `next scene none`, which was technically correct but less useful.

## What Shipped

- Rendered the Jam overview as `next scene waits for 2 scenes` for the known too-few-scenes case.
- Preserved compact `scene/energy` labels for queueable Scene jumps and the generic `none` fallback for unknown cases.
- Extended the single-Scene Jam regression so Overview, Suggested, Footer, and Help stay aligned.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app renders_jam_shell_with_single_scene_jump_waiting_cue`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI wording slice only; no Scene selection policy, source analysis, audio behavior, or broad Jam redesign changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
