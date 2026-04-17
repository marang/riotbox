# `RIOTBOX-71` Ticket Archive

- Ticket: `RIOTBOX-71`
- Title: `Move pending W-30 resample cue summary into the core Jam view model`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-71/move-pending-w-30-resample-cue-summary-into-the-core-jam-view-model`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-71-w30-resample-cue-summary`
- Linear branch: `feature/riotbox-71-move-pending-w-30-resample-cue-summary-into-the-core-jam`
- Assignee: `Markus`
- Labels: `None`
- PR: `#64`
- Merge commit: `4416d17`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`, `self-review`, `GitHub Actions Rust CI`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-67`

## Why This Ticket Existed

The periodic review in `RIOTBOX-68` found one remaining W-30 presentation leak: the shell still scanned `ActionQueue` directly to discover a pending resample cue even though the rest of the lane summaries already came through the core Jam view model. That small exception would have made later W-30 shell work easier to drift into ad hoc queue reads.

## What Shipped

- Added explicit pending W-30 resample cue data to `JamViewModel.lanes`.
- Stopped `ui.rs` from scanning `ActionQueue` directly for the pending W-30 resample cue label.
- Extended the core Jam-view regression fixture so the new lane summary field stays covered.
- Recorded the boundary cleanup in `docs/research_decision_log.md`.

## Notes

- This slice intentionally changed presentation wiring only. It did not add new W-30 behavior or a second resample path.
- Future shell summaries should keep extending the core Jam-view contract instead of introducing new direct queue inspection in the UI layer.
