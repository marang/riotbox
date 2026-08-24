# `RIOTBOX-1459` P023: Re-establish durable weak-source and bad-timing outcomes

- Ticket: `RIOTBOX-1459`
- Title: `P023: Re-establish durable weak-source and bad-timing outcomes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1459/p023-re-establish-durable-weak-source-and-bad-timing-outcomes`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-08-24`
- Started: `2026-08-24`
- Finished: `2026-08-24`
- Branch: `feature/riotbox-1459-durable-negative-outcomes`
- Linear branch: `feature/riotbox-1459-p023-re-establish-durable-weak-source-and-bad-timing`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#1446 (https://github.com/marang/riotbox/pull/1446)`
- Merge commit: `450ce8c5b521e95b2a67175026022be3d781c773`
- Deleted from Linear: `2026-08-24`
- Verification: `exact source and Holdout-metadata preflight; live and restart observer validation; distinct structured needs_fix review validation; fail-closed human-pass rejection; live source-family and aggregate readiness validation; just ci; GitHub rust-ci (PR #1446)`
- Docs touched: `docs/reviews/riotbox_1459_durable_negative_outcome_recheck_2026-08-24.md`, `docs/phase_definition_of_done.md`, `docs/execution_roadmap.md`, `docs/README.md`
- Follow-ups: `RIOTBOX-1033 owns stronger musical downbeat evidence while preserving manual-confirm fallback; weak_source and bad_timing remain unresolved in live readiness`

## Why This Ticket Existed

Fresh durable evidence was required because the old Beat20 negative-family review paths had expired and could not be reconstructed honestly from Markdown or hashes.

## What Shipped

- Ran one exact registered Development Beat20 assignment and process restart with stopped transport, source-only routing, idle generated lanes, and no fallback.
- Recorded one bounded human product review as distinct weak_source and bad_timing needs_fix records after the listener placed the downbeat at the file boundary while the probe selected beat 3.
- Kept both records out of the live bank, revalidated blocked readiness, and routed detector-quality follow-up to existing RIOTBOX-1033.

## Notes

- This ticket closes as a completed negative qualification, not as the two
  originally expected family passes. Neither needs-fix record was promoted.
