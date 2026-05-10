# `RIOTBOX-277` Use Jam view capture count for first-run onramp

- Ticket: `RIOTBOX-277`
- Title: `Use Jam view capture count for first-run onramp`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-277/use-jam-view-capture-count-for-first-run-onramp`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-277-use-jam-view-capture-count-for-first-run-onramp`
- Linear branch: `feature/riotbox-277-use-jam-view-capture-count-for-first-run-onramp`
- PR: `#267`
- Merge commit: `6d3f231`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-278`

## Why This Ticket Existed

`first_run_onramp_stage` still read `session.captures.len()` directly even though `CaptureSummaryView.capture_count` already projected the same fact. The first-run path is perform-facing UX and should consume the Jam view projection layer.

## What Shipped

- Updated first-run onramp stage detection to use `jam_view.capture.capture_count`.
- Preserved current first-run behavior and tests.

## Verification

- `cargo fmt --all --check`
- `git diff --check`
- `cargo test -p riotbox-app first_run`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- One-line UI projection cleanup; no first-run redesign, Capture model change, or audio behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
