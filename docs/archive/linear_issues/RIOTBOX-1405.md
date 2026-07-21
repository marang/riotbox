# `RIOTBOX-1405` P023: Validate weak and bad-timing sources through honest degraded or reject behavior

- Ticket: `RIOTBOX-1405`
- Title: `P023: Validate weak and bad-timing sources through honest degraded or reject behavior`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1405/p023-validate-weak-and-bad-timing-sources-through-honest-degraded-or`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-07-11`
- Started: `2026-07-21`
- Finished: `2026-07-21`
- Branch: `feature/riotbox-1405-p023-validate-weak-and-bad-timing-sources-through-honest`
- Linear branch: `feature/riotbox-1405-p023-validate-weak-and-bad-timing-sources-through-honest`
- Assignee: `Markus`
- Labels: `Analysis`, `Audio`, `Feature`
- PR: `#1374 (https://github.com/marang/riotbox/pull/1374)`
- Merge commit: `d96660307b481163e2ec2cc67a52c30c32ad9148`
- Deleted from Linear: `2026-07-21`
- Verification: `just ci; GitHub rust-ci (fmt, tests, audio QA smoke, clippy); exact degraded product reviews with human pass`
- Docs touched: `docs/reviews/riotbox_1405_degraded_product_review_2026-07-21.md`
- Follow-ups: `RIOTBOX-1408`

## Why This Ticket Existed

Weak and bad-timing sources needed an honest exact-product-path outcome instead of forced demo music, synthetic fallback, or fixture-only evidence.

## What Shipped

- Added typed observer and compact TUI performance readiness for trusted, degraded, and unavailable source timing.
- Added hash-bound degraded product reviews that rederive full runtime evidence and reject fallback, generated lanes, unsafe trajectory state, or fixture reviewers.
- Recorded a bounded human product pass and promoted weak_source and bad_timing as reviewed_degraded_or_reject without rendered candidate WAVs or demo-ready claims.

## Notes

- The human pass covers understandable state, reason, and next action only; general TUI polish remains later work.
