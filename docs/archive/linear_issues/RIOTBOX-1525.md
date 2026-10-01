# `RIOTBOX-1525` Reject unrepresentable Feral grid frame/sample capacities before rendering

- Ticket: `RIOTBOX-1525`
- Title: `Reject unrepresentable Feral grid frame/sample capacities before rendering`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1525/reject-unrepresentable-feral-grid-framesample-capacities-before`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1525-grid-capacity-guard`
- Linear branch: `feature/riotbox-1525-reject-unrepresentable-feral-grid-framesample-capacities`
- Assignee: `Markus`
- Labels: `Bug`, `review-followup`, `timing`
- PR: `#1594 (https://github.com/marang/riotbox/pull/1594)`
- Merge commit: `e242fe596905636275e97704e3ad06b70ccfd0b3`
- Deleted from Linear: `2026-10-01`
- Verification: `Original four rejection tests red in Debug and actual Release; after guard 52 unit plus two integration tests pass both profiles. 31 previous CLI cases and 39 named synthetic hashes including 16 output WAVs unchanged.`; `Full source-free just ci and final warning/error scans green; native exact-head run 36812497713 both jobs success, zero reviews/comments, PR #1594 merged.`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`, `docs/reviews/riotbox_1525_grid_capacity_guard_2026-10-01.md`
- Follow-ups: `RIOTBOX-1526 handles separate W-30 render/PCM16 preflight. Representable-large allocation policy and source/directory ordering remain bounded limits; RIOTBOX-1509 remains unresolved. No real-source/human/device/music qualification.`

## Why This Ticket Existed

Architecture checkpoint demonstrated accepted tiny finite BPM with unrepresentable stereo buffers and scalar overflow/wrapping.

## What Shipped

- Existing Grid Result rejects unrepresentable configured-channel f32 byte layouts before rendering; no new musical threshold or memory-budget policy.
- Six adjacent pure regressions and eight guarded synthetic CLI rejection/preservation paths; ordinary rounding and bytes retained.

## Notes

- None
