# `RIOTBOX-87` Add replay-safe W-30 slice-pool browse regression fixtures

- Ticket: `RIOTBOX-87`
- Title: `Add replay-safe W-30 slice-pool browse regression fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-87/add-replay-safe-w-30-slice-pool-browse-regression-fixtures`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-87-w30-slice-pool-fixtures`
- Linear branch: `feature/riotbox-87-add-replay-safe-w-30-slice-pool-browse-regression-fixtures`
- Assignee: `Markus`
- Labels: `None`
- PR: `#81`
- Merge commit: `0ead9ef64a6d13bf525b259d850086d27ee71977`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#224`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-88`

## Why This Ticket Existed

`RIOTBOX-85` shipped the first bounded W-30 slice-pool browse control and `RIOTBOX-86` made that browse seam legible in the shell, but the seam still lacked the same replay-safe fixture coverage that the rest of the W-30 MVP already used. The next smallest honest step was to widen the shared regression corpus instead of opening new behavior.

## What Shipped

- extended the shared `w30_regression.json` corpus with a committed `browse_slice_pool` case
- asserted committed app-layer browse state from that fixture data in `jam_app`
- asserted shell-visible Jam, Capture, and Log browse output from the same fixture data in `ui`

## Notes

- this slice is verification-only; it does not change the shipped W-30 runtime seam
- deeper preview profiling remains the next bounded follow-up on top of the same lineage seam
