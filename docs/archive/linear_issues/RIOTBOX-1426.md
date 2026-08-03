# `RIOTBOX-1426` P023: Derive a source-backed downbeat phase for externally supplied tempo

- Ticket: `RIOTBOX-1426`
- Title: `P023: Derive a source-backed downbeat phase for externally supplied tempo`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1426/p023-derive-a-source-backed-downbeat-phase-for-externally-supplied`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Superseded`
- Created: `2026-07-27`
- Started: `2026-07-27`
- Finished: `2026-07-27`
- Branch: `feature/riotbox-1426-p023-derive-source-backed-downbeat-phase-for-external-tempo`
- Linear branch: `feature/riotbox-1426-p023-derive-source-backed-downbeat-phase-for-externally`
- Assignee: `Owner`
- Labels: `Audio`, `Feature`, `review-followup`
- PR: `#1380 (https://github.com/marang/riotbox/pull/1380), merged only into the abandoned RIOTBOX-1422 stack`
- Merge commit: `760fec3b896d913860af190f13988779b001e90d`
- Deleted from Linear: `2026-07-27`
- Verification: `Historical focused timing tests, development matrix, full CI, and GitHub Rust CI passed; the merge commit is not an ancestor of main.`
- Docs touched: `Abandoned stack only: tempo-guided development manifest, Source Graph and source-timing specs, engineering notes, decision log, and RIOTBOX-1426 review.`
- Follow-ups: `RIOTBOX-1429`, then `RIOTBOX-1428`

## Why This Ticket Existed

Provide a typed source-backed phase when external tempo was supplied, without
fabricating manual downbeat truth before another exact-path attempt.

## What Shipped

- Nothing from this ticket shipped to `main`.
- PR #1380 merged only into the later-abandoned RIOTBOX-1422 feature stack.

## Notes

- Linear originally recorded this ticket as `Done` and archived it on
  `2026-07-27`; the repo disposition is corrected to `Superseded` because its
  target branch never reached `main`.
- Stack-only timing work may be considered for narrow re-extraction from commit `47b170db54a331ea798bec069237086249528521`.
- The later dirty H28 product-BPM correction must also be re-extracted narrowly; it was never a clean child-ticket merge.
- See the [RIOTBOX-1422 closeout](../../reviews/riotbox_1422_h27_h30_rejected_experiment_closeout_2026-08-02.md).
