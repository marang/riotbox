# `RIOTBOX-862` Improve real-source short-loop timing confidence on Beat20

- Ticket: `RIOTBOX-862`
- Title: `Improve real-source short-loop timing confidence on Beat20`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-862/improve-real-source-short-loop-timing-confidence-on-beat20`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-862-improve-real-source-short-loop-timing-confidence-on-beat20`
- Linear branch: `feature/riotbox-862-improve-real-source-short-loop-timing-confidence-on-beat20`
- Assignee: `Markus`
- Labels: `Analysis`, `Audio`
- PR: `#856 (https://github.com/marang/riotbox/pull/856)`
- Merge commit: `89a39717d77b7a7d7840509e41a5e2727cdac28f`
- Deleted from Linear: `2026-05-21`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Improved the real-source Source Timing confidence path for Beat20 without auto-trusting ambiguous downbeat evidence.

## What Shipped

- Classified near-stable competing downbeat phases as reviewable ambiguity; Beat20 now reports needs_review/ambiguous while remaining manual_confirm_only; updated tests, local expectations, benchmark notes, and Jam recipe guidance.

## Notes

- None
