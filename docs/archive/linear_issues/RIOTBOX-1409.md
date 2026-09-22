# `RIOTBOX-1409` Replace stringly Source Graph and Session source identities with typed contracts

- Ticket: `RIOTBOX-1409`
- Title: `Replace stringly Source Graph and Session source identities with typed contracts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1409/replace-stringly-source-graph-and-session-source-identities-with-typed`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-19`
- Started: `2026-09-22`
- Finished: `2026-09-22`
- Branch: `feature/riotbox-1409-typed-source-identities`
- Linear branch: `feature/riotbox-1409-replace-stringly-source-graph-and-session-source-identities`
- Assignee: `Markus`
- Labels: `Core`, `Improvement`, `review-followup`
- PR: `#1537 (https://github.com/marang/riotbox/pull/1537)`
- Merge commit: `d8a4db83399bf5f7d69fba06ef46d5489ec587f4`
- Deleted from Linear: `2026-09-22`
- Verification: `Two final normal-parallel just ci passes; 767 App, 269 Audio, 470 Core and 14 Sidecar library tests plus synthetic QA, strict Clippy and sequential solo review.`
- Docs touched: `Source Graph, Session and arrangement specs; RBX-377/378; docs/reviews/riotbox_1409_typed_source_identity_2026-09-22.md.`
- Follow-ups: `None`

## Why This Ticket Existed

Replace stringly persisted source identities without breaking supported V1 graph or Session restore contracts.

## What Shipped

- Typed graph endpoints, shared decode profiles, explicit Scene-to-Section bindings and transactional legacy replay migration.

## Notes

- No audible change or human-listening claim. Feature PR #1537 green and merged at d8a4db83. Follow-up numeric audit RIOTBOX-1420 proceeds independently.
