# `RIOTBOX-82` Unify W-30 phrase-cue conflict blocking between loop freeze and resample

- Ticket: `RIOTBOX-82`
- Title: `Unify W-30 phrase-cue conflict blocking between loop freeze and resample`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-82/unify-w-30-phrase-cue-conflict-blocking-between-loop-freeze-and`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-82-w30-phrase-cue-conflicts`
- Linear branch: `feature/riotbox-82-unify-w-30-phrase-cue-conflict-blocking-between-loop-freeze`
- Assignee: `Markus`
- Labels: `None`
- PR: `#76`
- Merge commit: `f551475159c884dbf7ec6818c9dfe4a512aa36fe`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#209`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-83`, `RIOTBOX-84`, `RIOTBOX-85`

## Why This Ticket Existed

The follow-up periodic review found that `w30.loop_freeze` and `promote.resample` still used separate pending guards even though both mutate capture lineage and preview state on the same W-30 `NextPhrase` seam. That meant both actions could be queued together and then commit in one phrase boundary with order-dependent side effects.

## What Shipped

- replaced the split W-30 phrase-capture pending checks with one shared exclusivity guard
- made `w30.loop_freeze` and `promote.resample` mutually exclusive when they target the W-30 lane phrase seam
- added focused regressions for both queue-order directions so the conflict stays locked down

## Notes

- this slice is intentionally narrow and changes no other W-30 runtime behavior
- the next review-driven follow-ups are to move Capture pending summaries behind projected app/core state and to scope W-30 diagnostics to the current lane target
