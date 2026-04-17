# `RIOTBOX-83` Move Capture pending summaries behind projected app/core state

- Ticket: `RIOTBOX-83`
- Title: `Move Capture pending summaries behind projected app/core state`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-83/move-capture-pending-summaries-behind-projected-appcore-state`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-83-capture-pending-projection`
- Linear branch: `feature/riotbox-83-move-capture-pending-summaries-behind-projected-appcore`
- Assignee: `Markus`
- Labels: `None`
- PR: `#77`
- Merge commit: `aa27ce5d0e602cbe09c385dbeec2358d9498b8ef`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#212`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-84`, `RIOTBOX-85`

## Why This Ticket Existed

The follow-up periodic review found that the Capture shell still owned a second copy of capture-action filtering logic by reading `ActionQueue` directly for pending capture counts and pending capture items. That created another presentation-path leak after the earlier W-30 cleanup work and risked shell drift as more capture-related commands accumulate on the same seam.

## What Shipped

- projected pending capture counts through `JamViewModel.capture`
- projected up to four pending capture items with stable target and explanation text through the same core presentation model
- stopped reading `ActionQueue` directly from the Capture shell for those summaries
- extended core and shell regressions so the projection seam stays locked in place

## Notes

- this slice is presentation-boundary cleanup only and adds no new W-30 runtime behavior
- the next review-driven follow-up is to scope W-30 operation diagnostics to the current lane target instead of stale global history
