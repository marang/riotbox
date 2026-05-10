# `RIOTBOX-268` Tighten Capture audition next-step cues

- Ticket: `RIOTBOX-268`
- Title: `Tighten Capture audition next-step cues`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-268/tighten-capture-audition-next-step-cues`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-268-tighten-capture-audition-next-step-cues`
- Linear branch: `feature/riotbox-268-tighten-capture-audition-next-step-cues`
- PR: `#258`
- Merge commit: `bd9d509`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-269`

## Why This Ticket Existed

First-use feedback said capture still felt partly blind unless the user could immediately tell how to hear captured material. Riotbox already had raw audition and promoted audition paths, but the Capture surface did not make the next audible step prominent enough.

## What Shipped

- Added explicit Capture `Do Next` guidance for queued raw and promoted W-30 auditions.
- Reworded fallback Capture steps toward audible intent: `hear it`, `keep it`, and `play it`.
- Clarified pending capture and promote handoffs so the next audible step is visible before switching to Log.
- Extended focused Capture snapshot assertions for raw audition, promoted audition, capture, and promotion cues.

## Verification

- `cargo test -p riotbox-app renders_capture_do_next_with_pending_capture_state -- --nocapture`
- `cargo test -p riotbox-app renders_capture_shell_snapshot_with_capture_context -- --nocapture`
- `cargo test -p riotbox-app renders_capture_shell_snapshot_with_raw_capture_audition_cue -- --nocapture`
- `cargo test -p riotbox-app renders_capture_shell_snapshot_with_w30_audition_cue -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- UI guidance slice only; no audio behavior, W-30 sampler redesign, keymap changes, or broad Capture layout changes shipped.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.
