# `RIOTBOX-1393` P023: Make MC-202 source phrase groove map use sub-beat anchor timing

- Ticket: `RIOTBOX-1393`
- Title: `P023: Make MC-202 source phrase groove map use sub-beat anchor timing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1393/p023-make-mc-202-source-phrase-groove-map-use-sub-beat-anchor-timing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1393-p023-mc202-sub-beat-groove-map`
- Linear branch: `feature/riotbox-1393-p023-make-mc-202-source-phrase-groove-map-use-sub-beat`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1357 (https://github.com/marang/riotbox/pull/1357)`
- Merge commit: `9103b4df767616f1b29ffba91fda941d3f74d293`
- Deleted from Linear: `2026-07-05`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Preserve sub-beat Source Graph anchor timing in MC-202 source phrase groove placement so pushed/offbeat answer evidence can change rhythm cells and rendered output instead of collapsing to coarse beat-index placement.

## What Shipped

- MC-202 groove mapping now uses matching BeatPoint plus anchor time_seconds for 16-step placement when available, keeps beat/bar fallback for incomplete timing data, adds a regression for same coarse beat index with different sub-beat answer timing, and updates the P023 roadmap plus MC-202 source phrase plan.

## Notes

- None
