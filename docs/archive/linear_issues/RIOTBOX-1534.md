# `RIOTBOX-1534` Give Feral manifest assertions ordinary explicit test ownership

- Ticket: `RIOTBOX-1534`
- Title: `Give Feral manifest assertions ordinary explicit test ownership`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1534/give-feral-manifest-assertions-ordinary-explicit-test-ownership`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1534-manifest-assertion-owners`
- Linear branch: `feature/riotbox-1534-give-feral-manifest-assertions-ordinary-explicit-test`
- Assignee: `Markus`
- Labels: `Improvement`
- PR: `#1612 (https://github.com/marang/riotbox/pull/1612)`
- Merge commit: `50befb256ebcee56eb2f7492b4de9774d6debd25`
- Deleted from Linear: `2026-10-01`
- Verification: `54 exact Debug/Release tests; 25 full definition bodies preserved; 31 CLI outcomes and 39 artifact hashes unchanged; full source-free just ci and both native gates passed; solo Rust/design/workflow review.`
- Docs touched: `docs/reviews/riotbox_1534_manifest_assertion_owners_2026-10-01.md`
- Follow-ups: `Newly reproduced pre-existing source/output collision requires a separate high-priority integrity fix; optional remaining test namespace cleanup is deferred.`

## Why This Ticket Existed

Replace textual manifest test inclusions with explicit ordinary owners while preserving the complete assertion contract.

## What Shipped

- Three ordinary test modules, explicit actual-owner imports and unchanged test names/bodies; include inventory reduced to five sites in two owners.

## Notes

- No real source, Holdout, device, DAW or human listening claim. No proactive subagents.
