# `RIOTBOX-352` Split jam_app test module below 500-line file budget

- Ticket: `RIOTBOX-352`
- Title: `Split jam_app test module below 500-line file budget`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-352/split-jam-app-test-module-below-500-line-file-budget`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-352-split-jam-app-tests`
- Linear branch: `feature/riotbox-352-split-jam_app-test-module-below-500-line-file-budget`
- Assignee: `Markus`
- Labels: `review-followup`, `workflow`
- PR: `#342`
- Merge commit: `6d079852739c1d48de09ecee190b51fe2246d4bb`
- Deleted from Linear: `2026-04-29`
- Verification: `cargo test -p riotbox-app`, `cargo test`, `just ci`, GitHub Actions `rust-ci`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-353`, `RIOTBOX-354`, `RIOTBOX-355`, `RIOTBOX-356`

## Why This Ticket Existed

`crates/riotbox-app/src/jam_app/tests.rs` had grown into a large test hotspot. Even though the tests were valuable, the single file consumed too much context whenever Jam app behavior needed review.

## What Shipped

- replaced the large `jam_app/tests.rs` body with a small include index
- moved existing Jam app tests into behavior-area files under `crates/riotbox-app/src/jam_app/tests/`
- preserved test behavior while making individual fixture families easier to inspect

## Notes

- this was a mechanical test-organization slice
- no product behavior changed
