# `RIOTBOX-1272` MC-202 phrase memory variation for repeated live triggers

- Ticket: `RIOTBOX-1272`
- Title: `MC-202 phrase memory variation for repeated live triggers`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1272/mc-202-phrase-memory-variation-for-repeated-live-triggers`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-15`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1272-mc202-phrase-memory-variation`
- Linear branch: `feature/riotbox-1272-mc-202-phrase-memory-variation-for-repeated-live-triggers`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1247 (https://github.com/marang/riotbox/pull/1247)`
- Merge commit: `c7bfbff3`
- Deleted from Linear: `2026-06-15`
- Verification: `cargo test -p riotbox-app mc202 -- --nocapture; git diff --check; just ci (/tmp/riotbox-1272-just-ci.log); GitHub rust-ci pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`
- Follow-ups: `RIOTBOX-1264 remains open for structured listening review and additional production-quality fixes toward producer-grade dense/non-dense MC-202 output.`

## Why This Ticket Existed

Repeated MC-202 live triggers needed to avoid static re-fire of the same source phrase footprint and expose phrase-memory pressure in replayable Session evidence.

## What Shipped

- Tightened phrase-memory rejection for too-close same-family source-derived candidates, added selected memory-distance provenance, strengthened commit-path tests for repeated trigger plan/render variation, and documented the behavior.

## Notes

- No human listening verdict or demo-readiness claim was added.
