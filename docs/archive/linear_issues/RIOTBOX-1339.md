# `RIOTBOX-1339` P023: Add shared master-bus soft limiter without masking weak-output gates

- Ticket: `RIOTBOX-1339`
- Title: `P023: Add shared master-bus soft limiter without masking weak-output gates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1339/p023-add-shared-master-bus-soft-limiter-without-masking-weak-output`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1339-master-bus-soft-limiter`
- Linear branch: `feature/riotbox-1339-p023-add-shared-master-bus-soft-limiter-without-masking-weak`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1360 (https://github.com/marang/riotbox/pull/1360)`
- Merge commit: `568fdae068b9f6e6040fe506a067def1dcd9da61`
- Deleted from Linear: `2026-07-05`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Riotbox needed a shared master-bus protection seam so product mixes can hit hard without relying on WAV clipping or per-render gain hacks.

## What Shipped

- Added shared runtime master-bus soft limiter, applied it to realtime/runtime mix and Feral-grid product mixes, exposed pre/post limiter metrics in reports/manifests, preserved weak-output gates, and validated with focused limiter/runtime/Feral-grid tests plus just audio-qa-ci and just ci.

## Notes

- None
