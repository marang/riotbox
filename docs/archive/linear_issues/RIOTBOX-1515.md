# `RIOTBOX-1515` Reject overflowing Observer/audio anchor totals without panic or wrapped acceptance

- Ticket: `RIOTBOX-1515`
- Title: `Reject overflowing Observer/audio anchor totals without panic or wrapped acceptance`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1515/reject-overflowing-observeraudio-anchor-totals-without-panic-or`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1515-anchor-overflow`
- Linear branch: `feature/riotbox-1515-reject-overflowing-observeraudio-anchor-totals-without-panic`
- Assignee: `Markus`
- Labels: `Bug`, `review-followup`
- PR: `#1576 (https://github.com/marang/riotbox/pull/1576)`
- Merge commit: `942dfe60fdecf0f6b7b9b6bc0d045475ead55899`
- Deleted from Linear: `2026-10-01`
- Verification: `Original Debug and actual Release red; both green: Observer 67 and CLI integration 2. All 21 previous CLI byte/status baselines identical. Source-free just ci passed, logs explicitly warning/error-free. Exact head 98e99b3dfc3b7fdb26ef2d9a0896fae068d6c7d1 native 36797441506 Rust/Windows Sidecar passed, zero reviews/comments.`
- Docs touched: `docs/reviews/riotbox_1515_observer_anchor_overflow_2026-10-01.md`
- Follow-ups: `RIOTBOX-1509 causal Windows startup diagnosis remains open; unexecuted 32-bit adjacent groove-count portability limitation retained explicitly. RIOTBOX-1517 owns next semantic QA migration.`

## Why This Ticket Existed

Existing unchecked Observer anchor sums panic in Debug and accept contradictory wrapped totals in Release.

## What Shipped

- Both additions checked; unrepresentable total uses existing malformed-evidence path. Six parser regressions and two actual CLI tests cover boundary validity, local degradation and strict rejection without report creation/overwrite.

## Notes

- Solo sequential Rust/correctness/architecture/workflow review and self-review: P2 fixed, no additional changed-diff finding. No schema/threshold/dependency/Stage-A/audio changes or new decision; no sources, holdout, device, DAW, playback or human/music pass.
