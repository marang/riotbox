# `RIOTBOX-254` Style latest landed result line on Jam

- Ticket: `RIOTBOX-254`
- Title: `Style latest landed result line on Jam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-254/style-latest-landed-result-line-on-jam`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-254-style-latest-landed-result-line-on-jam`
- Linear branch: `feature/riotbox-254-style-latest-landed-result-line-on-jam`
- PR: `#244`
- Merge commit: `ff7c4ea`
- Labels: `ux`
- Follow-ups: `RIOTBOX-255`

## Why This Ticket Existed

The Jam `Next` panel had hierarchy for Scene pending intent, timing rail, and Scene post-commit cues. The `landed ...` line still read as flat text, even though it tells the performer what just committed. Styling the landed result closes the small visual hierarchy loop for queued -> landed -> next.

## What Shipped

- Rendered the latest landed Jam line as styled spans instead of one flat string.
- Emphasized committed action result and Scene energy direction while preserving visible text.
- Kept string helpers for first-run and gesture copy that need plain text.
- Added focused UI style coverage for landed-result hierarchy.

## Verification

- `git diff --check main...HEAD`
- `cargo fmt --check`
- `cargo test -p riotbox-app latest_landed_line -- --nocapture`
- `cargo test -p riotbox-app post_commit -- --nocapture`
- `cargo test -p riotbox-app first_result -- --nocapture`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI styling slice only; no action history behavior, Log redesign, broad Jam layout changes, or audio behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
