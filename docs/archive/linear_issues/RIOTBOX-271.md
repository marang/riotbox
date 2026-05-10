# `RIOTBOX-271` Project W-30 pending audition intent into Jam view

- Ticket: `RIOTBOX-271`
- Title: `Project W-30 pending audition intent into Jam view`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-271/project-w-30-pending-audition-intent-into-jam-view`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-271-project-w-30-pending-audition-intent-into-jam-view`
- Linear branch: `feature/riotbox-271-project-w-30-pending-audition-intent-into-jam-view`
- PR: `#261`
- Merge commit: `1f2af3c`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-272`

## Why This Ticket Existed

The W-30 Capture seam review found that Capture `Do Next` reconstructed raw-vs-promoted pending audition intent by scanning generic pending actions and matching command strings. The perform-facing UI needed kind, target, and quantization as explicit Jam view data.

## What Shipped

- Added typed `W30PendingAuditionView` projection to the Jam lane summary.
- Updated Capture `Do Next` and compact pending W-30 cue rendering to consume the typed projection.
- Added focused test assertions for raw and promoted pending audition kind, target, and quantization.
- Recorded the view-contract decision in `docs/research_decision_log.md`.

## Verification

- `cargo fmt --check`
- `cargo test -p riotbox-core`
- `cargo test -p riotbox-app`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- View-model/UI seam slice only; no action system, keymap, audio behavior, or Capture layout redesign changed.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.
