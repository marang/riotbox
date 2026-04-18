# `RIOTBOX-123` Surface one bounded scene-result history cue in Log

- Ticket: `RIOTBOX-123`
- Title: `Surface one bounded scene-result history cue in Log`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-123/surface-one-bounded-scene-result-history-cue-in-log`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-123-scene-log-history`
- Linear branch: `feature/riotbox-123-surface-one-bounded-scene-result-history-cue-in-log`
- Assignee: `Markus`
- Labels: `None`
- PR: `#118`
- Merge commit: `afcc63ae119d179255236fdd64564cb7b946a73a`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, branch diff review, GitHub Actions `Rust CI` run `#327`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-129`

## Why This Ticket Existed

Once Scene Brain had `jump` and `restore` flowing on the current shell seam, Riotbox needed one compact way to reconstruct the most recent scene move without inventing a second scene-history subsystem. The `Log` surface was the right place for a bounded recent-result memory.

## What Shipped

- added a compact scene trail cue to the `Log` counts panel for the last committed `jump` and `restore` results
- kept the action-log truth surface intact instead of creating a parallel scene-history view
- refreshed the shared Scene Brain shell fixture so recent-result expectations stay replay-safe

## Notes

- the trail deliberately uses compressed `j` / `r` vocabulary to fit the current shell budget
- this slice improved traceability only; it did not change scene launch or restore behavior
