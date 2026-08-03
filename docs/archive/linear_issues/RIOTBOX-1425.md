# `RIOTBOX-1425` P023: Make W-30 hit-shaper reachability source-adaptive

- Ticket: `RIOTBOX-1425`
- Title: `P023: Make W-30 hit-shaper reachability source-adaptive`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1425/p023-make-w-30-hit-shaper-reachability-source-adaptive`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Superseded`
- Created: `2026-07-27`
- Started: `2026-07-27`
- Finished: `2026-07-27`
- Branch: `feature/riotbox-1425-p023-make-w30-hit-shaper-reachability-source-adaptive`
- Linear branch: `feature/riotbox-1425-p023-make-w-30-hit-shaper-reachability-source-adaptive`
- Assignee: `Owner`
- Labels: `Audio`, `Feature`, `review-followup`
- PR: `#1379 (https://github.com/marang/riotbox/pull/1379), merged only into the abandoned RIOTBOX-1422 stack`
- Merge commit: `e99a3e56bb8680402e5eac2f9492e9c8347cd2cc`
- Deleted from Linear: `2026-07-27`
- Verification: `Historical focused tests, development matrix, full CI, and GitHub Rust CI passed; the merge commit is not an ancestor of main.`
- Docs touched: `Abandoned stack only: hit-shaper development manifest, engineering notes, QA specs, decision log, and RIOTBOX-1425 review.`
- Follow-ups: `RIOTBOX-1429`, then `RIOTBOX-1428`

## Why This Ticket Existed

Make exact W-30 hit-shaper reachability source-adaptive before another frozen
holdout was consumed, without weakening source-ownership gates.

## What Shipped

- Nothing from this ticket shipped to `main`.
- PR #1379 merged only into the later-abandoned RIOTBOX-1422 feature stack.

## Notes

- Linear originally recorded this ticket as `Done` and archived it on
  `2026-07-27`; the repo disposition is corrected to `Superseded` because its
  target branch never reached `main`.
- Stack-only reachability work may be considered for narrow re-extraction from commit `58233c6dfb8a57ed976d10adbbf0fee15cf52d6a`.
- CI and matrix success established technical reachability, not a musical pass.
- See the [RIOTBOX-1422 closeout](../../reviews/riotbox_1422_h27_h30_rejected_experiment_closeout_2026-08-02.md).
