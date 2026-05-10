# `RIOTBOX-276` Project latest W-30 promoted capture label into Jam view

- Ticket: `RIOTBOX-276`
- Title: `Project latest W-30 promoted capture label into Jam view`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-276/project-latest-w-30-promoted-capture-label-into-jam-view`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-276-project-latest-w-30-promoted-capture-label-into-jam-view`
- Linear branch: `feature/riotbox-276-project-latest-w-30-promoted-capture-label-into-jam-view`
- PR: `#266`
- Merge commit: `47a9946`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-277`

## Why This Ticket Existed

`capture_routing_lines` still derived the compact `latest promoted` label by scanning `session.captures` in the TUI. That label is part of the W-30/Capture handoff surface and should follow the same projection pattern as typed target kind, pending audition intent, and handoff readiness.

## What Shipped

- Added `latest_w30_promoted_capture_label` to `CaptureSummaryView`.
- Updated Capture routing to render the `latest promoted` cue from the Jam view projection.
- Tightened focused regression coverage for the projected label and rendered cue.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-core builds_minimal_jam_view_model`
- `cargo test -p riotbox-app renders_capture_shell_snapshot_with_w30_audition_cue`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- UI projection slice only; no audio behavior, persistence, or Capture layout behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
