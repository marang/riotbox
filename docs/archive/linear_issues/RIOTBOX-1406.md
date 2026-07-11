# `RIOTBOX-1406` P023: Align concrete execution order with audible Golden Path priority

- Ticket: `RIOTBOX-1406`
- Title: `P023: Align concrete execution order with audible Golden Path priority`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1406/p023-align-concrete-execution-order-with-audible-golden-path-priority`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M1 | Reset + Real Listening`
- Status: `Done`
- Created: `2026-07-11`
- Started: `2026-07-11`
- Finished: `2026-07-11`
- Branch: `feature/riotbox-1406-p023-align-concrete-execution-order-with-audible-golden-path`
- Linear branch: `feature/riotbox-1406-p023-align-concrete-execution-order-with-audible-golden-path`
- Assignee: `Markus`
- Labels: `Bug`, `Docs`, `workflow`
- PR: `#1362 (https://github.com/marang/riotbox/pull/1362)`
- Merge commit: `b1c010417d31dca7a8e310f88e8a4580f081c6ff`
- Deleted from Linear: `2026-07-11`
- Verification: `just check: passed; git diff --check: passed; branch code-review: no findings; GitHub Actions Rust CI: passed`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `RIOTBOX-1398 remains the immediate next ticket, followed by RIOTBOX-1330, RIOTBOX-1333, and RIOTBOX-1335.`

## Why This Ticket Existed

Correct the post-merge RIOTBOX-1397 review finding so concrete roadmap and Linear selection cannot prioritize QA or infrastructure ahead of an unblocked audible Golden Path slice.

## What Shipped

- Kept RIOTBOX-1398 as the immediate real human-listening ticket and moved RIOTBOX-1330/1333/1335 directly behind it.
- Moved RIOTBOX-1403 and RIOTBOX-1399 to M4 and blocked both on RIOTBOX-1402 so they cannot preempt the first human-passed alpha.
- Placed RIOTBOX-1403 next to RIOTBOX-1405 and made RIOTBOX-1399 explicitly post-alpha unless measured validation cost becomes a named live-path blocker.

## Notes

- Maintenance/regression correction only; no sound improvement was claimed.
