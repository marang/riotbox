# `RIOTBOX-1294` Persist MC-202 source phrase plans through replay and restore

- Ticket: `RIOTBOX-1294`
- Title: `Persist MC-202 source phrase plans through replay and restore`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1294/persist-mc-202-source-phrase-plans-through-replay-and-restore`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-18`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1294-mc202-phrase-replay-restore`
- Linear branch: `feature/riotbox-1294-persist-mc-202-source-phrase-plans-through-replay-and`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1268 (https://github.com/marang/riotbox/pull/1268)`
- Merge commit: `73fffae75e42212f7011251a0c37ad907d93e70f`
- Deleted from Linear: `2026-06-18`
- Verification: `cargo fmt; focused riotbox-core replay/persistence/session tests; cargo test -p riotbox-app mc202_restore_replay; git diff --check; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/replay_model_spec.md, docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

MC-202 source-composed phrase plans need replay/restore truth so restored sessions do not silently keep stale runtime state or fall back to primitive bass material.

## What Shipped

- Core replay restores ActionCommitRecord.mc202_source_phrase_plan for MC-202 phrase actions, clears stale plans when commit records lack trusted plans, and persistence tests cover lane-state plus commit-record source phrase plan roundtrip.

## Notes

- SourceGraph reconstruction remains an app restore helper; core replay treats the commit record as the durable truth and clears absent plans.
