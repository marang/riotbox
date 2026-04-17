# `RIOTBOX-65` Ticket Archive

- Ticket: `RIOTBOX-65`
- Title: `Add first bounded W-30 internal resample action on the capture-lineage seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-65/add-first-bounded-w-30-internal-resample-action-on-the-capture-lineage`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-65-w30-internal-resample`
- Linear branch: `feature/riotbox-65-add-first-bounded-w-30-internal-resample-action-on-the`
- Assignee: `Markus`
- Labels: `None`
- PR: `#59`
- Merge commit: `42fd17a`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-66`, `RIOTBOX-67`

## Why This Ticket Existed

`RIOTBOX-63` prepared explicit capture lineage metadata and the typed W-30 resample-tap seam, but Riotbox still had no actual committed internal resample action exercising that path. The next honest slice was to prove the existing capture, queue, and replay-safe session seam could create lineage-safe resampled material without opening a second capture inventory, a hidden callback-only resample path, or an early W-30 lab surface.

## What Shipped

- Added the first bounded `promote.resample` queue path for the W-30 lane on the existing `NextPhrase` seam.
- Materialized committed resample actions into a new `CaptureRef` with cloned source-origin refs, extended lineage, and incremented `resample_generation_depth`.
- Updated the W-30 lane to point at the newly committed resample capture and refreshed the runtime resample-tap summary from that same committed seam.
- Surfaced the pending W-30 resample cue in the shell and event loop.
- Recorded the architectural decision in `docs/research_decision_log.md`.

## Notes

- This slice intentionally stops at the first committed internal resample action and does not add new W-30 audio behavior, a resample lab UI, or deeper pad-bank tooling.
- Later W-30 work should continue extending this committed capture-lineage seam rather than bypassing it with a second resample runtime or shell-only state.
